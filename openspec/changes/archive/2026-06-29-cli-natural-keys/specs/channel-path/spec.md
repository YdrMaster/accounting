# channel-path

交易链路功能——支持为交易设置可变长度的有序渠道序列，替代原有的单渠道关联，支持末端多项、渠道-账户关联、逐环节对账标记。

## ADDED Requirements

### Requirement: CLI 交易链路语法
`tx add` 和 `tx update` 的 `--channel` 参数 SHALL 使用 `->` 表示渠道链，末级多个渠道用 `&` 分隔。`->` 和 `&` 前后允许任意空格。系统 SHALL 按顺序生成 `channel_paths` 记录，末级多个渠道共享同一最大 `position`。

#### Scenario: 单渠道链路
- **WHEN** 用户执行 `tx add ... --channel 支付宝`
- **THEN** 创建一条 position=0、channel_id=支付宝 的 channel_paths 记录

#### Scenario: 三节点链路
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 建行卡"`
- **THEN** 创建三条记录：position=0 淘宝，position=1 支付宝，position=2 建行卡

#### Scenario: 末级多项链路
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 花呗 & 建行卡"`
- **THEN** 创建四条记录：position=0 淘宝，position=1 支付宝，position=2 花呗，position=2 建行卡

#### Scenario: 不含空格也合法
- **WHEN** 用户执行 `tx add ... --channel 淘宝->支付宝->花呗&建行卡`
- **THEN** 与带空格的写法产生相同的 channel_paths 记录

#### Scenario: 空节点报错
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> -> 支付宝"`
- **THEN** 返回错误 "渠道路径包含空节点"

#### Scenario: 非末级使用 & 报错
- **WHEN** 用户执行 `tx add ... --channel "淘宝 & 支付宝 -> 花呗"`
- **THEN** 返回错误 "& 只能在链路最后一级使用"

### Requirement: 渠道名称分隔符限制
由于 CLI 使用 `->` 和 `&` 作为渠道路径语法分隔符，`channels.name` 及新增渠道输入 SHALL 禁止包含子串 `->` 或 `&`。

#### Scenario: 创建含分隔符的渠道名被拒绝
- **WHEN** 用户尝试创建名为 `支付宝&微信` 的渠道
- **THEN** 系统返回错误 "渠道名称不能包含 & 或 ->"

#### Scenario: 创建普通渠道名成功
- **WHEN** 用户创建名为 `支付宝` 的渠道
- **THEN** 创建成功
