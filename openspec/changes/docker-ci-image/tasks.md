# Tasks: docker-ci-image

## 1. Release profile 与构建上下文

- [x] 1.1 workspace `Cargo.toml` 增加 `[profile.release]`：lto = "fat"、codegen-units = 1、strip = true
- [x] 1.2 新增 `.dockerignore`：target/、node_modules/、dist/、.git/、openspec/、docs/、plan/ 等

## 2. 多阶段 Dockerfile

- [x] 2.1 frontend stage：`node:22-alpine`，npm ci && npm run build，产出 /dist（利用 package-lock.json 分层缓存）
- [x] 2.2 backend stage：`rust:alpine`，cargo-chef 缓存依赖层，musl release 构建 accounting-api 与 auth-admin，验证二进制为静态（ldd 报 not a dynamic executable）
- [x] 2.3 final stage：`gcr.io/distroless/static-debian12`，COPY 两二进制 + dist，VOLUME /data，EXPOSE 3000，ENTRYPOINT 固定参数（--db /data/my.db --auth-db /data/auth.db --static-dir /app/dist --lang zh-cn --port 3000）

## 3. GitHub Actions workflow

- [x] 3.1 `.github/workflows/image.yml`：main/tag 推送时 build + push（ghcr.io，GITHUB_TOKEN），PR 仅 build；tag 策略 latest / 版本号 / sha-short；packages: write 权限声明
- [x] 3.2 利用 buildx + GHA cache（或 podman build --layers 等价）加速镜像构建

## 4. 本地验证（podman）

- [x] 4.1 `podman build` 完整跑通，镜像尺寸 < 20MB 确认（podman images）
- [x] 4.2 本地运行验证：挂空目录 /data → 启动 → podman exec auth-admin 建用户 → 浏览器登录、记一笔账、重启容器数据保留
- [x] 4.3 若 musl 构建失败，按 design D2 切换 glibc + distroless/cc 兜底并记录原因

## 5. 文档与首次发布

- [x] 5.1 部署文档（README 新章节或 docs/deployment.md）：ECS rootless podman 运行命令、TZ 注入、auth-admin 用法、GHCR public 设置、国内拉取加速、nonroot 变体的 chown 说明
- [ ] 5.2 push tag 触发首次 CI 构建，确认 GHCR 出现镜像并设置包可见性
