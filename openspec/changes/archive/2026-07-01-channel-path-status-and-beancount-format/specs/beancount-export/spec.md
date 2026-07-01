## MODIFIED Requirements

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

### Requirement: Export transactions with full fidelity
系统 SHALL 将每笔 Transaction 及其 Posting 导出为 beancount 交易指令，包含所有 metadata 以保留本系统独有概念。

#### Scenario: 导出含渠道链路的交易
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "微信", status: verified }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "微信√"`

#### Scenario: 导出多级渠道链路
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "淘宝", status: default }, { position: 1, channel: "支付宝", status: pending }, { position: 2, channel: "花呗", status: default }, { position: 2, channel: "建行卡", status: verified }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "淘宝 -> 支付宝* -> 花呗 & 建行卡√"`

#### Scenario: 导出含 pending 单渠道的交易
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "支付宝", status: pending }]
- **THEN** 交易指令 SHALL 包含 `channel_path: "支付宝*"`

## REMOVED Requirements

### Requirement: 渠道链路使用 JSON metadata
系统 SHALL 将 `channel_paths` 序列化为 JSON 字符串并作为 `channel_path` metadata 输出。

#### Scenario: 原 JSON 格式输出
- **WHEN** 交易有 ChannelPath [{ position: 0, channel: "微信", reconciled: true }]
- **THEN** 交易指令 SHALL 包含 `channel_path: '[{"position":0,"channel":"微信","reconciled":true}]'`

**Reason**: JSON 格式不易人工阅读和编辑，且与 CLI 的 `->` / `&` 语法不一致。

**Migration**: 新导出使用文本格式；旧备份中的 JSON 格式在导入时仍被兼容解析。
