## ADDED Requirements

### Requirement: TransactionDto 包含转账标记
TransactionDto SHALL 包含只读布尔字段 `is_transfer`，由服务端在组装 DTO 时按分录结构实时判定该交易是否为转账（判定规则见 `transaction-list-ui`）。该字段 SHALL NOT 落盘存储，SHALL 不改变既有字段的语义。

#### Scenario: 转账交易返回 true
- **WHEN** 查询一笔资产账户间的转账交易
- **THEN** TransactionDto.is_transfer 为 true

#### Scenario: 非转账交易返回 false
- **WHEN** 查询一笔普通收支交易
- **THEN** TransactionDto.is_transfer 为 false

#### Scenario: 期初余额返回 false
- **WHEN** 查询一笔资产账户与 `Equity:OpeningBalances` 配对的交易
- **THEN** TransactionDto.is_transfer 为 false
