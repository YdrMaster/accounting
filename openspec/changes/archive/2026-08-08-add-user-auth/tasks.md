# Tasks: add-user-auth

## 1. accounting-auth crate 骨架

- [x] 1.1 在 workspace 根 Cargo.toml 注册新 crate `accounting-auth`，建立目录结构（`src/lib.rs`、`src/db/`、`src/service/`、`src/api/`、`src/bin/auth-admin.rs`）
- [x] 1.2 添加依赖：sqlx（sqlite + runtime-tokio-rustls）、argon2、rand、sha2、axum、tower、tower-http、thiserror、tracing、serde、chrono、totp（totp-lite 或等价）
- [x] 1.3 定义 `AuthError`（thiserror）与 api 层 HTTP 错误映射（401/429/500，文案防枚举、不泄漏内部细节）

## 2. 数据层（auth.db）

- [x] 2.1 编写 sqlx migration：`users` 表（id、username 唯一、password_hash、display_name、totp_secret、totp_enabled、recovery_codes JSON、created_at）
- [x] 2.2 编写 sqlx migration：`sessions` 表（id、token_hash 唯一、user_id、pending、expires_at、last_totp_step、created_at）
- [x] 2.3 实现 `db` 模块：连接池初始化（`init(db_path)`）+ users/sessions 的 CRUD 函数
- [x] 2.4 db 集成测试：内存 SQLite（`:memory:`）跑 migration，覆盖 CRUD 与唯一约束（username 重复、token_hash 冲突）

## 3. 业务逻辑（service）

- [x] 3.1 密码哈希/校验（argon2id 默认参数）+ 单测
- [x] 3.2 session token 生成（256 位随机）与 SHA-256 哈希、7 天滑动过期判定 + 单测
- [x] 3.3 TOTP：secret 生成、otpauth:// URI 生成（issuer 可配置常量）、±1 窗口验证、时间步防重放；用 RFC 6238 已知向量做单测
- [x] 3.4 恢复码：生成 8 个、存哈希、用过即作废 + 单测
- [x] 3.5 频控：IP + 用户名双维度滑动窗口（每分钟 5 次），内存实现、接口可替换 + 单测

## 4. HTTP API（api 模块）

- [x] 4.1 `POST /api/auth/login`：密码校验 → 建 session 发 cookie；TOTP 用户返回 `require_totp` + pending_token；统一 401 文案；接频控
- [x] 4.2 `POST /api/auth/login/totp`：动态码/恢复码验证 → pending session 转正发 cookie；连错 5 次作废 pending
- [x] 4.3 `POST /api/auth/logout`（删 DB 记录）、`GET /api/auth/me`
- [x] 4.4 `POST /api/auth/totp/setup`、`POST /api/auth/totp/enable`（需已登录）
- [x] 4.5 认证中间件：cookie → 哈希查 session → 续期 → 注入 CurrentUser；拒绝 pending session
- [x] 4.6 端到端测试（tower oneshot 打 Router）：登录全流程、防枚举 401 一致性、TOTP 两步、恢复码一次性、未认证/pending 401、频控 429、登出即失效

## 5. auth-admin 工具

- [x] 5.1 `auth-admin user add --username --password`（用户名重复报错退出）、`user passwd`、`user list`
- [x] 5.2 手动验证：建用户 → 启动服务 → curl 登录/登出全流程

## 6. accounting-api 接入

- [x] 6.1 main.rs：启动时 `auth::init(auth_db_path)`（新增 `--auth-db` 参数，默认 `auth.db`），`router.merge(auth::router())`
- [x] 6.2 给全部业务路由套 `auth::middleware`（`/api/auth/*` 除外）
- [x] 6.3 验证 accounting-cli 直连 service 层不受影响

## 7. 前端（accounting-web）

- [x] 7.1 登录视图：用户名+密码表单、TOTP 两步输入切换、错误展示（沿用现有组件风格与 i18n）
- [x] 7.2 auth store（pinia）：登录状态、display_name、`GET /api/auth/me` 初始化
- [x] 7.3 路由守卫：未登录跳登录页并记录目标路径，登录后回跳
- [x] 7.4 api client 401 拦截：业务请求 401 清状态跳登录页；auth 接口 401 不跳转
- [x] 7.5 登出入口（导航/设置处）+ TOTP 绑定界面（setup 二维码渲染 + enable + 恢复码展示）
- [x] 7.6 vitest：登录表单、守卫跳转、TOTP 界面切换、401 拦截

## 8. 验证与收尾

- [x] 8.1 `cargo test` 全绿、`cargo clippy` 无警告
- [x] 8.2 前端 `vitest run`、`lint`、`build` 通过
- [x] 8.3 手工验收：auth-admin 建用户 → 启动 api → 浏览器走完登录 / TOTP 绑定 / 两步登录 / 登出全流程
- [x] 8.4 更新 AGENTS.md / README（新增 crate、auth.db、auth-admin、部署需 HTTPS 的说明）
