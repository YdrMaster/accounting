# cash-flow-report

## Purpose

资金流量表——按财务周期统计 Income 与 Expenses 根下各层级账户在指定周期内的收支汇总（净额绝对值）。该报表帮助用户直观了解周期内的收支构成，排除不计预算的标签分录，为现金流分析提供数据支撑。

## Requirements

### Requirement: 资金流量表输入参数
资金流量表 SHALL 接受以下输入：
- `date: NaiveDate`：确定具体周期的日期
- `period: FinancePeriod`：周期类型

#### Scenario: 查询月度资金流量
- **WHEN** 调用 `cash_flow_report(2026-06-15, FinancePeriod::Monthly)`
- **THEN** 统计 2026-06-01 至 2026-06-30 期间的资金流量

### Requirement: 资金流量表数据结构
系统 SHALL 定义以下数据结构：

```rust
pub struct CashFlowItem {
    pub account: Account,  // 账户信息（含每一层祖先的汇总行）
    pub amount: Decimal,   // 周期内净额汇总（绝对值）
}

pub struct CashFlowReport {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub income: Vec<CashFlowItem>,   // Income 根下各层级汇总
    pub expense: Vec<CashFlowItem>,  // Expenses 根下各层级汇总
}
```

#### Scenario: 资金流量表包含周期范围和收支两节明细
- **WHEN** 生成资金流量表
- **THEN** 返回包含 `period_start`、`period_end`、`income`（Income 根下各层级账户汇总）、`expense`（Expenses 根下各层级账户汇总）

### Requirement: 计算收支账户各层级汇总
系统 SHALL 分别对 Income 与 Expenses 根下的每个账户（含根本身及每一层祖先），统计周期内分录净额并按绝对值汇总，生成 `income` 与 `expense` 两节明细。

#### Scenario: 多层支出账户逐层汇总
- **WHEN** "Expenses:餐饮:外卖" 在周期内有分录 +500
- **THEN** expense 明细中 "外卖"、"餐饮"、"Expenses" 三行的 amount 均为 500

#### Scenario: 收入负金额归一化
- **WHEN** "Income:工资" 在周期内有分录 -15000
- **THEN** income 明细中 "工资" 行的 amount 为 15000

#### Scenario: 退款冲抵净额
- **WHEN** "Expenses:餐饮" 在周期内有分录 +500 与退款分录 -200
- **THEN** expense 明细中 "餐饮" 行的 amount 为 300

### Requirement: 使用共享的周期聚合查询
资金流量表 SHALL 使用 `sum_by_account_with_descendants` 共享查询方法，使每个账户的汇总包含其全部后代分录。

#### Scenario: 调用共享查询
- **WHEN** 生成资金流量表
- **THEN** 对 Income 与 Expenses 两根下的全部账户调用 `db.sum_by_account_with_descendants(account_ids, start_date, end_date, ...)`

### Requirement: 报表 API 明细以账户 id 关联
资金流量表 API（`GET /api/reports/cash-flow`）的每个明细项 SHALL 携带 `account_id` 与 `parent_id` 字段作为关联键，并携带按请求语言解析的 `name` 字段仅用于展示；前端与后端之间 SHALL NOT 使用账户名字或路径字符串做逻辑关联。

#### Scenario: 明细项包含 id 与父 id
- **WHEN** 调用 `GET /api/reports/cash-flow`
- **THEN** 每个明细项包含 `account_id`、`parent_id`（根账户为 null）、`name`、`amount` 四个字段

#### Scenario: 全层级覆盖
- **WHEN** Income 根下存在三级账户 "Income:工资:奖金"
- **THEN** income 明细包含该路径上每一层账户的明细项，且可通过 `parent_id` 链接成完整树

### Requirement: 排除不计预算的标签
资金流量表 SHALL 排除带有 "exclude-from-budget" 或 "不计预算" 标签的分录。

#### Scenario: 排除特定标签
- **WHEN** 某分录带有 "exclude-from-budget" 标签
- **THEN** 该分录的金额不计入资金流量统计

### Requirement: 明细行点击跳转交易筛选

现金流量表下方的收支明细列表行 SHALL 可点击（有 hover/cursor 可点击提示）。点击某账户行后，系统 MUST：

1. 将交易页面筛选条件**整体替换**为：日期范围 = 现金流量表当前周期的 `period_start` 至 `period_end`，账户 = 被点击账户及其全部后代账户（与报表聚合口径对齐），其余筛选维度（成员/标签/渠道/关键词/可报销）清空；
2. 将交易面板切换至可视位置（环形布局下转动至交易面板）；
3. 交易列表按新筛选条件自动刷新并显示「已筛选」标识。

系统 MUST NOT 修改旭日图的点击下钻行为。

#### Scenario: 点击父账户筛选子树交易

- **WHEN** 现金流量表当前周期为 2026-08-01 至 2026-08-31，用户点击明细行「餐饮」（含后代「餐饮:外卖」「餐饮:聚餐」）
- **THEN** 交易页面筛选变为 from=2026-08-01、to=2026-08-31、账户=餐饮及其全部后代，列表仅显示涉及该账户子树的交易，并显示「已筛选」标识

#### Scenario: 整体替换既有筛选

- **WHEN** 交易页面当前已设置成员筛选，用户点击现金流量表某账户行
- **THEN** 既有成员筛选被清除，新筛选仅包含周期与被点账户子树

#### Scenario: 点击叶子账户

- **WHEN** 用户点击无后代的叶子账户行
- **THEN** 账户筛选仅含该账户自身，交易列表显示涉及该账户的交易

#### Scenario: 面板切换

- **WHEN** 用户点击明细行且交易面板当前不在可视中心（窄屏/移动端）
- **THEN** 交易面板被转动至可视位置
