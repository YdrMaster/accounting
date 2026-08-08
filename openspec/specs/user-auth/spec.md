# user-auth 规格

## Purpose

定义记账系统的用户认证能力：用户名密码登录、可选 TOTP 双因素验证、session 管理、业务 API 认证保护、登录频控、命令行用户管理，以及独立的认证数据库存储。

## Requirements

### Requirement: 用户名密码登录

系统 SHALL 提供用户名 + 密码登录接口（`POST /api/auth/login`）。密码 MUST 以 argon2id 哈希存储，任何情况下 MUST NOT 明文存储或记录日志。登录成功时系统 MUST 创建 session 并通过 `Set-Cookie` 返回会话 cookie（`Secure; HttpOnly; SameSite=Lax; Path=/api`）。登录失败的响应 MUST 统一为 401 +「用户名或密码错误」，MUST NOT 区分"用户不存在"与"密码错误"。登录成功响应 MUST 包含用户的 `display_name` 与 `totp_enabled` 状态。

#### Scenario: 登录成功（未开 TOTP）

- **WHEN** 用户提交正确的用户名和密码，且该用户未开启 TOTP
- **THEN** 系统创建 session，返回 200 与 `Set-Cookie: session=...`（HttpOnly、Secure、SameSite=Lax），响应体包含 `display_name` 与 `totp_enabled: false`

#### Scenario: 密码错误

- **WHEN** 用户提交存在的用户名和错误的密码
- **THEN** 系统返回 401，文案为「用户名或密码错误」

#### Scenario: 用户不存在

- **WHEN** 用户提交不存在的用户名
- **THEN** 系统返回 401，文案与密码错误完全相同

### Requirement: TOTP 双因素验证

系统 SHALL 支持用户可选开启 TOTP（RFC 6238）作为第二因素。已登录用户 MUST 能通过 `POST /api/auth/totp/setup` 获取 otpauth:// URI，并通过 `POST /api/auth/totp/enable` 提交一次有效动态码完成开启；开启时系统 MUST 生成 8 个一次性恢复码（存哈希，用过即作废）。开启 TOTP 的用户登录时，密码校验通过后系统 MUST NOT 直接建立正式 session，而是返回 `require_totp: true` 与 5 分钟有效的 pending_token；`POST /api/auth/login/totp` 验证动态码或恢复码通过后才建立正式 session。TOTP 验证 MUST 允许 ±1 时间步窗口，且 MUST 拒绝同一时间步内已使用过的动态码。

#### Scenario: 开启 TOTP

- **WHEN** 已登录用户调用 setup 获取 URI，再提交 Authenticator 中的有效动态码调用 enable
- **THEN** 系统将用户标记为 `totp_enabled`，返回 8 个一次性恢复码

#### Scenario: 两步登录成功

- **WHEN** 已开启 TOTP 的用户密码登录（返回 pending_token）后，提交有效动态码到 `/api/auth/login/totp`
- **THEN** 系统建立正式 session，返回 200 与会话 cookie

#### Scenario: 动态码错误

- **WHEN** 用户提交错误的动态码
- **THEN** 系统返回 401；连续错误 5 次后 pending_token 作废，需重新密码登录

#### Scenario: 恢复码一次性使用

- **WHEN** 用户使用有效恢复码完成登录后，再次提交同一恢复码
- **THEN** 第二次提交返回 401

### Requirement: Session 管理

系统 SHALL 使用不透明随机 token（256 位）作为 session 凭证，数据库 MUST 只存储 token 的 SHA-256 哈希。Session 有效期 MUST 为 7 天滑动过期（每次认证请求续期）。登出接口（`POST /api/auth/logout`）MUST 删除服务端 session 记录，而非仅依赖清除 cookie。`GET /api/auth/me` MUST 返回当前登录用户信息，未认证时返回 401。

#### Scenario: 有效 session 访问

- **WHEN** 请求携带未过期 session cookie
- **THEN** 系统放行并续期该 session 的过期时间

#### Scenario: 登出后立即失效

- **WHEN** 用户登出后，再次携带同一 cookie 请求
- **THEN** 系统返回 401

#### Scenario: session 过期

- **WHEN** session 超过 7 天未活动
- **THEN** 系统返回 401，要求重新登录

### Requirement: 业务 API 认证保护

除 `/api/auth/*` 外的所有业务 API MUST 要求有效（非 pending）session，未认证请求 MUST 返回 401。认证中间件 MUST 向业务 handler 注入当前用户身份。pending 状态（等待 TOTP）的 session MUST NOT 通过业务 API 认证。

#### Scenario: 未认证访问业务接口

- **WHEN** 请求未携带 cookie 访问任意业务 API
- **THEN** 系统返回 401

#### Scenario: pending session 访问业务接口

- **WHEN** 请求携带处于 pending（等待 TOTP 验证）状态的 session
- **THEN** 系统返回 401

### Requirement: 登录频控

系统 SHALL 对登录接口（`/api/auth/login` 与 `/api/auth/login/totp`）实施频控：同一 IP + 用户名组合每分钟最多 5 次尝试，超限 MUST 返回 429 并携带 `Retry-After` 头。

#### Scenario: 频控触发

- **WHEN** 同一 IP + 用户名在 1 分钟内第 6 次尝试登录
- **THEN** 系统返回 429 与 `Retry-After` 头

### Requirement: auth-admin 用户管理

系统 SHALL 提供 `auth-admin` 命令行工具用于用户管理，至少支持 `user add --username --password`、`user passwd`、`user list`。系统 MUST NOT 在公网 HTTP API 上提供注册或改密接口。

#### Scenario: 创建首个用户

- **WHEN** 管理员执行 `auth-admin user add --username alice --password <pwd>`
- **THEN** 用户在 auth.db 中创建，可用该凭证登录

#### Scenario: 用户名重复

- **WHEN** 管理员创建一个已存在的用户名
- **THEN** 工具报错退出，不覆盖原用户

### Requirement: 独立认证数据库

认证数据 MUST 存储在独立的 `auth.db`（SQLite，sqlx 管理），与账簿数据库物理分离。Schema 变更 MUST 通过 sqlx migration 管理。账簿数据库 schema MUST NOT 因本变更而修改。

#### Scenario: 数据库分离

- **WHEN** 检查部署目录中的数据库文件
- **THEN** auth.db 与账簿 db 为独立文件，删除 auth.db 不影响账簿数据
