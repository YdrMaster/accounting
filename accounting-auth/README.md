# accounting-auth

独立认证子系统（单 crate 垂直切片）。提供用户名 + 密码登录、可选 TOTP 双因素、session cookie 与用户管理，供 [`accounting-api`](../accounting-api) 对公网部署做认证。

## 职责

- **独立数据库**：认证数据存于单独的 `auth.db`（与账簿 `my.db` 物理分离），由 sqlx migration 管理。
- **导入对外接口**（见 `src/lib.rs`）：
  - `AuthState` / `init`：初始化独立 auth.db（sqlx + SQLite）。
  - `router`：认证 HTTP API（`/api/auth/*`）。
  - `require_auth`：业务路由认证中间件，注入 `CurrentUser`。
- **内部划分**：`db`（sqlite 模式与表）、`service`（密码 argon2 哈希、TOTP RFC 6238、session、限流、恢复码）、`api`（handlers + middleware）。
- **用户管理工具** `auth-admin`（`src/bin/auth-admin.rs`）：公网部署不开放注册/改密接口，用本工具离线管理用户。

## 部署注意

- session cookie 带 `Secure` 属性，**必须经 HTTPS 访问**。
- 大陆 ECS 需域名 ICP 备案。

## 用户管理

```bash
auth-admin --db auth.db user add --username alice --password '<pwd>' --display-name '爱丽丝'
auth-admin --db auth.db user passwd --username alice --password '<new-pwd>'
auth-admin --db auth.db user list
```

## 设计文档

设计决策（单 crate 垂直切片的理由、TOTP 参数、安全决策、限流、session 策略、D1–D8、Risks / Trade-offs）见 [`add-user-auth`](../openspec/changes/archive/2026-08-08-add-user-auth/design.md)。规格见 `user-auth`、`web-auth-ui`。

## 分层上下文

见根 [`README.md`](../README.md) 的"认证（公网部署）"。独立认证子系统，不依赖 `accounting` / `accounting-sql`，自有 `auth.db`，对 `accounting-api` 提供 `require_auth`。
