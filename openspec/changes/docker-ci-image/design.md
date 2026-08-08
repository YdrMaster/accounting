# Design: docker-ci-image

## Context

Rust workspace（7 crate）+ Vue 前端。目标部署：阿里云 ECS（x86_64），rootless podman。关键事实（探索阶段已验证）：

- sqlx sqlite 静态编译、Cargo.lock 零 OpenSSL/rustls → 可做全静态 musl 二进制
- rust-i18n locales 编译期嵌入，无运行时资源文件
- 前端 dist 仅 1.1MB；release 二进制 11MB（未调优），auth-admin 5.4MB
- main.rs 的 npm 自动构建仅在 `accounting-web/` 源码存在时触发，镜像内只放 dist 即自动跳过
- chrono::Local 用于「今天」→ 需要 tzdata（distroless/static 自带）+ `TZ` 环境变量
- 本机 docker daemon 不可用、podman 4.9.3（rootless）可用；remote 在 GitHub → CI 选 GitHub Actions

## Goals / Non-Goals

**Goals:**

- CI（GitHub Actions）一键构建并推送 distroless 镜像到 GHCR，总尺寸目标 < 20MB。
- 镜像自包含：accounting-api + auth-admin + 前端 dist，单容器即可运行完整应用。
- ECS 部署只需 `podman pull` + `podman run`，零编译、零依赖安装。
- 镜像构建可复现：任何机器 `podman build` 得到相同结果。

**Non-Goals:**

- 多架构（ARM）构建、镜像签名、漏洞扫描。
- docker-compose / k8s 编排（自用单容器，run 命令 + 文档即可）。
- 自动部署到 ECS（CI 只出镜像，部署手动执行）。
- 应用代码改动。

## Decisions

### D1: distroless/static 默认 tag（容器内 root），不用 :nonroot

rootless podman 下容器 root 映射为宿主机当前用户，挂载空目录直接可写，文件属主正确——零 chown 配置。`:nonroot`（uid 65532）在 rootless 下映射到 subuid（165531），反而制造权限问题（需 `podman unshare chown`）。容器内 root 在 rootless podman 下无宿主机特权，自用场景隔离足够。部署文档中注明：若未来改用 rootful docker 并想要 nonroot，需 `chown -R 65532:65532` 数据目录。

### D2: musl 全静态编译；glibc + distroless/cc 作为兜底

`rust:alpine` stage 自带 musl target 与 musl-gcc，libsqlite3-sys 的 C 编译可直接工作。若 musl 构建在 CI 中意外踩坑，退路是 Debian glibc stage + `distroless/cc-debian12`（总尺寸 ~30MB，行为等价）——不恋战，直接切换。

### D3: 多阶段 Dockerfile，构建全部发生在 CI

```
stage frontend: node:22-alpine → npm ci && npm run build → /dist
stage backend:  rust:alpine    → cargo build --release（api + auth-admin，musl 静态）
stage final:    distroless/static-debian12 → COPY 两二进制 + dist
```

CI runner 拉 gcr.io 无网络障碍，国内网络问题被隔离在 CI 之外。Cargo 依赖用 `cargo-chef`（或手动 dummy-main 分层）缓存依赖层，避免每次全量重编译依赖。

### D4: release profile：性能优先、保留 panic 信息

`lto = "fat"`、`codegen-units = 1`、`strip = true`；`opt-level` 与 `panic` 保持默认（3 / unwind）。二进制从 11MB 降到约 7MB 是顺带收益，不是目标。

### D5: GHCR 托管，GITHUB_TOKEN 鉴权

推 `ghcr.io/<owner>/accounting`，tag 策略：`latest`（main 分支）+ `v*`（tag 触发）+ `sha-<short>`（每次构建）。GitHub 侧需将包可见性设为 public（或 ECS 配 PAT 拉取）——文档写明。ECS 拉取慢的应对：ACR 镜像加速或代理，写入部署文档 troubleshooting。

### D6: 运行时契约

- 入口：`accounting-api --db /data/my.db --auth-db /data/auth.db --static-dir /app/dist --lang zh-cn --port 3000`
- `/data` 声明 VOLUME（my.db、auth.db 持久化）
- `EXPOSE 3000`；时区经 `-e TZ=Asia/Shanghai` 注入
- 用户管理：`podman exec <容器> auth-admin --db /data/auth.db user add ...`

## Risks / Trade-offs

- [musl 下某个依赖编译失败] → D2 兜底路径（glibc + cc 镜像），已在决策中备案。
- [GHCR 包默认 private 导致 ECS 无法匿名拉取] → 部署文档写明首次需在 GitHub Packages 设置 public，或配置 podman 登录凭证。
- [ECS 拉 ghcr.io 慢/失败] → 文档给 ACR 加速/代理方案；不阻塞 CI 本身。
- [cargo-chef 增加复杂度] → 收益明确（依赖层缓存，CI 从 ~15min 降到 ~3min 增量），接受。
- [rootless podman 端口映射] → 3000 是非特权端口，rootless 可直接映射，无问题。

## Migration Plan

1. 合并 Dockerfile + workflow + profile 后，push tag（如 v0.1.0）触发首次构建。
2. GitHub Packages 设置镜像 public。
3. ECS：`podman pull` → `mkdir data && podman run` → `podman exec auth-admin user add` 建首个用户 → 配 HTTPS 反代。
4. 回滚：镜像按 tag 不可变，回滚即 pull 旧 tag；仓库侧 revert 不影响已发镜像。

## Open Questions

- 触发策略先按「main 推 latest、tag 推版本号」；是否要 PR 构建仅验证不推送——实现时按最小可用做（PR 只 build 不 push）。
