# account-type-resolution

## Purpose

账户类型（Asset/Equity/Income/Expense）解析机制——以单条 SQL 批量解析全部账户的根账户类型（消除逐账户 N+1 查询），并通过禁止修改系统根账户显示名保证类型推导锚点的稳定性，使预算/攒钱计划校验等依赖类型判定的路径不再被用户改名破坏。

## Requirements

### Requirement: 批量账户类型解析

系统 SHALL 提供单条 SQL 查询 `account_root_names_by_ids(account_ids, lang)`，返回每个输入账户的根账户（闭包表中 depth 最大的祖先）在指定语言的**系统名**（`is_system=1` 的名字行，即种子写入的名；用户后加的同名非系统名不参与推导）。`load_account_types` SHALL 调用该批量查询一次性解析全部账户类型，不再逐账户发起查询。

#### Scenario: 批量解析多个账户的根名

- **WHEN** 对 [Assets:Bank, Assets:Bank:Checking, Expenses:Food] 调用 account_root_names_by_ids(lang='en')
- **THEN** 返回 (Bank → "Assets")、(Checking → "Assets")、(Food → "Expenses")

#### Scenario: 预算校验单次往返

- **WHEN** 创建预算并触发账户类型校验
- **THEN** 类型解析只发起一次根名查询（而非每账户一次）

#### Scenario: 无对应语言名时返回空

- **WHEN** 某账户的根账户在指定语言无系统名
- **THEN** 该账户不出现在结果中（调用方按既有规则视为无法推导类型）

#### Scenario: 用户自建根账户不参与类型推导

- **WHEN** 用户自建一个根账户（非系统账户）并在其下创建子账户
- **THEN** 该根账户及其子账户不出现在类型解析结果中（类型推导只认种子系统根）

### Requirement: 系统根账户改名保护

系统 SHALL 拒绝修改系统根账户（`parent_id IS NULL AND is_system=1`）的显示名（任何语言），并返回语义明确的错误。非根账户的改名不受影响。

#### Scenario: 拒绝修改根账户英文名

- **WHEN** 对 Assets 根账户执行改名（lang='en'）
- **THEN** 返回错误，数据库中根账户名保持不变

#### Scenario: 拒绝修改根账户中文名

- **WHEN** 对 Expenses 根账户执行改名（lang='zh-CN'）
- **THEN** 返回错误，数据库中根账户名保持不变

#### Scenario: 非根账户改名正常

- **WHEN** 对 Assets:Bank 子账户执行改名
- **THEN** 改名成功

### Requirement: 改名保护的错误呈现

对系统根账户改名的请求，CLI 与 API SHALL 返回明确的本地化错误信息，指出系统根账户不可改名。

#### Scenario: CLI 报错

- **WHEN** 执行 `account rename Assets "我的资产" --lang en`
- **THEN** 显示本地化错误，提示系统根账户不可改名

#### Scenario: API 报错

- **WHEN** 通过账户改名端点请求修改系统根账户
- **THEN** 返回 HTTP 400，响应体包含本地化错误信息

### Requirement: 类型推导行为回归

批量解析与改名保护引入后，预算/攒钱计划的账户类型校验（限 Expenses/Assets 子树）、账户关闭校验、现金流量表分组的行为 SHALL 与变更前完全一致。

#### Scenario: 预算校验行为不变

- **WHEN** 创建限额挂在支出账户的预算
- **THEN** 校验通过；限额挂在资产账户时返回 Err(AccountNotExpense)

#### Scenario: 攒钱计划校验行为不变

- **WHEN** 创建账户集合含资产账户的攒钱计划
- **THEN** 校验通过；含支出账户时返回 Err(AccountNotAsset)
