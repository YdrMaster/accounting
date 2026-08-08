# container-deployment 规格

## Purpose

定义记账系统的容器化部署能力：将 Rust 后端、前端构建产物与用户管理工具打包为单个自包含、最小化的容器镜像，使用户仅需一条 `podman run` 命令即可运行完整应用。涵盖镜像内容与入口约定、数据持久化与用户管理方式，以及 GitHub Actions 自动构建、推送与镜像 tag 策略，并规定 release 构建参数的调优要求。

## Requirements

### Requirement: 自包含容器镜像

系统 SHALL 提供多阶段 `Dockerfile`，构建出基于 `gcr.io/distroless/static-debian12` 的自包含镜像，包含：`accounting-api` 与 `auth-admin` 静态二进制、前端构建产物 `/app/dist`。镜像 MUST NOT 包含 node、npm、Rust 工具链或前端源码。镜像总尺寸 SHOULD 小于 20MB（解压前）。镜像入口 MUST 为 accounting-api，默认参数：账簿库 `/data/my.db`、认证库 `/data/auth.db`、静态目录 `/app/dist`、语言 zh-cn、端口 3000。

#### Scenario: 镜像内容最小化

- **WHEN** 检查构建出的镜像文件系统
- **THEN** 仅含 distroless/static 基底文件、两个 Rust 二进制与 `/app/dist`，不含 node/npm/cargo/前端源码

#### Scenario: 单容器运行完整应用

- **WHEN** 以 `podman run -v <dir>:/data -p 3000:3000 <镜像>` 启动
- **THEN** 无需任何额外服务即可通过浏览器完成登录并使用全部功能

#### Scenario: 前端自动构建逻辑不触发

- **WHEN** 容器内启动 accounting-api
- **THEN** 因镜像不含 web 源码目录，npm 自动构建逻辑被跳过，直接托管 `/app/dist`

### Requirement: CI 自动构建与推送

系统 SHALL 提供 GitHub Actions workflow：push 到 main 分支或 `v*` tag 时自动构建镜像并推送 GHCR；PR 触发时 MUST 仅构建验证而不推送。tag 策略 MUST 包含：main → `latest`、`v*` tag → 对应版本号、每次构建 → `sha-<short>`。镜像 MUST 使用 GITHUB_TOKEN 鉴权推送，无需额外 secrets。CI MUST 缓存 Cargo 依赖层（cargo-chef 或等价分层）以避免每次全量编译依赖。

#### Scenario: main 分支推送

- **WHEN** 向 main 推送 commit
- **THEN** CI 构建镜像并推送 `ghcr.io/<owner>/accounting:latest` 与 `:sha-<short>`

#### Scenario: 版本 tag

- **WHEN** 推送 `v1.2.3` tag
- **THEN** CI 推送 `:v1.2.3`（及 `:latest`）

#### Scenario: PR 验证

- **WHEN** 提交 PR
- **THEN** CI 构建镜像验证可构建性但不推送到 registry

### Requirement: 数据持久化与用户管理

镜像 MUST 声明 `/data` 为数据卷，账簿库与认证库 MUST 存于 `/data` 下。镜像 MUST 包含 `auth-admin`，支持通过 `podman exec` 管理用户。部署文档 MUST 说明：rootless podman 下挂载宿主空目录的权限模型（容器 root 映射为宿主用户）、时区注入（`-e TZ`）、GHCR 包可见性设置、以及国内网络拉取 GHCR 的加速方案。

#### Scenario: 数据卷持久化

- **WHEN** 容器删除后以同一 `/data` 卷重新启动
- **THEN** 账簿数据与用户数据完整保留

#### Scenario: 容器内管理用户

- **WHEN** 执行 `podman exec <容器> auth-admin --db /data/auth.db user add --username alice --password <pwd>`
- **THEN** 用户创建成功，可立即用于登录

### Requirement: Release 构建调优

workspace `Cargo.toml` SHALL 配置 `[profile.release]`：`lto = "fat"`、`codegen-units = 1`、`strip = true`。`opt-level` 与 `panic` MUST 保持默认（3 / unwind），以保证性能优先与完整 panic 信息。

#### Scenario: 调优生效

- **WHEN** 执行 `cargo build --release`
- **THEN** 产物经 strip 且应用 LTO，panic 时仍输出完整调用栈信息
