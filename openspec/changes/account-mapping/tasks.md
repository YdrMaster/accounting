## 1. 领域模型

- [ ] 1.1 在 `accounting` crate 中新增 `PostingRole` 枚举（`IncomeExpense` / `Asset`），实现 `prefix()` 方法返回 `"收支"` / `"资产"`，实现 `from_key()` 从映射 key 解析，实现 `import_segment()` 返回 Import fallback 路径段
- [ ] 1.2 在 `accounting` crate 中新增 `AccountMapping` 结构体（`member_id`、`channel_id`、`category`、`account_id`）
- [ ] 1.3 改造 `BillPosting` 结构体：移除 `account_path: String`，替换为 `role: PostingRole` + `category: String`，保留 `commodity_symbol`、`amount`、`is_reimbursable`

## 2. 数据库层

- [ ] 2.1 在 `accounting-sql/src/schema.rs` 中新增 `account_mappings` 表 DDL（`member_id`、`channel_id`、`category`、`account_id`、`created_at`、`updated_at`，主键 `(member_id, channel_id, category)`，外键 member/channel ON DELETE CASCADE，外键 account 默认 RESTRICT）
- [ ] 2.2 在 `accounting-sql/src/schema.rs` 中新增 `account_mappings` 的 updated_at 触发器和索引
- [ ] 2.3 在 `accounting-sql/src/repo.rs` 中新增 `AccountMappingRow` 结构体及 CRUD 方法（`mapping_upsert`、`mapping_find`、`mapping_list`、`mapping_delete`、`mapping_count_by_account`）
- [ ] 2.4 在 `SqliteDatabase` 中暴露映射相关的公共方法

## 3. 适配器改造

- [ ] 3.1 改造 `alipay.rs`：`parse_alipay_row` 输出 `role + category` 替代 `account_path`，收支侧 Posting 的 `role=IncomeExpense, category=交易分类字段`，资产侧 Posting 的 `role=Asset, category=收/付款方式字段`
- [ ] 3.2 修正 `alipay.rs` 金额方向：支出交易中收支侧为正、资产侧为负；收入交易中收支侧为负、资产侧为正；退款遵循收入方向
- [ ] 3.3 更新 `alipay.rs` 测试用例，按新的 BillPosting 结构和修正后的金额方向编写断言

## 4. ImportService 改造

- [ ] 4.1 在 `ImportService` 中新增 `resolve_account_id` 方法：构造映射 key → 查映射表 → 有映射返回 AccountId → 无映射构造 Import fallback 路径并 ensure_cascading
- [ ] 4.2 改造 `submit_entry` 方法：对每个 BillPosting 调用 `resolve_account_id` 获取 account_id，替代原有的 `ensure_cascading(bp.account_path)`
- [ ] 4.3 更新 `ImportService` 测试用例，覆盖有映射、无映射、部分映射等场景

## 5. MappingService

- [ ] 5.1 在 `accounting-service` 中新增 `mapping_service.rs`，实现 `set`（通过 account_path 查找 AccountId + upsert）、`list`、`delete` 方法
- [ ] 5.2 `set` 方法：接受 account_path 字符串，使用 find_by_path 查找 AccountId，账户不存在返回 AccountNotFound 错误
- [ ] 5.3 为 `MappingService` 编写单元测试

## 6. 删除保护

- [ ] 6.1 在 `AccountService` 的账户删除流程中新增 `account_mappings` 引用检查，被映射引用时返回 "该账户被账户映射引用，请先删除相关映射" 错误
- [ ] 6.2 为删除保护编写测试

## 7. CLI

- [ ] 7.1 在 `accounting-cli` 中新增 `mapping` 子命令，支持 `set`、`list`、`delete` 三个子操作
- [ ] 7.2 `mapping set --member <ID> --channel <名称> --category <映射key> --account <账户路径>`：调用 MappingService::set
- [ ] 7.3 `mapping list --member <ID> --channel <名称>`：以表格形式输出映射列表
- [ ] 7.4 `mapping delete --member <ID> --channel <名称> --category <映射key>`：调用 MappingService::delete
- [ ] 7.5 为 CLI mapping 子命令编写集成测试