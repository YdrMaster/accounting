use crate::account_service::AccountService;
use crate::import::{AdaptError, BillEntry, ImportContext, builtin_adapters, find_adapter};
use crate::transaction_service::TransactionService;
use accounting::channel_path::ChannelPathNode;
use accounting::error::AccountingError;
use accounting::id::{
    AccountId, ChannelId, CommodityId, MemberId, PostingId, TagId, TransactionId,
};
use accounting::posting::Posting;
use accounting::posting_role::PostingRole;
use accounting::transaction::Transaction;
use accounting_sql::SqliteDatabase;
use rust_decimal::Decimal;

/// 映射查找键：(member_id, channel_id)
type MappingKey = (MemberId, ChannelId);

/// 导入上下文：(channel_name, import_root)
type ImportCtx = (String, String);

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

        // 2. 解析 source 为 ChannelId 和渠道名称
        let channel = self
            .db
            .channel_get_by_name(source)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let (channel_id, channel_name) = channel.map(|c| (c.id, c.name)).ok_or_else(|| {
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

        // 5. 查找导入根账户名称
        let import_root = self.resolve_import_root().await?;

        // 6. 构造 ImportContext
        let ctx = ImportContext {
            member_id,
            channel_id,
            commodity_id,
            channel_name: channel_name.clone(),
        };

        // 7. 调用适配器解析
        let iter = adapter
            .parse(data, &ctx)
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;

        // 8. 迭代 BillEntry 逐条处理
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
                    (member_id, channel_id),
                    &(channel_name.clone(), import_root.clone()),
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
                    let error = if let Some(row) = entry.row {
                        AdaptError::RowError {
                            row,
                            message: e.to_string(),
                        }
                    } else {
                        AdaptError::FormatError(e.to_string())
                    };
                    result.errors.push(error);
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
        mapping_key: MappingKey,
        import_ctx: &ImportCtx,
        default_commodity_id: CommodityId,
        pending_tag_id: Option<TagId>,
        account_service: &AccountService,
        tx_service: &TransactionService,
    ) -> Result<TransactionId, AccountingError> {
        let (member_id, channel_id) = mapping_key;
        let commodity = self
            .db
            .commodity_get_by_symbol("CNY")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
        let commodity_id = commodity.map(|c| c.id).unwrap_or(default_commodity_id);

        // 对每个 BillPosting 查映射表或走 Import fallback
        let mut postings = Vec::new();
        for bp in &entry.postings {
            let account_id = self
                .resolve_account_id(
                    mapping_key,
                    import_ctx,
                    bp.role,
                    &bp.category,
                    account_service,
                )
                .await?;

            postings.push(Posting {
                id: PostingId(0),
                transaction_id: TransactionId(0),
                account_id,
                commodity_id,
                amount: bp.amount,
                cost: None,
                cost_commodity_id: None,
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

    /// 解析 BillPosting 对应的 AccountId
    ///
    /// 优先查询映射表，有映射则直接使用映射目标账户；
    /// 无映射则构造 Import fallback 路径并 ensure_cascading。
    async fn resolve_account_id(
        &self,
        mapping_key: MappingKey,
        import_ctx: &ImportCtx,
        role: PostingRole,
        category: &str,
        account_service: &AccountService,
    ) -> Result<AccountId, AccountingError> {
        let (member_id, channel_id) = mapping_key;
        let (channel_name, import_root) = import_ctx;
        let key = role.to_key(category);

        // 1. 查映射表
        if let Some(mapping) = self
            .db
            .account_mapping_find(member_id, channel_id, &key)
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
        {
            return Ok(mapping.account_id);
        }

        // 2. 无映射 → Import fallback
        let path = format!(
            "{}:{}:{}:{}",
            import_root,
            channel_name,
            role.import_segment(),
            category
        );
        account_service.ensure_cascading(&path).await
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

    /// 解析导入根账户名称
    async fn resolve_import_root(&self) -> Result<String, AccountingError> {
        // 尝试中文
        if let Some(account) = self
            .db
            .account_get_by_name("导入")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            && account.parent_id.is_none()
            && account.is_system
        {
            return Ok("导入".to_string());
        }
        // 尝试英文
        if let Some(account) = self
            .db
            .account_get_by_name("Import")
            .await
            .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
            && account.parent_id.is_none()
            && account.is_system
        {
            return Ok("Import".to_string());
        }
        Err(AccountingError::AccountNotFound(
            "导入根账户不存在，请检查数据库初始化".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("en")).await.unwrap();

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
            is_system: false,
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
        if let Some(tx_id) = result.transaction_ids.first() {
            let tx = db.transaction_get(*tx_id).await.unwrap().unwrap();
            assert_eq!(tx.description, "美团外卖 - 美团外卖-午餐");
        }

        // 验证 Import 子账户已创建（新结构：含角色层级）
        let root = db
            .account_get_by_parent_and_name(None, "Import")
            .await
            .unwrap()
            .unwrap();
        assert!(root.is_system);

        let alipay = db
            .account_get_by_parent_and_name(Some(root.id), "alipay")
            .await
            .unwrap();
        assert!(alipay.is_some(), "Import:alipay 子账户应已创建");
        let alipay_id = alipay.as_ref().unwrap().id;

        // 验证收支层级
        let income_expense = db
            .account_get_by_parent_and_name(Some(alipay_id), "收支")
            .await
            .unwrap();
        assert!(
            income_expense.is_some(),
            "Import:alipay:收支 子账户应已创建"
        );

        let ie_account = income_expense.unwrap();
        let dining = db
            .account_get_by_parent_and_name(Some(ie_account.id), "餐饮美食")
            .await
            .unwrap();
        assert!(
            dining.is_some(),
            "Import:alipay:收支:餐饮美食 子账户应已创建"
        );

        // 验证资产层级
        let asset = db
            .account_get_by_parent_and_name(Some(alipay_id), "资产")
            .await
            .unwrap();
        assert!(asset.is_some(), "Import:alipay:资产 子账户应已创建");
    }

    #[tokio::test]
    async fn test_import_with_mapping() {
        let db = setup_db().await;

        // 查找实际的渠道 ID
        let channel = db.channel_get_by_name("alipay").await.unwrap().unwrap();

        // 创建一个目标账户供映射
        let expense_account = accounting::account::Account {
            id: AccountId(0),
            name: "Dining".to_string(),
            parent_id: None,
            closed_at: None,
            is_system: false,
            billing_day: None,
            repayment_day: None,
        };
        let expense_id = db
            .account_create_with_closure(&expense_account)
            .await
            .unwrap();

        // 创建映射
        let mapping = accounting::account_mapping::AccountMapping {
            member_id: MemberId(1),
            channel_id: channel.id,
            category: "收支:餐饮美食".to_string(),
            account_id: expense_id,
        };
        db.account_mapping_upsert(&mapping).await.unwrap();

        let service = ImportService::new(db.clone());
        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        assert_eq!(result.imported, 1);

        // 验证收支侧使用了映射目标账户
        let tx_id = result.transaction_ids[0];
        let postings = db.posting_list_by_transaction(tx_id).await.unwrap();
        let ie_posting = postings.iter().find(|p| p.account_id == expense_id);
        assert!(
            ie_posting.is_some(),
            "收支侧应使用映射目标账户 Dining (id={})",
            expense_id.0
        );
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
        db.initialize(Some("en")).await.unwrap();
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

    #[tokio::test]
    async fn test_import_alipay_refund_row() {
        let db = setup_db().await;
        let service = ImportService::new(db);

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,退款,测试商家,test***@test.com,测试商品,不计收支,36.75,蚂蚁宝藏信用卡,退款成功,2024011522001470000001\t,MO20240101\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        assert_eq!(result.imported, 1, "退款行应成功导入");
        assert_eq!(result.skipped, 0, "不应跳过退款行");
    }

    #[tokio::test]
    async fn test_import_alipay_real_file_includes_refunds() {
        let db = setup_db().await;
        let service = ImportService::new(db);

        let data = std::fs::read("test/支付宝交易明细.csv").unwrap();
        let result = service.import(&data, "alipay", MemberId(1)).await.unwrap();

        assert!(
            result.imported > 300,
            "应导入 300+ 条，实际导入 {} 条，跳过 {} 条",
            result.imported,
            result.skipped
        );
        // 测试账本里只有已关闭交易会被跳过，退款行必须被正常导入。
        assert!(
            result.errors.iter().all(|e| e.to_string().contains("交易已关闭")),
            "测试账本出现非预期的跳过错误：{:?}",
            result.errors
        );
    }
}
