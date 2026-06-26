## 1. 核心类型 (accounting crate)

- [x] 1.1 在 `accounting/src/id.rs` 中调用 `define_id!(BudgetId)` 新增 BudgetId 类型
- [x] 1.2 创建 `accounting/src/budget.rs`，定义 BudgetPeriod 枚举（Daily=1, WeeklyFromSunday=2, WeeklyFromMonday=3, Monthly=4, Yearly=5），实现 From<i64>/Display 和 `period_range(&self, date: NaiveDate) -> (NaiveDate, NaiveDate)` 方法
- [x] 1.3 在 `accounting/src/budget.rs` 中定义 Budget 结构体（id, name, period, commodity_id）和 BudgetLimit 结构体（budget_id, account_id, amount）
- [x] 1.4 在 `accounting/src/budget.rs` 中定义 BudgetError 枚举（EmptyName, EmptyLimits, AccountNotFound, DuplicateAccount, InvalidAmount, CommodityNotFound, BudgetNotFound, DatabaseError）
- [x] 1.5 在 `accounting/src/budget.rs` 中实现 `validate_budget` 验证函数（名称非空、限额非空、账户存在、无重复、金额>0、币种存在）
- [x] 1.6 在 `accounting/src/lib.rs` 中新增 `pub mod budget;`
- [x] 1.7 为 BudgetPeriod::period_range 编写单元测试（5 种周期的边界场景）
- [x] 1.8 为 BudgetPeriod 整数↔枚举转换编写单元测试
- [x] 1.9 为 validate_budget 编写单元测试（正常+6 种异常场景）

## 2. 数据库表与种子数据 (accounting-sql crate)

- [ ] 2.1 在 `accounting-sql/src/schema.rs` 的 SCHEMA_STATEMENTS 数组中添加 budgets 表和 budget_limits 表的 CREATE TABLE 语句
- [ ] 2.2 在 `accounting-sql/src/schema.rs` 中新增 SEED_TAGS_EXCLUDE_EN 和 SEED_TAGS_EXCLUDE_ZH 常量，包含 exclude-from-income-statement 和 exclude-from-budget 两条内置标签
- [ ] 2.3 修改 `accounting-sql/src/schema.rs` 中的 `insert_seed_data` 函数，在 ZH/EN 分支中分别执行新的种子标签 SQL
- [ ] 2.4 为 budgets 和 budget_limits 表的 created_at/updated_at 添加自动更新触发器（与现有表模式一致）

## 3. BudgetRepo (accounting-sql crate)

- [ ] 3.1 创建 `accounting-sql/src/repo/budget.rs`，实现 budget_create、budget_get、budget_list、budget_update、budget_delete、budget_get_limits 六个函数
- [ ] 3.2 在 `accounting-sql/src/repo.rs` 中添加 `pub mod budget;`
- [ ] 3.3 在 `accounting-sql/src/database.rs` 中为 SqliteDatabase 添加 Budget 相关方法（budget_create, budget_get, budget_list, budget_update, budget_delete, budget_get_limits），委托到 repo::budget
- [ ] 3.4 为 BudgetRepo CRUD 编写集成测试（create+get, list, update, delete, 级联删除 budget_limits）

## 4. 预算统计查询 (accounting-sql crate)

- [ ] 4.1 在 `accounting-sql/src/repo/posting.rs` 中新增 `sum_by_account_with_descendants` 函数（通过闭包表聚合后代账户、排除指定标签交易、仅统计指定币种）
- [ ] 4.2 在 `accounting-sql/src/database.rs` 中为 SqliteDatabase 添加 sum_by_account_with_descendants 委托方法
- [ ] 4.3 为 sum_by_account_with_descendants 编写集成测试（后代聚合、排除不计预算标签、仅统计本币）

## 5. 内置标签删除保护 (accounting-sql crate)

- [ ] 5.1 修改 `accounting-sql/src/repo/tag.rs` 中的 tag_delete 函数，删除前检查 is_system，若为 true 则返回错误（参照 channel_force_delete_by_id 模式）

## 6. BudgetService (accounting-service crate)

- [ ] 6.1 创建 `accounting-service/src/budget_service.rs`，定义 BudgetDetail、BudgetStatus、BudgetItemStatus 结构体
- [ ] 6.2 实现 BudgetService::create_budget（验证 → 事务 → 插入 budgets + budget_limits）
- [ ] 6.3 实现 BudgetService::update_budget（验证存在 → 事务 → 替换 budgets + budget_limits）
- [ ] 6.4 实现 BudgetService::delete_budget
- [ ] 6.5 实现 BudgetService::list_budgets
- [ ] 6.6 实现 BudgetService::get_budget_detail
- [ ] 6.7 实现 BudgetService::get_budget_status（查询 Budget → period_range → 获取不计预算标签 ID → 对每个 limit 调用 sum_by_account_with_descendants → 计算 remaining/percentage）
- [ ] 6.8 在 `accounting-service/src/lib.rs` 中添加 `pub mod budget_service;`
- [ ] 6.9 为 BudgetService 编写测试（create_budget、get_budget_status 各周期、排除标签、历史日期）

## 7. CLI budget 命令 (accounting-cli crate)

- [ ] 7.1 创建 `accounting-cli/src/cmd/budget.rs`，定义 BudgetCmd 枚举（Create, List, Show, Update, Delete）和对应 Args 结构体
- [ ] 7.2 实现 BudgetCmd::Create（解析 --name/--period/--commodity/--limit 参数，--limit 格式为 账户路径:金额，内部查找 account_id）
- [ ] 7.3 实现 BudgetCmd::List（调用 list_budgets，表格输出 ID/Name/Period/Commodity）
- [ ] 7.4 实现 BudgetCmd::Show（调用 get_budget_status，输出周期范围和各账户执行情况，超支标注 ⚠）
- [ ] 7.5 实现 BudgetCmd::Update（解析可选参数，--limit 替换所有限额）
- [ ] 7.6 实现 BudgetCmd::Delete
- [ ] 7.7 在 `accounting-cli/src/cmd/mod.rs` 中添加 `pub mod budget;`、BudgetRow 输出类型和 Budget(BudgetCmd) 命令变体
- [ ] 7.8 在 `accounting-cli/src/cmd/mod.rs` 的 Commands 枚举中添加 Budget 变体及 match 分支

## 8. 全量验证

- [ ] 8.1 `cargo fmt --workspace`
- [ ] 8.2 `cargo clippy --workspace -- -D warnings`
- [ ] 8.3 `cargo test --workspace`
- [ ] 8.4 手动验证 CLI: `accounting my.db budget create --name "月度生活" --period monthly --commodity 1 --limit Expenses:Food:2000`
- [ ] 8.5 手动验证 CLI: `accounting my.db budget show 1 --date 2026-06-26`