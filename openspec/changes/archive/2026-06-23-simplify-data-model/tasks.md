# Tasks: simplify-data-model

> 回溯归档：以下任务均已实现（commits `e0a18bd8`/`73b28742`/`7ab89940`/`bdbba816`/`108e6934` ~2026-06-23~24，Review 2026-06-24 确认 95~100%），checkbox 标记为 `[x]` 复盘。

## 0. 前置迁移 `fcb8fea6`

- [x] 0.1 cherry-pick `fcb8fea6`（删 `Account.position` + 前端拖拽排序），与 `4b9fa057`(remove-liability) 删除冲突以并集为准
- [x] 0.2 出口：`cargo fmt`/`cargo clippy --workspace --all-targets`/`cargo test --workspace` 全过

## 1. 阶段 1：移除冗余字段

- [x] 1.1 `posting.member_id`：删 `accounting/src/posting.rs` 字段、`schema.rs` 列、`repo/posting.rs` INSERT/SELECT、service/DTO/CLI 透传（统计已用 `t.member_id`）
- [x] 1.2 `posting.channel_id`：同上；`sum_by_channel()` 改 `t.channel_id` 并 JOIN `transactions`
- [x] 1.3 `transaction.is_template`：删字段/过滤/schema 列/INSERT-SELECT-UPDATE/repo 统计过滤/service/CLI `--template`/DTO/handler/前端 store
- [x] 1.4 `AccountType::is_permanent()`：删方法及测试
- [x] 1.5 DDL：`ALTER TABLE postings DROP COLUMN member_id; DROP COLUMN channel_id; ALTER TABLE transactions DROP COLUMN is_template;`
- [x] 1.6 出口：三件套全过

## 2. 阶段 2：审计字段改进

- [x] 2.1 11 表 `created_at`/`updated_at` DEFAULT `date('now')`→`datetime('now')`
- [x] 2.2 11 表 `update_*_updated_at` 触发器 `date('now')`→`datetime('now')`
- [x] 2.3 `settings` 表补 `created_at`/`updated_at` + 触发器
- [x] 2.4 测试断言更新为 `datetime('now')` 格式
- [x] 2.5 出口：三件套全过

## 3. 阶段 3：账户名称重构 `full_name` → `name`

- [x] 3.1 `Account.full_name` → `name` + `display_path()` 动态拼装
- [x] 3.2 schema：`name TEXT NOT NULL` + `UNIQUE(parent_id, name)`；种子调整
- [x] 3.3 `get_by_name` 改递归逐级 `get_by_parent_and_name`；新增 `find_root_name`/`find_root_id`（`ORDER BY depth DESC LIMIT 1`）
- [x] 3.4 `create_cascading` 重写：`split(':')` 后逐级查找/创建
- [x] 3.5 `closure.AccountNode.full_name` → `name`
- [x] 3.6 前端去所有 `split(':')`，用 `parent_id` 构树
- [x] 3.7 出口：三件套 + `npm run build` 全过

## 4. 阶段 4：账户类型重构（去 `account_type` 列）

- [x] 4.1 `Account` 去 `account_type` 字段；`AccountType` 只留 `FromStr`（去 `from_prefix`）
- [x] 4.2 schema 删列/`idx_accounts_type`/CHECK；种子调整
- [x] 4.3 `validate_account_close` 参数由 `AccountType` 改为 `find_root_name` + `AccountType::from_str` 推导后传入
- [x] 4.4 `repo/posting.rs` 三个统计 `JOIN account_ancestors` + `JOIN accounts ra` 取根节点 GROUP BY
- [x] 4.5 `account_service.create_cascading` 去类型一致性校验（结构天然保证）
- [x] 4.6 报表 `match account.account_type` 改根节点推导；DTO `account_type` 改只读推导字段
- [x] 4.7 出口：三件套 + `npm run build` 全过

## 5. RULE_06 集中治理（Review 随产出）

- [x] 5.1 新建 `accounting/src/datetime_utils.rs`：`start_of_day()`/`end_of_day()`（SAFETY 注释 + `expect`），`lib.rs` 导出
- [x] 5.2 替换 `repo/transaction.rs`（8 处）、`repo/posting.rs`（8 处）、`api/handlers/transaction.rs`（1 处）的 `and_hms_opt().unwrap()`
- [x] 5.3 豁免：CLI `cmd/tx.rs`/`main.rs`/`output.rs`、`pool.rs` 锁中毒、`report_service.rs` 测试内 `expect`
