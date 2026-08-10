# Proposal: remove-liability

> 回溯归档：本变更所述设计已实现（commit `4b9fa057` ~2026-06-22）。活规格 `account-type-resolution` 描述了类型推导的**现在**，但本归档补录"为什么从一开始就不保留 Liability"这一**决策来源**。`--skip-specs` 归档。

## Why

复式记账传统含 5 类账户（Asset / Liability / Equity / Income / Expense），但本项目作为个人/家庭记账系统，从未实际使用 `Liability`（负债）：

- 种子数据无 Liability 根账户，无信用卡/贷款等负债账户的级联创建路径。
- `billing_day` / `repayment_day` 通用化后并不依赖 Liability 类型。
- 资产负债表（BalanceSheet）保留负债向量却始终为空，是死代码分支。

5 类模型的维护成本（enum 分支、`close_conditions` 分支、i18n key、API 字段）与零使用量不匹配。移除 Liability 简化类型空间，使"账户类型由树根推导"（见 `simplify-data-model` 阶段 4）的不变量更自然——去掉一种类型即去掉一种推导分支。

## What Changes

- **枚举缩到 4 类**：`AccountType` 删除 `Liability`，重新连续编号 `Asset=1, Equity=2, Income=3, Expense=4`。
- **删除 `is_permanent()`**：移除 Liability 后该方法只剩 Asset 一种语义，失去区分度。
- **关闭规则简化**：`close_conditions()` 仅 `Asset` 要求余额为零，其余三类无条件关闭。
- **`from_prefix()` / `display_name()` / i18n**：删除 Liability 解析路径、显示名、`account_type_liability` 键。
- **`validation.rs`**：`validate_account_close()` 删 Liability 的 match arm。
- **Schema**：`accounts.account_type` CHECK 从 `BETWEEN 1 AND 5` 改 `BETWEEN 1 AND 4`；种子数据编号重排。
- **BalanceSheet**：删 `liabilities` 字段，只保留 `assets` 和 `equity`；保留"资产负债表"名称（见 D3）。
- **API/CLI/前端**：删除 Liability 相关 DTO 字段、命令变体、文案；前端原本就只有 4 个 tab 的事实得以坐实。

## Capabilities

### New Capabilities

（无——类型空间缩减是内部演进。）

### Modified Capabilities

（无——活规格 `account-type-resolution` 已描述类型推导的现状，本归档只补决策来源，不改活规格。）

## Impact

- `accounting/src/account_type.rs`：删 `Liability`、`is_permanent()`、`from_prefix`/`display_name` 的 Liability 分支。
- `accounting/src/validation.rs`：`validate_account_close` 去 Liability arm。
- `accounting-sql/src/schema.rs`：CHECK 改 4；种子编号重排（Equity 3→2、Income 4→3、Expense 5→4）。
- `accounting-sql/src/repo/account.rs`：`map_account` 类型映射重编。
- `accounting-service/src/report_service.rs`：`BalanceSheet` 删 `liabilities`；统计分支编号调整。
- `accounting-api`：`BalanceSheetResponse` 删 `liabilities`。
- `accounting-cli`：`AccountTypeArg` 删 Liability，`bs` 输出去负债块。
- `accounting-web`、i18n、`spec/core.md`、`spec/service.md`：去 Liability 引用。
- 兼容性：直接删除所有 Liability 账户及分录（已确认无历史数据）。
