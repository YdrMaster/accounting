# accounting

核心库：记账数据模型与算法。零 IO、零异步，可独立测试。位于依赖树根，被 `accounting-sql` / `accounting-service` / `accounting-beancount` 依赖；`accounting-auth` 为独立认证子系统，不依赖本 crate。

## 职责

- 数据模型：`Account`、`Transaction`、`Posting`、`Member`、`Channel`、`Tag`、`Commodity`、`Budget`、`SavingPlan`、`Attachment` 等。
- 复式记账核心：Posting 端点模型——`amount` 正负表资金方向，`cost` / `cost_commodity_id` 成对出现表达多币种双边等式（无独立换汇概念）。
- 纯算法：复式记账验证（`validation`）、余额计算（`balance`）、账户关闭校验、账户树后代聚合（`closure`）、分期期数推断、精度缩放。
- 账户类型：**4 类** `Asset` / `Equity` / `Income` / `Expense`。Asset 关户须各币种余额归零，其余三类无条件关闭。
- 多语言：`rust-i18n` 编译期嵌入 `locales/{en,zh-CN}.yaml`，提供实体名 i18n（见 `entity-names-i18n`）。

## 设计文档

完整的数据类型、核心行为规格与算法约束见 [`../spec/core.md`](../spec/core.md)。账户类型推导机制见 [`../openspec/specs/account-type-resolution/spec.md`](../openspec/specs/account-type-resolution/spec.md)。

## 分层上下文

见根 [`README.md`](../README.md) 的"分层架构"。本 crate 在依赖树根，向上分支为 `accounting-sql`（持久层）、`accounting-service`（业务编排）、`accounting-beancount`（导入导出）；`accounting-auth` 为独立认证岛，不依赖本 crate。
