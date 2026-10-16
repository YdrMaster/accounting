# Proposal: group2-equivalent-optimizations

## Why

`OPTIMIZATIONS.md` 分组 2 集中了已逐条到源码核实的等价优化点：API 层 10 处 HTTP 双重序列化（多余一次 `Value` 分配 + `unwrap` panic 风险）、sqlx 依赖 feature 过宽、多组重复代码与低效查询模式（`COUNT(*)` 做存在性判断、三份 `get_precision`、约 45 行退款校验复制等）。同时前端 `<title>` 仍是脚手架名 "try-frontend"、图标是 Vite 默认图标。趁发布后窗口期一并清理，除前端品牌项外全部保持行为不变。

## What Changes

- **accounting-api**：`BudgetResponse` / `SavingPlanResponse` 泛型化（`T: Serialize`），消除 budget.rs 与 saving_plan.rs 各 5 处 `serde_json::to_value(...).unwrap()`；两文件的 names 解析、状态日期解析等同构辅助函数收敛为共享 helper，`map_error` 泛型化，测试辅助 `respond` 随之泛型化。
- **Cargo.toml**：workspace sqlx 依赖改为 `default-features = false` + `features = ["runtime-tokio", "sqlite", "macros", "migrate"]`（`chrono`/`any`/`json` 均未使用，该配置已用 `cargo check --workspace --all-targets` 预验证）。
- **accounting-sql**：
  - 5 处存在性判断 `COUNT(*)` → `SELECT 1 ... LIMIT 1`：`posting_has_postings`（repo/posting.rs:104）、`initialize` 与 `is_initialized`（database.rs:104,121）、`names.rs` `ensure_available` 两个 SQL 分支（验证中新增发现，唯一性校验热路径）。
  - 删除空函数 `create_unique_indexes`（schema.rs:21）及其调用点。
  - `posting_sum_by_tag` / `posting_sum_by_member` / `posting_sum_by_channel` 三函数参数化为一个内部函数 + 三个薄封装（仅首列与 JOIN 不同）。
  - `get_precision` 三份实现（repo/posting.rs:899、repo/saving_plan.rs:51、repo/budget.rs:71,178 内联查询）收敛为一处共享 helper。
  - `format_deadline` / `parse_deadline`（repo/budget.rs ↔ repo/saving_plan.rs 完全重复）提为公共模块函数。
- **accounting-service**：`transaction_service.rs` create/update 中约 45 行逐字重复的退款/报销分录校验块抽为公共函数。
- **accounting-auth**：`session.rs` `to_hex` 由逐字节 `format!` 改为 `String::with_capacity` + `write!`。
- **accounting-cli**：`cmd/tx.rs` 约 165 行的 `run` 拆分为子函数，合并三份 `ChannelPathRow` 映射；`cmd/saving_plan.rs` `list` 复用首次拉取的计划列表，消除 `list_saving_plan_statuses` 内部重复查询。
- **accounting-web**：`<title>` 改为「图论记账」；favicon 替换 Vite 默认图标为「字母 A × 节点图」SVG（方案 A 五节点几何、全紫 `#646cff`、透明底、顶点中空并带极窄透明间隙环）+ 32px PNG 兜底；切换语言时同步 `document.documentElement.lang`。
- **明确不做**：database.rs ↔ repo 约 200 个包装方法的宏生成（分组 2.3 #8，延后至下一轮迭代）；`channel_count_transactions_by_id` 数链路节点而非交易数的语义问题（已记录，另行处理）；其余 `COUNT(*)`（测试代码与真计数函数）不动。

## Capabilities

### New Capabilities

（无——除品牌项外均为行为保持的重构，设计决策按项目约定落在 design.md，不进活规格。）

### Modified Capabilities

- `ui-polish`: 新增「浏览器标签与应用标识」Requirement——页面标题、favicon、`document.documentElement.lang` 与界面语言同步为可观测行为。

## Impact

- **代码**：accounting-api/handlers、accounting-sql（schema、database、names、repo）、accounting-service、accounting-auth、accounting-cli、accounting-web（index.html、i18n.ts、public/）。
- **依赖**：sqlx feature 收缩，全 workspace `cargo check` 预验证通过。
- **行为**：后端无外部可观测变化；前端标题/favicon/lang 为用户可见变化（品牌修正，属本次唯一越出「行为不变」原则的部分）。
- **测试**：不新增测试（分组 2 原则）；验收为现有 `cargo test --workspace` 与 `npm test` 全绿 + clippy 零警告 + 品牌项手动核对。
