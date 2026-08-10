# Design: remove-liability

## Context

回溯归档 commit `4b9fa057` ~2026-06-22。传统复式记账 5 类账户中，本系统作为个人/家庭记账从未真正使用 Liability——种子无负债根账户、无信用卡/贷款级联、BalanceSheet 的 `liabilities` 向量恒空。本设计决定彻底移除 Liability 类型。关联：`audit-time-fields` 同期，`simplify-data-model` 阶段 4 将类型改为根节点推导（本变更后 4 类更利推导）。

> 注：本归档补录"为何去掉 Liability"的决策。活规格 `account-type-resolution` 描述类型推导的**现状**，但未记录"曾存在第 5 类并主动移除"这一历史；该历史仅存于此。

## Goals / Non-Goals

**Goals:**

- `AccountType` 缩为 4 类，消除零使用量的 Liability 分支及其在 enum/校验/i18n/报表/DTO 中的扩散代码。
- 重新连续编号使数值稳定、阅读清晰。

**Non-Goals:**

- 不保留 Liability 作"未来扩展"占位（见 D1 备选）。
- 不改 `billing_day`/`repayment_day` 通用性（这些是账户通用字段，非负债专属）。
- 不引入"负资产"标记或子类型。
- 不改损益表逻辑。
- 不开发数据库迁移脚本（无历史数据）。

## Decisions

### D1: 彻底移除而非保留作占位

删除 `Liability` 枚举值、所有相关分支与 i18n key，重新连续编号 `Asset=1, Equity=2, Income=3, Expense=4`。

**备选**（否决）：保留 `Liability` 枚举以备未来信用卡/贷款功能。否决理由：

1. **零使用≠待用**：种子无负债根账户意味着即便保留枚举也无法级联创建负债账户，类型存在但路径不存在，是"半成品占位"。
2. **维护成本实证**：移除时确实需触碰 enum、`close_conditions`、`is_permanent`、`from_prefix`、`display_name`、i18n、schema CHECK、种子、repo 映射、报表分支、API DTO、CLI——证明 5 类模型的扩散成本，保留即持续承担。
3. **YAGNI**：真要做负债功能时，连根账户带类型一起加，比现在维护空壳分支更清晰。

### D2: 删除 `is_permanent()` 而非保留

移除 Liability 后 `is_permanent()` 只剩 `Asset` 一种返回 `true` 的语义，且当前调用点仅在测试中。保留失去区分度的方法违背内聚，故连方法带测试一并删除。

### D3: 保留"资产负债表"名称，只显示资产+权益

BalanceSheet 结构体删 `liabilities` 字段，仅保留 `assets` 与 `equity`；但**沿用**"资产负债表 / Balance Sheet"名称不改名。

**理由**：复式记账中"资产负债表"是约定俗成的报表名（资产 = 负债 + 权益），即使本系统无负债，名称仍是用户认知锚点。改名为"资产权益表"增加解释成本且有损专业性。空负债隐含"负债=0"，会计等式 `资产 = 权益` 仍成立。

### D4: 关闭规则——仅 Asset 要求余额为零

`close_conditions()` 改为：`Asset` → 余额为零；`Equity`/`Income`/`Expense` → 无限制。

**理由**：资产账户（现金/银行卡）关闭前须结清，否则余额丢失；权益/收入/支出是分类账户，关闭即停用，无需余额校验。原 Liability 余额校验本就因零使用而无意义，移除顺理成章。

### D5: 种子编号重排而非保留空位

种子 `SEED_ACCOUNTS_ROOT_EN/ZH` 把 Equity 从 3→2、Income 从 4→3、Expense 从 5→4，CHECK 约束 `BETWEEN 1 AND 5` 改 `BETWEEN 1 AND 4`。子账户种子同步。

**备选**（否决）：保留 1/3/4/5 留空 2。否决：编号空位是技术债，未来读者会困惑"为什么没有 2"，且约束上限留 5 等于变相保留 Liability 的 schema 痕迹。重排彻底清除。

### D6: 历史文档仅标注不更新正文

对 `plan/phase1-3`、`cli-design`、`docs/superpowers/plans/`、已归档 superpowers specs 中的 Liability 引用，仅在顶部加注"已废弃的 Liability 引用，仅供参考"，不改正文。

**理由**：历史文档是时间快照，改写正文会丢失"当时是这样设计的"信息（含为何曾用 5 类）；标注已废足以防误读。此约定也指导了本次清理对历史文档的处置（见 Part 4）。

## Risks / Trade-offs

- **若未来真需负债功能**：需回加 Liability 枚举+种子根账户+级联创建，工作量等于"新加一个类型"而非"翻出一个旧类型"。可接受——彼时需求会驱动完整设计，而非沿用残留。
- **BalanceSheet 名称与内容不完全对应**：名为"资产负债表"却无负债段。由 D3 解释，属可接受的约定俗成偏差。
- **直接删除存量 Liability 账户**：无迁移脚本，旧库若有负债数据会丢失。已确认无生产数据。

## Migration Plan

无数据迁移。Domain 删 Liability 分支与 `is_permanent`；Schema CHECK 改 4 + 种子编号重排；repo `map_account` 重映射；报表删 `liabilities`；API/CLI/前端去引用；i18n 删 key；`spec/core.md`/`spec/service.md` 同步；历史文档加注。

## Open Questions

- 无（回溯归档，决策已定型）。
