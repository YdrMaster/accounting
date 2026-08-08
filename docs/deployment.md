# 容器部署指南

本文档说明如何通过 CI 构建的容器镜像部署 accounting 到公网服务器（以阿里云 ECS + rootless podman 为例）。

## 镜像说明

- 镜像：`ghcr.io/<owner>/accounting`，基底为 `gcr.io/distroless/static-debian12`（无 shell、无包管理器，约 15MB）
- 内含：`accounting-api`（HTTP 服务 + 静态前端托管）、`auth-admin`（用户管理）、前端 dist
- tag 策略：`latest`（main 分支）、`v*`（版本 tag）、`sha-<short>`（每次构建）
- 数据：`/data` 卷，包含账簿库 `my.db` 与认证库 `auth.db`
- 端口：3000

## CI 构建

push 到 `main` 或 `v*` tag 时 GitHub Actions 自动构建并推送 GHCR；PR 仅验证可构建、不推送。

**首次发布后**：到 GitHub 仓库的 Packages 页面将 `accounting` 包可见性设为 **Public**（否则 ECS 需要 `podman login ghcr.io` 配置 PAT 才能拉取）。

## ECS 部署

### 1. 准备

```bash
# 安装 podman（Ubuntu 示例）
sudo apt install -y podman

# 数据目录（rootless podman 下容器 root 映射为当前用户，无需 chown）
mkdir -p ~/accounting/data
```

### 2. 拉取并运行

```bash
podman pull ghcr.io/<owner>/accounting:latest

podman run -d --name accounting \
  -v ~/accounting/data:/data \
  -p 3000:3000 \
  -e TZ=Asia/Shanghai \
  --restart unless-stopped \
  ghcr.io/<owner>/accounting:latest
```

注意：必须注入 `TZ`（报表/预算的「今天」依赖容器时区）。

### 3. 创建首个用户

```bash
podman exec accounting auth-admin --db /data/auth.db user add \
  --username alice --password '<强密码>' --display-name '爱丽丝'
```

其他命令：`user passwd --username alice --password '<新密码>'`、`user list`。

### 4. HTTPS（公网必需）

session cookie 带 `Secure` 属性，**必须通过 HTTPS 访问**。典型做法：

1. 域名完成 ICP 备案并解析到 ECS
2. 申请免费 DV 证书（阿里云 SSL 证书服务或 acme.sh + Let's Encrypt）
3. Nginx/Caddy 反代 `443 → 127.0.0.1:3000`，HTTP 全部跳转 HTTPS
4. 安全组只放行 443

Caddy 最简配置示例：

```caddy
accounting.example.com {
    reverse_proxy 127.0.0.1:3000
}
```

### 5. 升级

```bash
podman pull ghcr.io/<owner>/accounting:latest
podman stop accounting && podman rm accounting
# 用第 2 步相同命令重新运行（数据在卷中，不受影响）
```

## 常见问题

**Q: ECS 拉取 ghcr.io 很慢或失败**
国内网络访问 GHCR 不稳定。可选：给 podman 配 HTTP 代理（`~/.config/containers/containers.conf` 或环境变量 `HTTPS_PROXY`）；或在本机拉取后 `podman save | ssh | podman load` 离线传输；或自行转推到阿里云 ACR。

**Q: 想用非 root 容器用户（:nonroot 变体）**
本镜像以容器 root 运行，在 rootless podman 下等价于宿主机当前用户，无权限问题。若改用 rootful docker 或 k8s 并切换到 nonroot（uid 65532），需先执行 `chown -R 65532:65532 <数据目录>`（rootless podman 下用 `podman unshare chown -R 65532:65532 ...`）。

**Q: 如何备份**
备份 `~/accounting/data/` 目录即可（my.db + auth.db）。建议在服务停止或低峰期拷贝，或用 `sqlite3 .backup`。

**Q: 本地构建镜像（不走 CI）**
仓库根目录执行 `podman build -t accounting:local .`（需能拉取 node:22-alpine、lukemathwalker/cargo-chef、gcr.io/distroless/static-debian12；gcr.io 国内可用镜像站 `gcr.m.daocloud.io` 替代）。
