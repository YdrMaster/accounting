## 变更的需求

### 需求：从 beancount 指令导入交易
系统必须从 beancount 交易指令中解析 Transaction 和 Posting。每条交易指令必须包含 `member` metadata，且该 metadata 必须能解析为已导入成员；否则导入必须失败。

#### 场景：导入普通交易
- **当** beancount 文件中包含交易指令，narration 为 "盒马买菜"，包含两条分录，且 metadata 包含 `member: "张三"` 时
- **那么** 系统必须创建 Transaction { description: "盒马买菜", kind: Normal, member_id: <张三的ID> } 及对应 Posting

#### 场景：导入含 kind metadata 的交易
- **当** 交易指令包含 `kind: "refund"` metadata 时
- **那么** 系统必须设置 Transaction.kind 为 Refund

#### 场景：导入含 member metadata 的交易
- **当** 交易指令包含 `member: "张三"` metadata 时
- **那么** 系统必须查找名为 "张三" 的 Member 并设置 Transaction.member_id

#### 场景：导入缺少 member metadata 的交易
- **当** 交易指令不包含 `member` metadata 时
- **那么** 系统必须报错并终止导入

#### 场景：导入 member 未知的交易
- **当** 交易指令包含 `member: "不存在的成员"` metadata，且该成员未在 beancount 文件中通过 `custom "member"` 定义时
- **那么** 系统必须报错并终止导入

#### 场景：导入含 channel_path metadata 的交易
- **当** 交易指令包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'` metadata 时
- **那么** 系统必须查找名为 "微信" 的 Channel 并创建对应 ChannelPath 记录

#### 场景：导入含标签的交易
- **当** 交易指令包含 `#餐饮` 时
- **那么** 系统必须查找或创建名为 "餐饮" 的 Tag 并关联到该交易

#### 场景：导入含 cost 的分录
- **当** 分录行包含 `100 USD {720 CNY}` 时
- **那么** 系统必须设置 Posting.amount=100, cost=Some(720), cost_commodity 指向 CNY

#### 场景：导入含 reimbursable metadata 的分录
- **当** 分录 metadata 包含 `reimbursable: TRUE` 时
- **那么** 系统必须设置 Posting.is_reimbursable 为 true

## 移除的需求

_无_
