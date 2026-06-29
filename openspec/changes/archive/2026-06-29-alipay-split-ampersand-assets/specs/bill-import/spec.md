## ADDED Requirements

### Requirement: 支付宝适配器按 `&` 拆分收/付款方式

支付宝适配器 SHALL 在解析 `收/付款方式` 字段时，按 `&` 字符拆分为多个部分；每个非空部分各自生成一个 `role=Asset` 的 `BillPosting`。第一个部分承担该交易全部资产侧金额，其余部分金额为 `Decimal::ZERO`。空字段或拆分后全部为空时，仍回退为使用渠道名作为单一 Asset Posting 的 `category`。

#### Scenario: 收/付款方式含两个部分

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡(江苏银行)&超划算"`，交易金额为 `4.80`，方向为支出
- **THEN** 产出的 `BillEntry` 包含三个 `BillPosting`：
  - `{role=IncomeExpense, category="餐饮美食", amount=+4.80}`
  - `{role=Asset, category="蚂蚁宝藏信用卡(江苏银行)", amount=-4.80}`
  - `{role=Asset, category="超划算", amount=0.00}`

#### Scenario: 收/付款方式含三个部分

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡(江苏银行)&茶咖自由卡&超划算"`
- **THEN** 第一个部分 `"蚂蚁宝藏信用卡(江苏银行)"` 承担全部资产侧金额，其余两个部分生成金额为 `0.00` 的 Asset Posting

#### Scenario: 退款交易中收/付款方式含 `&`

- **WHEN** 适配器解析一笔退款账单，其 `收/付款方式` 为 `"招商银行信用卡分期&招商银行满减"`，退款金额为 `3158.00`
- **THEN** 第一个部分 `"招商银行信用卡分期"` 的 Asset Posting 金额为 `+3158.00`，其余部分金额为 `0.00`

#### Scenario: 收/付款方式为空

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为空字符串
- **THEN** 生成一个 `role=Asset` 的 `BillPosting`，其 `category` 为 `ImportContext` 中的渠道名（如 `"支付宝"`），金额按原有规则计算

#### Scenario: 收/付款方式无 `&`

- **WHEN** 适配器解析一行支付宝账单，其 `收/付款方式` 为 `"蚂蚁宝藏信用卡"` 且不含 `&`
- **THEN** 行为与改动前一致，仅生成一个 `role=Asset` 的 `BillPosting`
