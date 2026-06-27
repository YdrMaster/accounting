use accounting::channel_path::ChannelPathNode;
use accounting::error::AccountingError;
use accounting::id::{ChannelPathId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting::validation::validate_kind_consistency;
use accounting::validation::validate_reversal_cap;
use accounting::validation::validate_reversal_direction;
use accounting::validation::validate_transaction;
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;
use rust_i18n::t;

/// 交易服务
pub struct TransactionService {
    db: SqliteDatabase,
}

impl TransactionService {
    /// 创建服务实例
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 提交交易：验证 → 插入交易 + 分录 + 链路 → 提交
    pub async fn submit(
        &self,
        transaction: Transaction,
        mut postings: Vec<Posting>,
        tag_ids: Vec<TagId>,
        channel_path_nodes: Vec<ChannelPathNode>,
    ) -> Result<TransactionId, AccountingError> {
        validate_transaction(&postings)?;
        validate_reversal_direction(&postings)?;
        validate_kind_consistency(transaction.kind, &postings)?;

        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        let tx_id = tx
            .transaction_insert(&transaction, &tag_ids)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 写入 channel_paths（渠道存在性由 FK 约束保证）
        tx.channel_path_create_batch(tx_id, &channel_path_nodes)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证退款/报销分录的关联合法性
        for posting in &postings {
            if posting.linked_posting_id.is_none() {
                continue;
            }

            let linked_id = posting.linked_posting_id.ok_or_else(|| {
                AccountingError::InvalidTransaction(t!("refund_must_link").to_string())
            })?;

            let linked_posting = tx
                .posting_get(linked_id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!(
                        "{}",
                        t!("linked_posting_not_found", id = linked_id.0)
                    ))
                })?;

            if linked_posting.linked_posting_id.is_some() {
                return Err(AccountingError::InvalidTransaction(
                    t!("can_only_reverse_normal_posting").to_string(),
                ));
            }

            if linked_posting.account_id != posting.account_id {
                return Err(AccountingError::InvalidTransaction(
                    t!("refund_must_same_account").to_string(),
                ));
            }

            if (posting.amount > Decimal::ZERO && linked_posting.amount > Decimal::ZERO)
                || (posting.amount < Decimal::ZERO && linked_posting.amount < Decimal::ZERO)
            {
                return Err(AccountingError::InvalidTransaction(
                    t!("refund_direction_must_opposite").to_string(),
                ));
            }

            validate_reversal_cap(
                posting.amount,
                linked_posting.amount,
                linked_posting.reversal_total,
            )?;
        }

        for posting in &mut postings {
            posting.transaction_id = tx_id;
            tx.posting_insert(posting)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(tx_id)
    }

    /// 更新交易（全量替换）：删除旧分录+链路 → 更新交易 → 插入新分录+链路 → 提交
    pub async fn update(
        &self,
        transaction: Transaction,
        mut postings: Vec<Posting>,
        tag_ids: Vec<TagId>,
        channel_path_nodes: Vec<ChannelPathNode>,
    ) -> Result<(), AccountingError> {
        validate_transaction(&postings)?;
        validate_reversal_direction(&postings)?;
        validate_kind_consistency(transaction.kind, &postings)?;

        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 删除旧分录
        tx.posting_delete_by_transaction(transaction.id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 整体替换链路（删除旧 channel_paths，创建新的）
        tx.channel_path_delete_by_transaction(transaction.id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 更新交易
        tx.transaction_update(&transaction, &tag_ids)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 写入新链路
        tx.channel_path_create_batch(transaction.id, &channel_path_nodes)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 验证退款/报销分录的关联合法性
        for posting in &postings {
            if posting.linked_posting_id.is_none() {
                continue;
            }

            let linked_id = posting.linked_posting_id.ok_or_else(|| {
                AccountingError::InvalidTransaction(t!("refund_must_link").to_string())
            })?;

            let linked_posting = tx
                .posting_get(linked_id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!(
                        "{}",
                        t!("linked_posting_not_found", id = linked_id.0)
                    ))
                })?;

            if linked_posting.linked_posting_id.is_some() {
                return Err(AccountingError::InvalidTransaction(
                    t!("can_only_reverse_normal_posting").to_string(),
                ));
            }

            if linked_posting.account_id != posting.account_id {
                return Err(AccountingError::InvalidTransaction(
                    t!("refund_must_same_account").to_string(),
                ));
            }

            if (posting.amount > Decimal::ZERO && linked_posting.amount > Decimal::ZERO)
                || (posting.amount < Decimal::ZERO && linked_posting.amount < Decimal::ZERO)
            {
                return Err(AccountingError::InvalidTransaction(
                    t!("refund_direction_must_opposite").to_string(),
                ));
            }

            validate_reversal_cap(
                posting.amount,
                linked_posting.amount,
                linked_posting.reversal_total,
            )?;
        }

        // 插入新分录
        for posting in &mut postings {
            posting.transaction_id = transaction.id;
            tx.posting_insert(posting)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 删除交易（级联删除分录、附件、标签关联、链路由外键约束处理）
    pub async fn delete(&self, id: TransactionId) -> Result<(), AccountingError> {
        let mut tx = self
            .db
            .transaction()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.transaction_delete(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 列出交易（含分录和渠道链路）
    pub async fn list(
        &self,
        filter: TransactionFilter,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(Transaction, Vec<Posting>, Vec<ChannelPathNode>)>, AccountingError> {
        let limit = limit.map(|l| l as usize).unwrap_or(100);
        let offset = offset.map(|o| o as usize).unwrap_or(0);
        let transactions = self
            .db
            .transaction_list(&filter, limit, offset)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let mut result = Vec::new();
        for tx in transactions {
            let postings = self
                .db
                .posting_list_by_transaction(tx.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            let paths = self
                .db
                .channel_path_list_by_transaction(tx.id)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
            let nodes: Vec<ChannelPathNode> = paths
                .into_iter()
                .map(|p| ChannelPathNode {
                    position: p.position,
                    channel_id: p.channel_id,
                    reconciled: p.reconciled,
                })
                .collect();
            result.push((tx, postings, nodes));
        }
        Ok(result)
    }

    /// 查询单笔交易（含分录和渠道链路）
    pub async fn get(
        &self,
        id: TransactionId,
    ) -> Result<Option<(Transaction, Vec<Posting>, Vec<ChannelPathNode>)>, AccountingError> {
        let transaction = self
            .db
            .transaction_get(id)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        match transaction {
            Some(tx) => {
                let postings = self
                    .db
                    .posting_list_by_transaction(tx.id)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                let paths = self
                    .db
                    .channel_path_list_by_transaction(tx.id)
                    .await
                    .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
                let nodes: Vec<ChannelPathNode> = paths
                    .into_iter()
                    .map(|p| ChannelPathNode {
                        position: p.position,
                        channel_id: p.channel_id,
                        reconciled: p.reconciled,
                    })
                    .collect();
                Ok(Some((tx, postings, nodes)))
            }
            None => Ok(None),
        }
    }

    /// 标记/取消标记链路节点的对账状态
    pub async fn update_reconciled(
        &self,
        channel_path_id: ChannelPathId,
        reconciled: bool,
    ) -> Result<(), AccountingError> {
        self.db
            .channel_path_update_reconciled(channel_path_id, reconciled)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting::account::Account;
    use accounting::id::{AccountId, CommodityId, PostingId, TransactionId};
    use accounting::posting::Posting;
    use accounting::transaction::Transaction;
    use accounting::transaction::TransactionKind;
    use accounting_sql::SqliteDatabase;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_account(name: &str) -> Account {
        Account {
            id: AccountId(0),
            name: name.to_string(),
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
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        }
    }

    async fn create_test_account(db: &SqliteDatabase, name: &str) -> AccountId {
        let account = sample_account(name);
        db.account_create(&account).await.unwrap()
    }

    #[tokio::test]
    async fn test_submit_transaction() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:A").await;
        let id2 = create_test_account(&tx_service.db, "Assets:B").await;

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Test".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service
            .submit(tx, postings, vec![], vec![])
            .await
            .unwrap();
        assert!(tx_id.0 > 0);
    }

    #[tokio::test]
    async fn test_submit_unbalanced_fails() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:C").await;
        let id2 = create_test_account(&tx_service.db, "Assets:D").await;

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Bad".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-50")];

        let result = tx_service.submit(tx, postings, vec![], vec![]).await;
        assert!(matches!(
            result,
            Err(AccountingError::InvalidTransaction(_))
        ));
    }

    #[tokio::test]
    async fn test_update_transaction() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:E").await;
        let id2 = create_test_account(&tx_service.db, "Assets:F").await;

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "Original".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service
            .submit(tx.clone(), postings, vec![], vec![])
            .await
            .unwrap();

        // 更新交易
        let updated_tx = Transaction {
            id: tx_id,
            date_time: tx.date_time,
            description: "Updated".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let new_postings = vec![sample_posting(id1, "200"), sample_posting(id2, "-200")];

        tx_service
            .update(updated_tx, new_postings, vec![], vec![])
            .await
            .unwrap();

        // 验证更新后的分录
        let list = tx_service
            .db
            .posting_list_by_transaction(tx_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].amount, Decimal::from_str("200").unwrap());
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();
        let tx_service = TransactionService::new(db);

        let id1 = create_test_account(&tx_service.db, "Assets:G").await;
        let id2 = create_test_account(&tx_service.db, "Assets:H").await;

        let tx = Transaction {
            id: TransactionId(0),
            date_time: NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            description: "ToDelete".to_string(),
            kind: TransactionKind::Normal,
            member_id: None,
        };
        let postings = vec![sample_posting(id1, "100"), sample_posting(id2, "-100")];

        let tx_id = tx_service
            .submit(tx, postings, vec![], vec![])
            .await
            .unwrap();
        tx_service.delete(tx_id).await.unwrap();

        assert!(
            tx_service
                .db
                .transaction_get(tx_id)
                .await
                .unwrap()
                .is_none()
        );
    }
}
