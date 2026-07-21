# beancount-export

## Purpose

Beancount 格式导出功能——将数据库中的全部数据（商品、账户、成员、渠道、交易、附件）导出为 beancount 文本格式，实现完整数据保真与跨系统迁移。

## Requirements

### Requirement: Export commodities as beancount directives
系统 SHALL 将数据库中所有 Commodity 导出为 beancount `commodity` 指令，包含 `internal_id`、`name`、`precision` metadata。`commodity` 指令的日期 SHALL 使用该 Commodity 在数据库中的 `created_at` 日期；若 `created_at` 缺失，回退到 `1970-01-01`。

#### Scenario: 导出单个商品
- **WHEN** 数据库中存在 Commodity { symbol: "CNY", name: "人民币", precision: 2, id: 1, created_at: 2024-01-15 }
- **THEN** 导出文件中 SHALL 包含：
  ```
  2024-01-15 commodity CNY
    internal_id: 1
    name: "人民币"
    precision: 2
  ```

#### Scenario: 导出 created_at 缺失的商品
- **WHEN** 数据库中存在 Commodity { symbol: "USD", name: "美元", precision: 2 } 且 `created_at` 为 NULL
- **THEN** 导出文件中 SHALL 包含：
  ```
  1970-01-01 commodity USD
    internal_id: 2
    name: "美元"
    precision: 2
  ```

### Requirement: Export accounts as beancount open/close directives
系统 SHALL 将数据库中所有 Account 导出为 beancount `open` 指令（含 metadata），已关闭账户 SHALL 额外输出 `close` 指令。账户路径直接按数据库中的真实路径导出，根账户统一使用 `Assets / Equity / Income / Expenses`；不再存在 `AccountType::Import` 类型，导出逻辑 SHALL NOT 对 Import 账户做任何特殊路径转换。

#### Scenario: 导出普通资产账户
- **WHEN** 数据库中存在 Account { id: 1, name: "现金", parent: None, account_type: Asset, closed_at: None }
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD open 资产:现金
    internal_id: 1
    account_type: "Asset"
  ```

#### Scenario: 导出已关闭账户
- **WHEN** 账户 closed_at 为 2024-12-31
- **THEN** 导出文件中 SHALL 在 open 指令之后包含：
  ```
  2024-12-31 close 资产:现金
  ```

#### Scenario: 导出 Import fallback 子账户
- **WHEN** 数据库中存在路径为 `Expenses:Import:支付宝:餐饮美食` 的账户
- **THEN** 导出文件中 SHALL 使用 `Expenses:Import:支付宝:餐饮美食` 路径，并包含 `account_type: "Expense"` metadata

#### Scenario: 导出含账单日/还款日的账户
- **WHEN** 账户 billing_day 为 15，repayment_day 为 5
- **THEN** open 指令 SHALL 包含 `billing_day: 15` 和 `repayment_day: 5` metadata

### Requirement: Export members as beancount custom directives
系统 SHALL 将数据库中所有 Member 导出为 beancount `custom` 指令，包含 `internal_id` metadata。

#### Scenario: 导出成员
- **WHEN** 数据库中存在 Member { id: 1, name: "张三" }
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD custom "member" "张三"
    internal_id: 1
  ```

### Requirement: Export channels as beancount custom directives
系统 SHALL 将数据库中所有 Channel 导出为 beancount `custom` 指令，包含 `internal_id` metadata。

#### Scenario: 导出渠道
- **WHEN** 数据库中存在 Channel { id: 1, name: "微信", description: Some("微信支付") }
- **THEN** 导出文件中 SHALL 包含：
  ```
  YYYY-MM-DD custom "channel" "微信"
    internal_id: 1
    description: "微信支付"
  ```

### Requirement: Export transactions with full fidelity
系统 SHALL 将每笔 Transaction 及其 Posting 导出为 beancount 交易指令，包含所有 metadata 以保留本系统独有概念。

#### Scenario: 导出普通交易
- **WHEN** 存在交易 { description: "盒马买菜", kind: Normal, member: "张三", date_time: 2024-03-15 10:30:00 }，包含两条分录（支出 150 CNY，资产 -150 CNY）
- **THEN** 导出文件中 SHALL 包含：
  ```
  2024-03-15 10:30:00 * "" "盒马买菜"
    internal_id: 100
    kind: "normal"
    member: "张三"
    支出:食品    150.00 CNY
      internal_id: 200
      reimbursable: FALSE
    资产:银行   -150.00 CNY
      internal_id: 201
      reimbursable: FALSE
  ```

#### Scenario: 导出含标签的交易
- **WHEN** 交易关联标签 "餐饮"
- **THEN** 交易指令 SHALL 包含 `#餐饮`

#### Scenario: 导出含渠道链路的交易
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "微信", status: verified }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "微信√"`

#### Scenario: 导出多级渠道链路
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "淘宝", status: default }, { position: 1, channel: "支付宝", status: pending }, { position: 2, channel: "花呗", status: default }, { position: 2, channel: "建行卡", status: verified }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "淘宝 -> 支付宝* -> 花呗 & 建行卡√"`

#### Scenario: 导出含 pending 单渠道的交易
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "支付宝", status: pending }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "支付宝*"`

#### Scenario: 导出退款交易
- **WHEN** 交易 kind 为 Refund
- **THEN** 交易指令 SHALL 包含 `kind: "refund"`

#### Scenario: 导出含冲减关系的分录
- **WHEN** 分录 linked_posting_id 指向 posting 201
- **THEN** 交易指令 SHALL 包含 `reversal_of: '{"posting_id": N, "target_posting_id": 201}'`（N 为当前分录的 internal_id）

#### Scenario: 导出含 cost 的分录
- **WHEN** 分录 amount=100 USD, cost=Some(720 CNY), cost_commodity_id 指向 CNY
- **THEN** 分录行 SHALL 输出为 `100 USD {720 CNY}`

### Requirement: Export attachments as document directives
系统 SHALL 将附件二进制数据写入 `<导出目录>/attachments/` 子目录，并在 beancount 文件中使用 `document` 指令引用。

#### Scenario: 导出含附件的交易
- **WHEN** 交易关联附件 { id: 5, filename: "receipt.jpg", data: <binary> }
- **THEN** 系统 SHALL 将二进制数据写入 `<导出目录>/attachments/5_receipt.jpg`
- **THEN** beancount 文件中 SHALL 包含：
  ```
  YYYY-MM-DD document 支出:食品 "attachments/5_receipt.jpg"
  ```

### Requirement: Export output structure
系统 SHALL 在指定输出目录下生成 `backup.beancount` 主文件和 `attachments/` 子目录。

#### Scenario: 导出到指定目录
- **WHEN** 用户执行 `accounting-cli <db> beancount export ./output`
- **THEN** 系统 SHALL 创建 `./output/backup.beancount` 和 `./output/attachments/` 目录

### Requirement: Export ordering
系统 SHALL 按依赖顺序输出：commodity 指令 → account open 指令 → member/channel custom 指令 → 交易指令（按日期排序）→ account close 指令 → document 指令。

#### Scenario: 输出顺序正确
- **WHEN** 数据库包含商品、账户、成员、渠道、交易
- **THEN** 导出文件中 commodity 指令 SHALL 出现在 account open 之前
- **THEN** account open 指令 SHALL 出现在交易指令之前
- **THEN** account close 指令 SHALL 出现在交易指令之后
