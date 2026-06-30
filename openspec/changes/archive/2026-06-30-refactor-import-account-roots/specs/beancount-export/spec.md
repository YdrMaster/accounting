# beancount-export

## MODIFIED Requirements

### Requirement: Export accounts as beancount open/close directives
系统 SHALL 将数据库中所有 Account 导出为 beancount `open` 指令（含 metadata），已关闭账户 SHALL 额外输出 `close` 指令。系统不再对 Import 类型账户做特殊路径转换；所有账户直接按其实际路径导出，account_type 由根账户名决定。

#### Scenario: 导出普通资产账户
- **WHEN** 数据库中存在 Account { id: 1, name: "Cash", parent: Asset, account_type: Asset, closed_at: None }
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open Assets:Cash
    internal_id: 1
    account_type: "Asset"
  ```

#### Scenario: 导出导入 fallback 支出账户
- **WHEN** 数据库中存在路径 `Expenses:Import:Alipay:Food` 的账户
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open Expenses:Import:Alipay:Food
    account_type: "Expense"
  ```

#### Scenario: 导出导入 fallback 收入账户
- **WHEN** 数据库中存在路径 `Income:Import:Alipay:Salary` 的账户
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open Income:Import:Alipay:Salary
    account_type: "Income"
  ```

#### Scenario: 导出导入 fallback 退款账户
- **WHEN** 数据库中存在路径 `Expenses:Import:Alipay:退款` 的账户
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open Expenses:Import:Alipay:退款
    account_type: "Expense"
  ```

#### Scenario: 导出导入 fallback 资产账户
- **WHEN** 数据库中存在路径 `Assets:Import:Alipay:CreditCard` 的账户
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open Assets:Import:Alipay:CreditCard
    account_type: "Asset"
  ```

#### Scenario: 导出已关闭账户
- **WHEN** 账户 closed_at 为 2024-12-31
- **THEN** 导出文件中 SHALL 在 open 指令之后包含：
  ```
  2024-12-31 close Assets:Cash
  ```

#### Scenario: 导出含账单日/还款日的账户
- **WHEN** 账户 billing_day 为 15，repayment_day 为 5
- **THEN** open 指令 SHALL 包含 `billing_day: 15` 和 `repayment_day: 5` metadata

## REMOVED Requirements

### Requirement: 导出 Import 类型账户
系统 SHALL 对 AccountType::Import 类型的账户做特殊路径转换，将其从 `导入:xxx` 映射为 `Equity:Import:xxx`，并通过 `account_type` metadata 保留原始类型。

#### Scenario: 原 Import 类型账户导出
- **WHEN** 账户类型为 Import，路径为 `导入:Alipay`
- **THEN** 不再映射到 `Equity:Import:Alipay`

**Reason**: Import 账户类型已移除，所有账户均位于标准 beancount 根账户下，无需特殊转换。

**Migration**: 新系统中不存在 Import 类型账户；旧 `导入:` 数据不迁移。
