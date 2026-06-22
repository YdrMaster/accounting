> 注：本文档包含已废弃的 `Liability` / `负债` 账户类型引用，仅供参考。当前系统仅保留 `Asset`、`Equity`、`Income`、`Expense` 四种类型。
>
> 相关变更见：`docs/superpowers/specs/2026-06-22-remove-liability-design.md`

# accounting-stat 实现计划

> **面向 AI 代理的工作者：** 使用 `superpowers:subagent-driven-development` 或 `superpowers:executing-plans` 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 `accounting-service` 中扩展 `ReportService`，新增按标签/成员/渠道维度的收入支出统计功能，补全 CLI 报表命令。

**架构：** 不新建 crate（方案 B），直接在 `accounting-service/src/report_service.rs` 上扩展。`accounting-sql` 的 `PostingRepo` 新增 SQL 聚合查询方法，`accounting-cli` 新增 `report stat` 子命令。

**技术栈：** Rust 2024, rusqlite, rust_decimal, chrono, clap, tabled, serde, tokio

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `accounting-sql/src/repo/posting.rs` | `PostingRepo` trait 新增 `sum_by_tag`/`sum_by_member`/`sum_by_channel` 方法及 SQLite 实现 |
| `accounting-service/src/report_service.rs` | 新增 `TagStat`/`MemberStat`/`ChannelStat` 类型及 `stats_by_tag`/`stats_by_member`/`stats_by_channel` 方法 |
| `accounting-cli/src/cmd/report.rs` | `ReportCmd` 新增 `Stat` 子命令，调用 ReportService 新方法 |
| `accounting-cli/src/cmd/mod.rs` | 新增 `StatRow` 表格输出类型 |
| `accounting-cli/src/cmd/tx.rs` | `TxListArgs` 补全 `--template`/`--installment` 参数，`build_filter` 传递新增字段 |

---

## 任务 1：PostingRepo trait 扩展（方法签名 + 文档）

**文件：**
- 修改：`accounting-sql/src/repo/posting.rs`（trait 定义部分，约第 7-44 行）

**目标：** 在 `PostingRepo` trait 中声明三个统计方法，全部带 `///` 文档注释。

- [ ] **步骤 1：添加方法签名**

在 `PostingRepo` trait 的 `sum_with_ancestors` 方法之后添加：

```rust
    /// 按标签汇总分录金额（支持 TransactionFilter 过滤）
    ///
    /// 返回 `(TagId, CommodityId, account_type, Decimal)` 列表，
    /// 其中 `account_type` 为 4(Income) 或 5(Expense)，用于区分收入/支出方向。
    fn sum_by_tag(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(TagId, CommodityId, i64, Decimal)>, crate::error::DbError>;

    /// 按成员汇总分录金额（支持 TransactionFilter 过滤）
    fn sum_by_member(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(MemberId, CommodityId, i64, Decimal)>, crate::error::DbError>;

    /// 按渠道汇总分录金额（支持 TransactionFilter 过滤）
    fn sum_by_channel(
        &self,
        conn: &Connection,
        filter: &accounting::transaction_filter::TransactionFilter,
    ) -> Result<Vec<(ChannelId, CommodityId, i64, Decimal)>, crate::error::DbError>;
```

**注意：** `accounting::transaction_filter::TransactionFilter` 在 `accounting-sql` 中已可通过 `accounting` crate 访问（`accounting-sql` 依赖 `accounting`）。

- [ ] **步骤 2：Commit**

```bash
git add accounting-sql/src/repo/posting.rs
git commit -m "feat(sql): PostingRepo trait 新增按维度统计方法签名

- sum_by_tag/sum_by_member/sum_by_channel
- 接受 TransactionFilter 过滤参数
- 返回 (维度ID, 商品ID, account_type, 金额) 用于区分收入/支出"
```

---

## 任务 2：PostingRepo 统计查询实现 + 单元测试

**文件：**
- 修改：`accounting-sql/src/repo/posting.rs`（`SqlitePostingRepo` 实现部分）
- 新增测试：同文件底部 `#[cfg(test)]` 模块

**目标：** 实现三个统计查询方法，动态构建 WHERE 子句，编写单元测试验证聚合逻辑。

- [ ] **步骤 1：编写 `sum_by_tag` 实现**

参考 `TransactionRepo::list` 的动态 SQL 构建模式。SQL 结构：

```sql
SELECT tt.tag_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
JOIN transaction_tags tt ON tt.transaction_id = t.id
WHERE ...过滤条件...
GROUP BY tt.tag_id, p.commodity_id, a.account_type
```

过滤条件构建规则（与 `TransactionRepo::list` 一致）：
- `filter.start_date` → `t.date_time >= start.and_hms(0,0,0)`
- `filter.end_date` → `t.date_time <= end.and_hms(23,59,59)`
- `filter.account_id` → `p.account_id = ?`（需 `JOIN postings` 已存在）
- `filter.member_id` → `t.member_id = ?`
- `filter.channel_id` → `p.channel_id = ?`
- `filter.keyword` → `t.description LIKE %keyword%`
- `filter.is_template` → `t.is_template = ?`
- **忽略 `filter.tag_id`**（维度自身不用于过滤自身）
- **忽略 `filter.has_installment`**（当前 schema 无此字段的存储，后续实现）

金额还原：查询结果是 `i64`（数据库整数存储），通过 `get_precision` + `accounting::amount::from_db_amount` 还原为 `Decimal`。

- [ ] **步骤 2：编写 `sum_by_member` 实现**

与 `sum_by_tag` 结构相同，只是把 `tt.tag_id` 替换为 `t.member_id`，`JOIN transaction_tags` 替换为直接按 `t.member_id` 分组（或 `p.member_id`，与 `TransactionFilter` 的过滤语义保持一致）。

SQL：
```sql
SELECT t.member_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
WHERE ...过滤条件...
  AND t.member_id IS NOT NULL
GROUP BY t.member_id, p.commodity_id, a.account_type
```

**注意：** `member_id` 为 NULL 的交易不参与统计（无意义）。

- [ ] **步骤 3：编写 `sum_by_channel` 实现**

与 `sum_by_member` 类似，`t.member_id` 替换为 `p.channel_id`：

```sql
SELECT p.channel_id, p.commodity_id, a.account_type, SUM(p.amount) as total
FROM postings p
JOIN accounts a ON p.account_id = a.id
JOIN transactions t ON p.transaction_id = t.id
WHERE ...过滤条件...
  AND p.channel_id IS NOT NULL
GROUP BY p.channel_id, p.commodity_id, a.account_type
```

- [ ] **步骤 4：编写单元测试**

在 `posting.rs` 的 `#[cfg(test)]` 模块中新增测试：

```rust
#[test]
fn test_sum_by_tag() {
    let (conn, repo) = setup();
    // 插入标签
    conn.execute("INSERT INTO tags (name, is_system) VALUES ('餐饮', 0)", []).unwrap();
    let tag_id = TagId(conn.last_insert_rowid());
    // 插入交易 + 分录 + 标签关联（Income 和 Expense 各一笔）
    // 验证 sum_by_tag 返回正确的 (tag_id, commodity_id, account_type, amount)
}
```

类似地编写 `test_sum_by_member` 和 `test_sum_by_channel`。

- [ ] **步骤 5：运行测试**

```bash
cargo test -p accounting-sql
```
预期：全部通过。

- [ ] **步骤 6：运行 fmt + clippy**

```bash
cargo fmt
cargo clippy -p accounting-sql -- -D warnings
```

- [ ] **步骤 7：Commit**

```bash
git add accounting-sql/src/repo/posting.rs
git commit -m "feat(sql): 实现 PostingRepo 按维度统计查询

- sum_by_tag/sum_by_member/sum_by_channel SQL 实现
- 动态 WHERE 构建，支持 TransactionFilter 全部字段
- 单元测试验证聚合逻辑"
```

---

## 任务 3：ReportService 扩展（数据类型 + 统计方法 + 单元测试）

**文件：**
- 修改：`accounting-service/src/report_service.rs`

**目标：** 新增 `TagStat`/`MemberStat`/`ChannelStat` 类型和三个统计方法，编写单元测试。

- [ ] **步骤 1：新增数据类型**

在 `IncomeStatement` 结构体之后添加：

```rust
/// 标签统计项
#[derive(Debug, Clone)]
pub struct TagStat {
    /// 标签信息
    pub tag: accounting::tag::Tag,
    /// 该标签下 Income 类账户的汇总（收入）
    pub income: Vec<(CommodityId, Decimal)>,
    /// 该标签下 Expense 类账户的汇总（支出）
    pub expense: Vec<(CommodityId, Decimal)>,
}

/// 成员统计项
#[derive(Debug, Clone)]
pub struct MemberStat {
    /// 成员信息
    pub member: accounting::member::Member,
    /// 收入汇总
    pub income: Vec<(CommodityId, Decimal)>,
    /// 支出汇总
    pub expense: Vec<(CommodityId, Decimal)>,
}

/// 渠道统计项
#[derive(Debug, Clone)]
pub struct ChannelStat {
    /// 渠道信息
    pub channel: accounting::channel::Channel,
    /// 收入汇总
    pub income: Vec<(CommodityId, Decimal)>,
    /// 支出汇总
    pub expense: Vec<(CommodityId, Decimal)>,
}
```

- [ ] **步骤 2：实现 `stats_by_tag`**

```rust
pub async fn stats_by_tag(
    &self,
    filter: &TransactionFilter,
) -> Result<Vec<TagStat>, AccountingError> {
    let mut filter = filter.clone();
    filter.tag_id = None; // 忽略维度自身过滤

    let conn = self.db.connection();
    let raw = self
        .db
        .posting_repo()
        .sum_by_tag(&conn, &filter)
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

    // 按 TagId 分组，区分 Income(4) 和 Expense(5)
    let mut groups: HashMap<TagId, (Vec<(CommodityId, Decimal)>, Vec<(CommodityId, Decimal)>)> =
        HashMap::new();
    for (tag_id, commodity_id, account_type, amount) in raw {
        let entry = groups.entry(tag_id).or_default();
        match account_type {
            4 => entry.0.push((commodity_id, amount)), // Income
            5 => entry.1.push((commodity_id, amount)), // Expense
            _ => {}
        }
    }

    // 查询标签信息并构造结果
    let tag_repo = self.db.tag_repo();
    let mut result = Vec::new();
    for (tag_id, (income, expense)) in groups {
        let tag = tag_repo
            .get(&conn, tag_id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AccountingError::DatabaseError(format!("标签 {} 不存在", tag_id.0)))?;
        result.push(TagStat {
            tag,
            income,
            expense,
        });
    }

    Ok(result)
}
```

类似实现 `stats_by_member`（`filter.member_id = None`，查询 `member_repo.get`）和 `stats_by_channel`（`filter.channel_id = None`，查询 `channel_repo.get`）。

- [ ] **步骤 3：编写单元测试**

在 `report_service.rs` 的 `#[cfg(test)]` 模块中新增测试，复用现有的 `sample_account`/`sample_posting` 辅助函数：

```rust
#[tokio::test]
async fn test_stats_by_tag() {
    let db = SqliteDatabase::open_in_memory().unwrap();
    db.initialize("en").unwrap();
    let service = ReportService::new(db);

    // 创建 Income 账户、Expense 账户
    // 创建交易，打上标签
    // 验证 stats_by_tag 返回正确的 TagStat
}
```

- [ ] **步骤 4：运行测试**

```bash
cargo test -p accounting-service
```

- [ ] **步骤 5：运行 fmt + clippy**

```bash
cargo fmt
cargo clippy -p accounting-service -- -D warnings
```

- [ ] **步骤 6：Commit**

```bash
git add accounting-service/src/report_service.rs
git commit -m "feat(service): ReportService 新增按维度统计方法

- TagStat/MemberStat/ChannelStat 数据类型
- stats_by_tag/stats_by_member/stats_by_channel 实现
- TransactionFilter 复用，自动忽略与统计维度冲突的过滤字段
- 单元测试"
```

---

## 任务 4：CLI report 命令扩展（Stat 子命令 + 输出格式化）

**文件：**
- 修改：`accounting-cli/src/cmd/report.rs`
- 修改：`accounting-cli/src/cmd/mod.rs`

**目标：** 新增 `report stat` 子命令，支持 `--by-tag`/`--by-member`/`--by-channel` 及过滤参数。

- [ ] **步骤 1：新增 `StatRow` 输出类型**

在 `accounting-cli/src/cmd/mod.rs` 中新增：

```rust
/// 统计报表表格行
#[derive(Tabled, Serialize)]
pub struct StatRow {
    pub dimension_name: String,
    pub stat_type: String,
    pub commodity_id: i64,
    pub amount: String,
}
```

- [ ] **步骤 2：修改 `ReportCmd` 添加 `Stat` 子命令**

在 `accounting-cli/src/cmd/report.rs` 中：

```rust
#[derive(Subcommand)]
pub enum ReportCmd {
    /// 查询账户余额
    Balance(ReportBalanceArgs),
    /// 资产负债表
    Bs,
    /// 损益表
    Is,
    /// 按维度统计
    Stat(ReportStatArgs),
}

#[derive(Args)]
pub struct ReportStatArgs {
    /// 按标签统计
    #[arg(long, group = "dimension")]
    pub by_tag: bool,
    /// 按成员统计
    #[arg(long, group = "dimension")]
    pub by_member: bool,
    /// 按渠道统计
    #[arg(long, group = "dimension")]
    pub by_channel: bool,
    /// 起始日期
    #[arg(long)]
    pub from: Option<String>,
    /// 结束日期
    #[arg(long)]
    pub to: Option<String>,
    /// 指定账户
    #[arg(long)]
    pub account: Option<i64>,
    /// 指定成员
    #[arg(long)]
    pub member: Option<i64>,
    /// 指定标签
    #[arg(long)]
    pub tag: Option<String>,
    /// 指定渠道
    #[arg(long)]
    pub channel: Option<i64>,
    /// 关键词
    #[arg(long)]
    pub keyword: Option<String>,
}
```

`#[arg(group = "dimension")]` 确保 `--by-tag`、`--by-member`、`--by-channel` 三者必选其一且互斥。

- [ ] **步骤 3：实现 `ReportCmd::Stat` 分支**

在 `ReportCmd::run` 的 `match` 中添加 `ReportCmd::Stat(args)` 分支：

1. 解析日期字符串为 `NaiveDate`
2. 构建 `TransactionFilter`
3. 若 `args.tag` 有值，查询数据库解析为 `TagId`
4. 调用 `ReportService::stats_by_tag` / `stats_by_member` / `stats_by_channel`
5. 将结果展平为 `Vec<StatRow>`，按 `dimension_name` + `stat_type` 排序
6. 调用 `print_vec` 输出

展平逻辑示例：

```rust
let mut rows = Vec::new();
for stat in &stats {
    for (cid, amount) in &stat.income {
        rows.push(StatRow {
            dimension_name: stat.tag.name.clone(),
            stat_type: t!("income").to_string(),
            commodity_id: cid.0,
            amount: amount.to_string(),
        });
    }
    for (cid, amount) in &stat.expense {
        rows.push(StatRow {
            dimension_name: stat.tag.name.clone(),
            stat_type: t!("expense").to_string(),
            commodity_id: cid.0,
            amount: amount.to_string(),
        });
    }
}
```

- [ ] **步骤 4：运行 fmt + clippy + test**

```bash
cargo fmt
cargo clippy -p accounting-cli -- -D warnings
cargo test -p accounting-cli
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-cli/src/cmd/report.rs accounting-cli/src/cmd/mod.rs
git commit -m "feat(cli): report 新增 stat 子命令

- --by-tag/--by-member/--by-channel 互斥参数
- 支持 from/to/account/member/tag/channel/keyword 过滤
- StatRow 表格输出，收入/支出分别展示"
```

---

## 任务 5：tx list 参数补全

**文件：**
- 修改：`accounting-cli/src/cmd/tx.rs`

**目标：** 补全 `TxListArgs` 中缺失的 `--template` 和 `--installment` 参数，传递到 `TransactionFilter`。

- [ ] **步骤 1：检查缺失参数**

`TransactionFilter` 有 `is_template: Option<bool>` 和 `has_installment: Option<bool>`。
`TxListArgs` 当前没有这两个字段。

- [ ] **步骤 2：添加参数**

在 `TxListArgs` 中新增：

```rust
    /// 是否只显示模板交易
    #[arg(long)]
    pub template: bool,
    /// 是否只显示分期交易
    #[arg(long)]
    pub installment: bool,
```

- [ ] **步骤 3：修改 `build_filter` 传递新增字段**

```rust
fn build_filter(
    args: &TxListArgs,
    db: &SqliteDatabase,
) -> Result<TransactionFilter, AccountingError> {
    // ... 现有代码 ...
    filter.is_template = if args.template { Some(true) } else { None };
    filter.has_installment = if args.installment { Some(true) } else { None };
    Ok(filter)
}
```

- [ ] **步骤 4：运行 fmt + clippy + test**

```bash
cargo fmt
cargo clippy -p accounting-cli -- -D warnings
cargo test -p accounting-cli
```

- [ ] **步骤 5：Commit**

```bash
git add accounting-cli/src/cmd/tx.rs
git commit -m "feat(cli): tx list 补全 --template 和 --installment 参数

- TransactionFilter.is_template / has_installment 字段已在 repo 层支持
- CLI 参数补全，build_filter 正确传递"
```

---

## 任务 6：全量验证与最终提交

**目标：** 运行完整测试套件，确保所有 crate 无编译错误、无 clippy warning、全部测试通过。

- [ ] **步骤 1：格式化**

```bash
cargo fmt
```

- [ ] **步骤 2：Clippy 检查**

```bash
cargo clippy --workspace -- -D warnings
```

- [ ] **步骤 3：全量测试**

```bash
cargo test --workspace
```

预期：全部通过。

- [ ] **步骤 4：Commit（如 fmt/clippy 有修改）**

```bash
git add -A
git commit -m "chore: fmt + clippy 修复"
```

---

## 验收标准

1. `cargo test --workspace` 全部通过
2. `cargo clippy --workspace -- -D warnings` 零 warning
3. `cargo fmt` 后无未格式化文件
4. CLI 可执行以下命令：
   - `accounting my.db report stat --by-tag --from 2024-01-01 --to 2024-12-31`
   - `accounting my.db report stat --by-member --account 1`
   - `accounting my.db report stat --by-channel --tag 餐饮`
   - `accounting my.db tx list --template`
   - `accounting my.db tx list --installment`
5. 所有新增函数、类型、字段附带 `///` 文档注释
6. 新增代码符合现有代码风格（错误处理、命名、模式匹配等）

---

## 自检

**规格覆盖度：**
- ✅ 按标签/成员/渠道统计 — 任务 1-3
- ✅ TransactionFilter 复用 — 任务 1-2
- ✅ CLI report stat 命令 — 任务 4
- ✅ tx list 参数补全 — 任务 5
- ✅ 全量验证 — 任务 6

**占位符扫描：**
- ✅ 无 "TODO" / "待定" / "后续实现"
- ✅ 每个步骤包含实际代码或具体命令
- ✅ 类型名称前后一致

**类型一致性：**
- ✅ `TagStat`/`MemberStat`/`ChannelStat` 在任务 3 定义，任务 4 使用
- ✅ `StatRow` 在任务 4 定义并使用
- ✅ `TransactionFilter` 在任务 1-5 一致使用
