# Proposal: 支付宝导入按交易状态判定退款

## Why

支付宝适配器目前按"交易分类 == 退款"判定退款行，但实际账单中大量退款行的分类是真实消费分类（如餐饮美食、日用百货）。对两份真实账单（共 53 行 `退款成功`）交叉验证：19 行（36%）分类不是"退款"，被漏判；且这些行"收/支"均为"不计收支"，落入不计收支分支后按**支出**记账——钱退回来了，账上却记成一笔消费。精确的判定依据是"交易状态 == 退款成功"（数据验证：状态=退款成功 严格包含 分类=退款，无反向反例）。

## What Changes

- 支付宝适配器退款判定改为 `交易状态 == "退款成功"`（精确匹配），替换 `category == "退款"` 的分类名嗅探。
- `BillPosting` 新增 `is_refund: bool` 字段，由适配器显式标记，贯穿到映射 key 与 fallback 路径生成。
- `PostingRole::to_key` / `fallback_root` 改为接收显式 `is_refund` 标记（退款 → `Expenses` 根），删除 `is_refund_category` 分类名嗅探函数。
- 行为结果：退款行（无论分类）作为负支出落到 `Expenses:Import:<渠道>:<分类>`（或既有映射账户），语义为冲减对应消费分类。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `bill-import`: 支付宝适配器退款判定依据改为交易状态；`BillPosting` 增加 `is_refund` 字段；`PostingRole` 映射 key / fallback 规则改用显式退款标记，删除按分类名（"退款"/"Refund"）的特判。

## Impact

- **代码**: `accounting-service/src/import/alipay.rs`（判定逻辑）、`accounting-service/src/import/mod.rs`（BillPosting 字段）、`accounting/src/posting_role.rs`（to_key/fallback_root 签名与规则）、`accounting-service/src/import_service.rs`（两个调用点）
- **映射兼容**: 退款行映射 key 为 `Expenses:<分类>`，与被误判为正支出时的 key 相同，既有用户映射无需迁移；`分类=退款` 的旧行 key 仍为 `Expenses:退款`
- **依赖**: 无新增依赖
