# account-type-import

## Purpose

移除独立的 Import 账户类型——导入交易不再落入专用 Import 根账户，而是按其实际收支角色归入 Assets、Equity、Income 或 Expenses 下的 Import 子账户，保证账户分类语义准确、账户体系结构清晰。

## Requirements

### Requirement: 移除 Import 账户类型
系统 SHALL NOT 定义 `AccountType::Import` 枚举变体，也 SHALL NOT 提供 `导入/Import` 系统根账户。通过导入创建的账户 SHALL 按其实际角色归入 `Assets`、`Equity`、`Income` 或 `Expenses` 下的 `Import` 子账户。

#### Scenario: 系统初始化时不再创建 Import 根账户
- **WHEN** 系统初始化数据库
- **THEN** 只创建 `Assets`、`Equity`、`Income`、`Expenses` 四个系统根账户
- **THEN** 不存在 `Import` 或 `导入` 根账户

#### Scenario: 导入交易不再使用 Import 类型账户
- **WHEN** 导入支付宝账单且未设置分类映射
- **THEN** 支出/退款分录落入 `Expenses:Import:支付宝:<分类>`
- **THEN** 收入分录落入 `Income:Import:支付宝:<分类>`
- **THEN** 资产侧分录落入 `Assets:Import:支付宝:<分类>`
