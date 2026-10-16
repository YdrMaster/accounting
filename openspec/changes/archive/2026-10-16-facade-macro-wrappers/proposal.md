# Proposal: facade-macro-wrappers

## Why

`database.rs`（127 个 pub 方法）与 `transaction.rs`（83 个）共约 1300 行手写委托方法，与 repo 层 120 个函数逐字重复，仅连接获取方式不同（`&self` + `pool.acquire()` vs `&mut self.tx`）。分组 2 收尾时的多行签名精确审计显示 ~95% 为纯 1:1 转发，例外仅有四类：budget/saving_plan 写路径 7 处 `lang` 参数位置不一致、2 处参数互换（`commodity_upsert_by_symbol`、`channel_upsert_by_name`）、mapping 系 6 处改名委托（repo `mapping_*` → 门面 `account_mapping_*`）、以及 lifecycle + 既有 `display_name_wrappers!`。每新增一个 repo 函数都需手写两份包装，且签名漂移已经发生过 9 次。同批审计还确认 `channel_count_transactions_by_id` 名不副实（数 `channel_paths` 行数而非去重交易数），其唯一生产调用方 `delete_channel` 仅用作 `count > 0` 存在性守卫，数字从不展示。

## What Changes

- **repo 签名统一（先行）**：budget/saving_plan 写路径 7 个函数的 `lang` 参数移至末尾、`commodity_upsert_by_symbol` 与 `channel_upsert_by_name` 参数顺序对齐门面约定；service/repo 内部调用点随编译器适配。宏因此只需支持「逐字转发」一种形态。
- **channel 计数语义修正（随第 2 步迁移前落地）**：repo/两侧 wrapper 的 `channel_count_transactions_by_id` 改为 `channel_in_use`（`SELECT 1 ... LIMIT 1`，`Result<bool>`）；删除零生产调用方的 `channel_path_count_by_channel`（含其单测）；`handlers/channel.rs` `delete_channel` 守卫改布尔判断。用户可见行为零变化（守卫语义等价、`channel_in_use` 报错文案不变）。
- **新建 `repo/facade.rs`**：单一清单宏生成 `SqliteDatabase` / `SqliteTransaction` 双侧委托 impl；条目语法 `name(param: Type, ...) -> Ret;` 与改名变体 `name(...) -> Ret as repo_fn;`；清单置于门面契约专属文件，不混入 repo 根模块。
- **按域分批替换手写 wrapper**：member 域试点（验证宏展开、rust-analyzer 跳转、clippy 兼容）→ account → commodity → channel → channel_path → tag → attachment → transaction → posting → budget → saving_plan → mapping，每批 `cargo test --workspace` 全绿后推进；若试点发现 IDE 跳转不可用，降级为只生成 db 侧（tx 侧保留手写）。
- **保持手写**：lifecycle（`open`/`open_in_memory`/`initialize`/`is_initialized`/`transaction`/`commit`）与 `display_name_wrappers!` 生成的 7 个显示名方法。
- **明确不做**：tx 侧缺失的报告类方法不借机补齐（宏清单 = 现有手写方法的精确集合）；`account_ensure_rename_allowed` 维持 repo 内部不暴露。

## Capabilities

### New Capabilities

（无——纯内部结构与命名收敛，skip_specs：宏展开与原手写逐字等价，channel 守卫语义与报错文案不变，均无可观测行为。）

### Modified Capabilities

（无）

## Impact

- **代码**：accounting-sql（9 个 repo 函数签名、`facade.rs` 新建、`database.rs`/`transaction.rs` 大幅缩减、`channel_in_use` 替换两个计数函数）、accounting-api（`delete_channel` 守卫一处）、accounting-service（签名适配调用点）。
- **行为**：零外部可观测变化。
- **测试**：不新增测试；`channel_path_count_by_channel` 的单测随函数删除；现有测试断言零修改。
- **验收**：审计脚本核对宏清单条目数 == 删除的手写方法数；`cargo test --workspace` 与 clippy 全绿。
