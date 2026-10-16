# Tasks: group2-equivalent-optimizations

> 实施顺序即分组顺序：先最小且已预验证的 2.2，再泛型化主线（2.1 + helper 收敛），后机械清理，品牌项独立收尾。每分组结束跑一次 `cargo check --workspace --all-targets`，最终统一跑全量验收。

## 1. sqlx 依赖精简（design D3）

- [x] `Cargo.toml` workspace sqlx 改为 `default-features = false, features = ["runtime-tokio", "sqlite", "macros", "migrate"]`
- [x] `cargo check --workspace --all-targets` 零错误零警告

## 2. 响应枚举泛型化 + handler helper 收敛（design D1/D2）

- [x] `handlers/budget.rs`：`BudgetResponse<T: Serialize>` 泛型化；`map_error` 泛型化；`budget_names`、`parse_status_date` 改返回 `Result<_, String>`，调用点补 `map_err`；5 处 `to_value(...).unwrap()` 改直发 `Json(dto)`；`delete_budget` 保持 `json!` 直构
- [x] `handlers/saving_plan.rs`：`SavingPlanResponse<T: Serialize>` 同构改造；5 处 `to_value(...).unwrap()` 替换
- [x] 新建 `handlers/common.rs`：迁入两份 `ApiError` 与同构的 `parse_status_date`；names helper 统一 `Result<HashMap<Id, String>, String>` 签名
- [x] budget.rs 测试：`respond` 泛型化为 `respond<T: Serialize>(resp: BudgetResponse<T>)`；确认现有断言一字不改
- [x] `cargo check --workspace --all-targets` + `cargo test -p accounting-api` 全绿

## 3. accounting-sql 清理（design D4/D5/D6）

- [x] 存在性判断改短路查询（5 处）：`repo/posting.rs` `posting_has_postings`、`database.rs` `initialize`/`is_initialized`、`names.rs` `ensure_available` 两分支 → `SELECT 1 ... LIMIT 1` + `fetch_optional().is_some()`
- [x] 删除 `schema.rs` 空函数 `create_unique_indexes` 及调用点
- [x] `repo/posting.rs`：`posting_sum_by_tag/member/channel` 参数化为内部函数 + 三个薄封装（JOIN/GROUP BY 逐字保留）
- [x] `repo/commodity.rs` 新增 `commodity_precision(conn, CommodityId)`；替换 `repo/posting.rs:899`、`repo/saving_plan.rs:51`、`repo/budget.rs:71,178` 四处同形查询（budget.rs:302 JOIN 变体保留）
- [x] `format_deadline`/`parse_deadline` 提为 repo 公共函数（入参统一 `&str`），budget.rs 与 saving_plan.rs 改引用
- [x] `cargo check --workspace --all-targets` + `cargo test -p accounting-sql` 全绿

## 4. service / auth / cli 清理（design D7/D8/D9）

- [x] `transaction_service.rs`：抽 `validate_linked_postings`，替换 create/update 两处重复循环（调用时机不变）
- [x] `accounting-auth/src/service/session.rs`：`to_hex` 改 `String::with_capacity` + `write!`
- [x] `accounting-service`：新增 `list_saving_plan_statuses_for(plans, date)`，`compute_allocations` 接受计划入参，原方法委托
- [x] `cli/src/cmd/saving_plan.rs` `list`：改调 `_for` 复用首次计划列表
- [x] `cli/src/cmd/tx.rs`：`run` 各 match arm 拆为 `run_add/list/show/delete/update/reconcile` 子函数；三份 `ChannelPathRow` 映射合并为 `channel_path_rows`
- [x] `cargo check --workspace --all-targets` + `cargo test -p accounting-service -p accounting-cli -p accounting-auth` 全绿

## 5. 前端品牌项（design D10）

- [x] `index.html`：`<title>` 改「图论记账」；icon link 换 `favicon.svg` + `favicon.png`（32px）；删除 `public/vite.svg`（先 grep 确认无其他引用）
- [x] 新建 `public/favicon.svg`：方案 A 五节点几何、全 `#646cff`、透明底、顶点中空（r=6/描边 4）+ mask 挖 r=10 透明间隙环（定稿源码见 design D10）
- [x] 生成 `public/favicon.png`（32×32）：优先 rsvg-convert/resvg，退化 ImageMagick；均不可用则跳过并与用户说明
- [x] `src/i18n.ts`：`setLocale` 同步 `document.documentElement.lang`；初始化路径（`getInitialLocale` 消费处）同样设置
- [x] `npm test` 全绿；`npm run build` 无新增警告

## 6. 全量验收

- [x] `cargo clippy --workspace --all-targets` 零警告（workspace deny 门控生效）
- [x] `cargo test --workspace` 全绿，测试断言零修改（品牌项外）
- [x] `npm test` 全绿
- [x] 手测：浏览器打开前端（用户豁免，标题/favicon/lang 已由 dist 产物与 i18n.ts 代码路径静态验证）
- [x] 手测抽查后端行为等价：预算列表/详情/状态、攒钱计划列表、CLI `tx list`/`tx show`、`saving_plan list` 输出与改造前一致
- [x] 归档前确认 `channel_count_transactions_by_id` 语义问题与 2.3 #8 已记录在 OPTIMIZATIONS.md 待办（不实施）
