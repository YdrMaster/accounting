# 退款与报销功能实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在现有记账系统中实现退款与报销功能，支持分录级别的冲减关联。

**架构：** 在 `postings` 表新增 `kind`（Normal/Refund/Reimbursement）、`linked_posting_id`、`reversal_total` 三个字段，通过触发器自动维护 `reversal_total`。前端展示原交易时显示净支出（已扣除后续退款）。

**技术栈：** Rust (axum, rusqlite, rust_decimal), Vue 3 + Vite + TypeScript + AntDV + Pinia

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `accounting/src/posting.rs` | 定义 `PostingKind` 枚举，扩展 `Posting` 结构体 |
| `accounting/src/validation.rs` | 新增 `validate_reversal_direction` 纯逻辑验证 |
| `accounting/src/lib.rs` | 导出 `PostingKind` |
| `accounting-sql/src/schema.rs` | 数据库 schema 变更：新增字段、触发器、索引 |
| `accounting-sql/src/repo/posting.rs` | PostingRepo 适配新字段读写；新增 `get_posting` 方法 |
| `accounting-service/src/transaction_service.rs` | 构建 Posting 时适配新字段；提交前调用 reversal 验证（含数据库查询） |
| `accounting-api/src/dto.rs` | PostingDto/PostingRequest 新增 kind、linked_posting_id |
| `accounting-api/src/handlers/transaction.rs` | handler 解析并传递 kind、linked_posting_id |
| `accounting-web/src/stores/transaction.ts` | TypeScript 类型新增 kind、linked_posting_id |
| `accounting-web/src/views/TransactionForm.vue` | 分录行增加 kind 选择器，退款/报销选择原交易分录 |
| `accounting-web/src/components/TransactionDetail.vue` | 展示 reversal_total 和 kind 标记 |

---

## 任务 1：领域模型 — PostingKind 枚举与 Posting 扩展

**文件：**

- 修改：`accounting/src/posting.rs`
- 修改：`accounting/src/lib.rs`

---

- [ ] **步骤 1：定义 PostingKind 枚举并扩展 Posting 结构体**

将 `accounting/src/posting.rs` 完整替换为：

```rust
use crate::id::{AccountId, ChannelId, CommodityId, MemberId, PostingId, TransactionId};
use rust_decimal::Decimal;

/// 分录类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingKind {
    /// 普通分录
    Normal = 1,
    /// 退款分录（冲减原支出）
    Refund = 2,
    /// 报销分录（冲减原支出）
    Reimbursement = 3,
}

impl PostingKind {
    /// 从数据库整数值解析
    pub fn from_db(value: i32) -> Option<Self> {
        match value {
            1 => Some(PostingKind::Normal),
            2 => Some(PostingKind::Refund),
            3 => Some(PostingKind::Reimbursement),
            _ => None,
        }
    }
}

/// 分录（Posting / 端点）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    /// 分录唯一标识符
    pub id: PostingId,
    /// 所属交易 ID
    pub transaction_id: TransactionId,
    /// 所属账户 ID
    pub account_id: AccountId,
    /// 商品/货币 ID
    pub commodity_id: CommodityId,
    /// 金额（正数表示借方，负数表示贷方）
    pub amount: Decimal,
    /// 总价格（双边 Cost），单币种交易为 None
    pub cost: Option<Decimal>,
    /// cost 对应的商品 ID
    pub cost_commodity_id: Option<CommodityId>,
    /// 分录描述
    pub description: Option<String>,
    /// 关联成员 ID
    pub member_id: Option<MemberId>,
    /// 关联支付渠道 ID
    pub channel_id: Option<ChannelId>,
    /// 分录类型
    pub kind: PostingKind,
    /// 关联原分录 ID（退款/报销时指向被冲减的分录）
    pub linked_posting_id: Option<PostingId>,
    /// 累计被冲减金额（由触发器自动维护）
    pub reversal_total: Decimal,
}
```

- [ ] **步骤 2：在 lib.rs 中导出 PostingKind**

在 `accounting/src/lib.rs` 中，找到 `pub mod posting;` 所在位置，在其后添加：

```rust
pub use posting::PostingKind;
```

- [ ] **步骤 3：编译验证**

运行：`cd /home/mechdancer/repos/accounting && cargo check -p accounting 2>&1`
预期：通过（可能有 dead_code 警告，但无编译错误）

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting/src/posting.rs accounting/src/lib.rs
git commit -m "feat(domain): 新增 PostingKind 枚举并扩展 Posting 结构体"
```

---

## 任务 2：数据库层 — Schema 变更、触发器、索引

**文件：**

- 修改：`accounting-sql/src/schema.rs`

---

- [ ] **步骤 1：修改 postings 表定义，新增三个字段**

在 `accounting-sql/src/schema.rs` 的 `SCHEMA_SQL` 常量中，找到 `postings` 表定义，将：

```sql
CREATE TABLE IF NOT EXISTS postings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    commodity_id INTEGER NOT NULL REFERENCES commodities(id),
    amount INTEGER NOT NULL,
    cost INTEGER,
    cost_commodity_id INTEGER REFERENCES commodities(id),
    description TEXT,
    member_id INTEGER REFERENCES members(id),
    channel_id INTEGER REFERENCES channels(id),
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);
```

替换为：

```sql
CREATE TABLE IF NOT EXISTS postings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    commodity_id INTEGER NOT NULL REFERENCES commodities(id),
    amount INTEGER NOT NULL,
    cost INTEGER,
    cost_commodity_id INTEGER REFERENCES commodities(id),
    description TEXT,
    member_id INTEGER REFERENCES members(id),
    channel_id INTEGER REFERENCES channels(id),
    kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3),
    linked_posting_id INTEGER REFERENCES postings(id),
    reversal_total INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);
```

- [ ] **步骤 2：新增 postings 相关触发器**

在 `SCHEMA_SQL` 的触发器区域（`update_postings_updated_at` 触发器之后），添加：

```sql
CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_insert
AFTER INSERT ON postings
WHEN NEW.kind IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total + NEW.amount
    WHERE id = NEW.linked_posting_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_delete
AFTER DELETE ON postings
WHEN OLD.kind IN (2, 3) AND OLD.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total - OLD.amount
    WHERE id = OLD.linked_posting_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_update
AFTER UPDATE OF amount ON postings
WHEN NEW.kind IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total - OLD.amount + NEW.amount
    WHERE id = NEW.linked_posting_id;
END;
```

- [ ] **步骤 3：新增索引**

在 `SCHEMA_SQL` 的索引区域，添加：

```sql
CREATE INDEX IF NOT EXISTS idx_postings_kind ON postings(kind);
CREATE INDEX IF NOT EXISTS idx_postings_linked ON postings(linked_posting_id);
```

- [ ] **步骤 4：运行现有 schema 测试**

运行：`cd /home/mechdancer/repos/accounting && cargo test -p accounting-sql schema::tests 2>&1`
预期：全部通过

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-sql/src/schema.rs
git commit -m "feat(sql): postings 表新增 kind/linked_posting_id/reversal_total 及触发器"
```

---

## 任务 3：数据库仓库层 — PostingRepo 适配新字段

**文件：**

- 修改：`accounting-sql/src/repo/posting.rs`

---

- [ ] **步骤 1：修改 PostingRepo trait，新增 get 方法**

在 `accounting-sql/src/repo/posting.rs` 的 `PostingRepo` trait 定义中，在 `insert` 方法之后添加：

```rust
    /// 根据 ID 查询单个分录
    fn get(
        &self,
        conn: &Connection,
        id: PostingId,
    ) -> Result<Option<Posting>, crate::error::DbError>;
```

- [ ] **步骤 2：修改 insert 方法，写入新字段**

将 `insert` 方法中的 SQL 和参数替换为：

```rust
        conn.execute(
            "INSERT INTO postings
             (transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id, kind, linked_posting_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                posting.transaction_id.0,
                posting.account_id.0,
                posting.commodity_id.0,
                amount_i64,
                cost_i64,
                posting.cost_commodity_id.map(|id| id.0),
                posting.description,
                posting.member_id.map(|id| id.0),
                posting.channel_id.map(|id| id.0),
                posting.kind as i32,
                posting.linked_posting_id.map(|id| id.0),
            ],
        )?;
```

- [ ] **步骤 3：实现 get 方法**

在 `impl PostingRepo for SqlitePostingRepo` 中，在 `insert` 方法之后添加：

```rust
    fn get(
        &self,
        conn: &Connection,
        id: PostingId,
    ) -> Result<Option<Posting>, crate::error::DbError> {
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id, kind, linked_posting_id, reversal_total
             FROM postings WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id.0])?;
        if let Some(row) = rows.next()? {
            let precision = get_precision(conn, CommodityId(row.get::<_, i64>(3)?))?;
            let cost_commodity_id: Option<i64> = row.get(6)?;
            let cost_precision = cost_commodity_id
                .map(|cid| get_precision(conn, CommodityId(cid)))
                .transpose()?
                .unwrap_or(precision);
            Ok(Some(Posting {
                id: PostingId(row.get(0)?),
                transaction_id: TransactionId(row.get(1)?),
                account_id: AccountId(row.get(2)?),
                commodity_id: CommodityId(row.get(3)?),
                amount: accounting::amount::from_db_amount(row.get(4)?, precision),
                cost: row.get::<_, Option<i64>>(5)?.map(|c| accounting::amount::from_db_amount(c, cost_precision)),
                cost_commodity_id: cost_commodity_id.map(CommodityId),
                description: row.get(7)?,
                member_id: row.get::<_, Option<i64>>(8)?.map(accounting::id::MemberId),
                channel_id: row.get::<_, Option<i64>>(9)?.map(accounting::id::ChannelId),
                kind: accounting::posting::PostingKind::from_db(row.get(10)?)
                    .unwrap_or(accounting::posting::PostingKind::Normal),
                linked_posting_id: row.get::<_, Option<i64>>(11)?.map(PostingId),
                reversal_total: accounting::amount::from_db_amount(row.get(12)?, precision),
            }))
        } else {
            Ok(None)
        }
    }
```

- [ ] **步骤 4：修改 list_by_transaction 方法，查询并映射新字段**

将 SQL 替换为查询 13 个字段（新增 kind、linked_posting_id、reversal_total）：

```rust
        let mut stmt = conn.prepare(
            "SELECT id, transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id, kind, linked_posting_id, reversal_total
             FROM postings WHERE transaction_id = ?1"
        )?;
```

将 raw_rows 的 tuple 扩展为 13 个元素：

```rust
        let raw_rows: Vec<_> = stmt
            .query_map(params![transaction_id.0], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, i32>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                    row.get::<_, i64>(12)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
```

在构造 Posting 的循环中增加三个字段：

```rust
        for (
            id, tx_id, account_id, commodity_id, amount, cost, cost_commodity_id,
            description, member_id, channel_id, kind, linked_posting_id, reversal_total,
        ) in raw_rows
        {
            let precision = get_precision(conn, CommodityId(commodity_id))?;
            let cost_precision = cost_commodity_id
                .map(|cid| get_precision(conn, CommodityId(cid)))
                .transpose()?
                .unwrap_or(precision);
            postings.push(Posting {
                id: PostingId(id),
                transaction_id: TransactionId(tx_id),
                account_id: AccountId(account_id),
                commodity_id: CommodityId(commodity_id),
                amount: accounting::amount::from_db_amount(amount, precision),
                cost: cost.map(|c| accounting::amount::from_db_amount(c, cost_precision)),
                cost_commodity_id: cost_commodity_id.map(CommodityId),
                description,
                member_id: member_id.map(accounting::id::MemberId),
                channel_id: channel_id.map(accounting::id::ChannelId),
                kind: accounting::posting::PostingKind::from_db(kind)
                    .unwrap_or(accounting::posting::PostingKind::Normal),
                linked_posting_id: linked_posting_id.map(PostingId),
                reversal_total: accounting::amount::from_db_amount(reversal_total, precision),
            });
        }
```

- [ ] **步骤 5：对 list_by_account 做相同修改**

SQL、raw_rows tuple（13 个元素）、Posting 构造，与 `list_by_transaction` 完全一致。

- [ ] **步骤 6：修改测试辅助函数 `sample_posting`**

在 `#[cfg(test)]` 模块中的 `sample_posting` 函数，构造 Posting 时添加：

```rust
            kind: accounting::posting::PostingKind::Normal,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
```

- [ ] **步骤 7：运行 posting repo 测试**

运行：`cd /home/mechdancer/repos/accounting && cargo test -p accounting-sql repo::posting 2>&1`
预期：全部通过

- [ ] **步骤 8：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-sql/src/repo/posting.rs
git commit -m "feat(sql): PostingRepo 适配 kind/linked_posting_id/reversal_total 字段"
```

---

## 任务 4：验证层 — 退款/报销方向验证

**文件：**

- 修改：`accounting/src/validation.rs`

---

- [ ] **步骤 1：新增 validate_reversal_direction 函数**

在 `validate_transaction` 函数之后添加：

```rust
use crate::posting::{Posting, PostingKind};

/// 验证退款/报销分录的金额方向是否正确
///
/// 规则：Refund/Reimbursement 分录的金额不能为零。
/// 更严格的方向验证（与原分录方向相反）在 service 层通过数据库查询完成。
pub fn validate_reversal_direction(postings: &[Posting]) -> Result<(), AccountingError> {
    for posting in postings {
        if posting.kind == PostingKind::Normal {
            continue;
        }
        if posting.amount.is_zero() {
            return Err(AccountingError::InvalidTransaction(
                "退款/报销分录金额不能为零".to_string(),
            ));
        }
    }
    Ok(())
}
```

- [ ] **步骤 2：编译验证**

运行：`cd /home/mechdancer/repos/accounting && cargo check -p accounting 2>&1`
预期：通过

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting/src/validation.rs
git commit -m "feat(domain): 新增退款/报销金额方向验证"
```

---

## 任务 5：服务层 — TransactionService 适配与 Reversal 验证

**文件：**

- 修改：`accounting-service/src/transaction_service.rs`

---

- [ ] **步骤 1：在 submit 方法中调用 reversal 验证**

在 `validate_transaction(&postings)?;` 之后、创建数据库事务之前添加：

```rust
        validate_reversal_direction(&postings)?;
```

（需要确保 `accounting::validation::validate_reversal_direction` 已导入）

然后在数据库事务中，插入分录之前添加 reversal 数据库验证：

```rust
        // 验证退款/报销分录的关联合法性
        for posting in &postings {
            if posting.kind == accounting::posting::PostingKind::Normal {
                continue;
            }

            let linked_id = posting.linked_posting_id.ok_or_else(|| {
                AccountingError::InvalidTransaction(
                    "退款/报销分录必须关联原分录".to_string(),
                )
            })?;

            let linked_posting = tx
                .posting_repo()
                .get(&tx.conn(), linked_id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(
                        format!("关联的原分录 {} 不存在", linked_id.0)
                    )
                })?;

            if linked_posting.kind != accounting::posting::PostingKind::Normal {
                return Err(AccountingError::InvalidTransaction(
                    "只能冲减普通分录".to_string(),
                ));
            }

            if linked_posting.account_id != posting.account_id {
                return Err(AccountingError::InvalidTransaction(
                    "退款/报销必须冲减同一账户".to_string(),
                ));
            }

            if (posting.amount > Decimal::ZERO && linked_posting.amount > Decimal::ZERO)
                || (posting.amount < Decimal::ZERO && linked_posting.amount < Decimal::ZERO)
            {
                return Err(AccountingError::InvalidTransaction(
                    "退款/报销金额方向必须与原分录相反".to_string(),
                ));
            }
        }
```

- [ ] **步骤 2：在 update 方法中添加相同的验证**

在 `validate_transaction(&postings)?;` 之后添加 `validate_reversal_direction(&postings)?;`，在数据库事务中插入新分录之前添加与步骤 1 完全相同的 reversal 数据库验证代码块。

- [ ] **步骤 3：运行 transaction_service 测试**

运行：`cd /home/mechdancer/repos/accounting && cargo test -p accounting-service transaction_service 2>&1`
预期：全部通过

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-service/src/transaction_service.rs
git commit -m "feat(service): TransactionService 新增退款/报销关联验证"
```

---

## 任务 6：API 层 — DTO 与 Handler 适配

**文件：**

- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/transaction.rs`

---

- [ ] **步骤 1：修改 DTO，新增 kind 和 linked_posting_id**

将 `accounting-api/src/dto.rs` 中的 `PostingDto` 和 `PostingRequest` 修改为：

```rust
/// 分录响应。
#[derive(Serialize)]
pub struct PostingDto {
    /// 分录 ID。
    pub id: i64,
    /// 账户名称。
    pub account: String,
    /// 货币符号。
    pub commodity: String,
    /// 金额字符串。
    pub amount: String,
    /// 分录类型：normal / refund / reimbursement。
    pub kind: String,
    /// 关联原分录 ID。
    pub linked_posting_id: Option<i64>,
    /// 累计被冲减金额。
    pub reversal_total: String,
}

/// 分录请求。
#[derive(Deserialize)]
pub struct PostingRequest {
    /// 账户名称。
    pub account: String,
    /// 货币符号。
    pub commodity: String,
    /// 金额字符串。
    pub amount: String,
    /// 分录类型：normal / refund / reimbursement（默认 normal）。
    #[serde(default)]
    pub kind: String,
    /// 关联原分录 ID（退款/报销时必填）。
    pub linked_posting_id: Option<i64>,
}
```

- [ ] **步骤 2：修改 create_transaction handler 中的 Posting 构造**

在 `accounting-api/src/handlers/transaction.rs` 的 `create_transaction` 函数中， postings 构造循环内添加 kind 解析：

```rust
        let kind = match posting_req.kind.as_str() {
            "refund" => accounting::posting::PostingKind::Refund,
            "reimbursement" => accounting::posting::PostingKind::Reimbursement,
            _ => accounting::posting::PostingKind::Normal,
        };

        postings.push(Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: account.id,
            commodity_id: commodity.id,
            amount,
            cost: None,
            cost_commodity_id: None,
            description: None,
            member_id,
            channel_id: None,
            kind,
            linked_posting_id: posting_req.linked_posting_id.map(PostingId),
            reversal_total: Decimal::ZERO,
        });
```

- [ ] **步骤 3：修改 update_transaction handler 中的 Posting 构造**

与步骤 2 做相同的修改。

- [ ] **步骤 4：修改 list_transactions 和 get_transaction 中的 PostingDto 构造**

在构造 `PostingDto` 时添加新字段：

```rust
                .map(|p| PostingDto {
                    id: p.id.0,
                    account: accounts.get(&p.account_id.0).cloned().unwrap_or_default(),
                    commodity: commodities
                        .get(&p.commodity_id.0)
                        .cloned()
                        .unwrap_or_default(),
                    amount: p.amount.to_string(),
                    kind: match p.kind {
                        accounting::posting::PostingKind::Refund => "refund".to_string(),
                        accounting::posting::PostingKind::Reimbursement => "reimbursement".to_string(),
                        _ => "normal".to_string(),
                    },
                    linked_posting_id: p.linked_posting_id.map(|id| id.0),
                    reversal_total: p.reversal_total.to_string(),
                })
```

- [ ] **步骤 5：编译验证**

运行：`cd /home/mechdancer/repos/accounting && cargo check -p accounting-api 2>&1`
预期：通过

- [ ] **步骤 6：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-api/src/dto.rs accounting-api/src/handlers/transaction.rs
git commit -m "feat(api): PostingDto/Request 新增 kind/linked_posting_id/reversal_total"
```

---

## 任务 7：前端 — Store 类型适配

**文件：**

- 修改：`accounting-web/src/stores/transaction.ts`

---

- [ ] **步骤 1：修改 TypeScript 类型**

将 `accounting-web/src/stores/transaction.ts` 中的接口修改为：

```typescript
export interface Posting {
  id: number
  account: string
  commodity: string
  amount: string
  kind: string
  linked_posting_id?: number
  reversal_total: string
}

export interface PostingInput {
  account: string
  commodity: string
  amount: string
  kind?: string
  linked_posting_id?: number
}

export interface Transaction {
  id: number
  date_time: string
  description: string
  member_id?: number
  channel_id?: number
  is_template: boolean
  postings: Posting[]
  tags?: string[]
}

export interface CreateTransactionData {
  date_time: string
  description: string
  member_id?: number
  channel_id?: number
  postings: PostingInput[]
  tags: string[]
}
```

- [ ] **步骤 2：编译验证**

运行：`cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit 2>&1`
预期：通过（可能有未使用属性的警告，但无类型错误）

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/stores/transaction.ts
git commit -m "feat(web): transaction store 类型新增 kind/linked_posting_id/reversal_total"
```

---

## 任务 8：前端 — TransactionForm 分录行增加 kind 选择

**文件：**

- 修改：`accounting-web/src/views/TransactionForm.vue`

---

- [ ] **步骤 1：在分录行中添加 kind 选择器**

在分录列表的每一项（posting row）中，在账户选择器和金额输入之间（或旁边）添加一个 kind 下拉选择：

```vue
<a-select v-model:value="p.kind" style="width: 80px">
  <a-select-option value="normal">普通</a-select-option>
  <a-select-option value="refund">退款</a-select-option>
  <a-select-option value="reimbursement">报销</a-select-option>
</a-select>
```

默认值为 `"normal"`。

- [ ] **步骤 2：退款/报销时显示原分录选择**

当 `p.kind !== 'normal'` 时，显示一个输入框或选择器让用户输入 `linked_posting_id`：

```vue
<a-input-number
  v-if="p.kind !== 'normal'"
  v-model:value="p.linked_posting_id"
  placeholder="原分录 ID"
  style="width: 120px"
/>
```

（更优方案：弹出一个原交易选择器，选择交易后再选择该交易的具体分录。但作为第一步，先使用简单的 ID 输入。）

- [ ] **步骤 3：提交数据包含新字段**

确保 `handleSubmit` 中构造的 `postings` 数组包含 `kind` 和 `linked_posting_id`：

```typescript
const postings = postingList.value.map(p => ({
  account: p.account,
  commodity: p.commodity,
  amount: p.amount,
  kind: p.kind || 'normal',
  linked_posting_id: p.linked_posting_id,
}))
```

- [ ] **步骤 4：构建验证**

运行：`cd /home/mechdancer/repos/accounting/accounting-web && npm run build 2>&1`
预期：通过

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/views/TransactionForm.vue
git commit -m "feat(web): 交易表单分录行增加 kind 和 linked_posting_id"
```

---

## 任务 9：前端 — TransactionDetail 展示 kind 和 reversal_total

**文件：**

- 修改：`accounting-web/src/components/TransactionDetail.vue`

---

- [ ] **步骤 1：在交易 header 中展示 reversal_total**

对于每个 posting，如果是 Normal kind 且 `reversal_total !== '0'` 且 `reversal_total !== '0.00'`，在金额旁边显示净额：

```vue
<span class="transaction-amount" :class="amountColorClass">
  ¥{{ totalAmount.toFixed(2) }}
  <span v-if="netAmount !== totalAmount" class="net-amount">
    (净额 ¥{{ netAmount.toFixed(2) }})
  </span>
</span>
```

其中 `netAmount` 计算方式：`totalAmount + parseFloat(reversalTotal)`。

注意：`reversal_total` 是字符串，且可能是负数（如 `"-20.00"`）。

- [ ] **步骤 2：在展开的分录详情中展示 kind 标记**

在 `posting-row` 中，对每个 posting 显示 kind 标记：

```vue
<span v-if="p.kind === 'refund'" class="tag-refund">退</span>
<span v-if="p.kind === 'reimbursement'" class="tag-reimbursement">报</span>
```

以及 linked_posting_id：

```vue
<span v-if="p.linked_posting_id" class="linked-info">
  冲减分录 #{{ p.linked_posting_id }}
</span>
```

- [ ] **步骤 3：添加样式**

```css
.tag-refund {
  color: #fa8c16;
  font-size: 11px;
  border: 1px solid #fa8c16;
  padding: 0 4px;
  border-radius: 4px;
  margin-left: 4px;
}

.tag-reimbursement {
  color: #1890ff;
  font-size: 11px;
  border: 1px solid #1890ff;
  padding: 0 4px;
  border-radius: 4px;
  margin-left: 4px;
}

.net-amount {
  color: #999;
  font-size: 12px;
  margin-left: 4px;
}
```

- [ ] **步骤 4：构建验证**

运行：`cd /home/mechdancer/repos/accounting/accounting-web && npm run build 2>&1`
预期：通过

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/src/components/TransactionDetail.vue
git commit -m "feat(web): 交易详情展示 kind 标记、reversal_total 和净额"
```

---

## 任务 10：端到端验证

---

- [ ] **步骤 1：运行后端全部测试**

运行：`cd /home/mechdancer/repos/accounting && cargo test 2>&1`
预期：全部通过（90+ tests）

- [ ] **步骤 2：运行前端构建**

运行：`cd /home/mechdancer/repos/accounting/accounting-web && npm run build 2>&1`
预期：通过

- [ ] **步骤 3：运行 cargo clippy**

运行：`cd /home/mechdancer/repos/accounting && cargo clippy --all-targets 2>&1`
预期：无 error（允许现有 warning）

- [ ] **步骤 4：最终 Commit**

```bash
cd /home/mechdancer/repos/accounting
git add -A
git commit -m "feat: 实现退款与报销功能

- postings 表新增 kind/linked_posting_id/reversal_total
- 新增触发器自动维护 reversal_total
- 新增 PostingKind 枚举（Normal/Refund/Reimbursement）
- 新增退款/报销验证规则
- API 和前端适配 kind/linked_posting_id/reversal_total
- 前端展示净额和退/报标记"
```

---

## 自检

**规格覆盖度：**

- ✅ 数据模型（kind/linked_posting_id/reversal_total）— 任务 1、2、3
- ✅ 触发器维护 reversal_total — 任务 2
- ✅ 验证规则（数据库查询验证）— 任务 4、5
- ✅ API 适配 — 任务 6
- ✅ 前端表单（kind 选择）— 任务 8
- ✅ 前端详情展示（净额、退/报标记）— 任务 9
- ✅ 索引 — 任务 2

**占位符扫描：** 无 TODO、无"待定"、无"后续实现"。

**类型一致性：**

- `PostingKind` 在 domain、sql repo、service、api 中一致使用
- `linked_posting_id` 在 dto 中为 `Option<i64>`，在 domain 中为 `Option<PostingId>`
- `kind` 在 dto 中为 `String`（"normal"/"refund"/"reimbursement"），在 domain 中为 `PostingKind`

---

## 执行交接

**计划已完成并保存到 `docs/superpowers/plans/2026-06-05-refund-reimbursement-plan.md`。两种执行方式：**

**1. 子代理驱动（推荐）** - 每个任务调度一个新的子代理，任务间进行审查，快速迭代

**2. 内联执行** - 在当前会话中使用 executing-plans 执行任务，批量执行并设有检查点供审查

**选哪种方式？**
