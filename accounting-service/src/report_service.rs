use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, CommodityId, MemberId, TagId};
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;
use std::str::FromStr;

/// 私有类型别名：按 Income/Expense 分组后的 (商品, 金额) 列表
/// 元组第一项为 Income，第二项为 Expense
type IncomeExpensePairs = (Vec<(CommodityId, Decimal)>, Vec<(CommodityId, Decimal)>);

/// 账户余额项
#[derive(Debug, Clone)]
pub struct AccountBalance {
    /// 账户信息
    pub account: Account,
    /// 各商品余额列表
    pub balances: Vec<(CommodityId, Decimal)>,
}

/// 资产负债表
#[derive(Debug, Clone)]
pub struct BalanceSheet {
    /// 资产类账户余额
    pub assets: Vec<AccountBalance>,
    /// 权益类账户余额
    pub equity: Vec<AccountBalance>,
}

/// 损益表
#[derive(Debug, Clone)]
pub struct IncomeStatement {
    /// 收入类账户余额
    pub income: Vec<AccountBalance>,
    /// 支出类账户余额
    pub expenses: Vec<AccountBalance>,
}

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

/// 收支汇总
#[derive(Debug, Clone)]
pub struct Summary {
    /// 收入（资产类分录正金额之和）
    pub income: Decimal,
    /// 支出（资产类分录负金额之和的绝对值）
    pub expense: Decimal,
}

/// 报告服务
pub struct ReportService {
    db: SqliteDatabase,
}

impl ReportService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 收支汇总：查询指定日期范围内资产类分录的收入和支出
    pub async fn summary(
        &self,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Result<Summary, AccountingError> {
        let result = self
            .db
            .posting_summary(Some(from), Some(to))
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(Summary {
            income: result.income,
            expense: result.expense,
        })
    }

    /// 获取账户余额（含子账户聚合）
    pub async fn get_balance(
        &self,
        account_id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 通过闭包表聚合查询余额
        let totals = tx
            .posting_sum_with_ancestors(account_id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(totals.into_iter().collect())
    }

    /// 资产负债表
    pub async fn balance_sheet(&self) -> Result<BalanceSheet, AccountingError> {
        let accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut assets = Vec::new();
        let mut equity = Vec::new();

        for account in accounts {
            let balances = self
                .db
                .posting_sum_by_account(account.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if balances.iter().all(|(_, b)| b.is_zero()) {
                continue;
            }
            let item = AccountBalance {
                account: account.clone(),
                balances,
            };
            let root_name = self
                .db
                .account_find_root_name(account.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            match AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)? {
                AccountType::Asset => assets.push(item),
                AccountType::Equity => equity.push(item),
                _ => {}
            }
        }

        Ok(BalanceSheet { assets, equity })
    }

    /// 损益表
    pub async fn income_statement(&self) -> Result<IncomeStatement, AccountingError> {
        let accounts = self
            .db
            .account_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut income = Vec::new();
        let mut expenses = Vec::new();

        for account in accounts {
            let balances = self
                .db
                .posting_sum_by_account(account.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            if balances.iter().all(|(_, b)| b.is_zero()) {
                continue;
            }
            let item = AccountBalance {
                account: account.clone(),
                balances,
            };
            let root_name = self
                .db
                .account_find_root_name(account.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            match AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)? {
                AccountType::Income => income.push(item),
                AccountType::Expense => expenses.push(item),
                _ => {}
            }
        }

        Ok(IncomeStatement { income, expenses })
    }

    /// 按标签统计收入与支出
    pub async fn stats_by_tag(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<TagStat>, AccountingError> {
        let mut filter = filter.clone();
        filter.tag_ids.clear(); // 忽略维度自身过滤

        let raw = self
            .db
            .posting_sum_by_tag(&filter)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 TagId 分组，区分 Income 和 Expense
        let mut groups: HashMap<TagId, IncomeExpensePairs> = HashMap::new();
        for (tag_id, commodity_id, root_name, amount) in raw {
            let entry = groups.entry(tag_id).or_default();
            match AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)? {
                AccountType::Income => entry.0.push((commodity_id, amount)),
                AccountType::Expense => entry.1.push((commodity_id, amount)),
                _ => {}
            }
        }

        let tags: HashMap<TagId, accounting::tag::Tag> = self
            .db
            .tag_list()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        let mut result = Vec::new();
        for (tag_id, (income, expense)) in groups {
            let tag = tags.get(&tag_id).cloned().ok_or_else(|| {
                AccountingError::DatabaseError(t!("tag_not_found_id", id = tag_id.0).to_string())
            })?;
            result.push(TagStat {
                tag,
                income,
                expense,
            });
        }

        Ok(result)
    }

    /// 按成员统计收入与支出
    pub async fn stats_by_member(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<MemberStat>, AccountingError> {
        let mut filter = filter.clone();
        filter.member_ids.clear(); // 忽略维度自身过滤

        let raw = self
            .db
            .posting_sum_by_member(&filter)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 MemberId 分组，区分 Income 和 Expense
        let mut groups: HashMap<MemberId, IncomeExpensePairs> = HashMap::new();
        for (member_id, commodity_id, root_name, amount) in raw {
            let entry = groups.entry(member_id).or_default();
            match AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)? {
                AccountType::Income => entry.0.push((commodity_id, amount)),
                AccountType::Expense => entry.1.push((commodity_id, amount)),
                _ => {}
            }
        }

        let mut result = Vec::new();
        for (member_id, (income, expense)) in groups {
            let member = self
                .db
                .member_get(member_id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::DatabaseError(
                        t!("member_not_found_id", id = member_id.0).to_string(),
                    )
                })?;
            result.push(MemberStat {
                member,
                income,
                expense,
            });
        }

        Ok(result)
    }

    /// 按渠道统计收入与支出
    pub async fn stats_by_channel(
        &self,
        filter: &TransactionFilter,
    ) -> Result<Vec<ChannelStat>, AccountingError> {
        let mut filter = filter.clone();
        filter.channel_ids.clear(); // 忽略维度自身过滤

        let raw = self
            .db
            .posting_sum_by_channel(&filter)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 ChannelId 分组，区分 Income 和 Expense
        let mut groups: HashMap<ChannelId, IncomeExpensePairs> = HashMap::new();
        for (channel_id, commodity_id, root_name, amount) in raw {
            let entry = groups.entry(channel_id).or_default();
            match AccountType::from_str(&root_name).map_err(AccountingError::DatabaseError)? {
                AccountType::Income => entry.0.push((commodity_id, amount)),
                AccountType::Expense => entry.1.push((commodity_id, amount)),
                _ => {}
            }
        }

        let mut result = Vec::new();
        for (channel_id, (income, expense)) in groups {
            let channel = self
                .db
                .channel_get(channel_id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::DatabaseError(
                        t!("channel_not_found_id", id = channel_id.0).to_string(),
                    )
                })?;
            result.push(ChannelStat {
                channel,
                income,
                expense,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::channel::Channel;
    use accounting::id::{AccountId, CommodityId, MemberId, PostingId, TagId, TransactionId};
    use accounting::member::Member;
    use accounting::posting::Posting;
    use accounting::tag::Tag;
    use accounting::transaction::Transaction;
    use accounting::transaction::TransactionKind;
    use accounting_sql::SqliteDatabase;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_account(name: &str, parent_id: Option<AccountId>) -> Account {
        Account {
            id: AccountId(0),
            name: name.to_string(),
            parent_id,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        }
    }

    fn sample_posting(account_id: AccountId, amount: &str) -> Posting {
        Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id,
            commodity_id: CommodityId(1), // CNY seed
            amount: Decimal::from_str(amount).unwrap(),
            cost: None,
            cost_commodity_id: None,
            description: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_get_balance() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        let a1 = sample_account("I", None);
        let a2 = sample_account("J", None);
        let id1 = report_service.db.account_create(&a1).await.unwrap();
        let id2 = report_service.db.account_create(&a2).await.unwrap();

        // 直接通过 repo 插入交易和分录
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Report test".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = report_service
            .db
            .transaction_insert(&tx, &[])
            .await
            .unwrap();

        let mut p1 = sample_posting(id1, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(id2, "-100");
        p2.transaction_id = tx_id;

        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        let balance = report_service.get_balance(id1).await.unwrap();
        assert_eq!(balance[&CommodityId(1)], Decimal::from_str("100").unwrap());
    }

    #[tokio::test]
    async fn test_balance_sheet_excludes_zero_balance_and_income_expense() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        let assets_id = report_service
            .db
            .account_get_by_name("Assets")
            .await
            .unwrap()
            .expect("seeded Assets account should exist")
            .id;
        let income_id = report_service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .expect("seeded Income account should exist")
            .id;
        let expenses_id = report_service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .expect("seeded Expenses account should exist")
            .id;

        let bank = sample_account("Bank", Some(assets_id));
        let salary = sample_account("Salary", Some(income_id));
        let food = sample_account("Food", Some(expenses_id));

        let bank_id = report_service
            .db
            .account_create_with_closure(&bank)
            .await
            .unwrap();
        let salary_id = report_service
            .db
            .account_create_with_closure(&salary)
            .await
            .unwrap();
        let _food_id = report_service
            .db
            .account_create_with_closure(&food)
            .await
            .unwrap();

        // Equity:OpeningBalances is a seeded system account, reuse it instead of recreating
        let opening = report_service
            .db
            .account_get_by_name("Equity:OpeningBalances")
            .await
            .unwrap()
            .expect("seeded Equity:OpeningBalances account should exist");
        let opening_id = opening.id;

        // Tx1: Bank +100, OpeningBalances -100
        let tx1 = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Initial balance".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx1_id = report_service
            .db
            .transaction_insert(&tx1, &[])
            .await
            .unwrap();

        let mut p1 = sample_posting(bank_id, "100");
        p1.transaction_id = tx1_id;
        let mut p2 = sample_posting(opening_id, "-100");
        p2.transaction_id = tx1_id;
        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        // Tx2: Salary +200, Bank -200
        let tx2 = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 2)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Salary".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx2_id = report_service
            .db
            .transaction_insert(&tx2, &[])
            .await
            .unwrap();

        let mut p3 = sample_posting(salary_id, "200");
        p3.transaction_id = tx2_id;
        let mut p4 = sample_posting(bank_id, "-200");
        p4.transaction_id = tx2_id;
        report_service.db.posting_insert(&p3).await.unwrap();
        report_service.db.posting_insert(&p4).await.unwrap();

        let sheet = report_service.balance_sheet().await.unwrap();

        // Compile-time assertion that BalanceSheet has only assets and equity fields
        let BalanceSheet { assets, equity } = sheet;

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].account.name, "Bank");
        assert_eq!(assets[0].balances.len(), 1);
        assert_eq!(assets[0].balances[0].0, CommodityId(1));
        assert_eq!(assets[0].balances[0].1, Decimal::from_str("-100").unwrap());

        assert_eq!(equity.len(), 1);
        assert_eq!(equity[0].account.name, "OpeningBalances");
        assert_eq!(equity[0].balances.len(), 1);
        assert_eq!(equity[0].balances[0].0, CommodityId(1));
        assert_eq!(equity[0].balances[0].1, Decimal::from_str("-100").unwrap());
    }

    #[tokio::test]
    async fn test_income_statement_includes_income_and_expense() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        let income_id = report_service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .expect("seeded Income account should exist")
            .id;
        let expenses_id = report_service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .expect("seeded Expenses account should exist")
            .id;

        let salary = sample_account("Salary", Some(income_id));
        let food = sample_account("Food", Some(expenses_id));

        let salary_id = report_service
            .db
            .account_create_with_closure(&salary)
            .await
            .unwrap();
        let food_id = report_service
            .db
            .account_create_with_closure(&food)
            .await
            .unwrap();

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Income statement test".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = report_service
            .db
            .transaction_insert(&tx, &[])
            .await
            .unwrap();

        let mut p1 = sample_posting(salary_id, "500");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(food_id, "-500");
        p2.transaction_id = tx_id;
        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        let statement = report_service.income_statement().await.unwrap();

        assert_eq!(statement.income.len(), 1);
        assert_eq!(statement.income[0].account.name, "Salary");
        assert_eq!(statement.income[0].balances.len(), 1);
        assert_eq!(statement.income[0].balances[0].0, CommodityId(1));
        assert_eq!(
            statement.income[0].balances[0].1,
            Decimal::from_str("500").unwrap()
        );

        assert_eq!(statement.expenses.len(), 1);
        assert_eq!(statement.expenses[0].account.name, "Food");
        assert_eq!(statement.expenses[0].balances.len(), 1);
        assert_eq!(statement.expenses[0].balances[0].0, CommodityId(1));
        assert_eq!(
            statement.expenses[0].balances[0].1,
            Decimal::from_str("-500").unwrap()
        );
    }

    #[tokio::test]
    async fn test_stats_by_tag() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        // 创建 Income / Expense 子账户
        let income_root_id = report_service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .expect("seeded Income account should exist")
            .id;
        let expenses_root_id = report_service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .expect("seeded Expenses account should exist")
            .id;
        let income_acc = sample_account("Salary", Some(income_root_id));
        let expense_acc = sample_account("Food", Some(expenses_root_id));
        let income_id = report_service
            .db
            .account_create_with_closure(&income_acc)
            .await
            .unwrap();
        let expense_id = report_service
            .db
            .account_create_with_closure(&expense_acc)
            .await
            .unwrap();

        // 创建标签
        let tag = Tag {
            id: TagId(0),
            name: "餐饮".to_string(),
            description: None,
            is_system: false,
        };
        let tag_id = report_service.db.tag_create(&tag).await.unwrap();

        // 创建交易并关联标签
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Tag stat test".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = report_service
            .db
            .transaction_insert(&tx, &[tag_id])
            .await
            .unwrap();

        // 插入分录
        let mut p1 = sample_posting(income_id, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(expense_id, "-100");
        p2.transaction_id = tx_id;
        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        // 验证统计
        let stats = report_service
            .stats_by_tag(&TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].tag.name, "餐饮");
        assert_eq!(stats[0].income.len(), 1);
        assert_eq!(stats[0].income[0].0, CommodityId(1));
        assert_eq!(stats[0].income[0].1, Decimal::from_str("100").unwrap());
        assert_eq!(stats[0].expense.len(), 1);
        assert_eq!(stats[0].expense[0].0, CommodityId(1));
        assert_eq!(stats[0].expense[0].1, Decimal::from_str("-100").unwrap());
    }

    #[tokio::test]
    async fn test_stats_by_member() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        // 创建 Income / Expense 子账户
        let income_root_id = report_service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .expect("seeded Income account should exist")
            .id;
        let expenses_root_id = report_service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .expect("seeded Expenses account should exist")
            .id;
        let income_acc = sample_account("Salary", Some(income_root_id));
        let expense_acc = sample_account("Food", Some(expenses_root_id));
        let income_id = report_service
            .db
            .account_create_with_closure(&income_acc)
            .await
            .unwrap();
        let expense_id = report_service
            .db
            .account_create_with_closure(&expense_acc)
            .await
            .unwrap();

        // 创建成员
        let member = Member {
            id: MemberId(0),
            name: "Alice".to_string(),
        };
        let member_id = report_service.db.member_create(&member).await.unwrap();

        // 创建交易并设置成员
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Member stat test".to_string(),
            kind: TransactionKind::Normal,
            member_id: Some(member_id),
        };
        let tx_id = report_service
            .db
            .transaction_insert(&tx, &[])
            .await
            .unwrap();

        // 插入分录
        let mut p1 = sample_posting(income_id, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(expense_id, "-100");
        p2.transaction_id = tx_id;
        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        // 验证统计
        let stats = report_service
            .stats_by_member(&TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].member.name, "Alice");
        assert_eq!(stats[0].income.len(), 1);
        assert_eq!(stats[0].income[0].0, CommodityId(1));
        assert_eq!(stats[0].income[0].1, Decimal::from_str("100").unwrap());
        assert_eq!(stats[0].expense.len(), 1);
        assert_eq!(stats[0].expense[0].0, CommodityId(1));
        assert_eq!(stats[0].expense[0].1, Decimal::from_str("-100").unwrap());
    }

    #[tokio::test]
    async fn test_stats_by_channel() {
        let db = setup_db().await;
        let report_service = ReportService::new(db);

        // 创建 Income / Expense 子账户
        let income_root_id = report_service
            .db
            .account_get_by_name("Income")
            .await
            .unwrap()
            .expect("seeded Income account should exist")
            .id;
        let expenses_root_id = report_service
            .db
            .account_get_by_name("Expenses")
            .await
            .unwrap()
            .expect("seeded Expenses account should exist")
            .id;
        let income_acc = sample_account("Salary", Some(income_root_id));
        let expense_acc = sample_account("Food", Some(expenses_root_id));
        let income_id = report_service
            .db
            .account_create_with_closure(&income_acc)
            .await
            .unwrap();
        let expense_id = report_service
            .db
            .account_create_with_closure(&expense_acc)
            .await
            .unwrap();

        // 创建渠道
        let channel = Channel {
            id: ChannelId(0),
            name: "支付宝".to_string(),
            description: None,
            account_id: None,
            is_system: false,
        };
        let channel_id = report_service.db.channel_create(&channel).await.unwrap();

        // 创建交易
        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Channel stat test".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let tx_id = report_service
            .db
            .transaction_insert(&tx, &[])
            .await
            .unwrap();

        // 添加 channel_path
        let node = accounting::channel_path::ChannelPathNode {
            position: 0,
            channel_id,
            reconciled: false,
        };
        report_service
            .db
            .channel_path_create(tx_id, &node)
            .await
            .unwrap();

        // 插入分录
        let mut p1 = sample_posting(income_id, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(expense_id, "-100");
        p2.transaction_id = tx_id;
        report_service.db.posting_insert(&p1).await.unwrap();
        report_service.db.posting_insert(&p2).await.unwrap();

        // 验证统计
        let stats = report_service
            .stats_by_channel(&TransactionFilter::default())
            .await
            .unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].channel.name, "支付宝");
        assert_eq!(stats[0].income.len(), 1);
        assert_eq!(stats[0].income[0].0, CommodityId(1));
        assert_eq!(stats[0].income[0].1, Decimal::from_str("100").unwrap());
        assert_eq!(stats[0].expense.len(), 1);
        assert_eq!(stats[0].expense[0].0, CommodityId(1));
        assert_eq!(stats[0].expense[0].1, Decimal::from_str("-100").unwrap());
    }
}
