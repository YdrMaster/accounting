# Tasks: 支付宝导入按交易状态判定退款

## 1. 核心类型改造

- [x] 1.1 `accounting-service/src/import/mod.rs`：`BillPosting` 新增 `is_refund: bool` 字段（含文档注释：源数据是否为退款事件）
- [x] 1.2 `accounting/src/posting_role.rs`：`to_key` / `fallback_root` 签名增加 `is_refund: bool` 参数，IncomeExpense 规则改为 `is_refund || amount > 0 → Expenses`，否则 `Income`；删除 `is_refund_category` 函数；更新模块内单元测试（`fallback_root` 同时移除了不再使用的 `category` 参数）

## 2. 适配器与调用点

- [x] 2.1 `accounting-service/src/import/alipay.rs`：`is_refund = status == "退款成功"`（精确匹配），替换 `category == "退款"`；构造两处 BillPosting 时填充 `is_refund`（Asset 侧可恒为 false）；更新适配器测试，新增"真实分类退款行"用例
- [x] 2.2 `accounting-service/src/import_service.rs`：`resolve_account_id` 的两个调用点传入 `is_refund`；更新 import_service 测试（含退款行断言），新增 service 级"真实分类退款落 Expenses 且金额为负"用例
- [x] 2.3 `accounting-service/src/import/alipay.rs`：`parse_datetime` 支持新版导出格式（斜杠、月日不补零、无秒，如 `2026/7/25 11:43`）——试导入时发现新版账单全部 270 行因日期解析失败被跳过，经确认并入本 change；新增对应测试用例

## 3. 验证

- [x] 3.1 `cargo test -p accounting -p accounting-service` 全部通过（77 + 81）
- [x] 3.2 用 `.ignore/` 下两份真实账单做试导入（测试库），确认 53 行退款成功交易全部为负支出、落到 Expenses 侧账户（文件一：退款×26（含 7 笔 0 元）+ 日用百货×9；文件二：餐饮美食×9 + 退款×8 + 日用百货×1）
- [x] 3.3 全 workspace `cargo build` 确认无其他 BillPosting / to_key 调用点遗漏
