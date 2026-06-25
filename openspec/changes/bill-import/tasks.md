## 1. accounting-import crate 骨架

- [ ] 1.1 在 workspace Cargo.toml 中添加 `accounting-import` member，创建 crate 目录和 `Cargo.toml`（依赖 `accounting`、`chrono`、`rust_decimal`、`thiserror`）
- [ ] 1.2 定义 `BillAdapter` trait：`fn name(&self) -> &str`、`fn parse(&self, data: &[u8], ctx: &ImportContext) -> Result<Box<dyn Iterator<Item = Result<BillEntry, AdaptError>>>, AdaptError>`
- [ ] 1.3 定义 `ImportContext` 结构体：`member_id: MemberId`、`channel_id: ChannelId`、`commodity_id: CommodityId`
- [ ] 1.4 定义 `BillEntry` 结构体：`date_time: NaiveDateTime`、`description: String`、`kind: TransactionKind`、`postings: Vec<BillPosting>`、`tags: Vec<String>`
- [ ] 1.5 定义 `BillPosting` 结构体：`account_path: String`、`commodity_symbol: String`、`amount: Decimal`、`is_reimbursable: bool`
- [ ] 1.6 定义 `AdaptError` 错误类型：`RowError { row: usize, message: String }`、`FormatError(String)` 等 variant，实现 `std::error::Error` 和 `Display`
- [ ] 1.7 定义适配器注册机制：`fn builtin_adapters() -> Vec<Box<dyn BillAdapter>>` 返回所有内置适配器列表；`fn find_adapter(name: &str, adapters: &[Box<dyn BillAdapter>]) -> Option<&dyn BillAdapter>`
- [ ] 1.8 编写 unit test：构造 `Vec<u8>` 测试数据验证 `find_adapter` 按 name 查找适配器

## 2. 支付宝适配器

- [ ] 2.1 实现 `AlipayAdapter` 结构体，实现 `BillAdapter` trait
- [ ] 2.2 `parse` 内部将 `&[u8]` 转为 UTF-8 字符串，按行解析 CSV 格式，跳过表头和非数据行
- [ ] 2.3 解析支付宝 CSV 列：交易时间、交易分类、交易对方、商品说明、金额、收/支等，映射到 `BillEntry`
- [ ] 2.4 生成 `BillPosting`：支出侧 `account_path = "Import:支付宝:<分类>"`，金额为负；资产侧 `account_path = "Import:支付宝"`，金额为正
- [ ] 2.5 退款行（金额 > 0 且标记为收入）映射为 `TransactionKind::Refund`
- [ ] 2.6 解析错误时返回 `AdaptError::RowError { row, message }`，不 panic
- [ ] 2.7 编写 unit test：构造最小 CSV 测试数据，验证 `AlipayAdapter::parse` 输出的 `BillEntry`
- [ ] 2.8 更新 `accounting-import/Cargo.toml` 添加 CSV 解析所需依赖（如 `csv` crate 或手写解析）

## 3. seed data 更新

- [ ] 3.1 在 `accounting-sql/src/schema.rs` 的 `SEED_ACCOUNTS_ROOT_EN` / `SEED_ACCOUNTS_ROOT_ZH` 中添加 `Import` / `导入` 系统根账户（`is_system=1, parent_id=NULL`）
- [ ] 3.2 在 `SEED_TAGS_EN` / `SEED_TAGS_ZH` 中添加 `pending` / `待处理` 系统 Tag（`is_system=1`，description 说明用途）
- [ ] 3.3 更新 `test_seed_data` 测试中的计数断言以反映新增的 seed 数据
- [ ] 3.4 验证 `SEED_CLOSURE` 递归 CTE 会自动覆盖新增的 Import 根账户（因为 `WHERE is_system = 1` 已包含）

## 4. AccountService 扩展 — ensure_cascading

- [ ] 4.1 在 `accounting-service/src/account_service.rs` 中新增 `ensure_cascading` 方法：与 `create_cascading` 类似但跳过 `AccountType::from_str` 校验，支持在系统根账户（如 Import）下创建子账户
- [ ] 4.2 `ensure_cascading` 签名：`pub async fn ensure_cascading(&self, path: &str) -> Result<AccountId, AccountingError>`
- [ ] 4.3 实现逻辑：按 `:` 分割路径，逐级查找/创建账户（`is_system=false`），仅当路径首段已存在且为系统账户时跳过 AccountType 校验；首段不存在时报错
- [ ] 4.4 编写 unit test：验证 `ensure_cascading("Import:支付宝:餐饮美食")` 能在 Import 系统根账户下创建三级子账户
- [ ] 4.5 编写 unit test：验证重复调用 `ensure_cascading` 返回相同 AccountId（幂等）

## 5. ImportService

- [ ] 5.1 在 `accounting-service/src/` 中新建 `import_service.rs`，定义 `ImportService { db: SqliteDatabase }`
- [ ] 5.2 定义 `ImportResult` 结构体：`transaction_ids: Vec<TransactionId>`、`imported: usize`、`skipped: usize`、`errors: Vec<AdaptError>`
- [ ] 5.3 实现 `import` 方法：接受 `data: &[u8]`、`source: &str`、`member_id: MemberId`，返回 `ImportResult`
- [ ] 5.4 import 流程：(1) 从 `builtin_adapters()` 中按 `source` 查找适配器 (2) 通过 `channel_get_by_name` 或类似方式解析 source 为 ChannelId (3) 查找或使用默认 CommodityId (4) 构造 ImportContext (5) 调用 `adapter.parse(data, &ctx)` (6) 迭代 BillEntry 逐条处理
- [ ] 5.5 每个 BillEntry 处理：(1) 对每个 BillPosting.account_path 调用 `ensure_cascading` 创建/查找账户 (2) 将 commodity_symbol 解析为 CommodityId (3) 构建 Transaction + Postings (4) 解析 tags 名称为 TagId（含 "待处理" 系统 Tag） (5) 构建 ChannelPathNode (6) 调用 `TransactionService::submit`
- [ ] 5.6 skip-on-error：当 `BillEntry` 迭代返回 `Err(AdaptError)` 时，记录到 `errors` 列表，继续迭代；submit 失败时同样记录错误继续
- [ ] 5.7 更新 `accounting-service/src/lib.rs` 添加 `pub mod import_service;`
- [ ] 5.8 更新 `accounting-service/Cargo.toml` 添加 `accounting-import` 依赖
- [ ] 5.9 编写 integration test：构造内存 CSV + 内存 SQLite，完整运行 ImportService::import 并验证返回的 transaction_ids、Import 子账户创建、"待处理" Tag 关联

## 6. CLI import 子命令

- [ ] 6.1 在 `accounting-cli/src/cmd/` 中新建 `import.rs`，定义 `ImportArgs`（`--file <PathBuf>`、`--source <String>`、`--member <i64>`）
- [ ] 6.2 实现 `ImportCmd::run`：读取文件字节、调用 `ImportService::import`、格式化输出摘要（imported/skipped/errors）和 transaction_ids 列表
- [ ] 6.3 在 `accounting-cli/src/cmd/mod.rs` 中添加 `pub mod import;`，在 `Commands` 枚举中添加 `Import(import::ImportCmd)` 变体
- [ ] 6.4 在 `accounting-cli/src/main.rs` 的 match 分发中添加 `Commands::Import(cmd) => cmd.run(db, cli.format).await`
- [ ] 6.5 更新 `accounting-cli/Cargo.toml` 添加 `accounting-import` 依赖
- [ ] 6.6 编写 CLI 集成测试：验证 `import --source alipay --member 1 --file test.csv` 的完整流程
- [ ] 6.7 错误处理：不支持的 source、文件不存在、Channel 不存在等场景的友好错误输出

## 7. 验证与收尾

- [ ] 7.1 运行 `cargo fmt` 确保所有新代码格式化
- [ ] 7.2 运行 `cargo clippy` 确保零警告
- [ ] 7.3 运行 `cargo test` 确保所有测试通过（含新增测试和现有回归测试）
- [ ] 7.4 验证 `accounting import --source alipay --member 1 --file <path>` 端到端工作正常