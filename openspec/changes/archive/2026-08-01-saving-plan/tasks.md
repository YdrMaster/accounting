# Tasks: saving-plan

## 1. 核心域模型（accounting crate）

- [x] 1.1 `accounting/src/budget.rs`：`Budget` 结构体 `period` 改为 `Option<FinancePeriod>`、新增 `deadline: Option<NaiveDate>`；`BudgetError` 新增 `AccountNotExpense(AccountId)` 变体（含中文错误信息）；`validate_budget` 新增「限额账户须位于 Expenses 根子树」校验（通过闭包表/祖先查询判断）
- [x] 1.2 新增 `accounting/src/saving_plan.rs`：`SavingPlanId`（`define_id!`）、`SavingPlan` 结构体（id/period: Option/deadline: Option/commodity_id/target_amount）、`SavingPlanError` 枚举、`validate_saving_plan`（名非空、账户集合 ≥1、账户存在、无重复、target>0、账户须位于 Assets 根子树）；在 `lib.rs` 注册模块
- [x] 1.3 核心域单元测试：validate_budget 新校验（支出账户通过/资产账户拒绝）、validate_saving_plan 全部规则、Option period/deadline 字段行为

## 2. SQL 层（accounting-sql crate）

- [x] 2.1 `schema.rs`：`budgets` 表 DDL 改为 `period INTEGER NULL` + 新增 `deadline TEXT NULL`；`SCHEMA_STATEMENTS` 末尾追加 `saving_plans`、`saving_plan_accounts`（PK (plan_id, account_id)，ON DELETE CASCADE）、`saving_plan_names` 三表 DDL + 索引 + updated_at 触发器（镜像 budget 三表模式）
- [x] 2.2 ~~幂等迁移块~~（已取消：确认无存量数据，新 schema 直接生效，旧库文件删除重建即可）
- [x] 2.3 `names.rs`：新增 `SAVING_PLAN_NAMES: EntityNames` 常量；`database.rs` 宏行新增 `saving_plan_display_names`
- [x] 2.4 新增 `repo/saving_plan.rs`：saving_plans/saving_plan_accounts 的 CRUD（镜像 `repo/budget.rs`，period/deadline 可空序列化）
- [x] 2.5 `repo/posting.rs`：新增 `account_balance_by_ids(account_ids, commodity_id, as_of: NaiveDate)`——闭包表展开后代、`t.date_time <= end_of_day(as_of)`、按 commodity 过滤的余额合计；改造 budget 实际值查询支持 period 为空（不限下界，上界 min(date, deadline)）
- [x] 2.6 `repo/budget.rs`：budget CRUD 适配 period 可空 + deadline 列读写
- [x] 2.7 `database.rs`：`SqliteDatabase` 增加 `saving_plan_*` 包装方法；`budget_*` 方法签名适配
- [x] 2.8 SQL 层测试：新表 CRUD、可空字段读写往返、`account_balance_by_ids` 含后代与日期上界

## 3. Service 层（accounting-service crate）

- [x] 3.1 新增 `report/saving_plan.rs`：`SavingPlanService` CRUD（事务、先校验）+ `get_saving_plan_status(plan_id, date)`：expired 判定（date > deadline）、period 非空时计算 period_start/end、余额聚合调 `account_balance_by_ids`、gap/met 计算；`report/mod.rs` 注册
- [x] 3.2 `report/budget.rs`：适配 period 可空（一次性预算窗口=全部历史至 min(date, deadline)）、deadline 失效判定、`BudgetStatus` 增加 `expired: bool`、`period_start/period_end` 改 `Option<NaiveDate>`
- [x] 3.3 Service 层测试：攒钱计划状态（多账户合并、含后代、gap/met、expired、period 为 None）、预算一次性窗口与 expired、两类账户类型限制在 create/update 生效

## 4. API 层（accounting-api crate）

- [x] 4.1 `dto.rs`：SavingPlan DTO（period/deadline 可空字符串、target_amount、account_ids）、SavingPlanStatusDto（expired/period_start/end 可空/target/balance/gap/met）；Budget DTO 增加 deadline、period 改可空；BudgetStatusDto 增加 expired、period_start/end 可空；period/deadline 解析辅助函数
- [x] 4.2 新增 `handlers/saving_plan.rs`：GET/POST `/api/saving-plans`、GET/PUT/DELETE `/api/saving-plans/:id`、GET `/api/saving-plans/:id/status?date=`，错误映射与 budget handlers 一致（201/200/400/404）；`router.rs` merge
- [x] 4.3 `handlers/budget.rs`：适配新 DTO（deadline、period 可选）、status 响应含 expired
- [x] 4.4 API 集成测试：saving-plan 全端点（含 400 非资产账户、expired=true 场景）、budget 回归（旧客户端 body 不含 deadline/period 行为）

## 5. CLI 层（accounting-cli crate）

- [x] 5.1 新增 `cmd/saving_plan.rs`：`saving-plan create/list/show/update/delete`（--period 可选、--deadline 可选、--target、--account 可多次、show 显示 target/balance/gap/met 与「已失效」标注）；`cmd/mod.rs` 与 `main.rs` 注册；`resolver.rs` 增加 `resolve_saving_plan`
- [x] 5.2 `cmd/budget.rs`：create/update 增加 `--deadline`（update 支持 `none` 清除）、`--period` 改可选；show 对过期预算显示「已失效」
- [x] 5.3 locales（zh-CN/en）：saving-plan 全部词条 + budget 新增词条
- [x] 5.4 CLI 测试：saving-plan 各子命令、--deadline 设置/清除、一次性计划显示

## 6. 端到端验证

- [x] 6.1 `cargo fmt` + `cargo clippy` 无警告 + `cargo test --workspace` 全绿
- [x] 6.2 删除旧 `my.db`（无迁移，旧 schema 不再兼容），用 CLI 新建库验证 budget/saving-plan 命令可用
- [x] 6.3 手工场景验收：创建旅行基金（period 空 + deadline + 双账户 target 5000）与房租（monthly + 单账户 6000），show 输出符合预期

## 7. 终审 follow-up 修复

- [x] 7.1 repo 层事务化：`repo/budget.rs` 与 `repo/saving_plan.rs` 的 create/update 包进显式事务（sqlx `conn.begin()`/`commit()`），含 update 失败回滚的原子性测试
- [x] 7.2 budget delete 不存在返回 404（对齐 saving-plan delete 的预检做法），补 `delete_budget_not_found` 集成测试
- [x] 7.3 CLI `--period once` 表示一次性：create/update 均支持（update 可清回一次性），budget 与 saving-plan 一致，help 文案与 locales 同步
