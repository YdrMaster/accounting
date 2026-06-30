# beancount-import

Beancount 格式导入功能——从 beancount 文本文件中解析并导入全部数据（商品、账户、成员、渠道、交易、附件）到数据库，通过 internal_id 映射实现幂等导入与外键重建。

## Requirements

### Requirement: Import commodities from beancount directives
系统 SHALL 从 beancount `commodity` 指令中解析商品，使用 `commodity_upsert_by_symbol` 创建或更新商品。

#### Scenario: 导入商品
- **WHEN** beancount 文件中包含 `commodity CNY` 及 metadata `name: "人民币"`, `precision: 2`
- **THEN** 系统 SHALL 创建或更新 Commodity { symbol: "CNY", name: "人民币", precision: 2 }

### Requirement: Import accounts from beancount open/close directives
系统 SHALL 从 beancount `open` 指令中解析账户，使用 `account_get_or_create_by_path` 创建账户，路径保持原样，不再对 `account_type: "Import"` 做任何特殊还原。`billing_day` 和 `repayment_day` metadata SHALL 同步更新。`close` 指令 SHALL 设置账户的 `closed_at` 日期。

#### Scenario: 导入普通账户
- **WHEN** beancount 文件中包含 `open 资产:现金` 及 metadata `account_type: "Asset"`
- **THEN** 系统 SHALL 创建账户路径 `资产:现金`

#### Scenario: 导入 Import fallback 子账户
- **WHEN** beancount 文件中包含 `open Expenses:Import:支付宝:餐饮美食` 及 metadata `account_type: "Expense"`
- **THEN** 系统 SHALL 创建账户路径 `Expenses:Import:支付宝:餐饮美食`

#### Scenario: 导入含账单日/还款日的账户
- **WHEN** open 指令包含 `billing_day: 15` 和 `repayment_day: 5` metadata
- **THEN** 系统 SHALL 更新对应账户的 billing_day 和 repayment_day

#### Scenario: 导入账户关闭指令
- **WHEN** beancount 文件中包含 `2024-12-31 close 资产:现金`
- **THEN** 系统 SHALL 设置该账户的 closed_at 为 2024-12-31

### Requirement: Import members from beancount custom directives
系统 SHALL 从 beancount `custom "member"` 指令中解析成员，使用 `member_get_or_create_by_name` 创建或查找成员。

#### Scenario: 导入成员
- **WHEN** beancount 文件中包含 `custom "member" "张三"`
- **THEN** 系统 SHALL 创建或查找 Member { name: "张三" }

### Requirement: Import channels from beancount custom directives
系统 SHALL 从 beancount `custom "channel"` 指令中解析渠道，使用 `channel_upsert_by_name` 创建或查找渠道。

#### Scenario: 导入渠道
- **WHEN** beancount 文件中包含 `custom "channel" "微信"` 及 metadata `description: "微信支付"`
- **THEN** 系统 SHALL 创建或查找 Channel { name: "微信", description: Some("微信支付") }

### Requirement: Import transactions from beancount directives
系统必须从 beancount 交易指令中解析 Transaction 和 Posting。每条交易指令必须包含 `member` metadata，且该 metadata 必须能解析为已导入成员；否则导入必须失败。

#### Scenario: 导入普通交易
- **当** beancount 文件中包含交易指令，narration 为 "盒马买菜"，包含两条分录，且 metadata 包含 `member: "张三"` 时
- **那么** 系统必须创建 Transaction { description: "盒马买菜", kind: Normal, member_id: <张三的ID> } 及对应 Posting

#### Scenario: 导入含 kind metadata 的交易
- **当** 交易指令包含 `kind: "refund"` metadata 时
- **那么** 系统必须设置 Transaction.kind 为 Refund

#### Scenario: 导入含 member metadata 的交易
- **当** 交易指令包含 `member: "张三"` metadata 时
- **那么** 系统必须查找名为 "张三" 的 Member 并设置 Transaction.member_id

#### Scenario: 导入缺少 member metadata 的交易
- **当** 交易指令不包含 `member` metadata 时
- **那么** 系统必须报错并终止导入

#### Scenario: 导入 member 未知的交易
- **当** 交易指令包含 `member: "不存在的成员"` metadata，且该成员未在 beancount 文件中通过 `custom "member"` 定义时
- **那么** 系统必须报错并终止导入

#### Scenario: 导入含 channel_path metadata 的交易
- **当** 交易指令包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'` metadata 时
- **那么** 系统必须查找名为 "微信" 的 Channel 并创建对应 ChannelPath 记录

#### Scenario: 导入含标签的交易
- **当** 交易指令包含 `#餐饮` 时
- **那么** 系统必须查找或创建名为 "餐饮" 的 Tag 并关联到该交易

#### Scenario: 导入含 cost 的分录
- **当** 分录行包含 `100 USD {720 CNY}` 时
- **那么** 系统必须设置 Posting.amount=100, cost=Some(720), cost_commodity 指向 CNY

#### Scenario: 导入含 reimbursable metadata 的分录
- **当** 分录 metadata 包含 `reimbursable: TRUE` 时
- **那么** 系统必须设置 Posting.is_reimbursable 为 true

### Requirement: Rebuild internal ID references via internal_id mapping
系统 SHALL 通过 `internal_id` metadata 建立 old_id → new_id 映射表，用于重连所有外键引用。

#### Scenario: 导入时重建账户 ID 引用
- **WHEN** 账户 open 指令包含 `internal_id: 1`，导入后新 ID 为 42
- **THEN** 系统 SHALL 记录映射 1 → 42，后续分录引用 account internal_id=1 时使用 42

#### Scenario: 导入时重建冲减关系
- **WHEN** 交易包含 `reversal_of: '{"posting_id": 200, "target_posting_id": 201}'` metadata
- **THEN** 系统 SHALL 通过映射表将 200 和 201 转换为新 posting ID，并设置 linked_posting_id

#### Scenario: 导入时重建商品 ID 引用
- **WHEN** commodity 指令包含 `internal_id: 1`，导入后新 ID 为 5
- **THEN** 分录中引用 commodity internal_id=1 的 SHALL 使用 5

### Requirement: Import attachments from document directives
系统 SHALL 从 beancount `document` 指令中读取外部附件文件，并存入数据库。

#### Scenario: 导入含附件引用的交易
- **WHEN** beancount 文件中包含 `document 支出:食品 "attachments/5_receipt.jpg"`
- **THEN** 系统 SHALL 读取 `<导入文件所在目录>/attachments/5_receipt.jpg`
- **THEN** 系统 SHALL 创建 Attachment { transaction_id: <关联交易>, filename: "receipt.jpg", data: <文件内容> }

### Requirement: Skip duplicate transactions by internal_id
系统 SHALL 在导入时检查 `internal_id` metadata，如果目标数据库中已存在相同 `internal_id` 的交易，则跳过该交易。

#### Scenario: 重复导入同一文件
- **WHEN** 首次导入后再次导入相同的 beancount 文件
- **THEN** 系统 SHALL 跳过所有已存在的交易，不产生重复数据

#### Scenario: 无 internal_id 的交易
- **WHEN** beancount 交易指令不包含 `internal_id` metadata
- **THEN** 系统 SHALL 始终作为新交易插入（无法去重）

### Requirement: Import error handling
系统 SHALL 在解析 beancount 文件遇到无法识别的格式时输出错误信息并终止导入。

#### Scenario: 格式错误的 beancount 文件
- **WHEN** beancount 文件包含无法解析的行
- **THEN** 系统 SHALL 输出错误信息（包含行号和错误原因）并以非零退出码退出

#### Scenario: 附件文件不存在
- **WHEN** document 指令引用的附件文件不存在
- **THEN** 系统 SHALL 输出警告信息但继续导入其他数据

### Requirement: CLI command interface
系统 SHALL 提供 `accounting-cli <db> beancount import <input-file>` 命令。

#### Scenario: 执行导入命令
- **WHEN** 用户执行 `accounting-cli my.db beancount import ./output/backup.beancount`
- **THEN** 系统 SHALL 读取 beancount 文件并导入所有数据到 my.db

#### Scenario: 导入完成后输出摘要
- **WHEN** 导入完成
- **THEN** 系统 SHALL 输出导入统计信息（交易数、账户数、商品数等）
