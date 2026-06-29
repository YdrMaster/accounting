## Why

支付宝账单 CSV 的 `收/付款方式` 字段经常出现 `&` 分隔的复合值，例如 `蚂蚁宝藏信用卡(江苏银行)&超划算`、`招商银行信用卡&招商银行立减金`。当前 `AlipayAdapter` 把整个字符串当作一个资产分类，导致生成名为 `Import:支付宝:资产:蚂蚁宝藏信用卡(江苏银行)&超划算` 的账户，用户无法为其中的单个资产（信用卡、优惠券/补贴等）分别设置账户映射。本改动让 `&` 分隔的各个部分成为独立的 Asset Posting，从而支持分别映射到不同账户。

## What Changes

- 修改 `accounting-service/src/import/alipay.rs` 中的 `parse_alipay_row`：
  - 对 `收/付款方式` 按 `&` 拆分并 trim；
  - 第一个非空部分作为实际扣款资产，承担全部资产侧金额；
  - 其余部分生成金额为 `0` 的 Asset Posting，各自可被账户映射命中；
  - 空字段保持原有回退行为，使用渠道名作为资产分类。
- 在 `accounting-service/src/import/alipay.rs` 的单元测试中新增覆盖场景：
  - 两个部分（`A&B`）；
  - 三个部分（`A&B&C`）；
  - 退款交易中的 `&` 拆分；
  - 空 `收/付款方式` 的回退行为；
  - 无 `&` 时行为不变。
- `BillAdapter` trait、`ImportService`、`MappingService` 和数据库 schema 均保持不变。

## Capabilities

### New Capabilities

（无新能力）

### Modified Capabilities

- `bill-import`：支付宝适配器对 `收/付款方式` 字段的解析行为从“整体作为单一资产分类”变为“按 `&` 拆分生成多个 Asset Posting”。

## Impact

- 受影响代码：仅 `accounting-service/src/import/alipay.rs` 及其单元测试。
- 用户影响：新导入的支付宝交易在 `收/付款方式` 含 `&` 时会生成多个资产侧 Posting，历史已导入交易不受影响。
- 映射影响：用户可为 `资产:蚂蚁宝藏信用卡(江苏银行)`、`资产:超划算` 等分别设置映射；原复合名称 `资产:蚂蚁宝藏信用卡(江苏银行)&超划算` 的映射在拆分后不再被命中。
