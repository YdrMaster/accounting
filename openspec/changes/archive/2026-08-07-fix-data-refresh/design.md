# Design: fix-data-refresh

## Context

`accounting-web` 是纯手写 Vue 3 + Pinia 前端。各视图在 `onMounted` 调各自 store 的 load 方法；store 内部有缓存（transaction 的 `loadedRange`/`calendarDays`,report 的 `balanceSheet` 等）且缓存永不失效。`transaction.ts` 与 `report.ts` 已定义 `clearCache()` 但全代码库无调用方。导入路径（`channel.ts importFile` → `ConfigPanel.vue doImport`）成功后只弹 toast，不触碰任何业务 store。`transaction.create/update` 不像 `remove` 那样处理 `calendarDays`,BudgetView/SavingPlanView 只在挂载时 `loadStatuses()`。

## Goals / Non-Goals

**Goals:**

- 任何改变账目数据的操作成功后，所有依赖账目数据的视图在下次展示时是最新数据。
- 失效逻辑集中、可枚举，新增数据域时不容易漏。
- 不引入多余的重复请求：已失效的缓存按需重拉，而非每次操作后无差别全量并发刷新。

**Non-Goals:**

- 不改后端 API 契约与数据结构。
- 不引入 WebSocket/服务端推送等实时机制。
- 不处理多标签页之间的同步。

## Decisions

### D1: 用"数据版本号 + store 订阅"而非在调用点手动列举刷新

新增一个轻量模块（如 `stores/dataVersion.ts`）维护按数据域划分的失效事件，或一个 `invalidate(domain)` 函数。变更操作的执行点（store 的 create/update/remove、channel 的 importFile)只负责声明"哪个域脏了"；各消费 store 在脏标记下使缓存失效（清空本地缓存），视图层在挂载/激活时照常走"无缓存则拉取"的既有路径。

**为什么不选"在 ConfigPanel/TransactionView 里手动逐个调 store.refresh"**:explore 阶段已证实该模式是现状（CalendarView 刷了、TransactionView 没刷），新增视图时必然漏。集中声明失效域能把"谁需要刷新"收敛到一张表里。

**修正（实现时发现）**:`ResponsiveShell` 的 5 个 pane 全部常驻挂载（`v-for` + `<component :is>`，无 `v-if`),`onMounted` 只跑一次，"失效后等视图激活时重拉"永远等不到。因此编排采取**失效 + 主动静默重拉**，但只重拉"此前已加载过"的数据域（store 内用 `statusesLoaded`/`loaded`/缓存非空判断），避免多余请求。视图内的本地状态（日历 dailyStats）通过 `dataVersion` 版本号 watch 刷新。

### D2: 失效矩阵集中在 transaction/channel 等"写入方" store

失效矩阵（哪个写操作脏哪些域）：

| 写操作 | 脏数据域 |
|---|---|
| transaction create/update/remove | transactions、calendarDays、月度汇总、report(balanceSheet/cashFlow/netWorth)、budget statuses、savingPlan statuses、account 余额 |
| channel importFile | 同上一行全部（导入即批量交易变更），另加 mapping 可能新建的映射 |
| account create/update/remove | accounts、report |

实现上由写入方 store 在操作成功后调用其他 store 暴露的失效方法（`clearCache()` 已存在于 transaction/report;budget/savingPlan 需补一个状态失效或直接重拉 `loadStatuses()`;account 重拉 `loadAccounts()`)。

### D3: transaction store 内部统一 create/update/remove 的派生缓存策略

`create`/`update` 与 `remove` 对齐：操作成功后失效 `calendarDays`（让日历下次按需重拉），并对 `transactions` 列表做本地同步或失效重拉，保证三种操作行为一致。

### D4: 刷新失败降级

失效操作只清缓存、不抛错；后续的重新拉取失败不影响变更操作本身的成功反馈（toast 照弹），视图下次激活时自然重试。

## Risks / Trade-offs

- [失效范围过大导致多余请求] → 按需重拉（只清缓存，由视图激活时拉取），而不是操作后立即全量并发刷新；预算/攒钱状态因 hero 区直接展示，可在操作成功后立即静默重拉。
- [store 互相引用形成循环依赖] → 失效编排集中在一个模块（如 `stores/refresh.ts`）统一 import 各 store，业务 store 不互相 import。
- [导入大批量交易后立即重拉列表有体感延迟] → 导入完成后仅失效缓存 + 静默后台重拉当前视图数据，不阻塞 toast。
