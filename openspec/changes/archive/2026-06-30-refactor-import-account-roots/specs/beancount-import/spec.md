# beancount-import

## MODIFIED Requirements

### Requirement: Import accounts from beancount open/close directives
系统 SHALL 从 beancount `open` 指令中解析账户，使用 `account_get_or_create_by_path` 创建账户。`billing_day` 和 `repayment_day` metadata SHALL 同步更新。`close` 指令 SHALL 设置账户的 `closed_at` 日期。系统不再识别 `account_type: "Import"`，也不再把 `Equity:Import:` 前缀还原为 `导入:`。

#### Scenario: 导入普通资产账户
- **WHEN** beancount 文件中包含 `open Assets:Cash` 及 metadata `account_type: "Asset"`
- **THEN** 系统 SHALL 创建账户路径 `Assets:Cash`

#### Scenario: 导入普通收入账户
- **WHEN** beancount 文件中包含 `open Income:Salary` 及 metadata `account_type: "Income"`
- **THEN** 系统 SHALL 创建账户路径 `Income:Salary`

#### Scenario: 导入普通支出账户
- **WHEN** beancount 文件中包含 `open Expenses:Food` 及 metadata `account_type: "Expense"`
- **THEN** 系统 SHALL 创建账户路径 `Expenses:Food`

#### Scenario: 导入含账单日/还款日的账户
- **WHEN** open 指令包含 `billing_day: 15` 和 `repayment_day: 5` metadata
- **THEN** 系统 SHALL 更新对应账户的 billing_day 和 repayment_day

#### Scenario: 导入账户关闭指令
- **WHEN** beancount 文件中包含 `2024-12-31 close Assets:Cash`
- **THEN** 系统 SHALL 设置该账户的 closed_at 为 2024-12-31

## REMOVED Requirements

### Requirement: Import Import 类型账户
系统 SHALL 识别 `account_type` metadata 为 `"Import"` 的账户，并从 `Equity:Import:xxx` 路径还原为 `导入:xxx`。

#### Scenario: 导入 Import 类型账户
- **WHEN** beancount 文件中包含 `open Equity:Import:Alipay` 及 metadata `account_type: "Import"`
- **THEN** 系统不再还原为 `导入:Alipay`；该路径会按普通 `Equity:Import:Alipay` 处理或报错

**Reason**: Import 账户类型已移除，导入 fallback 账户直接使用标准 beancount 根账户。

**Migration**: 旧备份中的 `Equity:Import:` 账户建议在导入前手动替换为 `Assets:Import:`、`Income:Import:` 或 `Expenses:Import:` 下的新路径；新系统不自动迁移旧数据。
