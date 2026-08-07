# Spec: data-refresh

## ADDED Requirements

### Requirement: 交易变更后全量数据域刷新

当一笔交易被创建、更新或删除成功后，前端 SHALL 失效并重新加载所有依赖账目数据的展示数据，而不仅是交易列表本身。受影响的数据域 MUST 至少包括：交易列表、日历日统计（calendarDays)、月度汇总、资产负债表/收支报表、预算状态、攒钱计划状态、账户余额。

#### Scenario: 在交易列表页编辑一笔交易

- **WHEN** 用户在 TransactionView 通过表单修改一笔交易并保存成功
- **THEN** 交易列表、月度汇总 hero、预算状态、攒钱计划状态与报表缓存均被刷新为最新数据，无需手动重新进入页面

#### Scenario: 在日历视图编辑一笔交易

- **WHEN** 用户在 CalendarView 通过表单修改或新增一笔交易并保存成功
- **THEN** 日历对应日期的统计数据（calendarDays）被失效并重新加载，日历格子显示最新金额

#### Scenario: 删除一笔交易

- **WHEN** 用户删除一笔交易成功
- **THEN** 交易列表、日历、月度汇总、预算、攒钱计划与报表均反映删除后的数据

### Requirement: 交易 store 缓存行为一致

transaction store 的 `create`、`update` 与 `remove` MUST 对派生缓存（calendarDays、月度汇总等）采取一致的失效或同步策略，不允许出现 `remove` 清空日历缓存而 `create`/`update` 不清空的不一致行为。

#### Scenario: 新增交易后日历不残留旧状态

- **WHEN** 用户新增一笔属于当前已加载日历月份的交易
- **THEN** 该月 calendarDays 缓存被失效或同步更新，再次查看日历时数据正确

### Requirement: 账户变更后关联数据刷新

当账户被创建、修改或删除成功后，前端 SHALL 刷新账户列表以及依赖账户余额的展示数据（资产负债表、净资产趋势等报表缓存）。

#### Scenario: 修改账户后资产视图更新

- **WHEN** 用户在 AccountDrawer 中修改账户信息并保存成功
- **THEN** 账户网格与 AssetsView 中的报表数据在下次展示时为最新值

### Requirement: 刷新失败不阻断操作反馈

数据刷新（重新加载）的失败 MUST NOT 掩盖变更操作本身的成功反馈；刷新失败时 SHOULD 保留操作成功的提示，并可降级为下次进入视图时重新加载。

#### Scenario: 刷新请求失败

- **WHEN** 交易保存成功但随后的报表刷新请求失败
- **THEN** 用户仍看到保存成功的提示，已失效的缓存在下次访问对应视图时重新拉取
