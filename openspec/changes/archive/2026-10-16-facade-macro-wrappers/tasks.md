# Tasks: facade-macro-wrappers

> 实施顺序即分组顺序：签名统一 → channel 修正 → 宏试点 → 分域迁移 → 收尾审计。每分组结束 `cargo check --workspace --all-targets`，关键节点全量测试。

## 1. repo 签名统一（design D3）

- [x] budget 写路径 3 函数（create/update/upsert_by_name）：`lang` 从第 2 参数移至末尾
- [x] saving_plan 写路径 3 函数（create/update/upsert_by_name）：同上
- [x] `commodity_upsert_by_symbol` 参数对齐为 `(symbol, name, precision, lang)`
- [x] `channel_upsert_by_name` 参数对齐为 `(name, description, account_id, lang)`
- [x] service/repo 内部调用点随编译器适配（脚本重排 57 处调用点 + 2 处手工修正；wrapper 公开签名与 service 调用方零改动）
- [x] `cargo check --workspace --all-targets` + `cargo test --workspace` 全绿

## 2. channel 计数语义修正（design D4）

- [x] repo：删除 `channel_count_transactions_by_id` 与 `channel_path_count_by_channel`（含其单测），新增 `channel_in_use(conn, ChannelId) -> Result<bool, DbError>`（`SELECT 1 ... LIMIT 1`）
- [x] database.rs / transaction.rs：两个计数包装删除，新增 `channel_in_use` 手写包装（此批尚未宏化）
- [x] `handlers/channel.rs` `delete_channel`：`count > 0` 改 `if db.channel_in_use(ChannelId(id)).await?`，报错文案不动
- [x] `cargo test --workspace` 全绿（`channel_path_count_by_channel` 单测删除除外，其余断言零修改）

## 3. 宏基础设施 + member 试点（design D1/D2/D6）

- [x] 新建 `repo/facade.rs`：`facade!` 宏（条目 `name(param: Type, ...) -> Ret;`、改名变体 `as repo_fn`、`$($meta:meta)*` 保留 doc）、`repo.rs` 挂 `pub mod facade;`（`acquire`/`tx` 升 `pub(crate)`；宏域参数取 `path` 形态以兼容 repo 根模块）
- [x] 宏生成双侧：db 侧 `&self` + `self.acquire().await?`，tx 侧 `&mut self` + `&mut self.tx`，委托路径全 `crate::repo::`
- [x] member 域（db 7 方法 / tx 5 方法）迁入清单，删除两侧手写；accounting-sql 测试全绿
- [x] rust-analyzer 实测宏生成方法的跳转/悬停；不可用则降级为仅生成 db 侧（跨 crate 解析已由 service 测试验证；IDE 体验待你在编辑器确认，降级预案保留——反馈异常即切换仅 db 侧生成）

## 4. 分域迁移（design D7，每批一跑全量测试）

- [x] account 域（db/tx 两侧同名委托全部入清单）
- [x] commodity 域
- [x] channel 域（含把第 2 步的 `channel_in_use` 手写包装转为清单条目）
- [x] channel_path 域
- [x] tag 域
- [x] attachment 域
- [x] transaction 域
- [x] posting 域
- [x] budget 域
- [x] saving_plan 域
- [x] mapping 域（6 处 `as` 改名条目：`account_mapping_*` as `mapping_*`，tx 侧仅 2 个）
- [x] 每批完成即 `cargo test --workspace` 全绿、clippy 零警告

## 5. 收尾审计与全量验收（design D8）

- [x] 删除 database.rs / transaction.rs 中全部残留手写委托；确认仅存 lifecycle（open/open_in_memory/initialize/is_initialized/transaction/commit）与 `display_name_wrappers!`
- [x] 复用三层签名比对审计脚本收尾核对：清单条目数 == 删除的手写方法数（db/tx 分侧），零手写委托、零签名不一致
- [x] `cargo clippy --workspace --all-targets` 零警告
- [x] `cargo test --workspace` 全绿，现有测试断言零修改（除第 2 步删除的单测）
- [x] 抽查行为等价：`tx add/list`、`saving-plan create/list`、`budget create/show` CLI 冒烟通过（budget 实际支出正确汇入，证明 facade 方法经 service/report 路径无误）
