# channel-path

## Purpose

交易链路功能——支持为交易设置可变长度的有序渠道序列，替代原有的单渠道关联，支持末端多项、渠道-账户关联、逐环节对账标记。

## Requirements

### Requirement: 交易链路数据模型
系统 SHALL 支持为交易设置可变长度的有序渠道序列（交易链路），通过 `channel_paths` 表存储，每条记录包含 `id`、`transaction_id`、`position`、`channel_id`、`status` 五个字段。`status` 为枚举值：`default`（默认）、`pending`（待校验）、`verified`（已校验）。`transaction_id` 直接引用 `transactions(id)` 并声明 `ON DELETE CASCADE`。同一 `transaction_id` 下 `position` SHALL 从 0 开始递增，且允许同一 position 有多条记录（末端多项）。

#### Scenario: 创建包含 3 个节点的交易链路
- **WHEN** 创建一笔交易并指定渠道序列 [淘宝, 支付宝, 花呗]
- **THEN** 系统在 `channel_paths` 表中创建 3 条记录，status 均为 `default`：(transaction_id=T, position=0, channel_id=淘宝), (transaction_id=T, position=1, channel_id=支付宝), (transaction_id=T, position=2, channel_id=花呗)

#### Scenario: 创建包含 1 个节点的交易链路
- **WHEN** 创建一笔交易并指定渠道序列 [现金]
- **THEN** 系统在 `channel_paths` 表中创建 1 条记录，status 为 `default`：(transaction_id=T, position=0, channel_id=现金)

#### Scenario: 创建不指定渠道的交易
- **WHEN** 创建一笔交易且不指定任何渠道
- **THEN** `channel_paths` 表中不创建任何记录

#### Scenario: 第三方导入创建链路时状态为 pending
- **WHEN** 通过支付宝适配器导入交易，链路为 [支付宝]
- **THEN** `channel_paths` 记录 status 为 `pending`

### Requirement: 交易链末端多项
同一 `transaction_id` 下，最大 position 值 SHALL 允许有多条记录，表示交易链末端有多个并行渠道。非末端 position 不应有多条记录（由应用层保证）。

#### Scenario: 末端多项 — 淘宝→支付宝→(花呗+信用卡)
- **WHEN** 创建一笔交易并指定链路为"淘宝→支付宝→花呗+信用卡"（花呗和信用卡共享末端 position）
- **THEN** 系统在 `channel_paths` 表中创建 4 条记录：(transaction_id=T, position=0, channel_id=淘宝), (transaction_id=T, position=1, channel_id=支付宝), (transaction_id=T, position=2, channel_id=花呗), (transaction_id=T, position=2, channel_id=信用卡)

#### Scenario: 末端单项 — 常规链路
- **WHEN** 创建一笔交易并指定链路为"淘宝→支付宝→花呗"（末端只有一项）
- **THEN** 最大 position 只有 1 条记录，与常规链路行为一致

### Requirement: 交易与链路的关联
`channel_paths` 表 SHALL 通过 `transaction_id` 字段直接关联交易，声明外键 `REFERENCES transactions(id) ON DELETE CASCADE`。`transactions` 表 SHALL 移除原有的 `channel_id` 字段。

#### Scenario: 交易读取链路
- **WHEN** 查询一笔有渠道链路的交易
- **THEN** 返回结果中包含该交易的完整渠道序列，按 position 升序排列，同 position 内按 id 排序

#### Scenario: 交易无链路
- **WHEN** 查询一笔没有渠道链路的交易
- **THEN** 返回结果中渠道序列为空

#### Scenario: 删除交易时级联清理链路
- **WHEN** 删除一笔有渠道链路的交易
- **THEN** 数据库通过 `ON DELETE CASCADE` 自动删除 `channel_paths` 中该 `transaction_id` 的所有记录

### Requirement: 链路渠道存在性约束
`channel_paths.channel_id` SHALL 通过外键约束（`REFERENCES channels(id)`）保证引用的渠道存在。数据库层拒绝插入引用不存在渠道的链路记录。

#### Scenario: 创建交易时引用不存在的渠道
- **WHEN** 创建一笔交易并指定渠道序列 [淘宝, 不存在的渠道]
- **THEN** 数据库外键约束拒绝插入，系统返回渠道不存在的错误

#### Scenario: 更新交易链路时引用不存在的渠道
- **WHEN** 更新一笔交易的链路为 [超市, 不存在的渠道]
- **THEN** 数据库外键约束拒绝插入，系统返回渠道不存在的错误

### Requirement: 链路不可变更新
更新交易的链路时，系统 SHALL 整体替换该交易的链路数据，而非支持局部修改（如插入/删除中间节点）。

#### Scenario: 整体替换链路
- **WHEN** 将交易的链路从 [淘宝, 支付宝, 花呗] 更新为 [淘宝, 微信, 信用卡]
- **THEN** 系统删除该 transaction_id 对应的所有 channel_paths 记录，然后创建新的记录

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

### Requirement: 按渠道检索交易
系统 SHALL 支持检索所有链路中包含指定渠道的交易，用于按渠道进行人工对账。

#### Scenario: 按渠道检索交易
- **WHEN** 查询所有经过"支付宝"渠道的交易
- **THEN** 返回所有 `channel_paths` 中 `channel_id` 为"支付宝"的 `transaction_id` 对应的交易

#### Scenario: 按链路首节点检索交易
- **WHEN** 查询所有链路起点为"淘宝"的交易
- **THEN** 返回所有 `channel_paths` 中 `channel_id` 为"淘宝"且 `position = 0` 的 `transaction_id` 对应的交易

### Requirement: 删除渠道时的引用检查
系统 SHALL 在删除渠道时检查 `channel_paths` 表中是否有引用该渠道的记录。若有，SHALL 拒绝删除并返回渠道正在使用中的错误。

#### Scenario: 删除被链路引用的渠道
- **WHEN** 尝试删除渠道"支付宝"，且存在交易链路引用了该渠道
- **THEN** 系统拒绝删除，返回渠道正在使用中的错误

#### Scenario: 删除未被引用的渠道
- **WHEN** 尝试删除渠道"支付宝"，且没有任何交易链路引用该渠道
- **THEN** 系统允许删除

### Requirement: 渠道关联资产账户
`channels` 表 SHALL 新增可选的 `account_id` 字段，建立渠道与资产账户的一对一关联。`account_id` 允许为 NULL，表示该渠道未关联任何资产账户。

#### Scenario: 创建渠道时关联账户
- **WHEN** 创建渠道"花呗"并指定关联账户"Assets:花呗"
- **THEN** 渠道记录的 `account_id` 设为"Assets:花呗"的 ID

#### Scenario: 创建渠道时不关联账户
- **WHEN** 创建渠道"超市"且不指定关联账户
- **THEN** 渠道记录的 `account_id` 为 NULL

#### Scenario: 更新渠道的账户关联
- **WHEN** 将渠道"花呗"的关联账户从"Assets:花呗"改为"Assets:花呗2"
- **THEN** 渠道记录的 `account_id` 更新为新账户 ID

#### Scenario: 删除已关联渠道的账户
- **WHEN** 尝试删除一个被渠道 `account_id` 引用的资产账户
- **THEN** 系统根据外键约束拒绝删除或级联处理

### Requirement: 报表按渠道统计适配
现有的按渠道统计报表 SHALL 通过 `channel_paths` 表关联渠道，支持统计链路中任意位置包含指定渠道的交易数据。

#### Scenario: 按渠道统计收支
- **WHEN** 请求按渠道"支付宝"统计收支
- **THEN** 系统返回所有链路中包含"支付宝"渠道的交易的收支汇总

### Requirement: CLI 交易链路语法
`tx add` 和 `tx update` 的 `--channel` 参数 SHALL 使用 `->` 表示渠道链，末级多个渠道用 `&` 分隔。每个渠道名后可选择性地追加 `*` 表示 `pending` 或 `√` 表示 `verified`，无后缀表示 `default`。`->` 和 `&` 前后允许任意空格。

#### Scenario: 单渠道链路
- **WHEN** 用户执行 `tx add ... --channel 支付宝`
- **THEN** 创建一条 position=0、channel_id=支付宝、status=default 的 channel_paths 记录

#### Scenario: 单渠道链路带 pending 后缀
- **WHEN** 用户执行 `tx add ... --channel 支付宝*`
- **THEN** 创建一条 position=0、channel_id=支付宝、status=pending 的 channel_paths 记录

#### Scenario: 三节点链路
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 建行卡"`
- **THEN** 创建三条记录：position=0 淘宝 status=default，position=1 支付宝 status=default，position=2 建行卡 status=default

#### Scenario: 三节点链路混合状态
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝* -> 建行卡√"`
- **THEN** 创建三条记录：position=0 淘宝 status=default，position=1 支付宝 status=pending，position=2 建行卡 status=verified

#### Scenario: 末级多项链路
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 花呗 & 建行卡"`
- **THEN** 创建四条记录：position=0 淘宝 default，position=1 支付宝 default，position=2 花呗 default，position=2 建行卡 default

#### Scenario: 末级多项链路混合状态
- **WHEN** 用户执行 `tx add ... --channel "淘宝 -> 支付宝 -> 花呗* & 建行卡√"`
- **THEN** 创建四条记录：position=0 淘宝 default，position=1 支付宝 default，position=2 花呗 pending，position=2 建行卡 verified

#### Scenario: 不含空格也合法
- **WHEN** 用户执行 `tx add ... --channel 淘宝->支付宝->花呗&建行卡`
- **THEN** 与带空格的写法产生相同的 channel_paths 记录

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
