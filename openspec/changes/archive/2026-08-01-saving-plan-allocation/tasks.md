# Tasks: saving-plan-allocation

## 1. SQL 层：按账户分组余额

- [x] 1.1 `accounting-sql/src/repo/posting.rs`：新增 `account_balances_by_ids(account_ids, commodity_id, as_of) -> Vec<(AccountId, Decimal)>`——每个指定账户各自（含后代、闭包表展开、仅本币、`t.date_time <= end_of_day(as_of)`）的余额，按账户分组返回（不做跨账户去重）
- [x] 1.2 `accounting-sql/src/database.rs` + `transaction.rs`：对应包装方法
- [x] 1.3 SQL 层测试：多账户分组余额、含后代、币种过滤、日期上界、空集合

## 2. Service 层：全局分配算法

- [x] 2.1 `accounting-service/src/report/saving_plan.rs`：新增内部全局分配计算 `compute_allocations(date)`——筛选参与计划（未过期且有检查点；永久/过期计划排除）、按 commodity 分组、按（检查点, plan_id）升序、顺序占用（`allocated = min(target, available)`，欠费占光）、账户内分配偏好（优先与下一交集计划无关的账户，同类按账户 id 升序）
- [x] 2.2 `SavingPlanStatus` 增加 `allocated`/`satisfaction`/`accounts: Vec<SavingPlanAccountAllocation>`；`get_saving_plan_status` 改为基于全局分配（非参与计划按无竞争退化口径返回分配字段）
- [x] 2.3 新增 `list_saving_plan_statuses(date)` 复用同一计算
- [x] 2.4 Service 层测试（对照 spec 场景）：先到先得、欠费占光、跨币种隔离、周期计划检查点、永久/过期计划排除、分配偏好（A2000+B1000 经典例）、级联可用、批量与单条口径一致

## 3. API 层

- [x] 3.1 `dto.rs`：status 响应增加 allocated/satisfaction/accounts（AccountAllocationDto：account_id/balance/occupied_by_earlier/allocated，金额 string 序列化）
- [x] 3.2 `handlers/saving_plan.rs`：status handler 填充新字段
- [x] 3.3 API 集成测试：新字段形状、共享账户满足率场景、过期计划字段完整

## 4. CLI 层

- [x] 4.1 `cmd/saving_plan.rs`：show 增加满足率与每账户分配明细输出；list 增加满足率列（调 `list_saving_plan_statuses`）
- [x] 4.2 locales（zh-CN/en）：满足率、分配明细表头/标注词条
- [x] 4.3 CLI 测试：list 满足率列、show 分配明细（含经典三计划例）

## 5. 端到端验证

- [x] 5.1 `cargo fmt` + `cargo clippy --workspace --all-targets` 无警告 + `cargo test --workspace` 全绿
- [x] 5.2 CLI 冒烟：临时库复现 spec 经典例（计划 1/2/3、账户 A-E），show/list 输出与 spec 场景一致
