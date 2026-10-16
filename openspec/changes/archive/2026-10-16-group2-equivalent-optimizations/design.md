# Design: group2-equivalent-optimizations

## Context

基线：`cargo check --workspace --all-targets` 零警告；工作树干净（仅 OPTIMIZATIONS.md 未跟踪）。后端 handler 响应枚举 `BudgetResponse`（budget.rs:31-36）与 `SavingPlanResponse`（saving_plan.rs:32-37）的 `Ok` 变体为 `Json<serde_json::Value>`，迫使 10 处 `serde_json::to_value(...).unwrap()`；budget.rs 有 2 个以枚举作错误类型的辅助函数（`budget_names`、`parse_status_date`），saving_plan.rs 的对应函数返回 `String` 错误——两者不对称。sqlx workspace 依赖现带默认 features 且含未使用的 `chrono`。相关 spec 行为约束见 `ui-polish`（品牌项为本次唯一新增行为）。

## Goals / Non-Goals

**Goals:**

- 消除全部 `to_value().unwrap()` 双重序列化路径，handler 直发 `Json<T>`
- 收缩 sqlx 依赖面；重复代码收敛到单一实现；存在性判断查询走索引友好的短路形式
- 前端标识正式化（标题、favicon、`document.lang`）
- 全程行为保持：现有测试（Rust + 前端）不改断言即通过（品牌项除外）

**Non-Goals:**

- 不做 database.rs ↔ repo 约 200 个包装方法的宏生成（分组 2.3 #8，下一轮）
- 不引入统一 `AppError`（分组 4.2 方案，横切全部 handler，另行立项）
- 不修 `channel_count_transactions_by_id` 的计数语义（行为变化，另行处理）
- 不新增任何测试（分组 2 原则）

## Decisions

### D1：响应枚举泛型化 + 辅助函数错误类型统一为 String

`enum BudgetResponse<T: Serialize> { Created(Json<T>), Ok(Json<T>), NotFound(String), BadRequest(String) }`（`SavingPlanResponse` 同构）。`map_error` 泛型化为 `fn map_error<T: Serialize>(e: AccountingError) -> BudgetResponse<T>`——NotFound/BadRequest 变体不引用 `T`，调用点由函数返回类型推断，无需 turbofish。`budget_names`、`parse_status_date` 改为返回 `Result<_, String>`（与 saving_plan.rs 的 `plan_names`/`parse_request` 对齐），调用点自行 `map_err` 包成 `BadRequest`——顺带完成两文件同构样板的收敛（分组 2.3 #7）。
否决 `BudgetResponse<Infallible>` 占位：能编译但读者需解释占位语义；改 String 后占位彻底消失。否决保留非泛型枚举 + 手写两个变体：变体组合爆炸（`Ok(Vec)`/`Ok(单条)`/`Created`）。
测试侧：`respond` 改 `async fn respond<T: Serialize>(resp: BudgetResponse<T>)`；`delete_budget` 的 `json!({"deleted": true})` 保持 `serde_json::Value` 载荷（`BudgetResponse<serde_json::Value>`），不为此新建结构体。

### D2：共享 handler helper 落 `handlers/common.rs`

`ApiError`、`parse_status_date`（两文件完全同构）移入 `handlers/common.rs`；names 解析 helper 签名统一为 `async fn x_names(db, ids, lang) -> Result<HashMap<Id, String>, String>`。`map_error` 保留在各文件（判定的错误变体不同：Budget vs SavingPlan），不强行抽象。否决合并为一个带谓词参数的泛型 map_error：两处 8 行，抽象收益低于可读性损失。

### D3：sqlx features 精简（已预验证）

```toml
sqlx = { version = "0.9", default-features = false,
         features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
```

依据：`sqlx::migrate!` 需要 macros+migrate（accounting-auth/src/db/mod.rs:32）；全仓无编译期 `query!` 宏、无 `sqlx::types::Json`、无 any 驱动；日期一律手动转 String 后 bind，chrono feature 无消费者。2026-10-16 已以临时改动跑通 `cargo check --workspace --all-targets`（2m43s 零错误）后还原。否决逐 crate 收紧 features：workspace 统一声明已够，粒度收益不抵维护成本。

### D4：存在性判断 `COUNT(*)` → `SELECT 1 ... LIMIT 1`，范围 5 处

探索期全仓 20 处 `COUNT(*)` 逐一核实：生产代码中仅 5 处计数值只用于 >0 / ==0 判断——`posting_has_postings`（repo/posting.rs:104，`Ok(count > 0)`）、`initialize`（database.rs:104，`count == 0`）、`is_initialized`（database.rs:121，`count > 0`）、`names.rs` `ensure_available` 两个 SQL 分支（:178-190 父账户作用域、:192-201 全局，`count > 0`，唯一性校验热路径）。统一改为 `SELECT 1 FROM ... WHERE ... LIMIT 1` + `fetch_optional().is_some()`。
不改：`account_mapping.rs:126`、`channel.rs:158`、`channel_path.rs:99`（真计数，返回值即数量）；schema.rs:900-1024、repo/saving_plan.rs:761、repo/transaction.rs:319、repo/commodity.rs:205（`#[cfg(test)]` 内断言具体数量）。`ensure_available` 的 `count > 0` 判定语义与 `is_some()` 完全等价，行为不变。

### D5：`posting_sum_by_*` 参数化合并

抽内部函数 `async fn posting_sum_grouped(conn, filter, lang, first_col_expr: &str, extra_join: &str) -> Result<Vec<(TagId|MemberId|ChannelId 共同底层 i64, CommodityId, String, Decimal)>, DbError>`，三薄封装传入各自的 `tt.tag_id` / `t.member_id` / `cp.channel_id` 首列与 `JOIN transaction_tags` / `""` / `JOIN channel_paths` 子句，返回前各自映射为对应 ID 新类型。字面量子句由代码内常量拼接（与现状 QueryBuilder 用法一致，非用户输入路径）。channel 变体的 JOIN 逐字保留——不引入 DISTINCT，行数扇出与现有 SUM 结果完全一致（行为不变红线）。

### D6：`get_precision` 与 deadline helper 收敛位置

精度查询收敛为 `pub(crate) async fn commodity_precision(conn, CommodityId) -> Result<Option<i64>, DbError>`，放 `repo/commodity.rs`；替换 repo/posting.rs:899、repo/saving_plan.rs:51、repo/budget.rs:71,178 四个同形查询。budget.rs:302 经 `JOIN budgets` 按预算 id 查精度的变体保留原形（查询形状不同，强行统一需两套参数）。deadline：`format_deadline` / `parse_deadline` 提至 repo 公共模块（随 D5/D6 落点定为 `repo/mod.rs` 的公共项或独立 `repo/util.rs`，实施时以现有模块布局定），`parse_deadline` 入参统一为 `&str`（budget 版本现为 `String`，收窄为改签名调用点加 `&`，无行为影响）。

### D7：退款分录校验抽公共函数

`transaction_service.rs` 内新增 `async fn validate_linked_postings(tx: &SqliteTransaction, postings: &[Posting]) -> Result<(), AccountingError>`，逐字替换 create（:55-100）与 update（:154-199）两处循环。两处调用时机严格保持：create 在插入分录前、update 在删旧插新之间——均在事务内执行，`posting_get` 读到的是同事务可见状态。否决顺带改为批量取回 linked postings：那是分组 4.1 #3 的优化，会改变查询次数，越出等价边界。

### D8：CLI saving_plan list 复用首次查询

service 新增 `pub async fn list_saving_plan_statuses_for(&self, plans: &[SavingPlan], date: NaiveDate)`，`compute_allocations` 改为接受计划入参；现有 `list_saving_plan_statuses(date)` 委托为「先 list 再调 _for」，外部行为不变。CLI `cmd/saving_plan.rs list` 改调 `_for` 复用 :169 已取的计划列表。
否决「直接从 statuses 取 plan 拼列表」：statuses 可能不含非参与计划（无检查点等），会静默丢行，改变输出。

### D9：CLI tx.rs 拆分粒度

`TxCmd::run` 的每个 match arm 抽为独立 `async fn run_add / run_list / run_show / run_delete / run_update / run_reconcile`（签名统一 `(&SqliteDatabase 或 db 所有物, args, format, lang)`，按所有权实际调整）；三份逐字相同的 `ChannelPathRow` 映射闭包合并为 `fn channel_path_rows(paths: &[ChannelPathNode], names: &HashMap<ChannelId, String>) -> Vec<ChannelPathRow>`（run_show 内两处 :167-177 与 :181-193 也用它）。仅移动代码，不改输出格式。

### D10：品牌项实现

- `index.html`：`<title>图论记账</title>`；`<link rel="icon" type="image/svg+xml" href="/favicon.svg">` + `<link rel="icon" type="image/png" sizes="32x32" href="/favicon.png">`（PNG 兜底 Safari，SVG 优先）；删除 `public/vite.svg`（先确认无其他引用）。
- `favicon.svg`：64 viewBox，方案 A 五节点几何，全 `#646cff`；顶点中空（`r=6, stroke-width=4`），`<mask>` 以 `r=10` 圆挖断两腿形成 2 单位透明间隙环；其余四节点实心（横梁交点 r=4、两脚 r=5）、笔画宽 4.5 圆帽。源码定稿已与用户逐参数确认（`/tmp/icon-draft.html` E 卡片）。
- `favicon.png`（32×32）：优先 `rsvg-convert`/`resvg` 光栅化，退化 ImageMagick；工具链均不可用时与用户确认跳过 PNG（SVG 在 Chrome/Firefox/Edge 下自足）。
- lang 同步：`src/i18n.ts` `setLocale`（:44）内加 `document.documentElement.lang = locale`；初始化路径（`getInitialLocale` 消费处）同样设置。不改 i18n.spec.ts 现有断言，可选地由既有测试自然覆盖。
- 否决 `prefers-color-scheme` 双色方案（F）：用户已选定全紫单色（E），单色 `#646cff` 对白底 4.1:1、对 #1e1e1e 4.1:1，两侧均衡。

## Risks / Trade-offs

- **泛型化使每个 handler 返回具体类型不同**：router 注册只依赖 `IntoResponse`，无影响；clippy `use_self`/`uninlined_format_args` 为 deny 级，实施时注意枚举内部 `Self::` 写法。最大风险点是测试辅助与 `map_error` 的推断链，若某处推断失败就地 turbofish 标注，不改设计。
- **SQL 改写等价性**：`SELECT 1 ... LIMIT 1` 与 `COUNT(*)>0` 在 SQLite 语义等价（同一行可见性、同一事务模型）；D5 合并必须逐字保留 JOIN 与 GROUP BY，实施后以现有 report 测试为准。
- **D8 触及 service 内部结构**：`compute_allocations` 改签名波及调用方仅 2 处（CLI 与原方法委托），编译器兜底。
- **PNG 工具链缺失**：降级路径明确（跳过并告知），不阻塞其余任务。
- **前端 lang 同步的测试盲区**：vitest 环境下 `document.documentElement.lang` 可断言但现有 spec 未覆盖；按分组 2 原则不新增测试，靠手测验收。

## Migration Plan

无 schema 变化、无数据迁移、无 API 契约变化。sqlx feature 收缩只影响编译期。部署即生效，可随时回滚（git revert 单 commit）。

## Open Questions

- favicon.png 光栅化工具在本机是否可用（任务内探测，缺失则走跳过路径）。
- `repo` 公共 helper 的落点（`repo/mod.rs` vs `repo/util.rs`）视现有模块布局定，实施时顺现行约定，不再回询。
