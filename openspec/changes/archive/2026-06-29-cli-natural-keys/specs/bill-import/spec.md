# bill-import

账单导入功能——从外部渠道 App 导出的账单文件批量导入交易，通过适配器模式支持多渠道，导入的交易使用 Import 根账户隔离并标记待处理 Tag。

## MODIFIED Requirements

### Requirement: CLI import 子命令
系统 SHALL 在 CLI 中新增 `import` 子命令，接受 `--file <路径>`、`--source <来源>`、`--member <NAME>` 等参数，执行导入并输出结果摘要和交易 ID 列表。

#### Scenario: 基本导入命令
- **WHEN** 用户执行 `accounting import --source alipay --member Alice --file bill.csv`
- **THEN** 系统读取文件，选择支付宝适配器，执行导入，输出摘要信息和交易 ID 列表

#### Scenario: 成员名称不存在
- **WHEN** 用户执行 `accounting import --source alipay --member Bob --file bill.csv` 且成员 Bob 不存在
- **THEN** 系统返回错误 "成员 'Bob' 不存在"

#### Scenario: 不支持的来源
- **WHEN** 用户指定 `--source unknown_provider`
- **THEN** 系统返回错误"不支持的来源: unknown_provider"

#### Scenario: 文件不存在
- **WHEN** 用户指定的文件路径不存在
- **THEN** 系统返回文件读取错误
