use accounting::error::AccountingError;
use accounting::id::{TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting::validation::validate_reversal_direction;
use accounting::validation::validate_transaction;
use accounting_sql::database::Database;
use accounting_sql::transaction::Transaction as DbTransaction;
use rust_decimal::Decimal;

/// 交易服务
pub struct TransactionService<D: Database> {
    db: D,
}

impl<D: Database> TransactionService<D> {
    /// 创建服务实例
    pub fn new(db: D) -> Self {
        Self { db }
    }

    /// 提交交易：验证 → 插入交易 + 分录 → 提交
    pub async fn submit(
        &self,
        transaction: Transaction,
        mut postings: Vec<Posting>,
        tag_ids: Vec<TagId>,
    ) -> Result<TransactionId, AccountingError> {
        validate_transaction(&postings)?;
        validate_reversal_direction(&postings)?;

        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let tx_id = tx
            .transaction_repo()
            .insert(&tx.conn(), &transaction, &tag_ids)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证退款/报销分录的关联合法性
        for posting in &postings {
            if posting.kind == accounting::posting::PostingKind::Normal {
                continue;
            }

            let linked_id = posting.linked_posting_id.ok_or_else(|| {
                AccountingError::InvalidTransaction("退款/报销分录必须关联原分录".to_string())
            })?;

            let linked_posting = tx
                .posting_repo()
                .get(&tx.conn(), linked_id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!(
                        "关联的原分录 {} 不存在",
                        linked_id.0
                    ))
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

        for posting in &mut postings {
            posting.transaction_id = tx_id;
            tx.posting_repo()
                .insert(&tx.conn(), posting)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(tx_id)
    }

    /// 更新交易（全量替换）：删除旧分录 → 更新交易 → 插入新分录 → 提交
    pub async fn update(
        &self,
        transaction: Transaction,
        mut postings: Vec<Posting>,
        tag_ids: Vec<TagId>,
    ) -> Result<(), AccountingError> {
        validate_transaction(&postings)?;
        validate_reversal_direction(&postings)?;

        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 删除旧分录
        tx.posting_repo()
            .delete_by_transaction(&tx.conn(), transaction.id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 更新交易
        tx.transaction_repo()
            .update(&tx.conn(), &transaction, &tag_ids)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证退款/报销分录的关联合法性
        for posting in &postings {
            if posting.kind == accounting::posting::PostingKind::Normal {
                continue;
            }

            let linked_id = posting.linked_posting_id.ok_or_else(|| {
                AccountingError::InvalidTransaction("退款/报销分录必须关联原分录".to_string())
            })?;

            let linked_posting = tx
                .posting_repo()
                .get(&tx.conn(), linked_id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!(
                        "关联的原分录 {} 不存在",
                        linked_id.0
                    ))
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

        // 插入新分录
        for posting in &mut postings {
            posting.transaction_id = transaction.id;
            tx.posting_repo()
                .insert(&tx.conn(), posting)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 删除交易（级联删除分录、附件、标签关联由外键约束处理）
    pub async fn delete(&self, id: TransactionId) -> Result<(), AccountingError> {
        let tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.transaction_repo()
            .delete(&tx.conn(), id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 列出交易（含分录）
    pub async fn list(
        &self,
        filter: TransactionFilter,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(Transaction, Vec<Posting>)>, AccountingError> {
        let conn = self.db.connection();
        let limit = limit.map(|l| l as usize).unwrap_or(100);
        let offset = offset.map(|o| o as usize).unwrap_or(0);
        let transactions = self
            .db
            .transaction_repo()
            .list(&conn, &filter, limit, offset)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let mut result = Vec::new();
        for tx in transactions {
            let postings = self
                .db
                .posting_repo()
                .list_by_transaction(&conn, tx.id)
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            result.push((tx, postings));
        }
        Ok(result)
    }

    /// 查询单笔交易（含分录）
    pub async fn get(
        &self,
        id: TransactionId,
    ) -> Result<Option<(Transaction, Vec<Posting>)>, AccountingError> {
        let conn = self.db.connection();
        let transaction = self
            .db
            .transaction_repo()
            .get(&conn, id)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        match transaction {
            Some(tx) => {
                let postings = self
                    .db
                    .posting_repo()
                    .list_by_transaction(&conn, tx.id)
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                Ok(Some((tx, postings)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::account_type::AccountType;
    use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::Transaction;
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
            kind: accounting::posting::PostingKind::Normal,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    fn create_test_account(db: &SqliteDatabase, name: &str, ty: AccountType) -> AccountId {
        let account = sample_account(name, ty);
        db.account_repo()
            .create(&db.connection(), &account)
            .unwrap()
    }

    #[tokio::test]
    async fn test_submit_transaction() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:A", AccountType::Asset);
        let id2 = create_test_account(&tx_service.db, "Assets:B", AccountType::Asset);

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Test".to_string(),
            member_id: None,
            channel_id: None,
            is_template: false,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service.submit(tx, postings, vec![]).await.unwrap();
        assert!(tx_id.0 > 0);
    }

    #[tokio::test]
    async fn test_submit_unbalanced_fails() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:C", AccountType::Asset);
        let id2 = create_test_account(&tx_service.db, "Assets:D", AccountType::Asset);

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Bad".to_string(),
            member_id: None,
            channel_id: None,
            is_template: false,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-50")];

        let result = tx_service.submit(tx, postings, vec![]).await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));
    }

    #[tokio::test]
    async fn test_update_transaction() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:E", AccountType::Asset);
        let id2 = create_test_account(&tx_service.db, "Assets:F", AccountType::Asset);

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Original".to_string(),
            member_id: None,
            channel_id: None,
            is_template: false,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service
            .submit(tx.clone(), postings, vec![])
            .await
            .unwrap();

        // 更新交易
        let updated_tx = Transaction {
            id: tx_id,
            date_time: tx.date_time,
            description: "Updated".to_string(),
            member_id: None,
            channel_id: None,
            is_template: false,
        };
        let new_postings = vec![sample_posting(id1, "200"), sample_posting(id2, "-200")];

        tx_service
            .update(updated_tx, new_postings, vec![])
            .await
            .unwrap();

        // 验证更新后的分录
        let list = tx_service
            .db
            .posting_repo()
            .list_by_transaction(&tx_service.db.connection(), tx_id)
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].amount, Decimal::from_str("200").unwrap());
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        db.initialize("en").unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:G", AccountType::Asset);
        let id2 = create_test_account(&tx_service.db, "Assets:H", AccountType::Asset);

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "ToDelete".to_string(),
            member_id: None,
            channel_id: None,
            is_template: false,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service.submit(tx, postings, vec![]).await.unwrap();
        tx_service.delete(tx_id).await.unwrap();

        assert!(
            tx_service
                .db
                .transaction_repo()
                .get(&tx_service.db.connection(), tx_id)
                .unwrap()
                .is_none()
        );
    }
}
