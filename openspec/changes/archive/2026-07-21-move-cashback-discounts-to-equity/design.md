# Design: move-cashback-discounts-to-equity

## Context

种子数据在 `accounting-sql/src/schema.rs` 的 `insert_seed_data` 中定义，`child_specs` 以 `(英文名, 中文名, 父账户 id)` 声明 6 个系统子账户。当前 `Cashback` 挂在 `Assets` 下、`Discounts` 挂在 `Expenses` 下。

关键现状：

- 资产负债表（`accounting-service/src/report/balance_sheet.rs`）通过 `posting_sum_all_assets` 仅统计 Assets 根下的账户；现金流报表（`report/cash_flow.rs`）按根账户名分类（`root_name == "Assets"`）
- 账户关闭规则按类型区分：Asset 关闭要求各 commodity 余额为 0，Equity / Income / Expense 无条件关闭（`spec/core.md`）
- `reparent` API（`accounting-service/src/account_service.rs`）拒绝系统内置账户，用户无法自行移动这两个账户
- 项目处于开发阶段，已有「不兼容存量数据库」的既定原则（见 `builtin-data-english-storage` 规格），种子结构变更无需迁移

## Goals / Non-Goals

**Goals:**

- `Cashback`（返现）、`Discounts`（折扣）归入 `Equity` 根，种子树变为 4 根 + 6 子：`Equity:OpeningBalances`、`Equity:Cashback`、`Equity:Discounts`、`Expenses:Fees`、`Expenses:InstallmentFees`、`Assets:Cash`
- 返现/折扣余额自动退出资产负债表、现金流报表与收支统计（根账户归属变化的自然结果，不改报表代码）
- 叶子名称不变（英文系统名与中文名均不变），仅父账户变化

**Non-Goals:**

- 不提供存量数据库迁移；旧库需导出 → 重建 → 导入
- 不新增「收入：红包」内置账户——可自由支配的返利由用户按需自建收入账户，不属于内置数据
- 不改动报表、导入、关闭规则等任何业务逻辑代码
- 不改动 `exclude-from-income-statement` 标签的既有语义（它是逐笔交易的排除手段，与本变更的结构性排除互补）

## Decisions

### D1: 直接修改种子定义，而非提供迁移

`seeded_account_id` 按 `(系统英文名, parent_id)` 幂等查找；若保留旧账户并新增，会导致旧库出现重复账户。开发阶段直接改 `child_specs` 中两行的父引用为 `equity_id`，旧库重建。这与既有「不兼容存量数据库」原则一致，迁移代码的收益为零。

### D2: 移到 Equity 而非 Income，也不使用标签方案

- 语义：返现是伴随消费、限用途的受限返利；折扣是商家让利。两者都不是用户的收入或支出，而是直接计入净资产的调整项，与 `Equity:OpeningBalances` 同构
- 报表效果：Income 根仍会进入收支统计（违背目标）；Equity 根天然不进收支统计、不进本系统的资产负债表
- 备选方案「`exclude-from-income-statement` 标签」被否决：标签是逐笔手工操作，且无法解决 `Assets:Cashback` 扭曲资产负债表的问题；移账户是结构性的一劳永逸

### D3: 接受关闭规则的副作用

两个账户从 Asset 变为 Equity 后，关闭时不再要求余额为 0。对返利/折扣账户而言这反而合理——它们的余额是累计调整额，没有「清零才可关闭」的现实约束。无需为此引入特殊逻辑。

### D4: 收支统计口径收窄为「仅 Assets 根」，排除 Equity

实现验证时发现：收支汇总（`posting_daily_summary` 及 `transaction-summary-api` 规格）的口径是「资产类 = Assets + Equity 根分录」，导致 Assets↔Equity 交易（期初余额、以及移动后的返现/折扣）在收支统计中毛额双计（income 与 expense 各虚增一笔，净额为 0）。这与本变更「返现/折扣不进收支统计」的目标冲突。

决策：口径收窄为仅 Assets 根分录。语义依据——权益是「无因的收入」：它的功能是维持复式记账平衡（资产初始值不能凭空产生，须以权益减记配对），从权益生成的账目可抵扣支出但不导致资产减少。资产↔权益划转只体现资产侧变动（正为无因收入、负为无因支出），权益侧不重复计入。备选「维持 Assets+Equity 毛额双计」被否决：虚增收支总额，与目标直接冲突。

影响面仅一处实现：`accounting-sql/src/repo/posting.rs` 的 `posting_daily_summary` 查询（`GET /api/reports/summary` 端点已在历史变更中移除，无代码）。

## Risks / Trade-offs

- [旧库种子与新代码不一致：旧库已有 `Assets:Cashback`，新种子会另建 `Equity:Cashback`，产生重复账户] → 明确声明不兼容存量数据库，发布说明给出导出 → 重建 → 导入路径（沿用既有原则）
- [按全名硬编码引用这两个账户的代码/测试失效] → 全仓检索确认仅 `schema.rs`、seed 相关测试与文档引用；实现时同步更新
- [用户已习惯在支出统计中看到折扣抵减] → 属于目标行为本身；需要折扣可见性时可查看 Equity:Discounts 账户余额
