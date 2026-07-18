# builtin-data-english-storage

## Purpose

定义内置系统实体（账户、标签、渠道）的种子数据规范：以英文系统名为规范基准，配合多语言系统名字存储，种子内容与建库语言无关，数据库不存储显示语言设置。

## Requirements

### Requirement: 内置实体种子与多语言系统名字
数据库初始化 SHALL 创建系统内置实体（4 个根账户、6 个系统子账户、4 个系统标签、1 个内置渠道），并为每个内置实体按受支持语言（en、zh-CN）插入系统名字（`is_system=1`）且设为对应语言的显示名。英文系统名为规范基准：

- 根账户：`Assets`、`Equity`、`Income`、`Expenses`
- 系统子账户：`Equity:OpeningBalances`、`Expenses:Fees`、`Expenses:Discounts`、`Expenses:InstallmentFees`、`Assets:Cash`、`Assets:Cashback`
- 系统标签：`repayment`、`pending`、`exclude-from-income-statement`、`exclude-from-budget`
- 内置渠道：`Alipay`

种子内容 SHALL NOT 因建库语言不同而产生任何差异；初始化不再接收语言参数，数据库不存储显示语言设置（显示语言完全由调用方参数决定，见 `entity-names-i18n`）。

#### Scenario: 中文显示语言建库
- **WHEN** 用户以中文为显示语言初始化新数据库并使用
- **THEN** 系统账户 `Assets:Cash` 同时拥有英文系统名 `Cash` 和中文系统名 `现金`，分别为对应语言的显示名

#### Scenario: 建库无语言分支
- **WHEN** 分别以英文和中文为显示语言初始化两个数据库
- **THEN** 两个数据库的内容完全一致，且均不包含显示语言设置

### Requirement: 消除内置名的语言耦合查询
业务逻辑 SHALL NOT 依赖内置名字的具体语言值进行查询或匹配。系统标签 `pending` 的查询 SHALL 通过系统名字 `pending` 或系统实体身份完成，不再探测 `待处理`。

#### Scenario: pending 标签单名查询
- **WHEN** 导入服务解析待处理标签
- **THEN** 仅按系统名 `pending` 查询，无中文名探测分支

### Requirement: 不兼容存量数据库
本变更 SHALL NOT 提供存量数据库迁移。以旧版本 schema 创建的数据库无法由新版本直接使用，用户需自行导出数据并重建。

#### Scenario: 明确的不兼容声明
- **WHEN** 发布包含本变更的版本
- **THEN** 发布说明中明确 schema 不兼容及用户自助迁移路径（导出 → 重建 → 导入）
