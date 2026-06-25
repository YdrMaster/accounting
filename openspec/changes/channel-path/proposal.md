## Why

当前交易模型中，`channel_id` 只能关联单个渠道，无法表达真实交易中的完整链路。实际交易通常经过多个环节，例如"淘宝→支付宝→花呗"或"超市→微信→信用卡"，每个环节对应不同的账单来源，支持交叉对账以保证记账准确性。现有的单渠道设计无法支撑这种多环节链路的记录和对账需求。

## What Changes

- **BREAKING**：移除 `transactions.channel_id` 字段（当前代码中交易与渠道的单对一关联），替换为通过新的 `channel_paths` 表实现交易与渠道的多对多有序关联
- 新增 `channel_paths` 表：字段为 `id`、`transaction_id`（FK → transactions(id) ON DELETE CASCADE）、`position`、`channel_id`（FK → channels(id)）、`reconciled`（默认 0），存储交易链的有序渠道节点序列
- 交易链末端可以有多项：同一 transaction_id 下，最大 position 可以有多条记录共享同一 position 值（如"淘宝→支付宝→(花呗+信用卡)"，花呗和信用卡都是 position=2）
- `channel_paths.reconciled` 布尔字段，标记该环节是否已完成对账
- `channels` 表新增可选的 `account_id` 字段（FK → accounts(id)），建立渠道与资产账户的直接一对一关联
- 逐层适配 API、Service、Repository、CLI，将原有的单渠道读写改为链路读写，新增对账标记接口

## Capabilities

### New Capabilities
- `channel-path`: 交易链路功能——支持为交易设置可变长度的有序渠道序列，替代原有的单渠道关联，支持末端多项、渠道-账户关联、逐环节对账标记

### Modified Capabilities
<!-- 无现有 specs -->

## Impact

- **数据库层**：新增 `channel_paths` 表（直接用 `transaction_id` 关联交易）；`channels` 表增加 `account_id` 列；`transactions` 表移除 `channel_id` 列
- **核心模型层**（accounting crate）：`Transaction` 结构体移除 `channel_id`；新增 `ChannelPath` 模型；`Channel` 结构体增加 `account_id` 字段
- **SQL 层**（accounting-sql crate）：新增 `channel_paths` 的 Repository；修改交易 CRUD 以支持链路读写；修改报表查询（`posting_sum_by_channel` 等）
- **Service 层**（accounting-service crate）：`ChannelStat` 适配；对账标记读写逻辑
- **API 层**（accounting-api crate）：DTO 变更；路由/handler 适配
- **CLI 层**（accounting-cli crate）：交易命令适配链路输入/输出
