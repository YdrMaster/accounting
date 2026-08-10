# accounting

核心库：记账数据模型与算法。零 IO、零异步，可独立测试。位于分层架构最底层，被 `accounting-sql` / `accounting-service` / `accounting-api` / `accounting-cli` / `accounting-beancount` / `accounting-auth` 依赖。

## 职责

- 数据模型：`Account`、`Transaction`、`Posting`、`Member`、`Channel`、`Tag`、`Commodity`、`Budget`、`SavingPlan`、`Attachment` 等。
- 纯算法：复式记账验证（`validation`）、余额计算（`balance`）、账户关闭校验、闭包表计算（`closure`）、分期期数推断、精度缩放。
- 账户类型：**4 类** `Asset` / `Equity` / `Income` / `Expense`，由树根节点名推导（不存储在 `Account` 行上）。见活规格 `account-type-resolution`。
- 多语言：`rust-i18n` 编译期嵌入 `locales/{en,zh-CN}.yaml`，提供实体名 i18n（见 `entity-names-i18n`）。

## 设计文档

完整的数据类型、核心行为规格与算法约束见 [`../spec/core.md`](../spec/core.md)。账户类型推导机制的活规格见 [`../openspec/specs/account-type-resolution/spec.md`](../openspec/specs/account-type-resolution/spec.md)。

## 分层上下文

```
accounting          ← 本 crate：模型 + 算法（零 IO）
    ↑
accounting-sql      ← Repository + SQLite
    ↑
accounting-service  ← Service + 事务编排
    ↑
accounting-cli / accounting-api / accounting-beancount / accounting-auth
```

见根 [`README.md`](../README.md) 的"分层架构"与"各 crate 文档"。
