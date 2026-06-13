# 退款/报销 UI 重设计实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 重构 `kind` 从 postings 到 transactions，新增 `is_reimbursable` 字段，重新设计 Dashboard 退款/报销模式 UI 流程，用户通过点选分录替代手动输入 `linked_posting_id`。

**架构：** `kind` 从 `postings` 移到 `transactions` 实现交易级语义纯净，`linked_posting_id IS NOT NULL` 标识冲减分录，新增独立路由 `/transaction/refund` 和 `/transaction/reimbursement` 共用 `TransactionForm.vue` 组件。

**技术栈：** Rust (axum, rusqlite, rust_decimal), Vue 3 + Vite + TypeScript + AntDV + Pinia

**提交前必须：** 每次 commit 前运行 `cargo fmt`、`cargo clippy`、`cargo test`（全 workspace）。

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `accounting/src/transaction.rs` | `Transaction` 新增 `kind: TransactionKind`（从 posting 迁移） |
| `accounting/src/posting.rs` | `Posting` 删除 `kind`，新增 `is_reimbursable: bool` |
| `accounting/src/validation.rs` | 校验适配 `linked_posting_id` + 新增冲减金额上限、交易级一致性 |
| `accounting/src/lib.rs` | 导出 `TransactionKind`（替代原 `PostingKind`） |
| `accounting-sql/src/schema.rs` | Schema 迁移：transactions 加 kind，postings 删 kind 加 is_reimbursable |
| `accounting-sql/src/repo/transaction.rs` | TransactionRow 读写 kind；list 支持 has_reimbursable |
| `accounting-sql/src/repo/posting.rs` | Posting 读写删除 kind 列，新增 is_reimbursable 列 |
| `accounting-service/src/transaction_service.rs` | 适配字段变更 + 冲减金额上限校验 |
| `accounting-api/src/dto.rs` | TransactionDto 加 kind，PostingDto 删 kind 加 is_reimbursable |
| `accounting-api/src/handlers/transaction.rs` | 适配字段 + TxQuery 加 reimbursable + 新增 posting 详情接口 |
| `accounting-api/src/router.rs` | 新增 `/api/postings/:id` 路由 |
| `accounting-web/src/stores/transaction.ts` | TypeScript 类型适配 + fetchPosting |
| `accounting-web/src/router/index.ts` | 新增 `/transaction/refund`、`/transaction/reimbursement` |
| `accounting-web/src/components/Calendar.vue` | rangeMode → mode prop |
| `accounting-web/src/components/TransactionDetail.vue` | selectable 模式 + 可报销高亮 |
| `accounting-web/src/views/TransactionForm.vue` | 删除分录级 kind 选择器 + 报销按钮 + 专用模式 |
| `accounting-web/src/views/Dashboard.vue` | 4 模式切换 + selectedPostings + 底部抽屉 |
| `accounting-web/src/App.vue` | 暗色主题样式 |

---

### 任务 1：核心数据模型 — TransactionKind 迁移与 Posting 更新

**文件：**

- 修改：`accounting/src/posting.rs`
- 修改：`accounting/src/transaction.rs`
- 修改：`accounting/src/lib.rs`

---

- [ ] **步骤 1：从 posting.rs 移除 PostingKind，修改 Posting 结构体**

将 `accounting/src/posting.rs` 完整替换为：

```rust
use crate::id::{AccountId, ChannelId, CommodityId, MemberId, PostingId, TransactionId};
use rust_decimal::Decimal;

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
    /// 可报销标记（仅 Expense 类账户可设置）
    pub is_reimbursable: bool,
    /// 关联原分录 ID（非空表示该分录是冲减分录）
    pub linked_posting_id: Option<PostingId>,
    /// 累计被冲减金额（由触发器自动维护）
    pub reversal_total: Decimal,
}
```

- [ ] **步骤 2：在 transaction.rs 定义 TransactionKind 并扩展 Transaction**

将 `accounting/src/transaction.rs` 完整替换为：

```rust
use crate::id::{ChannelId, MemberId, TransactionId};
use chrono::NaiveDateTime;

/// 交易类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionKind {
    /// 普通交易
    Normal = 1,
    /// 退款交易
    Refund = 2,
    /// 报销交易
    Reimbursement = 3,
}

impl TransactionKind {
    /// 从数据库整数值解析
    pub fn from_db(value: i32) -> Option<Self> {
        match value {
            1 => Some(TransactionKind::Normal),
            2 => Some(TransactionKind::Refund),
            3 => Some(TransactionKind::Reimbursement),
            _ => None,
        }
    }
}

/// 交易
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// 交易唯一标识符
    pub id: TransactionId,
    /// 交易时间
    pub date_time: NaiveDateTime,
    /// 交易描述
    pub description: String,
    /// 交易类型（普通/退款/报销）
    pub kind: TransactionKind,
    /// 关联成员 ID
    pub member_id: Option<MemberId>,
    /// 支付渠道 ID
    pub channel_id: Option<ChannelId>,
    /// 是否为模板交易
    pub is_template: bool,
}
```

- [ ] **步骤 3：更新 lib.rs 导出**

编辑 `accounting/src/lib.rs`，将：

```rust
pub use posting::PostingKind;
```

替换为：

```rust
pub use transaction::TransactionKind;
```

- [ ] **步骤 4：编译验证**

```bash
cd /home/mechdancer/repos/accounting && cargo check 2>&1 | head -80
```

预期：因其他 crate 引用 `Posting.kind` 和 `PostingKind` 而出现编译错误（后续任务修复）。

- [ ] **步骤 5：Commit**

```bash
git add accounting/src/posting.rs accounting/src/transaction.rs accounting/src/lib.rs
git commit -m "refactor: move kind from Posting to Transaction; add TransactionKind enum and is_reimbursable field"
```

---

### 任务 2：更新核心校验逻辑

**文件：**

- 修改：`accounting/src/validation.rs`

---

- [ ] **步骤 1：更新 validation.rs**

将 `accounting/src/validation.rs` 完整替换为：

```rust
use crate::account_type::AccountType;
use crate::error::AccountingError;
use crate::posting::Posting;
use crate::transaction::TransactionKind;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 验证交易平衡性
///
/// 规则：
/// - 至少两个分录
/// - 同一 commodity 的金额之和为零
/// - 不同 commodity 的交易必须有 cost 字段建立等式
pub fn validate_transaction(postings: &[Posting]) -> Result<(), AccountingError> {
    if postings.len() < 2 {
        return Err(AccountingError::InvalidTransaction(
            "交易至少包含两个分录".to_string(),
        ));
    }

    // 按 commodity 分组求和
    let mut sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        *sums
            .entry(p.commodity_id.0)
            .or_insert_with(|| Decimal::ZERO) += p.amount;
    }

    // 检查是否所有 commodity 都能自平衡
    let unbalanced: Vec<_> = sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced.is_empty() {
        return Ok(());
    }

    // 多币种情况：检查 cost 是否能建立等式
    let mut cost_sums: HashMap<i64, Decimal> = HashMap::new();
    for p in postings {
        if let Some(cost) = p.cost {
            let cost_commodity = p.cost_commodity_id.map(|c| c.0).unwrap_or(p.commodity_id.0);
            *cost_sums
                .entry(cost_commodity)
                .or_insert_with(|| Decimal::ZERO) += cost;
        } else {
            *cost_sums
                .entry(p.commodity_id.0)
                .or_insert_with(|| Decimal::ZERO) += p.amount;
        }
    }

    let unbalanced_costs: Vec<_> = cost_sums.iter().filter(|(_, v)| !v.is_zero()).collect();
    if unbalanced_costs.is_empty() {
        Ok(())
    } else {
        Err(AccountingError::InvalidTransaction(
            "交易不平衡".to_string(),
        ))
    }
}

/// 验证交易级 kind 与分录结构的一致性
///
/// 规则：
/// - Normal 交易中所有分录的 linked_posting_id 必须为 None
/// - Refund/Reimbursement 交易必须至少有一个分录的 linked_posting_id 不为 None
pub fn validate_kind_consistency(kind: TransactionKind, postings: &[Posting]) -> Result<(), AccountingError> {
    let has_reversal = postings.iter().any(|p| p.linked_posting_id.is_some());
    match kind {
        TransactionKind::Normal => {
            if has_reversal {
                return Err(AccountingError::InvalidTransaction(
                    "普通交易不能包含冲减分录".to_string(),
                ));
            }
        }
        TransactionKind::Refund | TransactionKind::Reimbursement => {
            if !has_reversal {
                return Err(AccountingError::InvalidTransaction(
                    "退款/报销交易必须包含冲减分录".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// 验证冲减分录金额不为零
///
/// 更严格的方向验证（与原分录方向相反）在 service 层通过数据库查询完成。
pub fn validate_reversal_direction(postings: &[Posting]) -> Result<(), AccountingError> {
    for posting in postings {
        if posting.linked_posting_id.is_none() {
            continue;
        }
        if posting.amount.is_zero() {
            return Err(AccountingError::InvalidTransaction(
                "冲减分录金额不能为零".to_string(),
            ));
        }
    }
    Ok(())
}

/// 验证冲减金额不超过原分录剩余可冲减额度
///
/// `linked_amount`: 冲减分录金额
/// `original_amount`: 原分录金额
/// `existing_reversal_total`: 原分录已被其他冲减分录冲减的累计金额
pub fn validate_reversal_cap(
    linked_amount: Decimal,
    original_amount: Decimal,
    existing_reversal_total: Decimal,
) -> Result<(), AccountingError> {
    let used = existing_reversal_total.abs() + linked_amount.abs();
    let available = original_amount.abs();
    if used > available {
        return Err(AccountingError::InvalidTransaction(format!(
            "冲减金额超出原分录剩余额度: 已冲减 {}, 本次冲减 {}, 原金额 {}",
            existing_reversal_total.abs(),
            linked_amount.abs(),
            available,
        )));
    }
    Ok(())
}

/// 验证账户是否可以关闭
///
/// Asset 和 Liability 必须余额为零；Income 和 Expense 无限制
pub fn validate_account_close(
    account_type: AccountType,
    balances: &[(crate::id::CommodityId, Decimal)],
) -> Result<(), AccountingError> {
    match account_type {
        AccountType::Asset
        | AccountType::Liability
        | AccountType::Expense
        | AccountType::Equity => {
            let non_zero: Vec<_> = balances.iter().filter(|(_, b)| !b.is_zero()).collect();
            if !non_zero.is_empty() {
                return Err(AccountingError::AccountNotEmpty("账户余额非零".to_string()));
            }
        }
        AccountType::Income => {
            // Income 账户关闭无限制
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{AccountId, CommodityId, PostingId, TransactionId};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn posting(
        account_id: i64,
        commodity_id: i64,
        amount: &str,
        cost: Option<&str>,
        cost_commodity: Option<i64>,
        linked_posting_id: Option<i64>,
    ) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id: AccountId(account_id),
            commodity_id: CommodityId(commodity_id),
            amount: Decimal::from_str(amount).unwrap(),
            cost: cost.map(|c| Decimal::from_str(c).unwrap()),
            cost_commodity_id: cost_commodity.map(CommodityId),
            description: None,
            member_id: None,
            channel_id: None,
            is_reimbursable: false,
            linked_posting_id: linked_posting_id.map(PostingId),
            reversal_total: Decimal::ZERO,
        }
    }

    #[test]
    fn test_empty_postings_fails() {
        let postings: Vec<Posting> = vec![];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_single_posting_fails() {
        let postings = vec![posting(1, 1, "100", None, None, None)];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_balanced_same_commodity_passes() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_unbalanced_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-50", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_multi_commodity_with_cost_passes() {
        let postings = vec![
            posting(1, 1, "100", Some("70000"), Some(2), None),
            posting(2, 2, "-70000", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_ok());
    }

    #[test]
    fn test_multi_commodity_without_cost_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 2, "-700", None, None, None),
        ];
        assert!(validate_transaction(&postings).is_err());
    }

    #[test]
    fn test_normal_tx_with_reversal_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, Some(1)),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Normal, &postings).is_err());
    }

    #[test]
    fn test_refund_tx_without_reversal_fails() {
        let postings = vec![
            posting(1, 1, "100", None, None, None),
            posting(2, 1, "-100", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Refund, &postings).is_err());
    }

    #[test]
    fn test_refund_tx_with_reversal_passes() {
        let postings = vec![
            posting(1, 1, "-50", None, None, Some(1)),
            posting(2, 1, "50", None, None, None),
        ];
        assert!(validate_kind_consistency(TransactionKind::Refund, &postings).is_ok());
    }

    #[test]
    fn test_reversal_cap_exceeded_fails() {
        assert!(validate_reversal_cap(
            Decimal::from_str("60").unwrap(),
            Decimal::from_str("100").unwrap(),
            Decimal::from_str("50").unwrap(),
        ).is_err());
    }

    #[test]
    fn test_reversal_cap_within_limit_passes() {
        assert!(validate_reversal_cap(
            Decimal::from_str("40").unwrap(),
            Decimal::from_str("100").unwrap(),
            Decimal::from_str("50").unwrap(),
        ).is_ok());
    }

    #[test]
    fn test_reversal_direction_zero_rejected() {
        let postings = vec![
            posting(1, 1, "0", None, None, Some(1)),
            posting(2, 1, "0", None, None, None),
        ];
        assert!(validate_reversal_direction(&postings).is_err());
    }

    #[test]
    fn test_close_asset_with_zero_balance_ok() {
        let balances = vec![(CommodityId(1), Decimal::ZERO)];
        assert!(validate_account_close(AccountType::Asset, &balances).is_ok());
    }

    #[test]
    fn test_close_asset_with_non_zero_balance_fails() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Asset, &balances).is_err());
    }

    #[test]
    fn test_close_income_unconditionally_ok() {
        let balances = vec![(CommodityId(1), Decimal::from_str("100").unwrap())];
        assert!(validate_account_close(AccountType::Income, &balances).is_ok());
    }
}
```

- [ ] **步骤 2：运行测试验证**

```bash
cd /home/mechdancer/repos/accounting && cargo test -p accounting --lib validation::tests 2>&1 | tail -20
```

预期：新增测试 PASS（因 posting 字段变更可能导致编译错误，需先修复引用再跑）。

- [ ] **步骤 3：Commit**

```bash
git add accounting/src/validation.rs
git commit -m "feat: update validation - use linked_posting_id for reversal detection, add kind consistency and reversal cap checks"
```

---

### 任务 3：数据库 Schema 迁移

**文件：**

- 修改：`accounting-sql/src/schema.rs`

---

- [ ] **步骤 1：更新 SCHEMA_SQL**

在 `accounting-sql/src/schema.rs` 中：

1. 在 `CREATE TABLE transactions` 的 `is_template` 后新增 `kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3)`。
2. 在 `CREATE TABLE postings` 中删除 `kind` 列，新增 `is_reimbursable INTEGER NOT NULL DEFAULT 0`。
3. 删除 `CREATE INDEX idx_postings_kind`。
4. 新增 `CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind)`。
5. 新增 `CREATE INDEX IF NOT EXISTS idx_postings_reimbursable ON postings(is_reimbursable)`。

编辑 `accounting-sql/src/schema.rs` 的 `SCHEMA_SQL` 常量中相关部分：

```sql
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date_time TEXT NOT NULL,
    description TEXT NOT NULL,
    member_id INTEGER REFERENCES members(id),
    channel_id INTEGER REFERENCES channels(id),
    is_template INTEGER NOT NULL DEFAULT 0,
    kind INTEGER NOT NULL DEFAULT 1 CHECK(kind BETWEEN 1 AND 3),
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);
```

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
    is_reimbursable INTEGER NOT NULL DEFAULT 0,
    linked_posting_id INTEGER REFERENCES postings(id),
    reversal_total INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (date('now')),
    updated_at TEXT NOT NULL DEFAULT (date('now'))
);
```

在索引区域删除 `idx_postings_kind` 行，新增：

```sql
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind);
CREATE INDEX IF NOT EXISTS idx_postings_reimbursable ON postings(is_reimbursable);
```

由于项目使用 in-memory / 新建数据库方式，无需 migration 脚本。如果已存在数据库文件，删除后重建。

- [ ] **步骤 2：Commit**

```bash
git add accounting-sql/src/schema.rs
git commit -m "feat: schema migration - move kind to transactions, add is_reimbursable to postings"
```

---

### 任务 4：更新 PostingRepo（删 kind、加 is_reimbursable）

**文件：**

- 修改：`accounting-sql/src/repo/posting.rs`

---

- [ ] **步骤 1：更新 insert 方法**

编辑 `accounting-sql/src/repo/posting.rs` 的 `insert` 方法（约第 95-112 行），将 SQL 改为：

```sql
INSERT INTO postings
 (transaction_id, account_id, commodity_id, amount, cost, cost_commodity_id, description, member_id, channel_id, is_reimbursable, linked_posting_id)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
```

参数 `params!` 中将 `posting.kind as i32` 替换为 `posting.is_reimbursable as i32`。

- [ ] **步骤 2：更新 SELECT 查询（get / list_by_transaction / list_by_account）**

所有 SELECT 语句的列列表中：

- 将 `kind` 替换为 `is_reimbursable`
- linked_posting_id 索引号因删除 kind 前面一列而变化（列宽一致，无影响）

在 `map_posting` / 行映射逻辑中：

- 将 `kind: ...` 替换为 `is_reimbursable: row.get::<_, i32>(N)? != 0`
- 删除 `PostingKind::from_db(...)` 逻辑

- [ ] **步骤 3：更新测试辅助函数**

编辑测试中的 `sample_posting` / helper 函数，将 `kind: ...` 替换为 `is_reimbursable: false`。

- [ ] **步骤 4：运行 repo 测试**

```bash
cd /home/mechdancer/repos/accounting && cargo test -p accounting-sql --lib repo::posting::tests 2>&1 | tail -20
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-sql/src/repo/posting.rs
git commit -m "refactor: update PostingRepo - drop kind column, add is_reimbursable"
```

---

### 任务 5：更新 TransactionRepo（加 kind、has_reimbursable 过滤）

**文件：**

- 修改：`accounting-sql/src/repo/transaction.rs`

---

- [ ] **步骤 1：更新 insert SQL**

编辑 `insert` 方法（约第 57-67 行），SQL 改为：

```sql
INSERT INTO transactions (date_time, description, member_id, channel_id, is_template, kind)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
```

参数增加 `tx.kind as i32`。

- [ ] **步骤 2：更新 update SQL**

编辑 `update` 方法（约第 237-249 行），SQL 改为：

```sql
UPDATE transactions
 SET date_time = ?1, description = ?2, member_id = ?3, channel_id = ?4, is_template = ?5, kind = ?6
 WHERE id = ?7
```

参数增加 `tx.kind as i32`，`tx.id.0` 索引变为 `?7`。

- [ ] **步骤 3：更新 SELECT 查询和 map_transaction**

`get`/`list`/`count` 中的 SELECT 列列表增加 `kind`。

`map_transaction` 函数（约第 264-278 行）中：

- `is_template` 的索引从 `row.get::<_, i32>(5)?` 变为 `row.get::<_, i32>(6)?`
- 新增：`kind: TransactionKind::from_db(row.get::<_, i32>(5)?).unwrap_or(TransactionKind::Normal)`

- [ ] **步骤 4：添加 has_reimbursable 过滤**

在 `list` 方法中，`filter.has_reimbursable` 处理追加到动态条件构建中（约第 139 行之后）：

```rust
if let Some(true) = filter.has_reimbursable {
    if !joins.iter().any(|j| j.contains("postings")) {
        joins.push("JOIN postings p_reimb ON p_reimb.transaction_id = transactions.id");
    }
    conditions.push("p_reimb.is_reimbursable = 1");
}
```

在 `count` 方法中也添加相同逻辑。

- [ ] **步骤 5：更新测试 helper**

编辑 `sample_tx()` 增加 `kind: TransactionKind::Normal`。

- [ ] **步骤 6：运行测试**

```bash
cd /home/mechdancer/repos/accounting && cargo test -p accounting-sql --lib repo::transaction::tests 2>&1 | tail -20
```

- [ ] **步骤 7：Commit**

```bash
git add accounting-sql/src/repo/transaction.rs
git commit -m "feat: add kind to TransactionRepo, support has_reimbursable filter"
```

---

### 任务 6：更新 TransactionService

**文件：**

- 修改：`accounting-service/src/transaction_service.rs`

---

- [ ] **步骤 1：更新 submit 方法**

修改 `submit` 方法：

1. 在 `validate_transaction(&postings)?` 后新增 `validate_kind_consistency(transaction.kind, &postings)?;`
2. 冲减分录循环（约第 44-84 行）中：
   - 将 `posting.kind == PostingKind::Normal` 改为 `posting.linked_posting_id.is_none()`
   - 将 `linked_posting.kind != PostingKind::Normal` 改为 `linked_posting.linked_posting_id.is_some()`（检查被指向分录不是冲减分录）
   - 在方向检查通过后新增金额上限校验：

```rust
let remaining = linked_posting.amount.abs() - linked_posting.reversal_total.abs();
if posting.amount.abs() > remaining {
    return Err(AccountingError::InvalidTransaction(format!(
        "冲减金额 {} 超出原分录剩余额度 {}",
        posting.amount.abs(),
        remaining,
    )));
}
```

1. 导入调整：将 `use accounting::posting::PostingKind` 替换为 `use accounting::transaction::TransactionKind`。

- [ ] **步骤 2：更新 update 方法**（相同逻辑适配）

同样的改动应用到 `update` 方法。

- [ ] **步骤 3：更新测试 helper**

`sample_posting` 中将 `kind: PostingKind::Normal` 替换为 `is_reimbursable: false`。
`sample_tx` 构造中增加 `kind: TransactionKind::Normal`。

- [ ] **步骤 4：运行测试**

```bash
cd /home/mechdancer/repos/accounting && cargo test -p accounting-service --lib 2>&1 | tail -20
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-service/src/transaction_service.rs
git commit -m "feat: adapt TransactionService - kind on transaction, reversal cap validation"
```

---

### 任务 7：更新 API DTO 与 Handler

**文件：**

- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/transaction.rs`

---

- [ ] **步骤 1：更新 DTO**

编辑 `accounting-api/src/dto.rs`：

**TransactionDto** 新增 `kind`：

```rust
pub struct TransactionDto {
    pub id: i64,
    pub date_time: String,
    pub description: String,
    pub kind: String,
    pub member_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub is_template: bool,
    pub postings: Vec<PostingDto>,
}
```

**TransactionDetailDto** 同。

**CreateTransactionRequest** 新增 `kind`：

```rust
pub struct CreateTransactionRequest {
    pub date_time: String,
    pub description: String,
    #[serde(default)]
    pub kind: String,
    pub member_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub postings: Vec<PostingRequest>,
    pub tags: Vec<String>,
}
```

**PostingDto** 删除 `kind`，新增 `is_reimbursable`：

```rust
pub struct PostingDto {
    pub id: i64,
    pub account: String,
    pub commodity: String,
    pub amount: String,
    pub is_reimbursable: bool,
    pub linked_posting_id: Option<i64>,
    pub reversal_total: String,
}
```

**PostingRequest** 删除 `kind`，新增 `is_reimbursable`：

```rust
pub struct PostingRequest {
    pub account: String,
    pub commodity: String,
    pub amount: String,
    #[serde(default)]
    pub is_reimbursable: bool,
    pub linked_posting_id: Option<i64>,
}
```

- [ ] **步骤 2：更新 handler**

编辑 `accounting-api/src/handlers/transaction.rs`：

1. 更新 `TxQuery` 新增 `reimbursable` 参数：

```rust
pub struct TxQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub account: Option<i64>,
    pub member: Option<i64>,
    pub tag: Option<String>,
    pub keyword: Option<String>,
    pub reimbursable: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

1. 在 `list_transactions` 中，处理 `reimbursable` 参数（约第 91 行后）：

```rust
if let Some(true) = query.reimbursable {
    filter.has_reimbursable = Some(true);
}
```

1. 在 `list_transactions` 的结果映射中（约第 119-149 行）：
   - `TransactionDto` 新增 `kind` 字段映射
   - `PostingDto` 将 `kind: match p.kind { ... }` 替换为 `is_reimbursable: p.is_reimbursable`
   - `PostingDto` 删除 `kind` 行

2. 在 `create_transaction` 中（约第 166-203 行）：
   - 解析 `req.kind`：`"refund" => TransactionKind::Refund, "reimbursement" => TransactionKind::Reimbursement, _ => TransactionKind::Normal`
   - `Posting` 构造中将 `kind` 替换为 `is_reimbursable: posting_req.is_reimbursable`
   - `Transaction` 构造中新增 `kind`

3. `get_transaction` 同理适配字段。

4. 新增单个分录查询 handler：

```rust
async fn get_posting(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<PostingDto>, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    let posting = db
        .posting_repo()
        .get(&conn, PostingId(id))
        .map_err(|e| e.to_string())?
        .ok_or("Posting not found")?;

    let accounts: std::collections::HashMap<i64, String> = db
        .account_repo()
        .list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|a| (a.id.0, a.full_name))
        .collect();
    let commodities: std::collections::HashMap<i64, String> = db
        .commodity_repo()
        .list(&conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| (c.id.0, c.symbol))
        .collect();

    Ok(Json(PostingDto {
        id: posting.id.0,
        account: accounts.get(&posting.account_id.0).cloned().unwrap_or_default(),
        commodity: commodities.get(&posting.commodity_id.0).cloned().unwrap_or_default(),
        amount: posting.amount.to_string(),
        is_reimbursable: posting.is_reimbursable,
        linked_posting_id: posting.linked_posting_id.map(|id| id.0),
        reversal_total: posting.reversal_total.to_string(),
    }))
}
```

1. 路由注册增加：

```rust
.route("/api/postings/:id", get(get_posting))
```

- [ ] **步骤 3：编译验证全后端**

```bash
cd /home/mechdancer/repos/accounting && cargo check 2>&1 | tail -5
```

预期：编译通过，无错误。

- [ ] **步骤 4：运行全部后端测试**

```bash
cd /home/mechdancer/repos/accounting && cargo test 2>&1 | tail -20
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-api/src/dto.rs accounting-api/src/handlers/transaction.rs
git commit -m "feat: update API - TransactionDto gets kind, PostingDto gets is_reimbursable, add GET /api/postings/:id"
```

---

### 任务 8：前端类型定义与 Store 适配

**文件：**

- 修改：`accounting-web/src/stores/transaction.ts`

---

- [ ] **步骤 1：更新 TypeScript 接口**

将 `accounting-web/src/stores/transaction.ts` 完整替换为：

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Posting {
  id: number
  account: string
  commodity: string
  amount: string
  is_reimbursable: boolean
  linked_posting_id?: number
  reversal_total: string
}

export interface Transaction {
  id: number
  date_time: string
  description: string
  kind: string
  member_id?: number
  channel_id?: number
  is_template: boolean
  postings: Posting[]
  tags?: string[]
}

export interface PostingInput {
  account: string
  commodity: string
  amount: string
  is_reimbursable?: boolean
  linked_posting_id?: number
}

export interface CreateTransactionData {
  date_time: string
  description: string
  kind: string
  member_id?: number
  channel_id?: number
  postings: PostingInput[]
  tags: string[]
}

export const useTransactionStore = defineStore('transaction', () => {
  const transactions = ref<Transaction[]>([])
  const loading = ref(false)

  async function fetchTransactions(params?: Record<string, unknown>) {
    loading.value = true
    try {
      const res = await api.get<Transaction[]>('/transactions', { params })
      transactions.value = res.data
    } catch (e) {
      console.error('获取交易失败', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchPosting(id: number): Promise<Posting> {
    const res = await api.get<Posting>(`/postings/${id}`)
    return res.data
  }

  async function createTransaction(data: CreateTransactionData) {
    await api.post('/transactions', data)
  }

  async function updateTransaction(id: number, data: CreateTransactionData) {
    await api.put(`/transactions/${id}`, data)
  }

  async function deleteTransaction(id: number) {
    await api.delete(`/transactions/${id}`)
  }

  return { transactions, loading, fetchTransactions, fetchPosting, createTransaction, updateTransaction, deleteTransaction }
})
```

- [ ] **步骤 2：Commit**

```bash
git add accounting-web/src/stores/transaction.ts
git commit -m "feat: update TS types - Posting gets is_reimbursable, Transaction gets kind, add fetchPosting"
```

---

### 任务 9：新增前端路由

**文件：**

- 修改：`accounting-web/src/router/index.ts`

---

- [ ] **步骤 1：添加路由**

编辑 `accounting-web/src/router/index.ts`，在 routes 数组中增加：

```typescript
{ path: '/transaction/refund', component: TransactionForm },
{ path: '/transaction/reimbursement', component: TransactionForm },
```

完整 routes：

```typescript
const routes = [
  { path: '/', component: Dashboard },
  { path: '/transaction', component: TransactionForm },
  { path: '/transaction/refund', component: TransactionForm },
  { path: '/transaction/reimbursement', component: TransactionForm },
  { path: '/accounts', component: AccountTree },
  { path: '/tags', component: Tags },
  { path: '/reports', component: ReportView },
]
```

- [ ] **步骤 2：Commit**

```bash
git add accounting-web/src/router/index.ts
git commit -m "feat: add routes /transaction/refund and /transaction/reimbursement"
```

---

### 任务 10：Calendar 组件 — rangeMode → mode prop

**文件：**

- 修改：`accounting-web/src/components/Calendar.vue`
- 修改：`accounting-web/src/views/Dashboard.vue`（调整 Calendar 绑定）

---

- [ ] **步骤 1：修改 Calendar.vue**

将 prop `rangeMode: Boolean` 替换为 `mode: String`：

```typescript
props: {
  data: { type: Array as PropType<CalendarDay[]>, required: true },
  mode: { type: String as PropType<'normal' | 'range' | 'refund' | 'reimbursement'>, default: 'normal' },
}
```

在模板中将 `rangeMode` 引用替换为 `mode === 'range'`：

- 日期范围选择逻辑：`if (props.mode === 'range') { ... }`
- 范围模式样式：`:class="{ range: mode === 'range' }"`

emit 保持不变：`@select`、`@select-range`、`@clear`。

- [ ] **步骤 2：更新 Dashboard.vue 的 Calendar 绑定**

将 `<Calendar :range-mode="rangeMode" ...>` 改为 `<Calendar :mode="mode" ...>`。

- [ ] **步骤 3：Commit**

```bash
git add accounting-web/src/components/Calendar.vue accounting-web/src/views/Dashboard.vue
git commit -m "refactor: Calendar rangeMode boolean prop → mode string prop"
```

---

### 任务 11：TransactionDetail 组件 — selectable 模式 + 可报销高亮

**文件：**

- 修改：`accounting-web/src/components/TransactionDetail.vue`

---

在 `TransactionDetail.vue` 中：

- [ ] **步骤 1：新增 Props**

```typescript
props: {
  tx: { type: Object as PropType<Transaction>, required: true },
  selectable: { type: Boolean, default: false },
  selectableFilter: { type: Function as PropType<(p: Posting) => boolean>, default: null },
  selectedPostingIds: { type: Set as PropType<Set<number>>, default: () => new Set() },
}
emit: ['deleted', 'select-posting'],
```

- [ ] **步骤 2：分录行模板**

```vue
<div
  v-for="p in tx.postings"
  :key="p.id"
  class="posting-row"
  :class="{
    selectable: selectable,
    'selectable-disabled': selectableFilter && !selectableFilter(p),
    selected: selectedPostingIds.has(p.id),
    reimbursable: p.is_reimbursable,
    reversal: p.linked_posting_id != null,
  }"
  @click="onPostingClick(p)"
>
  <span class="posting-account">{{ p.account }}</span>
  <span class="posting-commodity">{{ p.commodity }}</span>
  <span class="posting-amount">{{ p.amount }}</span>
  <span v-if="p.linked_posting_id" class="badge" :class="tx.kind === 'reimbursement' ? 'badge-reimb' : 'badge-refund'">
    {{ tx.kind === 'reimbursement' ? '报' : '退' }}
  </span>
  <span v-if="p.is_reimbursable && !p.linked_posting_id" class="badge badge-reimb">报</span>
</div>
```

- [ ] **步骤 3：点击逻辑**

```typescript
function onPostingClick(p: Posting) {
  if (!props.selectable) return
  if (props.selectableFilter && !props.selectableFilter(p)) return
  emit('select-posting', p.id)
}
```

- [ ] **步骤 4：新增样式**

在 `<style scoped>` 中：

```css
.selectable { cursor: pointer; }
.selectable-disabled { opacity: 0.4; cursor: not-allowed; }
.selected { background: #bae7ff; border: 1px solid #1890ff; }
.reimbursable { background: #e6f7ff; }
.badge { font-size: 12px; padding: 0 4px; border-radius: 2px; margin-left: 4px; }
.badge-refund { color: #fa541c; border: 1px solid #fa541c; }
.badge-reimb { color: #1890ff; border: 1px solid #1890ff; }
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-web/src/components/TransactionDetail.vue
git commit -m "feat: add selectable mode and reimbursable highlighting to TransactionDetail"
```

---

### 任务 12：TransactionForm — 分录行改编 + 退款/报销专用模式

**文件：**

- 修改：`accounting-web/src/views/TransactionForm.vue`

---

- [ ] **步骤 1：删除分录级 kind 选择器**

删除每个分录行中的 kind 下拉框（`a-select` 绑定 `posting.kind`）。

- [ ] **步骤 2：新增「报销」按钮**

在分录行右侧（仅当选中账户的 `account_type === 'Expense'` 时显示）：

```vue
<a-button
  v-if="getAccountType(posting.account) === 'Expense'"
  :type="posting.is_reimbursable ? 'primary' : 'default'"
  size="small"
  @click="posting.is_reimbursable = !posting.is_reimbursable"
>
  报销
</a-button>
```

需要从 `useAccountStore` 获取账户列表，通过账户名查找 `account_type`。

- [ ] **步骤 3：检测专用模式**

在 `onMounted` / `setup` 中：

```typescript
const route = useRoute()
const isRefundMode = computed(() => route.path === '/transaction/refund')
const isReimbursementMode = computed(() => route.path === '/transaction/reimbursement')
const postingIds = computed(() => {
  const ids = route.query.posting_ids as string
  return ids ? ids.split(',').map(Number) : []
})
const formKind = computed(() => {
  if (isRefundMode.value) return 'refund'
  if (isReimbursementMode.value) return 'reimbursement'
  return 'normal'
})
const formTitle = computed(() => {
  if (isRefundMode.value) return '录入退款'
  if (isReimbursementMode.value) return '录入报销'
  return isEdit.value ? '编辑交易' : '记一笔'
})
```

- [ ] **步骤 4：加载原分录与分组初始化**

在专用模式下，`onMounted` 中调用 `fetchPosting(id)` 获取每个原分录详情，构建分组数据结构：

```typescript
interface PostingGroup {
  original: Posting        // 原分录详情
  reversal: PostingInput   // 自动生成的只读冲减分录
  assets: PostingInput[]   // 手动添加的资产分录
}

const groups = ref<PostingGroup[]>([])

onMounted(async () => {
  if (postingIds.value.length > 0) {
    for (const id of postingIds.value) {
      const original = await store.fetchPosting(id)
      groups.value.push({
        original,
        reversal: {
          account: original.account,
          commodity: original.commodity,
          amount: '0', // 实时计算为 -sum(assets)
          is_reimbursable: false,
          linked_posting_id: original.id,
        },
        assets: [],
      })
    }
  }
})
```

- [ ] **步骤 5：分组渲染模板**

```vue
<div v-for="(group, gi) in groups" :key="gi" class="posting-group">
  <div class="group-header">
    <span>{{ group.original.account }}</span>
    <span>{{ group.original.amount }}</span>
    <span>已冲减: {{ group.original.reversal_total }}</span>
  </div>
  <div class="reversal-row">
    <span>{{ group.reversal.account }}</span>
    <span>{{ group.reversal.commodity }}</span>
    <span>{{ computeReversalAmount(group) }}</span>
  </div>
  <div v-for="(asset, ai) in group.assets" :key="ai">
    <!-- 可编辑的资产分录行：树选择器（仅 Asset）、货币锁定、金额可编辑 -->
  </div>
  <a-button @click="addAssetRow(group)">+ 添加分录</a-button>
</div>
```

- [ ] **步骤 6：提交逻辑**

```typescript
async function handleSubmit() {
  const allPostings: PostingInput[] = []
  for (const group of groups.value) {
    // 冲减分录
    const reversalAmount = group.assets.reduce((sum, a) => sum + parseFloat(a.amount || '0'), 0)
    allPostings.push({
      account: group.reversal.account,
      commodity: group.reversal.commodity,
      amount: String(-reversalAmount),
      is_reimbursable: false,
      linked_posting_id: group.original.id,
    })
    // 资产分录
    for (const asset of group.assets) {
      allPostings.push({ ...asset })
    }
  }
  await store.createTransaction({
    date_time: dateTime.value.format('YYYY-MM-DD HH:mm:ss'),
    description: description.value,
    kind: formKind.value,
    member_id: memberId.value,
    channel_id: channelId.value,
    postings: allPostings,
    tags: selectedTagNames.value,
  })
  router.push('/')
}
```

在普通模式下，表单提交的 `kind` 为 `'normal'`，postings 中的 `is_reimbursable` 和 `linked_posting_id` 正常赋值。

- [ ] **步骤 7：Commit**

```bash
git add accounting-web/src/views/TransactionForm.vue
git commit -m "feat: remove per-posting kind selector, add reimbursable button, refund/reimbursement dedicated form"
```

---

### 任务 13：Dashboard — 模式切换 + 分录选择 + 底部抽屉

**文件：**

- 修改：`accounting-web/src/views/Dashboard.vue`

---

- [ ] **步骤 1：工具栏 4 模式按钮**

将现有「范围选择」按钮替换为 `a-radio-group`：

```vue
<a-radio-group v-model:value="mode" button-style="solid" size="small">
  <a-radio-button value="normal">普通</a-radio-button>
  <a-radio-button value="range">范围</a-radio-button>
  <a-radio-button value="refund">退款</a-radio-button>
  <a-radio-button value="reimbursement">报销</a-radio-button>
</a-radio-group>
```

- [ ] **步骤 2：状态管理**

```typescript
type DashboardMode = 'normal' | 'range' | 'refund' | 'reimbursement'
const mode = ref<DashboardMode>('normal')
const selectedPostingIds = ref<Set<number>>(new Set())
const drawerExpanded = ref(false)
```

- [ ] **步骤 3：Calendar 绑定调整**

```vue
<Calendar
  :data="calendarData"
  :mode="mode"
  @select="mode === 'normal' || mode === 'refund' || mode === 'reimbursement' ? handleSelect($event) : undefined"
  @select-range="mode === 'range' ? handleSelectRange($event) : undefined"
  @clear="handleClear"
/>
```

- [ ] **步骤 4：交易列表 selectable**

```vue
<TransactionDetail
  v-for="tx in group.transactions"
  :key="tx.id"
  :tx="tx"
  :selectable="mode === 'refund' || mode === 'reimbursement'"
  :selectable-filter="mode === 'reimbursement' ? (p: Posting) => p.is_reimbursable : undefined"
  :selected-posting-ids="selectedPostingIds"
  @deleted="handleDeleted"
  @select-posting="togglePostingSelection"
/>
```

- [ ] **步骤 5：底部抽屉组件**

```vue
<div v-if="isSelectMode && selectedPostingIds.size > 0" class="bottom-drawer" :class="{ collapsed: !drawerExpanded }">
  <div class="drawer-left">
    <span class="count-badge">{{ selectedPostingIds.size }}</span>
    <span class="toggle-icon" @click="drawerExpanded = !drawerExpanded">
      {{ drawerExpanded ? '▶' : '◀' }}
    </span>
  </div>
  <div v-if="drawerExpanded" class="drawer-body">
    <div class="drawer-actions">
      <a-button type="primary" style="background: #52c41a; border-color: #52c41a" @click="confirmSelection">
        确定
      </a-button>
      <a-button @click="cancelSelection">取消</a-button>
    </div>
    <div class="drawer-cards" @wheel="onCardScroll" @mousedown="onCardDragStart">
      <div v-for="p in selectedPostingsInfo" :key="p.id" class="selected-card">
        <div>{{ p.account }}</div>
        <div>{{ p.date }}</div>
        <div>{{ p.description }}</div>
        <div>{{ p.amount }}</div>
      </div>
    </div>
  </div>
</div>
```

- [ ] **步骤 6：辅助逻辑**

```typescript
const selectedPostingsInfo = computed(() => {
  const result: any[] = []
  for (const tx of allTransactions) {
    for (const p of tx.postings) {
      if (selectedPostingIds.value.has(p.id)) {
        result.push({ ...p, date: tx.date_time.slice(0, 10), description: tx.description })
      }
    }
  }
  return result
})

function togglePostingSelection(id: number) {
  const next = new Set(selectedPostingIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedPostingIds.value = next
}

function confirmSelection() {
  const ids = Array.from(selectedPostingIds.value).join(',')
  const path = mode.value === 'refund' ? '/transaction/refund' : '/transaction/reimbursement'
  router.push(`${path}?posting_ids=${ids}`)
}

function cancelSelection() {
  selectedPostingIds.value = new Set()
  mode.value = 'normal'
}
```

- [ ] **步骤 7：报销模式数据筛选**

```typescript
watch(mode, (newMode) => {
  if (newMode === 'reimbursement') {
    store.fetchTransactions({ reimbursable: true, ...currentFilter.value })
  } else {
    store.fetchTransactions(currentFilter.value)
  }
})
```

- [ ] **步骤 8：Commit**

```bash
git add accounting-web/src/views/Dashboard.vue
git commit -m "feat: Dashboard mode switching, posting selection, bottom drawer"
```

---

### 任务 14：暗色主题样式

**文件：**

- 修改：`accounting-web/src/App.vue`

---

- [ ] **步骤 1：添加暗色主题覆盖**

在 `App.vue` 的 `<style>` 中（或单独样式块），添加：

```css
html.dark .selectable.selected {
  background: #15395b;
  border-color: #177ddc;
}
html.dark .reimbursable {
  background: #111d2c;
}
html.dark .bottom-drawer {
  background: #1f1f1f;
  border-color: #434343;
}
html.dark .selected-card {
  background: #262626;
  border-color: #434343;
}
html.dark .group-header {
  background: #262626;
  color: #d9d9d9;
}
```

- [ ] **步骤 2：Commit**

```bash
git add accounting-web/src/App.vue
git commit -m "style: dark theme styles for drawer, selectable postings, reimbursable highlight"
```

---

## 验证命令

后端全量测试：

```bash
cd /home/mechdancer/repos/accounting && cargo test 2>&1 | tail -30
```

前端编译检查：

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit 2>&1 | tail -20
```
