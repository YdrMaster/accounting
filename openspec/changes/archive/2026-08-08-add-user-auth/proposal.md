# Proposal: add-user-auth

## Why

项目要部署到公网（阿里云 ECS），当前 accounting-api 完全没有任何认证机制，所有 API 接口裸奔。需要先落地一个面向自用场景的登录系统，同时架构上为将来的产品化（多租户、第三方 IdP、短信登录）预留可替换的边界。

## What Changes

- 新增独立 crate `accounting-auth`，包含 db（sqlx + 独立 auth.db）、业务逻辑（密码校验、TOTP、session 签发）、axum Router 与认证中间件、以及 `auth-admin` 管理工具。
- 登录方式：用户名 + 密码（argon2id），可选开启 TOTP 双因素验证（RFC 6238），附一次性恢复码。
- session 采用不透明随机 token + httpOnly cookie（7 天滑动过期），DB 只存哈希。
- accounting-api 的 main 入口 merge auth router，并给全部业务路由套上认证中间件（**BREAKING**：此后所有 `/api/*` 业务接口要求有效 session，未认证返回 401）。
- accounting-web 新增登录页、路由守卫、api client 的 401 拦截。
- 不开放注册接口；首个用户由 `auth-admin user add` 创建。
- 暂不做：短信验证码登录、扫码登录、开放注册、用户与账簿/租户的关联。

## Capabilities

### New Capabilities

- `user-auth`: 用户认证——用户名密码登录、TOTP 双因素、session 管理（签发/校验/续期/登出）、登录频控、auth-admin 用户管理。
- `web-auth-ui`: 前端认证界面——登录页、TOTP 两步输入、路由守卫与 401 处理。

### Modified Capabilities

（无）

## Impact

- **新增 crate**：`accounting-auth`（sqlx、argon2、totp、rand、axum、tower、thiserror、tracing 依赖）。
- **accounting-api**：`src/main.rs` 增加 router merge 与中间件挂载（约两行核心改动）；其余代码不动。
- **accounting-web**：新增登录视图、auth store、路由守卫；api client 增加 401 拦截。
- **数据库**：新增独立文件 `auth.db`（与账簿 my.db 物理分离），含 `users`、`sessions` 两张表，sqlx migrate 管理。
- **部署**：要求 HTTPS（备案域名 + 证书），cookie `Secure` 依赖部署层保证。
- **预留**：认证子系统可整体拆分为独立服务或替换为 OIDC IdP，业务代码不感知；短信登录/多租户映射在未来变更中引入。
