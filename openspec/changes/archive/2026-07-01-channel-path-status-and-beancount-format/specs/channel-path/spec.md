## MODIFIED Requirements

### Requirement: 交易链路数据模型
系统 SHALL 支持为交易设置可变长度的有序渠道序列（交易链路），通过 `channel_paths` 表存储，每条记录包含 `id`、`transaction_id`、`position`、`channel_id`、`status` 五个字段。`status` 为枚举值：`default`（默认）、`pending`（待校验）、`verified`（已校验）。`transaction_id` 直接引用 `transactions(id)` 并声明 `ON DELETE CASCADE`。同一 `transaction_id` 下 `position` SHALL 从 0 开始递增，且允许同一 position 有多条记录（末端多项）。

#### Scenario: 创建包含 3 个节点的交易链路
- **WHEN** 创建一笔交易并指定渠道序列 [淘宝, 支付宝, 花呗]
- **THEN** 系统在 `channel_paths` 表中创建 3 条记录，status 均为 `default`：(transaction_id=T, position=0, channel_id=淘宝), (transaction_id=T, position=1, channel_id=支付宝), (transaction_id=T, position=2, channel_id=花呗)

#### Scenario: 第三方导入创建链路时状态为 pending
- **WHEN** 通过支付宝适配器导入交易，链路为 [支付宝]
- **THEN** `channel_paths` 记录 status 为 `pending`

### Requirement: 逐环节对账标记
`channel_paths` 表 SHALL 包含 `status` 字段，标记该环节的状态：`default`（无需特别校验）、`pending`（待人工校验）、`verified`（已校验）。创建链路节点时 `status` 默认为 `default`，从第三方渠道导入时默认为 `pending`，用户可独立标记每个环节的状态。

#### Scenario: 创建链路时默认 default
- **WHEN** 通过 CLI 创建一笔交易并指定渠道序列 [淘宝, 支付宝, 花呗]
- **THEN** 所有 3 条 channel_paths 记录的 `status` 均为 `default`

#### Scenario: 标记单个环节为 verified
- **WHEN** 用户将链路中“支付宝”环节标记为已校验
- **THEN** 对应 channel_paths 记录的 `status` 更新为 `verified`，其他环节的 `status` 不受影响

#### Scenario: 查询 pending 环节
- **WHEN** 查询某笔交易中所有待校验的环节
- **THEN** 返回该交易 `transaction_id` 下 `status = pending` 的 channel_paths 记录

### Requirement: CLI 交易链路语法
`tx add` 和 `tx update` 的 `--channel` 参数 SHALL 使用 `->` 表示渠道链，末级多个渠道用 `&` 分隔。每个渠道名后可选择性地追加 `*` 表示 `pending` 或 `√` 表示 `verified`，无后缀表示 `default`。`->` 和 `&` 前后允许任意空格。

#### Scenario: 单渠道链路带 pending 后缀
- **WHEN** 用户执行 `tx add ... --channel 支付宝*`
- **THEN** 创建一条 position=0、channel_id=支付宝、status=pending 的 channel_paths 记录

#### Scenario: 三节点链路混合状态
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝* -> 建行卡√"`
- **THEN** 创建三条记录：position=0 淘宝 status=default，position=1 支付宝 status=pending，position=2 建行卡 status=verified

#### Scenario: 末级多项链路混合状态
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 花呗* & 建行卡√"`
- **THEN** 创建四条记录：position=0 淘宝 default，position=1 支付宝 default，position=2 花呗 pending，position=2 建行卡 verified

#### Scenario: 渠道名含后缀字符报错
- **WHEN** 用户执行 `tx add ... --channel "支付宝* -> 微信"` 且存在名为“支付宝*”的渠道
- **THEN** 系统按“支付宝”查找渠道，并将该节点 status 设为 pending；若不存在“支付宝”渠道则报错

#### Scenario: 空节点报错
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> -> 支付宝"`
- **THEN** 返回错误 "渠道路径包含空节点"

#### Scenario: 非末级使用 & 报错
- **WHEN** 用户执行 `tx add ... --channel "淘宝 & 支付宝 -> 花呗"`
- **THEN** 返回错误 "& 只能在链路最后一级使用"

### Requirement: 渠道名称分隔符限制
由于 CLI 使用 `->`、`&`、`*`、`√` 作为渠道路径语法字符，`channels.name` 及新增渠道输入 SHALL 禁止包含子串 `->`、`&`、`*` 或 `√`。

#### Scenario: 创建含分隔符的渠道名被拒绝
- **WHEN** 用户尝试创建名为 `支付宝&微信` 的渠道
- **THEN** 系统返回错误 "渠道名称不能包含 &、->、* 或 √"

#### Scenario: 创建普通渠道名成功
- **WHEN** 用户创建名为 `支付宝` 的渠道
- **THEN** 创建成功

## REMOVED Requirements

### Requirement: 交易链路数据模型中的 reconciled 布尔字段
系统 SHALL 使用 `reconciled` 布尔字段标记环节是否已对账。

#### Scenario: 创建链路时默认未对账
- **WHEN** 创建一笔交易并指定渠道序列 [淘宝, 支付宝, 花呗]
- **THEN** 所有 3 条 channel_paths 记录的 `reconciled` 均为 0（未对账）

#### Scenario: 标记单个环节为已对账
- **WHEN** 用户将链路中“支付宝”环节标记为已对账
- **THEN** 对应 channel_paths 记录的 `reconciled` 更新为 1，其他环节的 `reconciled` 不受影响

**Reason**: 布尔对账标记无法表达“第三方导入待校验”与“默认未对账”的区别，因此升级为 `default/pending/verified` 三态。

**Migration**: 旧数据中 `reconciled=1` 迁移为 `status=verified`；`reconciled=0` 迁移为 `status=default`。
