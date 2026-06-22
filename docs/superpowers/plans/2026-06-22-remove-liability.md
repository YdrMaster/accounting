# 移除负债账户类型实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 从整个仓库中彻底移除 `AccountType::Liability` 及其所有相关引用，使系统只保留 `Asset`、`Equity`、`Income`、`Expense` 四种账户类型。

**架构：** 按代码分层逐步清理：先改 `accounting` domain 层（枚举、验证），再改 `accounting-sql` 数据库层（schema、映射），然后改 `accounting-service` 报表与统计，再改 API、CLI、前端与文档。每次提交前执行完整验证（`cargo fmt` → `cargo test` → `cargo clippy` → `npm run build`）。

**技术栈：** Rust（workspace：accounting / accounting-sql / accounting-service / accounting-api / accounting-cli），SQLite，Vue 3 + Vite + TypeScript。

---

## 文件结构与变更清单

| 文件 | 职责 | 变更 |
|------|------|------|
| `accounting/src/account_type.rs` | 账户类型枚举 | 删除 `Liability`，重排编号，删除 `is_permanent`，更新 `close_conditions` / `display_name` / `from_prefix` |
| `accounting/src/validation.rs` | 关闭验证 | 仅 `Asset` 需余额为零，删除 `Liability` match arm |
| `accounting/locales/zh-CN.yaml` | 中文翻译 | 删除 `account_type_liability` |
| `accounting/locales/en.yaml` | 英文翻译 | 删除 `account_type_liability` |
| `accounting-sql/src/schema.rs` | 数据库 schema + 种子数据 | `CHECK` 改为 `1..=4`，种子数据编号同步 |
| `accounting-sql/src/repo/account.rs` | 账户类型映射 | `map_account` 删除 `Liability`，重排编号 |
| `accounting-sql/src/repo/posting.rs` | 分录仓库测试辅助 | 测试辅助函数中 `Income/Expense` 编号改为 `3/4` |
| `accounting-service/src/report_service.rs` | 报表服务 | `BalanceSheet` 删除 `liabilities`，统计分组编号改为 `3/4` |
| `accounting-api/src/handlers/report.rs` | 报表 API | `BalanceSheetResponse` 删除 `liabilities` |
| `accounting-cli/src/cmd/mod.rs` | CLI 账户类型参数 | `AccountTypeArg` 删除 `Liability` |
| `accounting-cli/src/cmd/report.rs` | CLI 资产负债表输出 | 删除 `bs.liabilities` 输出块 |
| `accounting-cli/README.md` | CLI 文档 | 删除 `Liability` / `负债` 引用 |
| `accounting-web/README.md` | 前端文档 | 删除 `Liabilities` 引用 |
| `spec/core.md` | 核心规格 | 删除 `Liability`，关闭规则改为仅 `Asset` |
| `spec/service.md` | 服务规格 | 资产负债表描述改为 `Asset / Equity`，关闭规则同步 |
| `spec/refund-reimbursement-design.md` | 退款/报销规格 | 相关描述同步 |
| `README.md` / `plan/*.md` / `docs/superpowers/**` | 历史/参考文档 | 标注已过时，不修改正文 |

---

## 任务 1：Domain 层 — AccountType 枚举与关闭验证

**文件：**
- 修改：`accounting/src/account_type.rs`
- 修改：`accounting/src/validation.rs`

---

- [ ] **步骤 1：替换 `account_type.rs` 完整内容**

```rust
/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// 资产类账户
    Asset = 1,
    /// 权益类账户
    Equity = 2,
    /// 收入类账户
    Income = 3,
    /// 支出类账户
    Expense = 4,
}

impl AccountType {
    /// 返回该类型账户的关闭条件说明
    pub fn close_conditions(self) -> String {
        match self {
            AccountType::Asset => rust_i18n::t!("close_condition_balance_zero").to_string(),
            AccountType::Equity | AccountType::Income | AccountType::Expense => {
                rust_i18n::t!("close_condition_unlimited").to_string()
            }
        }
    }

    /// 返回本地化的显示名称
    pub fn display_name(self) -> String {
        let key = match self {
            AccountType::Asset => "account_type_asset",
            AccountType::Equity => "account_type_equity",
            AccountType::Income => "account_type_income",
            AccountType::Expense => "account_type_expense",
        };
        rust_i18n::t!(key).to_string()
    }

    /// 根据账户名前缀解析账户类型（支持中英文、单复数）
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        let lower = prefix.to_lowercase();
        match lower.as_str() {
            // 英文（单复数兼容）
            "asset" | "assets" => Some(Self::Asset),
            "equity" => Some(Self::Equity),
            "income" => Some(Self::Income),
            "expense" | "expenses" => Some(Self::Expense),
            // 中文（与 seed 数据和 display_name 一致）
            "资产" => Some(Self::Asset),
            "权益" => Some(Self::Equity),
            "收入" => Some(Self::Income),
            "支出" => Some(Self::Expense),
            _ => None,
        }
    }
}

/// 分期方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallmentMethod {
    /// 等额本息
    EqualPrincipalAndInterest = 1,
    /// 等额本金
    EqualPrincipal = 2,
    /// 免息分期
    InterestFree = 3,
    /// 自定义分期
    Custom = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_close_conditions() {
        rust_i18n::set_locale("zh-CN");
        assert_eq!(AccountType::Asset.close_conditions(), "余额为零");
        assert_eq!(AccountType::Equity.close_conditions(), "无限制");
        assert_eq!(AccountType::Income.close_conditions(), "无限制");
        assert_eq!(AccountType::Expense.close_conditions(), "无限制");
    }
}
```

- [ ] **步骤 2：更新 `validation.rs` 中的 `validate_account_close` 函数**

将函数体替换为：

```rust
/// 验证账户是否可以关闭
///
/// 仅 Asset 要求余额为零；Equity、Income、Expense 无条件关闭
pub fn validate_account_close(
    account_type: AccountType,
    balances: &[(crate::id::CommodityId, Decimal)],
) -> Result<(), AccountingError> {
    match account_type {
        AccountType::Asset => {
            let non_zero: Vec<_> = balances.iter().filter(|(_, b)| !b.is_zero()).collect();
            if !non_zero.is_empty() {
                return Err(AccountingError::AccountNotEmpty(
                    t!("account_balance_non_zero").to_string(),
                ));
            }
        }
        AccountType::Equity | AccountType::Income | AccountType::Expense => {}
    }
    Ok(())
}
```

- [ ] **步骤 3：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting
cargo test -p accounting
cargo clippy -p accounting --all-targets
```

预期：`cargo test -p accounting` 全部通过；`cargo clippy` 无新增 error。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting/src/account_type.rs accounting/src/validation.rs
git commit -m "feat(domain): 移除 Liability 账户类型，重排枚举编号，仅 Asset 关闭需余额为零"
```

---

## 任务 2：数据库层 — Schema、映射与测试辅助函数

**文件：**
- 修改：`accounting-sql/src/schema.rs`
- 修改：`accounting-sql/src/repo/account.rs`
- 修改：`accounting-sql/src/repo/posting.rs`

---

- [ ] **步骤 1：修改 `schema.rs` 中 `accounts` 表的 CHECK 约束**

将：
```sql
account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 5),
```
替换为：
```sql
account_type INTEGER NOT NULL CHECK(account_type BETWEEN 1 AND 4),
```

- [ ] **步骤 2：修改 `schema.rs` 中的英文种子数据**

将 `SEED_ACCOUNTS_ROOT_EN` 替换为：
```rust
const SEED_ACCOUNTS_ROOT_EN: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('Assets', 1, NULL, 1),
('Equity', 2, NULL, 1),
('Income', 3, NULL, 1),
('Expenses', 4, NULL, 1);
"#;
```

将 `SEED_ACCOUNTS_CHILD_EN` 替换为：
```rust
const SEED_ACCOUNTS_CHILD_EN: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('Equity:OpeningBalances', 2, (SELECT id FROM accounts WHERE full_name = 'Equity'), 1),
('Expenses:Fees', 4, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Expenses:Discounts', 4, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Expenses:InstallmentFees', 4, (SELECT id FROM accounts WHERE full_name = 'Expenses'), 1),
('Assets:Cash', 1, (SELECT id FROM accounts WHERE full_name = 'Assets'), 1),
('Assets:Cashback', 1, (SELECT id FROM accounts WHERE full_name = 'Assets'), 1);
"#;
```

- [ ] **步骤 3：修改 `schema.rs` 中的中文种子数据**

将 `SEED_ACCOUNTS_ROOT_ZH` 替换为：
```rust
const SEED_ACCOUNTS_ROOT_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('资产', 1, NULL, 1),
('权益', 2, NULL, 1),
('收入', 3, NULL, 1),
('支出', 4, NULL, 1);
"#;
```

将 `SEED_ACCOUNTS_CHILD_ZH` 替换为：
```rust
const SEED_ACCOUNTS_CHILD_ZH: &str = r#"
INSERT OR IGNORE INTO accounts (full_name, account_type, parent_id, is_system) VALUES
('权益:期初余额', 2, (SELECT id FROM accounts WHERE full_name = '权益'), 1),
('支出:手续费', 4, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('支出:折扣', 4, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('支出:分期手续费', 4, (SELECT id FROM accounts WHERE full_name = '支出'), 1),
('资产:现金', 1, (SELECT id FROM accounts WHERE full_name = '资产'), 1),
('资产:返现', 1, (SELECT id FROM accounts WHERE full_name = '资产'), 1);
"#;
```

- [ ] **步骤 4：修改 `repo/account.rs` 的 `map_account` 函数**

将：
```rust
    let account_type = match type_int {
        1 => AccountType::Asset,
        2 => AccountType::Liability,
        3 => AccountType::Equity,
        4 => AccountType::Income,
        5 => AccountType::Expense,
        _ => AccountType::Asset,
    };
```
替换为：
```rust
    let account_type = match type_int {
        1 => AccountType::Asset,
        2 => AccountType::Equity,
        3 => AccountType::Income,
        4 => AccountType::Expense,
        _ => AccountType::Asset,
    };
```

- [ ] **步骤 5：修改 `repo/posting.rs` 测试辅助函数中的编号**

将 `insert_income_account` 的注释和 SQL 从 `4` 改为 `3`：
```rust
    /// 插入 Income 类型账户（account_type = 3）
    fn insert_income_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, is_system) VALUES (?1, 3, 0)",
            [name],
        )
        .unwrap();
        AccountId(conn.last_insert_rowid())
    }
```

将 `insert_expense_account` 的注释和 SQL 从 `5` 改为 `4`：
```rust
    /// 插入 Expense 类型账户（account_type = 4）
    fn insert_expense_account(conn: &Connection, name: &str) -> AccountId {
        conn.execute(
            "INSERT INTO accounts (full_name, account_type, is_system) VALUES (?1, 4, 0)",
            [name],
        )
        .unwrap();
        AccountId(conn.last_insert_rowid())
    }
```

- [ ] **步骤 6：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting-sql
cargo test -p accounting-sql
cargo clippy -p accounting-sql --all-targets
```

预期：全部通过。

- [ ] **步骤 7：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-sql/src/schema.rs accounting-sql/src/repo/account.rs accounting-sql/src/repo/posting.rs
git commit -m "feat(sql): 移除 Liability 类型，账户类型编号重排为 1-4"
```

---

## 任务 3：Service 层 — 资产负债表与统计

**文件：**
- 修改：`accounting-service/src/report_service.rs`

---

- [ ] **步骤 1：修改 `BalanceSheet` 结构体**

将：
```rust
pub struct BalanceSheet {
    /// 资产类账户余额
    pub assets: Vec<AccountBalance>,
    /// 负债类账户余额
    pub liabilities: Vec<AccountBalance>,
    /// 权益类账户余额
    pub equity: Vec<AccountBalance>,
}
```
替换为：
```rust
pub struct BalanceSheet {
    /// 资产类账户余额
    pub assets: Vec<AccountBalance>,
    /// 权益类账户余额
    pub equity: Vec<AccountBalance>,
}
```

- [ ] **步骤 2：修改 `balance_sheet` 方法**

将方法体替换为：
```rust
    pub async fn balance_sheet(&self) -> Result<BalanceSheet, AccountingError> {
        let conn = self.db.connection();
        let accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut assets = Vec::new();
        let mut equity = Vec::new();

        for account in accounts {
            let balances = self
                .db
                .posting_repo()
                .sum_by_account(&conn, account.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if balances.iter().all(|(_, b)| b.is_zero()) {
                continue;
            }
            let item = AccountBalance {
                account: account.clone(),
                balances,
            };
            match account.account_type {
                AccountType::Asset => assets.push(item),
                AccountType::Equity => equity.push(item),
                _ => {}
            }
        }

        Ok(BalanceSheet { assets, equity })
    }
```

- [ ] **步骤 3：修改统计方法中的 account_type 编号**

在 `stats_by_tag`、`stats_by_member`、`stats_by_channel` 中，把：
```rust
            match account_type {
                4 => entry.0.push((commodity_id, amount)), // Income
                5 => entry.1.push((commodity_id, amount)), // Expense
                _ => {}
            }
```
替换为：
```rust
            match account_type {
                3 => entry.0.push((commodity_id, amount)), // Income
                4 => entry.1.push((commodity_id, amount)), // Expense
                _ => {}
            }
```

三个方法都要改。

- [ ] **步骤 4：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting-service
cargo test -p accounting-service
cargo clippy -p accounting-service --all-targets
```

预期：全部通过。

- [ ] **步骤 5：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-service/src/report_service.rs
git commit -m "feat(service): 资产负债表移除 liabilities，统计分组适配新编号"
```

---

## 任务 4：API 层 — 资产负债表响应

**文件：**
- 修改：`accounting-api/src/handlers/report.rs`

---

- [ ] **步骤 1：修改 `BalanceSheetResponse` 结构体**

将：
```rust
struct BalanceSheetResponse {
    assets: Vec<AccountBalanceItem>,
    liabilities: Vec<AccountBalanceItem>,
    equity: Vec<AccountBalanceItem>,
}
```
替换为：
```rust
struct BalanceSheetResponse {
    assets: Vec<AccountBalanceItem>,
    equity: Vec<AccountBalanceItem>,
}
```

- [ ] **步骤 2：修改 `balance_sheet` handler**

将返回体：
```rust
    Ok(Json(BalanceSheetResponse {
        assets: sheet.assets.into_iter().map(into_item).collect(),
        liabilities: sheet.liabilities.into_iter().map(into_item).collect(),
        equity: sheet.equity.into_iter().map(into_item).collect(),
    }))
```
替换为：
```rust
    Ok(Json(BalanceSheetResponse {
        assets: sheet.assets.into_iter().map(into_item).collect(),
        equity: sheet.equity.into_iter().map(into_item).collect(),
    }))
```

- [ ] **步骤 3：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting-api
cargo check -p accounting-api
cargo clippy -p accounting-api --all-targets
```

预期：编译通过，clippy 无新增 error。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-api/src/handlers/report.rs
git commit -m "feat(api): 资产负债表响应移除 liabilities 字段"
```

---

## 任务 5：CLI

**文件：**
- 修改：`accounting-cli/src/cmd/mod.rs`
- 修改：`accounting-cli/src/cmd/report.rs`

---

- [ ] **步骤 1：修改 `cmd/mod.rs` 中的 `AccountTypeArg`**

将：
```rust
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum AccountTypeArg {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl From<AccountTypeArg> for accounting::account_type::AccountType {
    fn from(arg: AccountTypeArg) -> Self {
        match arg {
            AccountTypeArg::Asset => accounting::account_type::AccountType::Asset,
            AccountTypeArg::Liability => accounting::account_type::AccountType::Liability,
            AccountTypeArg::Equity => accounting::account_type::AccountType::Equity,
            AccountTypeArg::Income => accounting::account_type::AccountType::Income,
            AccountTypeArg::Expense => accounting::account_type::AccountType::Expense,
        }
    }
}
```
替换为：
```rust
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum AccountTypeArg {
    Asset,
    Equity,
    Income,
    Expense,
}

impl From<AccountTypeArg> for accounting::account_type::AccountType {
    fn from(arg: AccountTypeArg) -> Self {
        match arg {
            AccountTypeArg::Asset => accounting::account_type::AccountType::Asset,
            AccountTypeArg::Equity => accounting::account_type::AccountType::Equity,
            AccountTypeArg::Income => accounting::account_type::AccountType::Income,
            AccountTypeArg::Expense => accounting::account_type::AccountType::Expense,
        }
    }
}
```

- [ ] **步骤 2：修改 `cmd/report.rs` 中的 `ReportCmd::Bs` 输出**

删除 `for item in &bs.liabilities { ... }` 整个代码块。

修改后 `ReportCmd::Bs` 分支只保留 `assets` 和 `equity` 的输出：
```rust
            ReportCmd::Bs => {
                let service = accounting_service::report_service::ReportService::new(db);
                let bs = service.balance_sheet().await?;
                let mut rows = Vec::new();
                for item in &bs.assets {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[资产] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &bs.equity {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[权益] {}", item.account.full_name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line(t!("no_data").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
```

- [ ] **步骤 3：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting-cli
cargo check -p accounting-cli
cargo clippy -p accounting-cli --all-targets
```

预期：编译通过。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-cli/src/cmd/mod.rs accounting-cli/src/cmd/report.rs
git commit -m "feat(cli): 移除 CLI 中的 Liability 参数与资产负债表负债输出"
```

---

## 任务 6：国际化

**文件：**
- 修改：`accounting/locales/zh-CN.yaml`
- 修改：`accounting/locales/en.yaml`

---

- [ ] **步骤 1：删除中文翻译中的 `account_type_liability`**

在 `accounting/locales/zh-CN.yaml` 中删除行：
```yaml
account_type_liability: "负债"
```

- [ ] **步骤 2：删除英文翻译中的 `account_type_liability`**

在 `accounting/locales/en.yaml` 中删除行：
```yaml
account_type_liability: "Liability"
```

- [ ] **步骤 3：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt -p accounting
cargo test -p accounting
cargo clippy -p accounting --all-targets
```

预期：全部通过。

- [ ] **步骤 4：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting/locales/zh-CN.yaml accounting/locales/en.yaml
git commit -m "i18n: 移除 liability 账户类型翻译"
```

---

## 任务 7：前端文档

**文件：**
- 修改：`accounting-web/README.md`

---

- [ ] **步骤 1：删除 `accounting-web/README.md` 中的 `Liabilities` 引用**

找到并删除类似以下内容：
```markdown
- **账户树**：层级展示所有账户（Assets / Liabilities / Equity / Income / Expenses）
```

改为：
```markdown
- **账户树**：层级展示所有账户（Assets / Equity / Income / Expenses）
```

同样删除/修改“资产负债表（BS）：按账户类型汇总资产与负债”等包含负债概念的描述，使其与新的 `assets + equity` 语义一致。

- [ ] **步骤 2：运行前端构建验证**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npm run build
```

预期：构建通过。

- [ ] **步骤 3：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add accounting-web/README.md
git commit -m "docs(web): 移除 README 中的 Liabilities 引用"
```

---

## 任务 8：核心规格与参考文档

**文件：**
- 修改：`spec/core.md`
- 修改：`spec/service.md`
- 修改：`spec/refund-reimbursement-design.md`
- 修改：`README.md`
- 修改：`accounting-cli/README.md`
- 标注：`plan/*.md`、`docs/superpowers/plans/*.md`、`docs/superpowers/specs/2026-06-13-account-cards-design.md` 等历史文档

---

- [ ] **步骤 1：更新 `spec/core.md`**

- 删除 `AccountType` 枚举中的 `Liability = 2`。
- 将关闭规则中所有 `Asset / Liability / Equity` 改为 `Asset`（因为已确认仅 `Asset` 需余额为零）。
- 更新 mermaid 图中 `Asset / Liability / Equity` 分支为 `Asset`。
- 更新错误类型表格中 `Asset/Liability/Equity` 为 `Asset`。

- [ ] **步骤 2：更新 `spec/service.md`**

- 将 `// Asset/Liability/Equity 需余额为 0` 改为 `// Asset 需余额为 0`。
- 将 `balance_sheet` 注释“查询所有 Asset/Liability/Equity 账户的余额”改为“查询所有 Asset/Equity 账户的余额”。
- 将 `matches!(account.account_type, Asset | Liability | Equity)` 改为 `matches!(account.account_type, Asset | Equity)`。

- [ ] **步骤 3：更新 `spec/refund-reimbursement-design.md`**

- 将“资产视角：退款到账按实际时间统计，用于资产负债表/现金流量表”中涉及负债的表述同步为新的资产负债表语义。
- 将 `Asset/Liability/Equity` 改为 `Asset/Equity`。

- [ ] **步骤 4：更新 `README.md` 与 `accounting-cli/README.md`**

- 删除所有 `liability` / `负债` / `Liability` 引用。
- `accounting-cli/README.md` 中“关闭账户（Asset/Liability 需余额为零）”改为“关闭账户（Asset 需余额为零）”。
- 账户类型示例列表从 `asset, liability, equity, income, expense` 改为 `asset, equity, income, expense`。

- [ ] **步骤 5：标注历史文档**

在以下历史/参考文档顶部添加标注（不修改正文）：

```markdown
> 注：本文档包含已废弃的 `Liability` / `负债` 账户类型引用，仅供参考。当前系统仅保留 `Asset`、`Equity`、`Income`、`Expense` 四种类型。
```

涉及的文件包括：
- `plan/phase1.md`
- `plan/phase2.md`
- `plan/phase3.md`
- `plan/cli-design.md`
- `docs/superpowers/plans/2026-06-13-account-cards.md`
- `docs/superpowers/plans/2026-06-12-refund-reimbursement-ui-redesign.md`
- `docs/superpowers/specs/2026-06-13-account-cards-design.md`
- 其他 grep 命中 `Liability` / `负债` 的 `docs/superpowers/**` 历史文档

- [ ] **步骤 6：运行完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt --all
cargo test
cargo clippy --all-targets
cd accounting-web && npm run build
```

预期：全部通过。

- [ ] **步骤 7：Commit**

```bash
cd /home/mechdancer/repos/accounting
git add spec/ README.md accounting-cli/README.md docs/superpowers/plans/ docs/superpowers/specs/2026-06-13-account-cards-design.md
git commit -m "docs: 同步移除 Liability 的规格与历史文档标注"
```

---

## 任务 9：最终端到端验证

**文件：** 无新增，仅运行验证。

---

- [ ] **步骤 1：运行 Rust 完整验证**

```bash
cd /home/mechdancer/repos/accounting
cargo fmt --all
cargo test
cargo clippy --all-targets
```

预期：`cargo test` 全部通过；`cargo clippy` 无新增 error。

- [ ] **步骤 2：运行前端构建验证**

```bash
cd /home/mechdancer/repos/accounting/accounting-web
npm run build
```

预期：构建通过。

- [ ] **步骤 3：最终 Commit（如果前面按任务分别提交了，此步可省略或只做最终整理）**

如果用户希望最后统一提交，可执行：

```bash
cd /home/mechdancer/repos/accounting
git add -A
git commit -m "feat: 移除 Liability 账户类型及其所有引用

- 删除 AccountType::Liability 枚举值，编号重排为 Asset=1/Equity=2/Income=3/Expense=4
- 数据库 schema、种子数据、仓库映射同步更新
- 资产负债表移除 liabilities，仅保留 assets/equity
- CLI 移除 Liability 参数与负债输出
- 仅 Asset 关闭时要求余额为零
- 删除 i18n 中 liability 翻译
- 同步更新规格文档并标注历史文档"
```

---

## 自检

**规格覆盖度：**

- ✅ Domain 枚举删除与重排 — 任务 1
- ✅ 关闭验证规则 — 任务 1
- ✅ 数据库 schema / 种子 / 映射 — 任务 2
- ✅ 报表服务移除 liabilities — 任务 3
- ✅ API 响应移除 liabilities — 任务 4
- ✅ CLI 移除 Liability — 任务 5
- ✅ i18n 移除 liability 翻译 — 任务 6
- ✅ 前端文档 — 任务 7
- ✅ 规格与历史文档标注 — 任务 8
- ✅ 每次提交前完整验证 — 各任务步骤 3/6

**占位符扫描：** 无 TODO、无“待定”、无“类似任务 N”、无未定义类型。

**类型一致性：**

- `AccountType` 编号在 domain、schema、repo、`report_service` 统计分组中一致。
- `BalanceSheet` 结构体在 service、API 中一致地只有 `assets` / `equity`。
- `AccountTypeArg` 与 domain `AccountType` 一一对应。
