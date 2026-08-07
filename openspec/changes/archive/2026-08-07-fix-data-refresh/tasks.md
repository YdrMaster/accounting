# Tasks: fix-data-refresh

## 1. 失效编排模块

- [x] 1.1 新建 `accounting-web/src/stores/refresh.ts`（或等价模块）,集中实现失效矩阵:`invalidateTransactionsChanged()` 与 `invalidateAccountsChanged()`,统一 import 各 store 并调用其失效/重拉方法,业务 store 之间不互相 import
- [x] 1.2 为 budget、savingPlan store 补充失效入口(失效 statuses 缓存并支持静默重拉 `loadStatuses()`);account store 确认 `loadAccounts()` 可强制重拉

## 2. transaction store 行为一致化

- [x] 2.1 `create` 成功后失效 `calendarDays`(与 `remove` 对齐),并同步/失效 `transactions` 列表
- [x] 2.2 `update` 成功后失效 `calendarDays` 并同步/失效 `transactions` 列表
- [x] 2.3 `create`/`update`/`remove` 三者统一调用失效编排,使 report、budget、savingPlan、account 余额缓存在交易变更后失效

## 3. 导入路径接通刷新

- [x] 3.1 `channel.ts` `importFile` 成功(含部分成功的 skip-on-error)后调用 `invalidateTransactionsChanged()`(或等价编排),覆盖交易、日历、月度汇总、账户、报表、预算、攒钱计划
- [x] 3.2 `ConfigPanel.vue` `doImport` 保持现有 toast 行为,确认导入后切换到交易/资产/预算视图时展示最新数据

## 4. 账户变更路径接通刷新

- [x] 4.1 账户 create/update/remove 成功后调用 `invalidateAccountsChanged()`:重拉账户列表并失效 report 缓存

## 5. 视图层收尾

- [x] 5.1 TransactionView `onFormSaved` 不再假设"store 已同步",确保月度汇总 hero 依赖的数据在保存后刷新
- [x] 5.2 BudgetView / SavingPlanView(或 PlansView)在交易变更后能看到最新 statuses(通过失效后重拉验证)
- [x] 5.3 刷新失败降级:失效只清缓存不抛错,重拉失败不影响变更操作的成功 toast

## 6. 验证

- [x] 6.1 按 spec 场景手动验证:列表页编辑交易、日历页新增/编辑交易、删除交易、导入账单(含部分失败)、修改账户,各视图数据均更新
- [x] 6.2 运行前端已有测试与 `cargo test`(如涉及共享代码),确认无回归
