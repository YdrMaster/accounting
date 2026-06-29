# account-mapping

账户映射功能——为 (成员, 渠道) 绑定分类字符串→账户编号映射，使导入交易可按映射自动归入用户指定账户，无映射时走 Import fallback 账户。

## MODIFIED Requirements

### Requirement: CLI mapping 子命令
系统 SHALL 在 CLI 中新增 `mapping` 子命令，支持 `set`、`list`、`delete` 三个子操作。`set` 接受 `--member`、`--channel`、`--category`、`--account` 参数；`list` 接受 `--member`、`--channel` 参数；`delete` 接受 `--member`、`--channel`、`--category` 参数。`--member` 使用成员名称而非 ID。

#### Scenario: CLI 设置映射
- **WHEN** 用户执行 `accounting mapping set --member Alice --channel 支付宝 --category "收支:餐饮美食" --account "Expenses:餐饮"`
- **THEN** 系统创建映射并输出成功信息

#### Scenario: CLI 列出映射
- **WHEN** 用户执行 `accounting mapping list --member Alice --channel 支付宝`
- **THEN** 系统以表格形式输出该 (成员, 渠道) 下所有映射

#### Scenario: CLI 删除映射
- **WHEN** 用户执行 `accounting mapping delete --member Alice --channel 支付宝 --category "收支:餐饮美食"`
- **THEN** 系统删除映射并输出成功信息

#### Scenario: 成员名称不存在
- **WHEN** 用户执行 `accounting mapping list --member Bob --channel 支付宝` 且成员 Bob 不存在
- **THEN** 返回错误 "成员 'Bob' 不存在"
