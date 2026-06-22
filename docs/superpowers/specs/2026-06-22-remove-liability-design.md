# 移除负债账户类型设计文档

> 目标：从整个仓库中彻底移除 `AccountType::Liability` 及其所有相关引用，使系统只保留 `Asset`、`Equity`、`Income`、`Expense` 四种账户类型。

---

## 关键决策

| 决策 | 结论 |
|------|------|
| 历史数据 | 无需迁移，直接删除所有 `Liability` 账户及分录 |
| `billing_day` / `repayment_day` | 保留为 `Account` 的通用字段，所有类型共享 |
| 资产负债表 | 保留“资产负债表”名称，只显示 `assets` 和 `equity` |
| 关闭规则 | 仅 `Asset` 关闭时要求余额为零，其余类型无条件关闭 |
| 枚举编号 | 重新连续编号：`Asset=1, Equity=2, Income=3, Expense=4` |
| 历史文档 | 仅标注已过时，不主动更新正文 |

---

## 1. Domain 层（`accounting` crate）

### `accounting/src/account_type.rs`

- 删除 `Liability` 枚举值，剩余类型及编号：

  ```rust
  Asset = 1,
  Equity = 2,
  Income = 3,
  Expense = 4,
  ```

- 删除 `is_permanent()` 方法。移除 `Liability` 后该方法仅剩 `Asset` 一种语义，失去区分度，且目前只在测试中使用。
- `close_conditions()` 改为：
  - `Asset` → “余额为零”
  - `Equity` / `Income` / `Expense` → “无限制”
- `display_name()` 删除 `Liability` 分支及对应 i18n key。
- `from_prefix()` 删除 `"liability" | "liabilities" | "负债"` 的解析路径。
- 同步删除涉及 `Liability` 的单元测试。

### `accounting/src/validation.rs`

- `validate_account_close()` 只保留 `Asset` 分支，删除 `Liability` 的 match arm。
- 注释同步更新为“仅 `Asset` 关闭时要求余额为零”。
- 测试用例删除 `Liability` 相关断言。

---

## 2. 数据库层（`accounting-sql` crate）

### `accounting-sql/src/schema.rs`

- `accounts.account_type` 的 `CHECK` 约束从 `BETWEEN 1 AND 5` 改为 `BETWEEN 1 AND 4`。
- 种子数据 `SEED_ACCOUNTS_ROOT_EN` / `SEED_ACCOUNTS_ROOT_ZH` 中：
  - `Equity` 从 `3` 改为 `2`
  - `Income` 从 `4` 改为 `3`
  - `Expenses` 从 `5` 改为 `4`
- 子账户种子中的 `account_type` 编号同步更新。

### `accounting-sql/src/repo/account.rs`

- `map_account()` 删除 `2 => AccountType::Liability`，重新映射：
  - `1 => Asset`
  - `2 => Equity`
  - `3 => Income`
  - `4 => Expense`
  - `_ => Asset`
- 测试中涉及 `Liability` 的用例删除或改为 `Equity`。

### `accounting-sql/src/repo/posting.rs`

- 测试辅助函数中插入 `Income` / `Expense` 账户的 SQL 编号从 `4/5` 改为 `3/4`。
- 涉及 `account_type` 的注释同步更新为新的编号含义。

---

## 3. Service 层（`accounting-service` crate）

### `accounting-service/src/report_service.rs`

- `BalanceSheet` 结构体删除 `liabilities` 字段，只保留 `assets` 和 `equity`。
- `balance_sheet()` 方法删除 `liabilities` 向量及 `AccountType::Liability` 分支。
- `stats_by_tag` / `stats_by_member` / `stats_by_channel` 中按 `account_type` 数值分组的分支从 `4/5` 改为 `3/4`。
- 测试数据删除 `Liability` 样本账户。

### `accounting-service/src/account_service.rs`

- 关闭账户逻辑本身无需改动，`validate_account_close` 会按新规则执行。

---

## 4. API 层（`accounting-api` crate）

### `accounting-api/src/handlers/report.rs`

- `BalanceSheetResponse` 删除 `liabilities` 字段。
- `balance_sheet` handler 删除 `sheet.liabilities` 的映射。

其余 handler（账户、交易等）没有直接硬编码 `Liability`，无需改动。

---

## 5. CLI（`accounting-cli` crate）

### `accounting-cli/src/cmd/mod.rs`

- `AccountTypeArg` 枚举删除 `Liability` 及对应 `From<AccountTypeArg>` 实现。

### `accounting-cli/src/cmd/report.rs`

- `ReportCmd::Bs` 输出循环删除 `bs.liabilities` 块。

### `accounting-cli/README.md`

- 删除所有提到 `Liability` / `负债` 的 CLI 示例和说明。

---

## 6. 前端（`accounting-web`）

- `AccountTree.vue` 当前已有 4 个 tab（Asset / Income / Expense / Equity），无需改动。
- `ReportView.vue` 使用通用 `flattenReport` 渲染，API 少一个 `liabilities` 字段后会自动适配，无需改动。
- 更新 `README.md` 中关于 `Liabilities` 的描述。

---

## 7. 国际化

### `accounting/locales/zh-CN.yaml` / `en.yaml`

- 删除 `account_type_liability` 键。

---

## 8. 文档更新

- `spec/core.md`：删除 `Liability` 枚举值，关闭规则改为仅 `Asset`。
- `spec/service.md`：资产负债表描述改为查询 `Asset / Equity`。
- `spec/refund-reimbursement-design.md`：涉及 `Asset / Liability / Equity` 的描述同步调整为 `Asset / Equity`。
- `README.md`、`accounting-cli/README.md`、`accounting-web/README.md`：删除 `Liability` / `负债` 引用。
- 历史计划文档（`plan/phase1.md`、`phase2.md`、`phase3.md`、`cli-design.md`）以及 `docs/superpowers/plans/`、`docs/superpowers/specs/2026-06-13-account-cards-design.md` 等历史文档：仅在顶部或相关段落标注“本文档包含已废弃的 `Liability` 类型引用，仅供参考”，不修改正文。

---

## 9. 验证

每次提交前必须执行完整验证：

- `cargo fmt`：Rust 代码格式化，确保无未格式化的变更。
- `cargo test`：全 workspace 测试通过。
- `cargo clippy --all-targets`：无新增 error。
- `cd accounting-web && npm run build`：前端构建通过。

> 在实现计划的每个 commit 步骤之前，都需要先运行上述完整验证，确认通过后再提交。

---

## 范围外（明确不做）

- 不开发数据库迁移脚本（已确认无历史数据）。
- 不改 `billing_day` / `repayment_day` 字段的通用性。
- 不改损益表逻辑。
- 不引入“负资产”标记或新的账户子类型。
- 不重新设计“资产 = 权益”的会计等式展示样式。
