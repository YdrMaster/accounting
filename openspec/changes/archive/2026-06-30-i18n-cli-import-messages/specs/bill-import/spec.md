# bill-import

## MODIFIED Requirements

### Requirement: skip-on-error 策略
导入过程中，单行解析错误 SHALL 不中断整批导入。系统 SHALL 以结构化方式记录错误（包含行号和错误类型/数据），继续处理后续行，最终由 CLI 根据当前语言环境汇总输出成功数和错误详情。

#### Scenario: 跳过格式错误的行
- **WHEN** 第 5 行金额字段为空、第 18 行日期格式无效
- **THEN** 系统跳过这两行并继续处理其他行，英文环境下 CLI 报告 "Imported 23 transactions, skipped 2 (row 5: amount parse failed; row 18: date parse failed)"

#### Scenario: 所有行均成功
- **WHEN** 所有行解析成功
- **THEN** 英文环境下 CLI 报告 "Imported N transactions, skipped 0"

### Requirement: CLI import 子命令
系统 SHALL 在 CLI 中新增 `import` 子命令，接受 `--file <路径>`、`--source <来源>`、`--member <NAME>` 等参数，执行导入并输出结果摘要和交易 ID 列表。所有输出文本 SHALL 通过 `accounting-cli` 的 locale 文件进行本地化。

#### Scenario: 基本导入命令
- **WHEN** 用户执行 `accounting import --source alipay --member Alice --file bill.csv`
- **THEN** 系统读取文件，选择支付宝适配器，执行导入，输出本地化的摘要信息和交易 ID 列表

#### Scenario: 成员名称不存在
- **WHEN** 用户执行 `accounting import --source alipay --member Bob --file bill.csv` 且成员 Bob 不存在
- **THEN** 系统返回本地化错误，英文环境下为 "Member not found: Bob"

#### Scenario: 不支持的来源
- **WHEN** 用户指定 `--source unknown_provider`
- **THEN** 系统返回本地化错误，英文环境下为 "Unsupported source: unknown_provider"

#### Scenario: 文件不存在
- **WHEN** 用户指定的文件路径不存在
- **THEN** 系统返回本地化的文件读取错误
