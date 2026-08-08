# Proposal: docker-ci-image

## Why

项目要部署到阿里云 ECS，但本机（WSL）全量编译内存消耗大，已有 OOM 崩溃前科，不应在部署机上编译。需要 CI 自动构建一个尽量小的容器镜像（distroless 无发行版基底），ECS 上 rootless podman 直接 pull 运行。

## What Changes

- 新增多阶段 `Dockerfile`：node 构建前端 dist → rust musl 静态编译 `accounting-api` + `auth-admin` → `gcr.io/distroless/static-debian12` 打包（预期总尺寸 ~15MB）。
- 新增 GitHub Actions workflow：push main / tag 触发构建，推送 `ghcr.io/ydrmaster/accounting`（GITHUB_TOKEN 自动鉴权，零 secrets 配置）。
- workspace `Cargo.toml` 增加 `[profile.release]` 调优：`lto = "fat"`、`codegen-units = 1`、`strip = true`（panic 保持默认 unwind，性能优先于尺寸）。
- 新增 `.dockerignore`，避免 `target/`、`node_modules/` 进入构建上下文。
- 新增部署文档：ECS 上 rootless podman 运行方式、数据卷、auth-admin 用法、时区设置。
- 不改任何应用代码（main.rs 的 npm 自动构建逻辑在镜像内因无 web 源码自动跳过，行为天然正确）。

## Capabilities

### New Capabilities

- `container-deployment`: 容器镜像构建（多阶段 Dockerfile、release profile、.dockerignore）、CI 自动构建与推送（GitHub Actions → GHCR）、ECS 部署运行方式（rootless podman、数据卷、用户管理、时区）。

### Modified Capabilities

（无）

## Impact

- **新增文件**：`Dockerfile`、`.dockerignore`、`.github/workflows/image.yml`、部署文档（docs/ 或 README 章节）。
- **修改文件**：workspace `Cargo.toml`（release profile，影响所有 crate 的 release 构建产物，不改变行为）。
- **CI**：GitHub Actions 新增工作流；GHCR 包托管（需在 GitHub 侧将包设为 public 或配置 ECS 拉取凭证）。
- **运行环境**：ECS 需 rootless podman；数据卷 `/data` 存放 my.db + auth.db。
