# 退款与报销功能设计文档

## 背景

现有记账系统只支持普通交易（支出、收入、转账）。实际消费场景中，退款和报销是常见需求：

- **退款**：商家退回部分/全部款项（如外卖退了一个菜）
- **报销**：第三方赔付部分/全部支出（如医保报销、单位报销）

它们的共性是：都是某笔支出的"反作用"，会减少原支出金额，但增加某资产账户金额。

## 设计目标

1. 消费者视角：只关心"某笔开销实际花了多少"，不关心退款到账哪个资产
2. 资产视角：退款到账按实际时间统计，用于资产负债表/现金流量表
3. 开支视角：退款金额回溯到原支出的时间维度，避免"6 月报销 5 月饭费导致 6 月饭费为负"的怪象
4. 灵活性：一笔退款/报销可以冲减多笔原支出的多个分录

## 数据模型

### postings 表变更

```sql
ALTER TABLE postings ADD COLUMN kind INTEGER NOT NULL DEFAULT 1
    CHECK(kind BETWEEN 1 AND 3);
-- 1=Normal, 2=Refund, 3=Reimbursement

ALTER TABLE postings ADD COLUMN linked_posting_id INTEGER REFERENCES postings(id);

ALTER TABLE postings ADD COLUMN reversal_total INTEGER NOT NULL DEFAULT 0;
-- 累计被冲减金额（由触发器自动维护）
```

### transactions 表

**不变**。交易本身不再区分类型，类型标注在分录级别。

### 领域模型

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingKind {
    Normal = 1,
    Refund = 2,
    Reimbursement = 3,
}

pub struct Posting {
    pub id: PostingId,
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub commodity_id: CommodityId,
    pub amount: Decimal,
    pub cost: Option<Decimal>,
    pub cost_commodity_id: Option<CommodityId>,
    pub description: Option<String>,
    pub member_id: Option<MemberId>,
    pub channel_id: Option<ChannelId>,
    pub kind: PostingKind,                // 新增
    pub linked_posting_id: Option<PostingId>,  // 新增
    pub reversal_total: Decimal,          // 新增
}
```

## 验证规则

提交/更新交易时，对所有 `kind != Normal` 的 posting 强制检查：

1. `linked_posting_id` 必须非空
2. 被指向的 posting 必须存在
3. 被指向的 posting 的 `kind` 必须是 `Normal`
4. 两个 posting 的 `account_id` 必须相同（同一账户）
5. 本 posting 的金额方向必须是冲减：
   - Expense 账户下 `amount < 0`（支出减少）
   - Income 账户下 `amount > 0`（收入减少，虽罕见但支持）

**不检查**增加的资产账户是什么，由用户自由填写。

## 触发器（维护 reversal_total）

```sql
-- 插入冲减分录时累加
CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_insert
AFTER INSERT ON postings
WHEN NEW.kind IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total + NEW.amount
    WHERE id = NEW.linked_posting_id;
END;

-- 删除冲减分录时回退
CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_delete
AFTER DELETE ON postings
WHEN OLD.kind IN (2, 3) AND OLD.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total - OLD.amount
    WHERE id = OLD.linked_posting_id;
END;

-- 更新冲减分录金额时调整
CREATE TRIGGER IF NOT EXISTS trg_postings_reversal_update
AFTER UPDATE OF amount ON postings
WHEN NEW.kind IN (2, 3) AND NEW.linked_posting_id IS NOT NULL
BEGIN
    UPDATE postings
    SET reversal_total = reversal_total - OLD.amount + NEW.amount
    WHERE id = NEW.linked_posting_id;
END;
```

> 注：更新触发器仅监控 `amount` 字段变更。如果 `linked_posting_id` 或 `kind` 本身被修改（业务上不应发生），需通过应用层保证一致性。

## 统计规则

### 按资产账户统计（资产负债表、现金流量表）

**忽略 `kind`**，所有 posting 按自身交易时间和金额正常统计。

退款到账的 Asset posting（Normal kind）在退款发生的时间点增加资产。

### 按开支/收入账户统计（损益表、分类开支）

- `kind = Normal` 的 posting：按自身交易时间统计
- `kind = Refund/Reimbursement` 的 posting：将其金额 **回溯** 到 `linked_posting_id` 所属交易的时间维度

**示例：**

| 交易 | 时间 | 分录 | kind | linked_to | 账户统计 | 标签统计 |
|------|------|------|------|-----------|----------|----------|
| 吃饭 | 5 月 | Expense:正餐 +100 | Normal | - | 5 月正餐 100 | 5 月餐饮 100 |
| 退款 | 6 月 | Expense:正餐 -20 | Refund | 吃饭分录 | **5 月正餐 80** | **5 月餐饮 80** |
| 退款 | 6 月 | Asset:微信零钱 +20 | Normal | - | 6 月微信 +20 | 不计入 |

### 按标签/成员/渠道统计

与按账户统计逻辑一致：Refund/Reimbursement posting 的金额回溯到原 posting 所属交易的时间维度。

## 双向查询

### 原分录 → 冲减分录

```sql
-- 方式 1：直接读物化字段（展示净额）
SELECT amount, reversal_total FROM postings WHERE id = ?;
-- 净额 = amount + reversal_total

-- 方式 2：查所有冲减分录（展示明细）
SELECT * FROM postings WHERE linked_posting_id = ?;
```

### 冲减分录 → 原分录

```sql
SELECT p.*, t.date_time
FROM postings p
JOIN transactions t ON p.transaction_id = t.id
WHERE p.id = ?;  -- ? = linked_posting_id
```

## API 与前端交互

### 创建交易

 posting 数据中可以携带：

- `kind`: "normal" | "refund" | "reimbursement"（默认 normal）
- `linked_posting_id`: 指向被冲减的原 posting ID（仅 kind != normal 时必填）

### 交易详情展示

- Normal posting：如 `reversal_total != 0`，显示"已被冲减 ¥X"
- Refund/Reimbursement posting：显示"退"或"报"标记，点击可跳转原交易
- 原交易详情：列出所有冲减它的分录及所属交易

### 记一笔表单

- 在分录行增加 kind 选择（默认 Normal）
- 当用户选择 Refund/Reimbursement 时，弹出原交易选择器
- 选择原交易后，自动筛选出该交易的 Expense/ Income 分录供用户选择被冲减对象
- 自动填充相同的账户和金额方向（用户可修改金额）

## 索引

```sql
CREATE INDEX IF NOT EXISTS idx_postings_kind ON postings(kind);
CREATE INDEX IF NOT EXISTS idx_postings_linked ON postings(linked_posting_id);
```

## 边界情况

1. **删除原交易**：外键 `ON DELETE CASCADE` 会级联删除其 posting，但冲减分录的 `linked_posting_id` 将指向不存在的记录。应用层应在删除原交易前检查 `reversal_total`，如非零则拒绝删除或要求先删除关联的退款/报销交易。
2. **超额冲减**：系统不限制冲减总额不超过原金额。`reversal_total` 可以超过原 `amount`（例如原支出 100，两笔退款各 60）。应用层可选择性提示，但不强制拒绝。
3. **修改原交易金额**：修改 Normal posting 的金额不会影响已存在的冲减分录。`reversal_total` 保持不变。
4. **多币种冲减**：冲减分录的 `commodity_id` 应与原 posting 一致。应用层验证，数据库层不强制外键约束（已有 commodity_id 字段）。
