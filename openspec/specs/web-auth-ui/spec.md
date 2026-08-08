# web-auth-ui 规格

## Purpose

定义记账系统 Web 前端的认证交互能力：登录页、TOTP 两步输入、路由守卫与 401 拦截、登出入口。前端通过会话 cookie 与后端认证接口（user-auth 能力）配合，保证未认证用户无法访问业务页面。

## Requirements

### Requirement: 登录页

前端 SHALL 提供登录页，包含用户名、密码输入与提交。登录成功后 MUST 跳转至用户原本要访问的页面（或首页）。登录失败 MUST 展示服务端返回的错误文案，且 MUST NOT 在前端区分"用户不存在"与"密码错误"。

#### Scenario: 登录成功跳转

- **WHEN** 用户在登录页提交正确凭证
- **THEN** 前端保存登录状态，跳转至目标页面

#### Scenario: 登录失败提示

- **WHEN** 登录接口返回 401
- **THEN** 登录页展示统一错误文案，输入框保留用户名、清空密码

### Requirement: TOTP 两步输入

当登录响应为 `require_totp: true` 时，前端 MUST 切换至动态码输入界面（同一登录页内，不跳转），支持提交 6 位动态码或恢复码。动态码错误 MUST 提示并可重试；pending_token 过期 MUST 返回密码输入界面。

#### Scenario: 切换两步输入

- **WHEN** 登录接口返回 `require_totp: true`
- **THEN** 登录页隐藏密码表单，显示动态码输入框

#### Scenario: pending 过期

- **WHEN** pending_token 过期后提交动态码，接口返回 401
- **THEN** 前端提示重新登录并返回密码输入界面

### Requirement: 路由守卫与 401 拦截

前端 SHALL 对所有页面实施路由守卫：未登录访问任何页面 MUST 重定向至登录页并记录目标路径。api client MUST 拦截任意接口的 401 响应，清除本地登录状态并跳转登录页。登录/TOTP 接口本身的 401 MUST NOT 触发跳转。

#### Scenario: 未登录访问页面

- **WHEN** 未登录用户直接访问 `/transactions`
- **THEN** 前端重定向至登录页，登录成功后回到 `/transactions`

#### Scenario: 会话过期

- **WHEN** 已登录页面上的任一业务请求返回 401
- **THEN** 前端清除登录状态并跳转登录页

### Requirement: 登出

前端 SHALL 提供登出入口，调用 `POST /api/auth/logout` 后清除本地登录状态并跳转登录页。

#### Scenario: 登出

- **WHEN** 用户点击登出
- **THEN** 系统调用登出接口，前端跳转登录页，后续业务请求不再携带旧会话
