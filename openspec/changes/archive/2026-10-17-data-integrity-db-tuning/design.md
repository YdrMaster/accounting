# Design: data-integrity-db-tuning

## Context

分组 2 之后的三层结构（service → db/tx 门面 → repo）改变了分组 3 文档的前提。调用图核实：`transaction_insert`/`update`、`channel_path_create_batch` 的生产调用全部在事务路径（service submit/update 用 tx 门面；beancount import 用 db 门面 autocommit，本 change 将切换为 tx）；`account_set_owners` 有真实 autocommit 调用方（`handlers/account.rs:111,145`、`config/service.rs:498`），另有 tx 路径调用方（`config/service.rs:329`）；`account_delete` 仅 handler autocommit 一处。schema 的 `account_owners`/`account_ancestors` 已带 `ON DELETE CASCADE`（schema.rs:286-297）。索引缺口：7 张 `*_names` 表 `name` 列与 `transactions.member_id` 无索引；`repo/channel.rs:90` 对 `LOWER(cn.name)` 套函数使 NOCASE 索引失效。auth 中间件每请求 3 次 DB 往返（middleware.rs:81/94/99），且现行 spec 契约为「每次认证请求续期」——本 change 以 MODIFIED 增量改契约为 TTL/2 节流。

## Goals / Non-Goals

**Goals:**

- 多步写库路径的原子性：`account_set_owners` 中途失败不破坏原 owners；beancount 导入失败零残留
- SQLite 池与 PRAGMA 调优（`synchronous`/`busy_timeout`/连接数），WAL 下消除逐行 fsync
- 按名查询与 member 过滤走索引；auth 中间件 3 次往返降为 1 次 + 续期节流
- 本组允许的新测试全部落地（#3/#5 回滚、3.4 续期 e2e）

**Non-Goals:**

- 不做支付宝导入分批事务（#6 降级观察项：3.2 落地后实测耗时，不达标另行立项，届时需给 `TransactionService::submit` 拆 inner 避免嵌套 begin）
- 不给 repo 多写函数内部加 `begin()`（与 service 外层事务撞嵌套 `BEGIN`，SQLite 直接报错）
- 不动 `account_delete` 的事务结构（CASCADE 后单语句天然原子）

## Decisions

### D1：#1/#4 取消单独实施，由 #5 吸收

`transaction_insert`/`update`、`channel_path_create_batch` 唯一的 autocommit 生产调用方是 beancount import（`import.rs:150,208`），#5 单事务化后其写库全部进入事务。service 路径本就原子。repo 层保持「吃连接不事务」的门面契约，在 `repo/transaction.rs` 与 `repo/channel_path.rs` 顶部以注释固化：「多写函数须在事务内使用；autocommit 单独调用不做原子性保证」。
否决「repo 函数内部 begin」：service 在外层事务内经 tx 门面调这些函数，嵌套 `BEGIN` 报错。否决「inner/autocommit 双函数」：仅为测试调用方引入双层复杂度，无生产收益。

### D2：#3 `account_set_owners` 双层原子化

`repo::account::account_set_owners_inner(conn, account_id, owner_ids)`——pub(crate)，删除旧 owners + 单条多值 INSERT（`(@0,*),(@1,*)...` 逐参数绑定），供事务内使用；`repo::account::account_set_owners(conn, ...)`——pub，`begin`/`commit` 包 inner。facade 清单：db 侧条目默认指向公开函数；tx 侧条目 `account_set_owners(...) -> () as account_set_owners_inner;`。
否决「只改 db 门面为手写事务包装」：手写包装正是分组 2 刚消除的形态；`as` 改名是既有宏能力。

### D3：#5 beancount 导入单事务 + 整体回滚

`import.rs` 入口处 `db.transaction()` 开启一次事务，函数内全部 `db.` 门面写调用切换为 `tx.`（tx 侧方法已由门面清单覆盖：`transaction_insert`/`channel_path_create_batch`/`account_update_by_path`/`attachment_create` 等，实施时全量核对 `import.rs` 的 `db\.` 调用面）；成功 `commit`，任一 `?` 传播错误随 drop 回滚。失败语义从「部分落库」变为「全量回滚 + 报错」，与 spec 既有解析期「报错并终止导入」同向。`let _ = db.attachment_create(...)`（import.rs:266）的吞错改为 `?` 传播——写库失败（区别于附件文件读取失败，后者本就是警告继续）在事务语义下应触发回滚。
否决「失败行计入 errors 继续」：与原子性目标直接矛盾。否决「SAVEPOINT 分段」：SQLite 嵌套 savepoint 复杂度无收益，导入是全有或全无的语义。

### D4：3.2 池配置

`open_in_memory`：`SqlitePoolOptions::new().max_connections(1)`——`:memory:` 每连接独立库，第二连接即 "no such table"；现有测试全部串行 await，事务挂起期间不触 db 门面（全量测试验证，若有用例死锁则修测试顺序而非放宽连接数）。文件库 `open`：`synchronous(SqliteSynchronous::Normal)`（WAL 标准搭配；断电窗口丢最近已提交事务，个人记账接受，归档时记录取舍）、`.busy_timeout(Duration::from_secs(5))`、`max_connections(4)`（单写者模型，4 连接覆盖读并发）。
否决 `synchronous=OFF`：丢已提交事务不可接受。

### D5：3.3 索引与等值改写

`SCHEMA_STATEMENTS` 索引区追加 8 条：`idx_{account,tag,channel,commodity,member,budget,saving_plan}_names_name`（`name` 列）+ `idx_transactions_member`（`member_id`）。`IF NOT EXISTS` 使老库下次启动自愈，无迁移文件。`channel.rs:90` 改 `cn.name = ?1`——`channel_names.name` 为 `COLLATE NOCASE`（schema 已核实），等值比较直接命中新索引。
风险：`LOWER(cn.name)` 改写依赖 NOCASE 对非 ASCII 的大小写折叠语义与 `LOWER()` 一致（SQLite NOCASE 仅折叠 ASCII——与 `LOWER()` 同为 ASCII-only，语义等价已核实）。

### D6：3.4 auth 中间件

`sessions.rs` 新增 `find_session_with_user_by_token_hash(token_hash)`：单条 JOIN 返回 session + user 所需字段，保持 pending/expired 判定语义不变。中间件：JOIN 一次取回 → 判定 → 仅当 `sess.expires_at - now < SESSION_TTL_SECS / 2` 时 `touch_session`。`hash_token` 逐请求成本维持（分组 2 已优化 to_hex）。
否决「缓存 session 于内存」：单机个人应用收益低，引入失效复杂度。

### D7：新增测试边界

- `account_set_owners`：中途失败（如注入重复约束或非法成员）后原 owners 保持不变——放在 repo 测试层，用事务内构造失败。
- beancount 导入：构造中途必失败的导入（如交易引用不存在账户），断言失败后零残留（交易/账户/成员计数不变）。
- 3.4：`api_e2e` 增补两条——剩余 > TTL/2 的请求不改写过期时间；剩余 < TTL/2 的请求续期至完整 TTL。
其余现有测试断言零修改；`open_in_memory` 单连接若暴露串行假设破坏的用例，修测试顺序并记录。

## Risks / Trade-offs

- **单连接内存池的串行假设**：任何「tx 未提交期间又 acquire」的代码路径会死锁到超时；全量测试为探测网。文件库 4 连接 + busy_timeout 兜底写竞争。
- **synchronous=NORMAL 的断电窗口**：WAL 已提交事务可能在断电时丢失（应用崩溃不丢）；个人记账可接受，取舍已写入本文件，归档后为决策来源。
- **auth JOIN 查询的字段面**：中间件需要 user 的哪些字段以保持既有判定（pending 状态、用户存在性）必须逐一核对，避免 JOIN 后丢字段导致判定漂移。
- **import 切 tx 门面的遗漏风险**：`import.rs` 的 `db.` 调用面需全量盘点（含错误分支内的调用），遗漏一处即 autocommit 旁路——实施以 grep 清单逐条勾销。
- **索引生效验证**：`EXPLAIN QUERY PLAN` 抽查按名查询走索引（实施时顺手验证一次）。

## Migration Plan

索引由 `IF NOT EXISTS` 老库自愈；池配置与 PRAGMA 即时生效、git revert 可回滚；`account_set_owners`/import 行为强化无数据迁移。建议按「3.2 → #2/#3 → #5 → 3.3 → 3.4」分 commit，便于二分。

## Open Questions

无——探索期三项决策（#6 观察项、#5 整体回滚、新增测试清单）均已由用户确认。
