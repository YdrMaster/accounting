## 1. 修改支付宝适配器资产拆分逻辑

- [x] 1.1 在 `accounting-service/src/import/alipay.rs` 的 `parse_alipay_row` 中，将 `payment_method` 按 `&` 拆分并 trim，过滤空字符串。
- [x] 1.2 当拆分结果非空时，第一个部分生成承担全部 `asset_amount` 的 Asset Posting，其余每个部分生成金额为 `Decimal::ZERO` 的 Asset Posting。
- [x] 1.3 当 `payment_method` 为空或拆分后全部为空时，保持原有回退行为，使用 `channel_name` 作为单一 Asset Posting 的 `category`。
- [x] 1.4 确保 `BillEntry` 的 `postings` 顺序为：先 IncomeExpense Posting，后按拆分顺序排列的 Asset Postings。

## 2. 补充单元测试

- [x] 2.1 新增测试：收/付款方式含两个部分（`A&B`），验证生成 1 个 IncomeExpense + 2 个 Asset Posting，且金额分配正确。
- [x] 2.2 新增测试：收/付款方式含三个部分（`A&B&C`），验证第一个部分拿全额，后两个部分金额为 0。
- [x] 2.3 新增测试：退款交易中收/付款方式含 `&`，验证资产侧金额为正且拆分正确。
- [x] 2.4 新增测试：收/付款方式为空，验证回退到渠道名且只生成一个 Asset Posting。
- [x] 2.5 新增测试：收/付款方式无 `&`，验证行为与改动前一致。

## 3. 验证与清理

- [x] 3.1 运行 `cargo test -p accounting-service import::alipay` 确保适配器测试通过。
- [x] 3.2 运行 `cargo test -p accounting-service import_service` 确保导入服务测试通过。
- [x] 3.3 运行 `cargo clippy -p accounting-service` 检查无新增警告。
