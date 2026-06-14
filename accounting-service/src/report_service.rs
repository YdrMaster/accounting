use accounting::account::Account;
use accounting::account_type::AccountType;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, CommodityId, MemberId, TagId};
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::database::Database;
use accounting_sql::transaction::Transaction;
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;

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
    /// 负债类账户余额
    pub liabilities: Vec<AccountBalance>,
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

/// 报告服务
pub struct ReportService<D: Database> {
    db: D,
}

impl<D: Database> ReportService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 获取账户余额（含子账户聚合）
    pub async fn get_balance(
        &self,
        account_id: AccountId,
    ) -> Result<HashMap<CommodityId, Decimal>, AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 通过闭包表聚合查询余额
        let totals = tx
            .posting_repo()
            .sum_with_ancestors(&tx.conn(), account_id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(totals.into_iter().collect())
    }

    /// 资产负债表
    pub async fn balance_sheet(&self) -> Result<BalanceSheet, AccountingError> {
        let conn = self.db.connection();
        let accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut assets = Vec::new();
        let mut liabilities = Vec::new();
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
                AccountType::Liability => liabilities.push(item),
                AccountType::Equity => equity.push(item),
                _ => {}
            }
        }

        Ok(BalanceSheet {
            assets,
            liabilities,
            equity,
        })
    }

    /// 损益表
    pub async fn income_statement(&self) -> Result<IncomeStatement, AccountingError> {
        let conn = self.db.connection();
        let accounts = self
            .db
            .account_repo()
            .list(&conn)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let mut income = Vec::new();
        let mut expenses = Vec::new();

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
        filter.tag_id = None; // 忽略维度自身过滤

        let conn = self.db.connection();
        let raw = self
            .db
            .posting_repo()
            .sum_by_tag(&conn, &filter)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 TagId 分组，区分 Income(4) 和 Expense(5)
        let mut groups: HashMap<TagId, IncomeExpensePairs> = HashMap::new();
        for (tag_id, commodity_id, account_type, amount) in raw {
            let entry = groups.entry(tag_id).or_default();
            match account_type {
                4 => entry.0.push((commodity_id, amount)), // Income
                5 => entry.1.push((commodity_id, amount)), // Expense
                _ => {}
            }
        }

        let tags: HashMap<TagId, accounting::tag::Tag> = self
            .db
            .tag_repo()
            .list(&conn)
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
        filter.member_id = None; // 忽略维度自身过滤

        let conn = self.db.connection();
        let raw = self
            .db
            .posting_repo()
            .sum_by_member(&conn, &filter)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 MemberId 分组，区分 Income(4) 和 Expense(5)
        let mut groups: HashMap<MemberId, IncomeExpensePairs> = HashMap::new();
        for (member_id, commodity_id, account_type, amount) in raw {
            let entry = groups.entry(member_id).or_default();
            match account_type {
                4 => entry.0.push((commodity_id, amount)), // Income
                5 => entry.1.push((commodity_id, amount)), // Expense
                _ => {}
            }
        }

        let mut result = Vec::new();
        for (member_id, (income, expense)) in groups {
            let member = self
                .db
                .member_repo()
                .get(&conn, member_id)
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
        filter.channel_id = None; // 忽略维度自身过滤

        let conn = self.db.connection();
        let raw = self
            .db
            .posting_repo()
            .sum_by_channel(&conn, &filter)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 按 ChannelId 分组，区分 Income(4) 和 Expense(5)
        let mut groups: HashMap<ChannelId, IncomeExpensePairs> = HashMap::new();
        for (channel_id, commodity_id, account_type, amount) in raw {
            let entry = groups.entry(channel_id).or_default();
            match account_type {
                4 => entry.0.push((commodity_id, amount)), // Income
                5 => entry.1.push((commodity_id, amount)), // Expense
                _ => {}
            }
        }

        let mut result = Vec::new();
        for (channel_id, (income, expense)) in groups {
            let channel = self
                .db
                .channel_repo()
                .get(&conn, channel_id)
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
    use accounting::account_type::AccountType;
    use accounting::channel::Channel;
    use accounting::id::{AccountId, CommodityId, MemberId, PostingId, TagId, TransactionId};
    use accounting::member::Member;
    use accounting::posting::Posting;
    use accounting::tag::Tag;
    use accounting::transaction::Transaction;
    use accounting::transaction::TransactionKind;
    use accounting_sql::impls::sqlite::SqliteDatabase;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_account(name: &str, account_type: AccountType) -> Account {
        Account {
            id: AccountId(0),
            full_name: name.to_string(),
            account_type,
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
            position: 0,
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
            member_id: None,
            channel_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    #[tokio::test]
    async fn test_get_balance() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let report_service = ReportService::new(db);

        let a1 = sample_account("Assets:I", AccountType::Asset);
        let a2 = sample_account("Assets:J", AccountType::Asset);
        let id1 = report_service
            .db
            .account_repo()
            .create(&report_service.db.connection(), &a1)
            .unwrap();
        let id2 = report_service
            .db
            .account_repo()
            .create(&report_service.db.connection(), &a2)
            .unwrap();

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
            channel_id: None,
            is_template: false,
        };
        let tx_id = report_service
            .db
            .transaction_repo()
            .insert(&report_service.db.connection(), &tx, &[])
            .unwrap();

        let mut p1 = sample_posting(id1, "100");
        p1.transaction_id = tx_id;
        let mut p2 = sample_posting(id2, "-100");
        p2.transaction_id = tx_id;

        report_service
            .db
            .posting_repo()
            .insert(&report_service.db.connection(), &p1)
            .unwrap();
        report_service
            .db
            .posting_repo()
            .insert(&report_service.db.connection(), &p2)
            .unwrap();

        let balance = report_service.get_balance(id1).await.unwrap();
        assert_eq!(balance[&CommodityId(1)], Decimal::from_str("100").unwrap());
    }

    #[tokio::test]
    async fn test_stats_by_tag() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let report_service = ReportService::new(db);

        // 准备数据（放在独立作用域中，确保 conn 在 async 调用前释放）
        let _ = {
            let conn = report_service.db.connection();

            // 创建 Income / Expense 账户
            let income_acc = sample_account("Income:Salary", AccountType::Income);
            let expense_acc = sample_account("Expense:Food", AccountType::Expense);
            let income_id = report_service
                .db
                .account_repo()
                .create(&conn, &income_acc)
                .unwrap();
            let expense_id = report_service
                .db
                .account_repo()
                .create(&conn, &expense_acc)
                .unwrap();

            // 创建标签
            let tag = Tag {
                id: TagId(0),
                name: "餐饮".to_string(),
                description: None,
                is_system: false,
            };
            let tag_id = report_service.db.tag_repo().create(&conn, &tag).unwrap();

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
                channel_id: None,
                is_template: false,
            };
            let tx_id = report_service
                .db
                .transaction_repo()
                .insert(&conn, &tx, &[tag_id])
                .unwrap();

            // 插入分录
            let mut p1 = sample_posting(income_id, "100");
            p1.transaction_id = tx_id;
            let mut p2 = sample_posting(expense_id, "-100");
            p2.transaction_id = tx_id;
            report_service.db.posting_repo().insert(&conn, &p1).unwrap();
            report_service.db.posting_repo().insert(&conn, &p2).unwrap();

            (income_id, expense_id, tag_id)
        };

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
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let report_service = ReportService::new(db);

        // 准备数据（放在独立作用域中，确保 conn 在 async 调用前释放）
        let _ = {
            let conn = report_service.db.connection();

            // 创建 Income / Expense 账户
            let income_acc = sample_account("Income:Salary", AccountType::Income);
            let expense_acc = sample_account("Expense:Food", AccountType::Expense);
            let income_id = report_service
                .db
                .account_repo()
                .create(&conn, &income_acc)
                .unwrap();
            let expense_id = report_service
                .db
                .account_repo()
                .create(&conn, &expense_acc)
                .unwrap();

            // 创建成员
            let member = Member {
                id: MemberId(0),
                name: "Alice".to_string(),
            };
            let member_id = report_service
                .db
                .member_repo()
                .create(&conn, &member)
                .unwrap();

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
                channel_id: None,
                is_template: false,
            };
            let tx_id = report_service
                .db
                .transaction_repo()
                .insert(&conn, &tx, &[])
                .unwrap();

            // 插入分录
            let mut p1 = sample_posting(income_id, "100");
            p1.transaction_id = tx_id;
            let mut p2 = sample_posting(expense_id, "-100");
            p2.transaction_id = tx_id;
            report_service.db.posting_repo().insert(&conn, &p1).unwrap();
            report_service.db.posting_repo().insert(&conn, &p2).unwrap();

            (income_id, expense_id, member_id)
        };

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
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let report_service = ReportService::new(db);

        // 准备数据（放在独立作用域中，确保 conn 在 async 调用前释放）
        let _ = {
            let conn = report_service.db.connection();

            // 创建 Income / Expense 账户
            let income_acc = sample_account("Income:Salary", AccountType::Income);
            let expense_acc = sample_account("Expense:Food", AccountType::Expense);
            let income_id = report_service
                .db
                .account_repo()
                .create(&conn, &income_acc)
                .unwrap();
            let expense_id = report_service
                .db
                .account_repo()
                .create(&conn, &expense_acc)
                .unwrap();

            // 创建渠道
            let channel = Channel {
                id: ChannelId(0),
                name: "支付宝".to_string(),
                description: None,
            };
            let channel_id = report_service
                .db
                .channel_repo()
                .create(&conn, &channel)
                .unwrap();

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
                channel_id: None,
                is_template: false,
            };
            let tx_id = report_service
                .db
                .transaction_repo()
                .insert(&conn, &tx, &[])
                .unwrap();

            // 插入分录并关联渠道
            let mut p1 = sample_posting(income_id, "100");
            p1.transaction_id = tx_id;
            p1.channel_id = Some(channel_id);
            let mut p2 = sample_posting(expense_id, "-100");
            p2.transaction_id = tx_id;
            p2.channel_id = Some(channel_id);
            report_service.db.posting_repo().insert(&conn, &p1).unwrap();
            report_service.db.posting_repo().insert(&conn, &p2).unwrap();

            (income_id, expense_id, channel_id)
        };

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
