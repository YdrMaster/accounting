# user-auth（增量）

## MODIFIED Requirements

### Requirement: Session 管理

系统 SHALL 使用不透明随机 token（256 位）作为 session 凭证，数据库 MUST 只存储 token 的 SHA-256 哈希。Session 有效期 MUST 为 7 天滑动过期：认证请求放行时，仅当 session 剩余有效期不足 TTL/2 时系统才续期至完整 TTL（滑动续期节流，降低写放大）；剩余充足时放行但不改写过期时间。登出接口（`POST /api/auth/logout`）MUST 删除服务端 session 记录，而非仅依赖清除 cookie。`GET /api/auth/me` MUST 返回当前登录用户信息，未认证时返回 401。

#### Scenario: 有效 session 访问

- **WHEN** 请求携带未过期 session cookie
- **THEN** 系统放行；剩余有效期不足 TTL/2 时续期至完整 7 天，剩余充足时过期时间保持不变

#### Scenario: 续期节流边界

- **WHEN** 同一 session 先后发出两次请求，第一次时剩余有效期大于 TTL/2，第二次时剩余不足 TTL/2
- **THEN** 第一次请求后 session 过期时间不变，第二次请求后被续期至完整 7 天

#### Scenario: 登出后立即失效

- **WHEN** 用户登出后，再次携带同一 cookie 请求
- **THEN** 系统返回 401

#### Scenario: session 过期

- **WHEN** session 超过 7 天未获得续期
- **THEN** 系统返回 401，要求重新登录
