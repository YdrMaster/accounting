# accounting-stat 设计文档

> Phase 2 第一子系统：查询统计、报表生成、按维度统计。

## 1. 背景与目标

Phase 1 已实现基础记账功能（账户、交易、成员、标签等），但报表统计能力较弱：

- `report bs` / `report is` 已实现，但停留在 `accounting-service` 中
- 缺少按标签、成员、渠道等维度的收入/支出汇总
- `tx list` 已支持多条件过滤，但 CLI 参数未完全暴露

**目标**：在 `accounting-service` 中扩展 `ReportService`，新增按维度统计功能，补全 CLI 报表命令。

## 2. 方案选择

**选择方案 B：并入 `accounting-service`**。

不新建 `accounting-stat` crate，原因：

- Web UI 和 CLI 等价，都需要读写功能，不存在"只读查询独立复用"的场景
- 少维护一个 crate，降低复杂度
- `Database` trait 已统一读写接口，拆分 crate 的隔离收益有限

## 3. 已有功能迁移

`accounting-service/src/report_service.rs` 中已有：

- `AccountBalance`、`BalanceSheet`、`IncomeStatement` 类型
- `ReportService::get_balance()` — 账户余额（含子账户聚合）
- `ReportService::balance_sheet()` — 资产负债表
- `ReportService::income_statement()` — 损益表

**无需迁移，直接在此文件上扩展**。

## 4. 新增接口设计

### 4.1 `ReportService` 扩展方法

```rust
use accounting::transaction_filter::TransactionFilter;

impl<D: Database> ReportService<D> {
    /// 按标签统计收入/支出（日期范围、账户、成员、渠道、关键词等过滤）
    pub async fn stats_by_tag(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<TagStat>, AccountingError>;

    /// 按成员统计收入/支出
    pub async fn stats_by_member(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<MemberStat>, AccountingError>;

    /// 按渠道统计收入/支出
    pub async fn stats_by_channel(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<ChannelStat>, AccountingError>;
}
```

**过滤语义**：

- `stats_by_tag` 忽略 `filter.tag_id`（维度自身不用于过滤自身）
- `stats_by_member` 忽略 `filter.member_id`
- `stats_by_channel` 忽略 `filter.channel_id`
- 其他字段（`start_date`、`end_date`、`account_id`、`keyword`、`has_installment`、`is_template`）全部生效

### 4.2 新增数据类型

```rust
/// 标签统计项
#[derive(Debug, Clone)]
pub struct TagStat {
    pub tag: Tag,
    /// 该标签下 Income 类账户的汇总（收入）
    pub income: Vec<(CommodityId, Decimal)>,
    /// 该标签下 Expense 类账户的汇总（支出）
    pub expense: Vec<(CommodityId, Decimal)>,
}

/// 成员统计项
#[derive(Debug, Clone)]
pub struct MemberStat {
    pub member: Member,
    pub income: Vec<(CommodityId, Decimal)>,
    pub expense: Vec<(CommodityId, Decimal)>,
}

/// 渠道统计项
#[derive(Debug, Clone)]
pub struct ChannelStat {
    pub channel: Channel,
    pub income: Vec<(CommodityId, Decimal)>,
    pub expense: Vec<(CommodityId, Decimal)>,
}
```

`income` 和 `expense` 分别按 `CommodityId` 分组，同一商品的收入/支出独立展示，避免跨币种混算。

## 5. `accounting-sql` 新增 Repo 方法

在 `PostingRepo` trait 中新增统计查询方法：

```rust
/// 按标签汇总分录（支持日期范围过滤）
fn sum_by_tag(
    &self,
    conn: &Connection,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Vec<(TagId, CommodityId, i64, Decimal)>, DbError>;

/// 按成员汇总
fn sum_by_member(...);

/// 按渠道汇总
fn sum_by_channel(...);
```

**实现思路**：
SQL 通过 `JOIN postings → accounts → transactions → transaction_tags → tags` 关联，按 `(tag_id, commodity_id, account_type)` 分组 `SUM(amount)`。`account_type` 用于区分 Income/Expense 方向。

```sql
SELECT
    tt.tag_id,
    p.commodity_id,
    a.account_type,
    SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
JOIN transaction_tags tt ON tt.transaction_id = t.id
WHERE t.date_time BETWEEN ? AND ?
GROUP BY tt.tag_id, p.commodity_id, a.account_type
```

`ReportService` 拿到原始数据后，按 `account_type` 拆分为 `income`（Income=4）和 `expense`（Expense=5）两个列表。

## 6. CLI 命令映射

### 6.1 新增报表命令

```bash
# 按标签统计
accounting my.db report stat --by-tag [--from YYYY-MM-DD] [--to YYYY-MM-DD]
  [--account <ID>] [--member <ID>] [--channel <ID>] [--keyword <TEXT>]

# 按成员统计
accounting my.db report stat --by-member [--from YYYY-MM-DD] [--to YYYY-MM-DD]
  [--account <ID>] [--tag <ID>] [--channel <ID>] [--keyword <TEXT>]

# 按渠道统计
accounting my.db report stat --by-channel [--from YYYY-MM-DD] [--to YYYY-MM-DD]
  [--account <ID>] [--member <ID>] [--tag <ID>] [--keyword <TEXT>]
```

`report stat` 为互斥子命令或互斥参数（`--by-tag` / `--by-member` / `--by-channel` 三选一）。

### 6.2 `tx list` 增强（无需新增 repo 方法）

`TransactionFilter` 已支持全部过滤字段，CLI 只需暴露已有参数：

```bash
accounting my.db tx list
  [--from YYYY-MM-DD] [--to YYYY-MM-DD]
  [--account <ID>] [--member <ID>] [--tag <ID>]
  [--keyword <TEXT>] [--template] [--installment]
  [--limit <N>] [--offset <N>]
```

当前 CLI 已支持大部分参数，检查是否有缺失字段（如 `--installment`）并补全。

## 7. WAL 模式

`accounting-sql/src/schema.rs` 的 `SCHEMA_SQL` 已添加：

```sql
PRAGMA journal_mode = WAL;
```

开启 WAL 后，多个 CLI 进程可以并发读取数据库（如同时执行报表查询），写入操作不会阻塞读取。

## 8. 错误处理

- `ReportService` 中所有数据库错误统一映射为 `AccountingError::DatabaseError(msg)`
- 统计方法中，若过滤条件导致无交易匹配，返回空列表（不是错误）
- 按维度统计时，若某个维度（如标签）下无任何交易，该维度不出现于结果中（不返回零值占位）

## 9. 测试策略

### 9.1 单元测试（`report_service.rs`）

- 准备内存数据库 + seed 数据
- 插入多笔交易（不同标签、成员、渠道、日期）
- 验证 `stats_by_tag` / `stats_by_member` / `stats_by_channel` 的聚合结果
- 验证 `TransactionFilter` 各字段的组合过滤效果

### 9.2 集成测试（`accounting-cli`）

- 通过 CLI 命令执行报表查询，验证输出格式（table / json）
- 验证 `--by-tag` / `--by-member` / `--by-channel` 参数互斥

## 10. 边界与限制

- **现金流量表推算**：Phase 2 不做，留待后期
- **对账功能**：Phase 2 不做，留待支持多方数据导入后
- **跨币种换算**：统计结果按原币种展示，不做汇率换算（汇率功能尚未实现）
- **性能**：统计查询使用 SQL 聚合，数据量大时依赖 SQLite 索引，若后续遇到性能瓶颈再优化
