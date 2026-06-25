use crate::account_service::AccountService;
use crate::import::{AdaptError, BillEntry, ImportContext, builtin_adapters, find_adapter};
use crate::transaction_service::TransactionService;
use accounting::channel_path::ChannelPathNode;
use accounting::error::AccountingError;
use accounting::id::{ChannelId, CommodityId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;

/// 导入结果
pub struct ImportResult {
    pub transaction_ids: Vec<TransactionId>,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<AdaptError>,
}

/// 导入服务 — 编排完整导入流程
pub struct ImportService {
    db: SqliteDatabase,
}

impl ImportService {
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// 执行导入
    pub async fn import(
        &self,
        data: &[u8],
        source: &str,
        member_id: MemberId,
    ) -> Result<ImportResult, AccountingError> {
        // 1. 查找适配器
        let adapters = builtin_adapters();
        let adapter = find_adapter(source, &adapters).ok_or_else(|| {
            AccountingError::InvalidTransaction(format!("不支持的来源: {source}"))
        })?;

        // 2. 解析 source 为 ChannelId
        let channel = self
            .db
            .channel_get_by_name(source)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let channel_id = channel.map(|c| c.id).ok_or_else(|| {
            AccountingError::AccountNotFound(format!("渠道 '{source}' 不存在，请先创建对应渠道"))
        })?;

        // 3. 查找默认 CommodityId（CNY）
        let commodity = self
            .db
            .commodity_get_by_symbol("CNY")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let commodity_id = commodity
            .map(|c| c.id)
            .ok_or_else(|| AccountingError::AccountNotFound("默认商品 CNY 不存在".to_string()))?;

        // 4. 查找 "待处理" 系统 Tag
        let pending_tag_id = self.resolve_pending_tag_id().await?;

        // 5. 构造 ImportContext
        let ctx = ImportContext {
            member_id,
            channel_id,
            commodity_id,
        };

        // 6. 调用适配器解析
        let iter = adapter
            .parse(data, &ctx)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 7. 迭代 BillEntry 逐条处理
        let account_service = AccountService::new(self.db.clone());
        let tx_service = TransactionService::new(self.db.clone());

        let mut result = ImportResult {
            transaction_ids: Vec::new(),
            imported: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        for entry_result in iter {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    result.skipped += 1;
                    result.errors.push(e);
                    continue;
                }
            };

            match self
                .submit_entry(
                    &entry,
                    member_id,
                    channel_id,
                    commodity_id,
                    pending_tag_id,
                    &account_service,
                    &tx_service,
                )
                .await
            {
                Ok(tx_id) => {
                    result.transaction_ids.push(tx_id);
                    result.imported += 1;
                }
                Err(e) => {
                    result.skipped += 1;
                    result.errors.push(AdaptError::FormatError(e.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// 将单个 BillEntry 提交为交易
    #[allow(clippy::too_many_arguments)]
    async fn submit_entry(
        &self,
        entry: &BillEntry,
        member_id: MemberId,
        channel_id: ChannelId,
        default_commodity_id: CommodityId,
        pending_tag_id: Option<TagId>,
        account_service: &AccountService,
        tx_service: &TransactionService,
    ) -> Result<TransactionId, AccountingError> {
        let commodity = self
            .db
            .commodity_get_by_symbol("CNY")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let commodity_id = commodity.map(|c| c.id).unwrap_or(default_commodity_id);

        // 对每个 BillPosting.account_path 调用 ensure_cascading 创建/查找账户
        let mut postings = Vec::new();
        for bp in &entry.postings {
            let account_id = account_service.ensure_cascading(&bp.account_path).await?;

            postings.push(Posting {
                id: PostingId(0),
                transaction_id: TransactionId(0),
                account_id,
                commodity_id,
                amount: bp.amount,
                cost: None,
                cost_commodity_id: None,
                description: None,
                is_reimbursable: bp.is_reimbursable,
                linked_posting_id: None,
                reversal_total: Decimal::ZERO,
            });
        }

        // 构建 Transaction
        let transaction = Transaction {
            id: TransactionId(0),
            date_time: entry.date_time,
            description: entry.description.clone(),
            kind: entry.kind,
            member_id: Some(member_id),
        };

        // 构建 tag_ids（含 "待处理" 系统 Tag）
        let mut tag_ids: Vec<TagId> = Vec::new();
        if let Some(ptid) = pending_tag_id {
            tag_ids.push(ptid);
        }

        // 解析 BillEntry 中的额外 tags
        for tag_name in &entry.tags {
            if let Some(tag) = self
                .db
                .tag_get_by_name(tag_name)
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                && !tag_ids.contains(&tag.id)
            {
                tag_ids.push(tag.id);
            }
        }

        // 构建 ChannelPathNode
        let channel_path_nodes = vec![ChannelPathNode {
            position: 0,
            channel_id,
            reconciled: false,
        }];

        tx_service
            .submit(transaction, postings, tag_ids, channel_path_nodes)
            .await
    }

    /// 解析 "待处理" 系统 Tag ID
    async fn resolve_pending_tag_id(&self) -> Result<Option<TagId>, AccountingError> {
        // 尝试中文
        if let Some(tag) = self
            .db
            .tag_get_by_name("待处理")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
        {
            return Ok(Some(tag.id));
        }
        // 尝试英文
        if let Some(tag) = self
            .db
            .tag_get_by_name("pending")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
        {
            return Ok(Some(tag.id));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize("en").await.unwrap();

        // 创建成员
        let member = accounting::member::Member {
            id: MemberId(0),
            name: "测试用户".to_string(),
        };
        db.member_create(&member).await.unwrap();

        // 创建 "alipay" 渠道
        let channel = accounting::channel::Channel {
            id: ChannelId(0),
            name: "alipay".to_string(),
            description: Some("支付宝".to_string()),
            account_id: None,
        };
        db.channel_create(&channel).await.unwrap();

        db
    }

    #[tokio::test]
    async fn test_import_alipay_csv() {
        let db = setup_db().await;
        let service = ImportService::new(db.clone());

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
            "2024-01-16 09:00:00,交通出行,滴滴出行,chu***@didichuxing.com,快车费,支出,28.50,蚂蚁宝藏信用卡,交易成功,2024011622001470000002\t,MO20240102\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.transaction_ids.len(), 2);

        // 验证交易带 "pending" Tag
        for tx_id in &result.transaction_ids {
            let tx = db.transaction_get(*tx_id).await.unwrap().unwrap();
            assert_eq!(tx.description, "美团外卖 - 美团外卖-午餐");
            break;
        }

        // 验证 Import 子账户已创建
        let root = db
            .account_get_by_parent_and_name(None, "Import")
            .await
            .unwrap()
            .unwrap();
        assert!(root.is_system);

        let alipay = db
            .account_get_by_parent_and_name(Some(root.id), "支付宝")
            .await
            .unwrap();
        assert!(alipay.is_some(), "Import:支付宝 子账户应已创建");

        let other = db
            .account_get_by_parent_and_name(Some(alipay.unwrap().id), "餐饮美食")
            .await
            .unwrap();
        assert!(other.is_some(), "Import:支付宝:餐饮美食 子账户应已创建");
    }

    #[tokio::test]
    async fn test_import_unsupported_source() {
        let db = setup_db().await;
        let service = ImportService::new(db);

        let result = service.import(b"test", "unknown", MemberId(1)).await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        match err {
            AccountingError::InvalidTransaction(msg) => {
                assert!(msg.contains("不支持的来源"), "actual: {msg}");
            }
            _ => panic!("预期 InvalidTransaction 错误，实际：{err:?}"),
        }
    }

    #[tokio::test]
    async fn test_import_channel_not_found() {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize("en").await.unwrap();
        let service = ImportService::new(db);

        let result = service.import(b"test", "alipay", MemberId(1)).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_skip_on_error() {
        let db = setup_db().await;
        let service = ImportService::new(db);

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
            "2024-01-16 09:00:00,日用百货,测试商家,test***@test.com,已关闭交易,支出,50.00,蚂蚁宝藏信用卡,交易关闭,2024011622001470000002\t,MO20240102\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        assert_eq!(result.imported, 1, "只应成功导入 1 条");
        assert_eq!(result.skipped, 1, "应跳过 1 条");
        assert_eq!(result.errors.len(), 1);
    }
}
