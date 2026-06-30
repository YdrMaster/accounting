# account-mapping

## MODIFIED Requirements

### Requirement: 映射 category 格式
`category` 字段 SHALL 使用 `"<role>:<原始分类>"` 格式。role 为 `"Asset"`（资产侧）、`"Income"`（收入侧）或 `"Expenses"`（支出侧/退款侧）。资产侧映射 key 格式为 `"Assets:<付款方式名>"`，收入侧映射 key 格式为 `"Income:<分类名>"`，支出侧及退款映射 key 格式为 `"Expenses:<分类名>"`。

#### Scenario: 资产侧映射 key
- **WHEN** 适配器输出 `role=Asset, category="蚂蚁宝藏信用卡"`
- **THEN** 映射 key 为 `"Assets:蚂蚁宝藏信用卡"`

#### Scenario: 收入侧映射 key
- **WHEN** 适配器输出 `role=IncomeExpense, amount<0, category="工资"`
- **THEN** 映射 key 为 `"Income:工资"`

#### Scenario: 支出侧映射 key
- **WHEN** 适配器输出 `role=IncomeExpense, amount>0, category="餐饮美食"`
- **THEN** 映射 key 为 `"Expenses:餐饮美食"`

#### Scenario: 退款作为支出侧映射 key
- **WHEN** 适配器输出 `role=IncomeExpense, category="退款"`
- **THEN** 映射 key 为 `"Expenses:退款"`
