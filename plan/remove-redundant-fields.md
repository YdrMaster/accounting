# 移除冗余字段

本文档记录需要从核心数据模型中移除的冗余字段，包括移除原因和影响范围。

## 判据

**事实**：交易发生时客观确定的、不可事后随意更改的信息。不同用户对同一笔交易会填入相同的值。
**用户自定义信息**：用户主观附加的分类、标注、偏好。不同用户对同一笔交易可能给出不同的值。

核心数据库只存储事实。冗余字段（与上级实体重复的信息）也应移除。

---

## 1. `posting.member_id` — 移除

### 原因

与 `transaction.member_id` 冗余。"谁做了这笔交易"是交易级别的事实，一笔交易内的所有分录共享同一个行为人。分录只需要知道"哪个账户、多少钱"，不需要重复记录成员信息。

### 影响范围

- `accounting/src/posting.rs` — 移除 `member_id` 字段
- `accounting-sql/src/schema.rs` — postings 表移除 `member_id` 列
- `accounting-sql/src/repo/posting.rs` — INSERT/SELECT 语句移除 member_id（约 10 处）
- `accounting-service/src/transaction_service.rs` — 构造 Posting 时不再设置 member_id
- `accounting-api/src/dto.rs` — Posting DTO 移除 member_id
- `accounting-api/src/handlers/transaction.rs` — 不再从请求中读取 posting 的 member_id
- `accounting-cli/src/cmd/tx.rs` — 不再从命令行读取 posting 的 member_id

> 注：`sum_by_member()` 统计查询已经使用 `t.member_id`（transaction 级别），不受影响。

---

## 2. `posting.channel_id` — 移除

### 原因

与 `transaction.channel_id` 冗余。支付渠道是交易级别的事实——"这笔钱通过支付宝付的"描述的是整笔交易，不是单个分录。

### 影响范围

- `accounting/src/posting.rs` — 移除 `channel_id` 字段
- `accounting-sql/src/schema.rs` — postings 表移除 `channel_id` 列
- `accounting-sql/src/repo/posting.rs` — INSERT/SELECT 语句移除 channel_id（约 10 处）；`sum_by_channel()` 需从 `p.channel_id` 改为 `t.channel_id`
- `accounting-service/src/transaction_service.rs` — 构造 Posting 时不再设置 channel_id
- `accounting-api/src/dto.rs` — Posting DTO 移除 channel_id
- `accounting-api/src/handlers/transaction.rs` — 不再从请求中读取 posting 的 channel_id
- `accounting-cli/src/cmd/tx.rs` — 不再从命令行读取 posting 的 channel_id

> **注意**：`sum_by_channel()` 当前按 `p.channel_id` 分组，移除 posting.channel_id 后需改为 `GROUP BY t.channel_id`，并确保 JOIN 了 transactions 表。

---

## 3. `transaction.is_template` — 移除

### 原因

`is_template` 是用户主观意愿——"我想把这笔交易当作模板"，不是交易客观事实。两个用户对同一笔交易不会达成"这必须是模板"的共识。此外，当前代码中没有创建模板或从模板实例化交易的逻辑，仅用于查询过滤，属于未完成功能。

### 影响范围

- `accounting/src/transaction.rs` — 移除 `is_template` 字段
- `accounting/src/transaction_filter.rs` — 移除 `is_template` 过滤字段
- `accounting-sql/src/schema.rs` — transactions 表移除 `is_template` 列
- `accounting-sql/src/repo/transaction.rs` — INSERT/SELECT/UPDATE 移除 is_template（约 8 处）
- `accounting-sql/src/repo/posting.rs` — 统计查询移除 is_template 过滤条件（3 处）
- `accounting-service/src/transaction_service.rs` — 构造 Transaction 时不再设置 is_template
- `accounting-cli/src/cmd/tx.rs` — 移除 `--template` 参数
- `accounting-cli/src/cmd/mod.rs` — 移除显示字段
- `accounting-api/src/dto.rs` — 移除 DTO 字段
- `accounting-api/src/handlers/transaction.rs` — 移除 is_template 处理
- `accounting-web/src/stores/transaction.ts` — 移除 is_template 字段

---

## 4. `AccountType::is_permanent()` — 移除

### 原因

死代码——全项目 grep 仅在测试中调用，没有任何业务逻辑使用。且语义有误：按会计准则永久账户应包括 Asset、Liability、Equity，但代码中 `matches!(Asset | Liability)` 漏掉了 Equity。如果未来需要，可从 AccountType 枚举直接推导。

### 影响范围

- `accounting/src/account_type.rs` — 删除 `is_permanent()` 方法（第 18-20 行）和对应测试（第 86-90 行）

---

## 变更总结

| 移除项 | 所在实体 | 影响文件数 | 迁移注意 |
|--------|---------|-----------|---------|
| `posting.member_id` | Posting | ~6 | 统计查询已用 `t.member_id`，无需改动 |
| `posting.channel_id` | Posting | ~6 | `sum_by_channel()` 需从 `p.channel_id` 改为 `t.channel_id` |
| `transaction.is_template` | Transaction | ~10 | 无迁移逻辑，纯删除 |
| `is_permanent()` | AccountType | 1 | 纯删除 |

数据库迁移：
- `ALTER TABLE postings DROP COLUMN member_id`
- `ALTER TABLE postings DROP COLUMN channel_id`
- `ALTER TABLE transactions DROP COLUMN is_template`

> SQLite 从 3.35.0 (2021-03-12) 开始支持 `DROP COLUMN`，rusqlite bundled 版本应已支持。
