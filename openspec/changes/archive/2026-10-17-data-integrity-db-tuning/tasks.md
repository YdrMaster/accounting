# Tasks: data-integrity-db-tuning

> 实施顺序即分组顺序：3.2 配置 → #2/#3 原子化 → #5 单事务 → 3.3 索引 → 3.4 auth。每分组结束跑一次全量测试。

## 1. 3.2 SQLite 池与 PRAGMA（design D4）

- [x] `open_in_memory`：池限 `max_connections(1)`
- [x] 文件库 `open`：`synchronous(Normal)`、`busy_timeout(5s)`、`max_connections(4)`
- [x] `cargo test --workspace` 全绿（单连接池暴露 1 处：`test_account_root_names_by_ids_wrapper` 在事务挂起期间走 db 门面，已按预案调整断言顺序）

## 2. 3.1 #2+#3 账户写路径原子化（design D2）

- [x] `repo/account.rs` `account_delete`：删除 `account_owners`/`account_ancestors` 两条冗余 DELETE（CASCADE 兜底）
- [x] `account_set_owners` 拆双层：`account_set_owners_inner`（pub(crate)，多值 INSERT 合并循环）+ 公开 `account_set_owners`（begin/commit 包 inner）
- [x] facade 清单：db 侧默认条目指向公开函数；tx 侧 `as account_set_owners_inner`
- [x] 新增回滚测试：写库中途失败后原 owners 不变
- [x] `cargo test --workspace` 全绿

## 3. 3.1 #5 beancount 导入单事务（design D3）

- [x] 盘点 `import.rs` 全部 `db.` 调用面（grep 清单逐条勾销）
- [x] 入口 `db.transaction()` 开事务，全部写库调用切 `tx.` 门面，成功 `commit`、错误随 drop 回滚
- [x] `let _ = db.attachment_create(...)` 改 `?` 传播（附件文件读取失败的警告继续语义不变）
- [x] 新增零残留测试：构造中途必失败的导入，断言失败后交易/账户/成员计数不变
- [x] `cargo test --workspace` 全绿 + CLI `beancount import` 冒烟一次

## 4. 3.3 索引与等值改写（design D5）

- [x] `SCHEMA_STATEMENTS` 追加 7 张 `*_names.name` 索引 + `idx_transactions_member`（均 `IF NOT EXISTS`）
- [x] `repo/channel.rs:90`：`LOWER(cn.name)=LOWER(?1)` → `cn.name = ?1`
- [x] 老库启动自愈验证：空文件经 CLI initialize 后 8 索引齐备（初版插点在 saving_plan_names 建表前导致中止，已移至数组末尾）；EXPLAIN 确认 channel 按名查询与 transactions 按 member 查询均走 `SEARCH ... USING INDEX`
- [x] `cargo test --workspace` 全绿

## 5. 3.4 auth 中间件（design D6）

- [x] `auth/db/sessions.rs` 新增 `find_session_with_user_by_token_hash` 单条 JOIN（pending/expired 判定所需字段逐一核对）
- [x] 中间件改单查询；`expires_at - now < SESSION_TTL_SECS/2` 才 `touch_session`
- [x] `api_e2e` 增补：剩余 > TTL/2 不改写过期时间；剩余 < TTL/2 续期至完整 TTL
- [x] `cargo test -p accounting-auth` 全绿（`api_e2e.rs` 含新断言）

## 6. 全量验收与观察项

- [x] `cargo clippy --workspace --all-targets` 零警告
- [x] `cargo test --workspace` 全绿；本 change 新增 3 个测试之外的断言零修改
- [x] 观察项（不实施）：OPTIMIZATIONS.md 遗留待办已记录——3.2 已消除逐行 fsync，待真实样例文件实测耗时后再定批量化（submit 拆 inner）
