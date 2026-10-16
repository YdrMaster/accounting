# Design: facade-macro-wrappers

## Context

分组 2 收尾时的多行签名精确审计（脚本比对三层签名、归一化类型路径）确认：database.rs 115 个 + transaction.rs 80 个同名委托方法中约 95% 与 repo 函数逐字 1:1（仅类型路径书写差异）；真实例外为 lang 参数位置 7 处、参数互换 2 处、mapping 改名委托 6 处、lifecycle/显示名宏等自定义 ~13 处。既有先例：`database.rs` 的 `display_name_wrappers!`（7 实体批量显示名方法）。wrapper 的存在理由是类型化门面——service/cli/api 只见 `SqliteDatabase`/`SqliteTransaction`，不见 sqlx 连接类型；宏化不改变该架构，只把手工簿记变为一张清单。

## Goals / Non-Goals

**Goals:**

- 手写委托方法全部由单一清单宏生成，repo 新增函数只需加一行清单
- 宏语义唯一：逐字转发（签名统一后不存在参数重排形态）
- channel 计数 API 语义化：守卫用途显式化为 `channel_in_use`
- 每一步独立可编译、可测试，行为不变红线全程成立

**Non-Goals:**

- 不补齐 tx 侧缺失的报告类方法（宏清单 = 现有手写精确集合）
- 不动 `display_name_wrappers!` 与 lifecycle 方法
- 不把 repo 连接类型暴露给 service 层（门面架构不变）
- 不为宏引入过程宏依赖

## Decisions

### D1：单一 `facade!` 清单宏，双侧生成

每侧独立一次宏调用、成对使用：`facade! { db: 域名 { 条目 } }` 生成 `impl SqliteDatabase`（`&self` + `self.acquire().await?`），`facade! { tx: 域名 { 条目 } }` 生成 `impl SqliteTransaction`（`&mut self` + `&mut self.tx`）。不做单次调用双侧生成——db 侧 115 个与 tx 侧 80 个方法集本不对称（报告类方法不入事务），强制对称会违背「清单 = 现有手写精确集合」原则。
否决 proc-macro：新增构建依赖与维护面，收益不抵成本。否决 trait 方案（`ConnProvider::with_conn`）：pool 侧 acquire 返回拥有型 `PoolConnection`、tx 侧是 `&mut Transaction`，统一抽象需 async-trait 或 HRTB 闭包，复杂度高于宏且丢失方法清单形态。

### D2：宏条目语法与文档保留

```rust
facade! {
    /// 获取渠道的全部名字（任意语言）
    channel_names_by_id(id: ChannelId) -> Vec<String>;
    account_mapping_list_all() -> Vec<(AccountId, MemberId)> as mapping_list_all;
}
```

- 条目 `name(param: Type, ...) -> Ret;`，参数逐字转发，类型含泛型/元组/引用切片均可（`ty` fragment 匹配完整类型）；同一调用内条目形态必须齐整（普通条目与 `as` 改名条目分两次调用，宏按形态分支匹配）
- 条目前置 `///` doc 注释经 `$($meta:meta)*` 捕获原样重发，现有手写 doc 全量保留
- `as repo_fn` 变体仅 mapping 系 6 处使用，门面名不变、调用方零改动
- 展开体即现有手写形态：`self.acquire().await?` / `repo::x::y(&mut conn, ...)`；`use` 路径在宏内写全 `crate::repo::...`，不依赖调用侧导入

### D3：签名统一先行，宏不做参数映射

8 个 repo 函数对齐门面约定：budget 的 create/update/upsert_by_name 与 saving_plan 的 create/update/upsert_by_name 共 6 个，`lang` 从第 2 参数移至**末尾**；`commodity_upsert_by_symbol` 调整为 `(symbol, name, precision, lang)`、`channel_upsert_by_name` 调整为 `(name, description, account_id, lang)`。方向选择依据：门面/service/CLI 的调用约定已以 lang 收尾为主，repo 是少数派；且 service 调用点由编译器兜底，属机械适配。
否决宏支持重排语法（`repo(name, lang, ...)` 显式映射）：宏解析复杂度陡增，换取的只是免去一次性签名适配，不划算。

### D4：channel 计数语义化

repo 层：`channel_count_transactions_by_id` 删除，新增 `channel_in_use(conn, ChannelId) -> Result<bool, DbError>`（`SELECT 1 FROM channel_paths WHERE channel_id = ?1 LIMIT 1`）；`channel_path_count_by_channel` 删除（零生产调用方，仅 1 个单测，git 历史可找回）。两侧 wrapper：`channel_in_use` 进宏清单，两个计数方法不再出现。handler：`delete_channel` 的 `count > 0` 改 `if db.channel_in_use(ChannelId(id)).await?`，`channel_in_use` 报错文案不动。
等价论证：守卫只依赖「是否存在引用行」，路径行计数与存在性判断在 `>0` 语义下恒等价；命名修正消除「名字承诺交易数」的误用陷阱，未来真要展示计数可用既有 `channel_path_find_transactions_by_channel`（DISTINCT 正确版）。

### D5：清单落 `repo/facade.rs`

清单是门面契约（哪些 repo 函数暴露到哪一侧、以何名暴露），属独立关注点，新文件承载；`repo.rs` 挂 `pub mod facade;`。宏定义同文件，清单分域注释分组（与现有 `// === Channel ===` 风格一致）。

### D6：试点先行 + 降级路径

member 域（5 方法）首个迁移。试点验收含 rust-analyzer 跳转/悬停实测：宏生成方法可解析则铺开；若不可用，**降级为宏只生成 db 侧、tx 侧保留手写**（仍消除约半数重复），其余计划不变。降级决定在试点任务内一次性做出，不回滚已试点域。

### D7：分批迁移与批次顺序

按 repo 文件域推进：member（试点）→ account → commodity → channel → channel_path → tag → attachment → transaction → posting → budget → saving_plan → mapping（`as` 变体）。每批：删手写方法（两侧）→ 清单加条目 → `cargo test --workspace` 全绿 → 下一批。批次内含该域在 db/tx 两侧的全部同名委托方法；channel 域批叠加 D4 的 in_use 改造（其时 channel_in_use 已在第 2 步落地为手写，此批仅转为清单条目）。

### D8：验收口径

- `cargo test --workspace` 全绿、clippy 零警告（含宏展开代码，既有 deny 门控生效）
- 清点核对：宏清单条目数 == 删除的手写方法数（db/tx 分侧计数），审计脚本复用分组 2 的三层签名比对逻辑在收尾时跑一遍，输出应为「零手写委托方法、零签名不一致」
- 行为红线：现有测试断言零修改（`channel_path_count_by_channel` 的单测随函数删除除外）

## Risks / Trade-offs

- **rust-analyzer 对宏生成 impl 方法的支持**：macro_rules 生成的关联函数 IDE 通常可解析，但跳转/悬停体验需实测（D6 试点判定点）；最坏降级只生成 db 侧。
- **清单遗漏/多列**：宏清单若与 repo 实际函数漂移无编译报错（漏列=方法消失，调用方编译错可发现；多列=编译错直接暴露）。漏列风险由收尾审计兜底。
- **签名适配波及面**：9 个函数的调用点集中在 service 与 repo 内部，编译器驱动；`lang` 移尾在 Rust 无默认参数语法下是纯机械改动。
- **宏展开可读性**：出错定位从方法体变为清单行，代价可接受（委托方法本无逻辑）；`cargo expand` 可复核。
- **批次中途状态**：宏清单与手写方法在批次间共存（部分域已迁移、部分未迁移），同名方法不会冲突（迁移即删除手写）；每批全绿保证可暂停。

## Migration Plan

无 schema/数据/公共 API 变化（channel 计数函数为 crate 内 API，调用方仅一处 handler）。单 commit 或按批次分 commit 均可回滚；建议每阶段（签名统一 / channel 修正 / 试点 / 各批 / 收尾）独立 commit，便于二分定位。

## Open Questions

无——探索期四项决策（签名方向、删除计数函数、as 改名、清单位置）均已由用户确认。

### 实施期修订（apply 阶段落定）

- 宏域参数保持 `$domain:ident`（曾试 `path` 形态以兼容 repo 根模块，但 path 片段不能作路径中段，已回退）；根模块函数（`get_setting`/`set_setting`）走字面量域 `repo_root` 专用臂。
- `account_rename`（两侧）为复合流程——先 `account_ensure_rename_allowed` 再 `account_rename`——保留手写不入清单。签名审计对此类复合 wrapper 是盲区（多 repo 调用但签名 1:1），收尾审计已补「每方法 repo 调用数核对」确认全仓仅此一例。
- 部分手写 wrapper 带 `#[allow(clippy::too_many_arguments)]`，清单条目已按需补注。
