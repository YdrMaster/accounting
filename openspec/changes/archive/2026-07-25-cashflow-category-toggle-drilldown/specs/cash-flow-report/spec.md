# cash-flow-report delta

## MODIFIED Requirements

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

### Requirement: 使用共享的周期聚合查询
资金流量表 SHALL 使用 `sum_by_account_with_descendants` 共享查询方法，使每个账户的汇总包含其全部后代分录。

#### Scenario: 调用共享查询
- **WHEN** 生成资金流量表
- **THEN** 对 Income 与 Expenses 两根下的全部账户调用 `db.sum_by_account_with_descendants(account_ids, start_date, end_date, ...)`

## REMOVED Requirements

### Requirement: 计算每个资产账户的流入流出
**Reason**: 统计口径从资产账户改为收支账户；收支账户的单向性使 inflow/outflow 拆分没有展示场景，且原实现按净额符号伪拆分存在失真。改为每账户单金额（净额绝对值）汇总。

**Migration**: 由新增要求「计算收支账户各层级汇总」替代；调用方（API handler、CLI）改为消费 `income`/`expense` 明细的 `amount` 字段。

### Requirement: 计算总资产汇总行
**Reason**: 报表不再含 `total` 字段；Income/Expenses 两根账户自身的汇总行即各自节的总计。

**Migration**: 需要总额的消费者取 `income`/`expense` 中根账户（parent_id 为空）对应行的 `amount`。

## ADDED Requirements

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

### Requirement: 报表 API 明细以账户 id 关联
资金流量表 API（`GET /api/reports/cash-flow`）的每个明细项 SHALL 携带 `account_id` 与 `parent_id` 字段作为关联键，并携带按请求语言解析的 `name` 字段仅用于展示；前端与后端之间 SHALL NOT 使用账户名字或路径字符串做逻辑关联。

#### Scenario: 明细项包含 id 与父 id
- **WHEN** 调用 `GET /api/reports/cash-flow`
- **THEN** 每个明细项包含 `account_id`、`parent_id`（根账户为 null）、`name`、`amount` 四个字段

#### Scenario: 全层级覆盖
- **WHEN** Income 根下存在三级账户 "Income:工资:奖金"
- **THEN** income 明细包含该路径上每一层账户的明细项，且可通过 `parent_id` 链接成完整树
