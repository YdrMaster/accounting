# accounting-api

axum HTTP 服务入口。位于分层架构顶层，组合 [`accounting-service`](../accounting-service) 业务层、[`accounting-sql`](../accounting-sql) 数据层、[`accounting-auth`](../accounting-auth) 认证，并托管 [`accounting-web`](../accounting-web) 前端静态资源。

## 职责

- `axum` 路由：业务 API（账户、交易、分录、预算、攒钱计划、渠道、成员、标签、商品、映射、报表、导入等）+ `/api/auth/*` 认证 + `/api/health`。
- DTO 层（`dto.rs`）与 handler 层（`handlers/`）：请求/响应序列化、错误映射、本地化。
- 认证中间件：所有业务 API（`/api/auth/*` 与 `/api/health` 除外）要求有效 session cookie，由 `accounting-auth` 的 `require_auth` 注入 `CurrentUser`。
- 静态前端托管：`--static-dir` 指向 `accounting-web` 构建产物。

## 启动参数

| 参数 | 说明 | 默认 |
|------|------|------|
| `--db` | 账簿数据库路径 | `my.db` |
| `--auth-db` | 认证数据库路径 | `auth.db` |
| `--port` | 监听端口 | `3000` |
| `--static-dir` | 前端静态资源目录 | — |
| `--lang` | 默认界面语言 | `zh-CN` |

## 部署

公网部署需经 HTTPS（session cookie 带 `Secure`）。容器部署见 [`../docs/deployment.md`](../docs/deployment.md)（见 `container-deployment`）。

## 相关规格

- 认证：规格 `user-auth`、`web-auth-ui`；设计决策见 [`add-user-auth`](../openspec/changes/archive/2026-08-08-add-user-auth/design.md)。
- 业务 API：`transaction-api`、`budget-api`、`saving-plan-api`、`member-api`、`tag-api`、`channel-api`、`mapping-api`、`transaction-summary-api` 等。
- 容器与数据刷新：`container-deployment`、`data-refresh`。

## 分层上下文

见根 [`README.md`](../README.md) 的"Web 用法"、"认证（公网部署）"、"容器部署"与"各 crate 文档"。
