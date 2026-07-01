## MODIFIED Requirements

### Requirement: Import transactions from beancount directives
系统必须从 beancount 交易指令中解析 Transaction 和 Posting。每条交易指令必须包含 `member` metadata，且该 metadata 必须能解析为已导入成员；否则导入必须失败。

#### Scenario: 导入含 channel_path metadata 的交易
- **当** 交易指令包含 `channel_path: "淘宝 -> 支付宝* -> 花呗 & 建行卡√"` metadata 时
- **那么** 系统必须创建 4 条 ChannelPath 记录：淘宝 default、支付宝 pending、花呗 default、建行卡 verified

#### Scenario: 导入单渠道带 pending 状态
- **当** 交易指令包含 `channel_path: "支付宝*"` metadata 时
- **那么** 系统必须创建 1 条 status=pending 的 ChannelPath 记录

#### Scenario: 导入旧 JSON 格式 channel_path
- **当** 交易指令包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'` metadata 时
- **那么** 系统必须创建 1 条 channel=微信、status=verified 的 ChannelPath 记录

#### Scenario: 导入旧 JSON 格式未对账 channel_path
- **当** 交易指令包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":false}]'` metadata 时
- **那么** 系统必须创建 1 条 channel=微信、status=default 的 ChannelPath 记录

## REMOVED Requirements

### Requirement: channel_path 仅支持 JSON 格式
系统 SHALL 从 `channel_path` metadata 中解析 JSON 数组，每个元素包含 `position`、`channel`、`reconciled` 字段。

#### Scenario: JSON 格式解析
- **当** 交易指令包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'` metadata 时
- **那么** 系统必须查找名为 "微信" 的 Channel 并创建对应 ChannelPath 记录

**Reason**: 统一使用 CLI 的 `->` / `&` 文本格式，使 beancount 文件可人工编辑。

**Migration**: 新 beancount 文件使用文本格式；旧 JSON 格式仍被兼容解析，`reconciled=true` 映射为 `verified`，`reconciled=false` 映射为 `default`。
