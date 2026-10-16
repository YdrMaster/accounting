# Proposal: data-integrity-db-tuning

## Why

`OPTIMIZATIONS.md` 分组 3（数据完整性 + DB 效率 + 认证中间件）的落地。分组 2 之后的架构演进修正了文档的前提：`transaction_insert`/`update` 与 `channel_path_create_batch` 的生产调用已全部走 service 事务路径或 beancount import（后者本 change 将单事务化），文档原方案「repo 内部 `begin()`」会与 service 外层事务撞嵌套 `BEGIN`——故 #1/#4 不再单独实施，由 #5 吸收；`account_delete` 因 CASCADE 已确认缩水为删冗余 DELETE；`account_set_owners` 经调用图核实存在真实 autocommit 调用方（API handlers×3、config×1），中途失败会留下部分替换的 owners，是真窟窿。SQLite 连接池无 `synchronous`/`busy_timeout`/连接数调优，WAL 下默认 FULL 同步使逐行事务（支付宝导入）每行一次 fsync。

## What Changes

- **3.2 SQLite 配置**：`open_in_memory` 池限 `max_connections(1)`（`:memory:` 多连接即多库）；文件库 `synchronous=NORMAL`（WAL 标准搭配，断电或丢最近已提交事务，个人记账接受并记录）、`busy_timeout(5s)`、`max_connections(4)`。
- **3.1 #2 缩水实施**：`account_delete` 删除 `account_owners`/`account_ancestors` 两条冗余 DELETE（schema 已 `ON DELETE CASCADE`），单语句天然原子，不包事务。
- **3.1 #3 双层原子化**：`account_set_owners` 拆 `inner`（pub(crate)，事务内用）+ autocommit 公开函数（`begin`/`commit` 包 inner，循环 INSERT 合并为单条多值 INSERT）；tx 门面 `as` 改名指向 inner、db 门面指向 autocommit 版。补回滚测试：中途失败原 owners 不变。
- **3.1 #5 beancount 导入单事务**：`import.rs` 开启一次事务、全部写库切换 tx 门面、结束 commit；执行期任一失败整体回滚并报错（与 spec 既有「解析错误终止导入」语义同向，附件缺失警告继续的既有行为不变）。补零残留回滚测试。#1/#4 随之吸收，不单独实施。
- **3.3 索引**：`SCHEMA_STATEMENTS` 追加 7 张 `*_names` 表 `name` 列索引 + `transactions(member_id)`（均 `IF NOT EXISTS`，老库启动自愈）；`repo/channel.rs:90` 的 `LOWER(cn.name)=LOWER(?1)` 改为 `cn.name = ?1`（NOCASE 列，等值可走索引）。
- **3.4 认证中间件**：`find_session_by_token_hash` + `find_user_by_id` 合并为单条 JOIN 查询；滑动续期节流为「剩余 < TTL/2 才 touch」（写放大从每请求 1 次降至约每半 TTL 1 次）；补续期 e2e 断言（接近过期的 session 仍续期、过半前的请求不续期）。spec 契约「每次认证请求续期」随之 MODIFIED。
- **3.1 #6 降级为观察项**：支付宝导入分批事务暂缓——3.2 的 `synchronous=NORMAL` 消除逐行 fsync 后实测导入耗时，不达标再另行立项批量化（需给 `TransactionService::submit` 拆 inner，避免嵌套 begin）。
- **测试**：本组允许新增测试——#3 回滚测试、#5 零残留测试、3.4 续期 e2e；其余现有测试断言零修改。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `user-auth`: MODIFIED「session 滑动过期」——续期触发条件从「每次认证请求」改为「距过期不足 TTL/2 时」，放行语义不变。
- `beancount-import`: ADDED「导入原子性」Requirement——执行期失败全量回滚零残留（解析期「报错并终止导入」的既有 Scenario 语义不变）。

## Impact

- **代码**：accounting-sql（database.rs 池配置、schema.rs 索引、repo/account.rs、repo/channel.rs）、accounting-beancount（import.rs）、accounting-service（import_service.rs 观察项测量）、accounting-auth（db/sessions.rs JOIN、api/middleware.rs）、accounting-api（无改动，`delete_channel` 式守卫不涉及）。
- **行为**：`account_set_owners`/import 中途失败从「留半成品」变为「原子回滚」（健壮性强化）；session 续期频率降为半 TTL 节流（放行与最终滑动过期语义不变）；索引与池配置为部署参数级变化。
- **风险**：`open_in_memory` 单连接后，事务挂起期间再走 db 门面的测试会死锁至超时（全量测试暴露即改写测试顺序）；`synchronous=NORMAL` 的断电窗口需在归档 design 中记录取舍。
- **验收**：`cargo test --workspace` 全绿（含 3 个新测试）、clippy 零警告、`accounting-auth/tests/api_e2e.rs` 全绿、老库启动自动补建索引验证。
