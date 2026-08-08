# Design: add-user-auth

## Context

accounting 项目为 Rust workspace（accounting / accounting-sql / accounting-service / accounting-api / accounting-cli + Vue 前端 accounting-web）。accounting-api 是 axum HTTP 服务，托管 Vue dist 静态文件并暴露 `/api/*`，**当前完全没有认证**。目标部署形态：阿里云单台 ECS，域名 + ICP 备案 + HTTPS，自用为主（自己 + 家人），预留产品化。

用户明确的架构约束：

- 认证是独立垂直系统：**独立数据库文件**（auth.db，与账簿 my.db 物理分离），与现有代码**只在 web 层耦合**（共享 main 入口）。
- **单一 crate**：`accounting-auth` 一个 crate 内含 sqlx 数据层、业务逻辑、axum api，不再拆 core/sql/api 三层（与主项目 rusqlite 分层不同，是有意为之）。
- 登录方式**只做用户名 + 密码**，短信验证码暂不做；TOTP 双因素保留。

## Goals / Non-Goals

**Goals:**

- 所有业务 API 必须有有效 session 才能访问（401 防线）。
- 用户名 + argon2id 密码登录，可选 TOTP 第二步（RFC 6238），附一次性恢复码。
- session 为不透明 token + httpOnly cookie，DB 存哈希，7 天滑动过期。
- 认证子系统可整体拆走（独立服务）或替换（OIDC IdP），业务代码不感知。
- 登录接口频控，防暴力破解与用户名枚举。

**Non-Goals:**

- 短信验证码登录、扫码登录、开放注册、找回密码（改密走 auth-admin）。
- 用户与账簿数据/租户的关联（自用阶段登录只守门）。
- RBAC / 多角色权限。
- Redis 等外部会话存储（单进程内存频控即可）。

## Decisions

### D1: 单 crate 垂直切片，web 层唯一耦合

`accounting-auth` 内部按 `db` / `service` / `api` / `bin(auth-admin)` 模块划分，对外只暴露 `init(db_path)`、`router()`、`middleware`。accounting-api 的 main 只做：`router.merge(auth::router())` + 业务路由套 `auth::middleware`。

- 备选 A（写进 accounting-api）：最快但认证与业务混杂，产品化要重写，否决。
- 备选 B（独立 IdP/OIDC，如 Keycloak）：行业标准但对自用过重（多一个服务要运维），否决。
- 备选 C（镜像主项目分三个 crate）：被用户否决——过度拆分，单 crate 足够。

### D2: sqlx + 独立 SQLite 文件，不沿用主项目 rusqlite

auth.db 与 my.db 分离，schema 迁移用 sqlx migrate，HTTP 侧代码用连接池更顺手。账簿侧继续 rusqlite 不动。将来拆成独立服务时 auth.db 直接随服务走，账簿零影响。

### D3: 不透明 session token + cookie，不用 JWT

256 位随机 token（`rand`），cookie 存原文、DB 存 SHA-256 哈希；登出/改密即删 DB 记录立即失效。JWT 无法服务端撤销，对"丢了手机要立刻踢掉会话"的场景不如服务端 session 直接；单体部署也不需要 JWT 的无状态优势。

Cookie 属性：`Secure; HttpOnly; SameSite=Lax; Path=/api`，7 天滑动过期（每次请求续期）。

### D4: argon2id 密码哈希

`argon2` crate 默认参数。不用 bcrypt（有 72 字节截断坑）、不用快哈希。

### D5: TOTP 作为唯一 2FA，附恢复码

RFC 6238，验证窗口 ±1 步防时钟漂移，记录上次时间步防重放。secret 160 位随机，DB 明文存（自用单库可接受，文档注明产品化需加密）。8 个一次性恢复码（存哈希、用过作废）解决丢手机问题。

备选：短信验证码作为 2FA——已被用户砍掉（阿里云短信个人资质可办但本期不需要）。

### D6: 两步登录用 pending session 而非独立状态表

密码校验通过但用户开了 TOTP 时，建一条 `pending=true` 的 session 记录（5 分钟过期），返回 pending_token；`/api/auth/login/totp` 验证通过后将该记录转正。中间件只放行 `pending=false` 的 session。避免引入第三张表。

### D7: 频控用进程内内存计数器

登录接口按 IP + 用户名双维度滑动窗口（每分钟 5 次），429 + `Retry-After`。单进程自用不需要 Redis；多实例部署时产品化阶段再换。

### D8: 不开放注册，auth-admin 管理用户

`accounting-auth` 带 `auth-admin` bin：`user add --username --password`、`user passwd`、`user list`。公网无注册接口，攻击面最小。

## Risks / Trade-offs

- [cookie `Secure` 在 HTTP 下不生效，本地/内网调试登录会失败] → 部署文档写明必须 HTTPS；dev 模式 vite proxy 同源转发，本地 http://localhost 浏览器会放行 Secure 限制（localhost 例外），不受影响。
- [TOTP secret 明文存库] → 自用单库可接受；设计文档与代码注释注明产品化时需加密（KMS 或 envelope encryption）。
- [内存频控重启清零、且无法多实例共享] → 自用可接受；service 层把频控抽象成可替换实现，产品化换 Redis。
- [单 crate 与主项目分层惯例不一致] → 有意决策（用户明确指示），README 注明理由，避免后来者"按惯例"拆分。
- [所有 API 加认证是 BREAKING，现有前端/CLI 直连会 401] → 前端同期加登录页与守卫；accounting-cli 为本地工具直连 service 层，不经 HTTP，不受影响。
- [session 滑动过期续期写库增加写放大] → 每请求一次 UPDATE，SQLite 自用量级无压力。

## Migration Plan

1. 新增 crate 与 auth.db migration，全部测试通过。
2. accounting-api merge router + 中间件，同步上线前端登录页（同一版本发布，避免旧前端裸奔 401）。
3. 部署：ECS 上跑 `auth-admin user add` 建首个用户 → 配 HTTPS 反代/证书 → 启动。
4. 回滚：git 回退到加中间件之前的版本即可；auth.db 独立文件，删除无任何副作用。

## Open Questions

- TOTP 绑定流程中 otpauth:// URI 的 issuer 名称（产品名）待定，实现时用可配置常量。
- 是否需要在登录响应中返回 display_name 供前端显示（倾向是，规格中明确）。
