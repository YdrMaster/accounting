# builtin-data-english-storage (delta)

## MODIFIED Requirements

### Requirement: 内置实体种子与多语言系统名字
数据库初始化 SHALL 创建系统内置实体（4 个根账户、6 个系统子账户、4 个系统标签、1 个内置渠道），并为每个内置实体按受支持语言（en、zh-CN）插入系统名字（`is_system=1`）且设为对应语言的显示名。英文系统名为规范基准：

- 根账户：`Assets`、`Equity`、`Income`、`Expenses`
- 系统子账户：`Equity:OpeningBalances`、`Equity:Cashback`、`Equity:Discounts`、`Expenses:Fees`、`Expenses:InstallmentFees`、`Assets:Cash`
- 系统标签：`repayment`、`pending`、`exclude-from-income-statement`、`exclude-from-budget`
- 内置渠道：`Alipay`

种子内容 SHALL NOT 因建库语言不同而产生任何差异；初始化不再接收语言参数，数据库不存储显示语言设置（显示语言完全由调用方参数决定，见 `entity-names-i18n`）。

`Cashback`（返现）与 `Discounts`（折扣）SHALL 归属 `Equity` 根账户：返现是伴随消费产生、限用途的受限返利，折扣是商家让利，二者均非用户的资产、收入或支出，而是与 `Equity:OpeningBalances` 同构的、直接计入净资产的调整项。归入 `Equity` 后，其余额 SHALL 不进入资产负债表（仅统计 Assets 根）与收支统计（仅统计 Income / Expenses 根）。

#### Scenario: 中文显示语言建库
- **WHEN** 用户以中文为显示语言初始化新数据库并使用
- **THEN** 系统账户 `Assets:Cash` 同时拥有英文系统名 `Cash` 和中文系统名 `现金`，分别为对应语言的显示名

#### Scenario: 返现与折扣账户挂在 Equity 根下
- **WHEN** 初始化新数据库
- **THEN** 存在系统子账户 `Equity:Cashback`（中文名 `返现`）与 `Equity:Discounts`（中文名 `折扣`），且不存在 `Assets:Cashback` 与 `Expenses:Discounts`

#### Scenario: 返现余额不进入资产负债表与收支统计
- **WHEN** 一笔交易包含 `Equity:Cashback` 的 posting
- **THEN** 该 posting 不计入资产负债表（Assets 根统计），也不计入收入/支出统计（Income / Expenses 根统计）

#### Scenario: 建库无语言分支
- **WHEN** 分别以英文和中文为显示语言初始化两个数据库
- **THEN** 两个数据库的内容完全一致，且均不包含显示语言设置
