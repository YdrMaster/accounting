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

/// 导入上下文：channel_name
type ImportCtx = String;

/// 导入致命错误
#[derive(Debug)]
pub enum ImportError {
    UnsupportedSource { source: String },
    ChannelNotFound { source: String },
    CnyCommodityNotFound,
    Parse { source: String },
    Database { source: String },
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::UnsupportedSource { source } => write!(f, "unsupported source: {source}"),
            ImportError::ChannelNotFound { source } => write!(f, "channel not found: {source}"),
            ImportError::CnyCommodityNotFound => write!(f, "default commodity CNY not found"),
            ImportError::Parse { source } => write!(f, "parse error: {source}"),
            ImportError::Database { source } => write!(f, "database error: {source}"),
        }
    }
}

impl std::error::Error for ImportError {}

/// 导入结果
#[derive(Debug)]
pub struct ImportResult {
    pub transaction_ids: Vec<TransactionId>,
    pub imported: usize,
    pub skipped: usize,
    pub pending_tag_name: Option<String>,
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
    ) -> Result<ImportResult, ImportError> {
        // 1. 查找适配器
        let adapters = builtin_adapters();
        let adapter =
            find_adapter(source, &adapters).ok_or_else(|| ImportError::UnsupportedSource {
                source: source.to_string(),
            })?;

        // 2. 解析 source 为 ChannelId（支持别名与大小写不敏感，命中名字表任意语言名字）
        let channel =
            self.db
                .channel_resolve_by_name(source)
                .await
                .map_err(|e| ImportError::Database {
                    source: e.to_string(),
                })?;
        // 渠道实体不再携带名字；Import 路径中的渠道段直接使用调用方给定的 source
        let (channel_id, channel_name) =
            channel.map(|c| (c.id, source.to_string())).ok_or_else(|| {
                ImportError::ChannelNotFound {
                    source: source.to_string(),
                }
            })?;

        // 3. 查找默认 CommodityId（CNY）
        let commodity =
            self.db
                .commodity_get_by_symbol("CNY")
                .await
                .map_err(|e| ImportError::Database {
                    source: e.to_string(),
                })?;
        let commodity_id = commodity
            .map(|c| c.id)
            .ok_or(ImportError::CnyCommodityNotFound)?;

        // 4. 按系统名 "pending" 查找待处理系统 Tag（不再探测 "待处理" 中文名）
        let pending_tag_id = self.resolve_pending_tag_id().await?;

        // 5. 构造 ImportContext
        let ctx = ImportContext {
            member_id,
            channel_id,
            commodity_id,
            channel_name: channel_name.clone(),
        };

        // 7. 调用适配器解析
        let iter = adapter.parse(data, &ctx).map_err(|e| ImportError::Parse {
            source: e.to_string(),
        })?;

        // 8. 迭代 BillEntry 逐条处理
        let account_service = AccountService::new(self.db.clone());
        let tx_service = TransactionService::new(self.db.clone());

        let mut result = ImportResult {
            transaction_ids: Vec::new(),
            imported: 0,
            skipped: 0,
            pending_tag_name: pending_tag_id.map(|_| "pending".to_string()),
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
                    &channel_name,
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
                        AdaptError::Row {
                            row,
                            detail: crate::import::RowErrorDetail::Other {
                                message: e.to_string(),
                            },
                        }
                    } else {
                        AdaptError::Row {
                            row: 0,
                            detail: crate::import::RowErrorDetail::Other {
                                message: e.to_string(),
                            },
                        }
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
                    bp.amount,
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
            member_id,
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

        // 构建 ChannelPathNode（第三方导入默认为 pending）
        let channel_path_nodes = vec![ChannelPathNode {
            position: 0,
            channel_id,
            status: accounting::channel_path::ChannelPathStatus::Pending,
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
        amount: Decimal,
        account_service: &AccountService,
    ) -> Result<AccountId, AccountingError> {
        let (member_id, channel_id) = mapping_key;
        let channel_name = import_ctx;
        let key = role.to_key(category, amount);

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
        let root = role.fallback_root(category, amount);
        let path = format!("{}:Import:{}:{}", root, channel_name, category);
        account_service.ensure_cascading(&path).await
    }

    /// 按系统名 "pending" 解析待处理系统 Tag ID
    ///
    /// 系统标签实体固定以 en 系统名 `pending` 落库（zh-CN 系统名为同一实体的
    /// 另一条名字记录），单名查询即可命中，无需按界面语言探测。
    async fn resolve_pending_tag_id(&self) -> Result<Option<TagId>, ImportError> {
        let tag = self
            .db
            .tag_get_by_name("pending")
            .await
            .map_err(|e| ImportError::Database {
                source: e.to_string(),
            })?;
        Ok(tag.map(|t| t.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accounting_sql::SqliteDatabase;

    async fn setup_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        // 创建成员
        db.member_get_or_create_by_name("测试用户", "zh-CN")
            .await
            .unwrap();

        // 创建 "alipay" 渠道
        db.channel_upsert_by_name("alipay", Some("支付宝"), None, "en")
            .await
            .unwrap();

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

        // 验证新 fallback 账户已创建
        let dining = db
            .account_get_by_name("Expenses:Import:alipay:餐饮美食")
            .await
            .unwrap();
        assert!(
            dining.is_some(),
            "Expenses:Import:alipay:餐饮美食 子账户应已创建"
        );

        let transport = db
            .account_get_by_name("Expenses:Import:alipay:交通出行")
            .await
            .unwrap();
        assert!(
            transport.is_some(),
            "Expenses:Import:alipay:交通出行 子账户应已创建"
        );

        let card = db
            .account_get_by_name("Assets:Import:alipay:蚂蚁宝藏信用卡")
            .await
            .unwrap();
        assert!(
            card.is_some(),
            "Assets:Import:alipay:蚂蚁宝藏信用卡 子账户应已创建"
        );
    }

    #[tokio::test]
    async fn test_import_with_mapping() {
        let db = setup_db().await;

        // 查找实际的渠道 ID
        let channel = db.channel_get_by_name("alipay").await.unwrap().unwrap();

        // 创建一个目标账户供映射（仅按 id 关联，无需名字）
        let expense_account = accounting::account::Account {
            id: AccountId(0),
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
            category: "Expenses:餐饮美食".to_string(),
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
        assert!(
            matches!(result.unwrap_err(), ImportError::UnsupportedSource { .. }),
            "expected UnsupportedSource error"
        );
    }

    #[tokio::test]
    async fn test_import_channel_not_found() {
        // open_in_memory 只创建 schema，不写入种子数据，因此没有 Alipay/支付宝渠道。
        // alipay 有适配器，应报 ChannelNotFound。
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        let service = ImportService::new(db);

        let result = service.import(b"test", "alipay", MemberId(1)).await;

        assert!(
            matches!(result.unwrap_err(), ImportError::ChannelNotFound { .. }),
            "expected ChannelNotFound when no channel exists"
        );
    }

    #[tokio::test]
    async fn test_import_with_channel_alias() {
        // 渠道解析走名字表命中（大小写不敏感）：英文别名 alipay 可命中种子渠道 Alipay/支付宝
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();

        db.member_get_or_create_by_name("测试用户", "zh-CN")
            .await
            .unwrap();

        let service = ImportService::new(db);
        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        assert_eq!(result.imported, 1);
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
        assert!(
            matches!(
                result.errors[0],
                AdaptError::Row {
                    detail: crate::import::RowErrorDetail::ClosedTransaction,
                    ..
                }
            ),
            "expected ClosedTransaction error, got {:?}",
            result.errors[0]
        );
    }

    #[tokio::test]
    async fn test_import_alipay_refund_row() {
        let db = setup_db().await;
        let service = ImportService::new(db.clone());

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

        let refund = db
            .account_get_by_name("Expenses:Import:alipay:退款")
            .await
            .unwrap();
        assert!(
            refund.is_some(),
            "退款应作为负支出归入 Expenses:Import:alipay:退款"
        );
    }

    /// 3.1：pending 标签只按系统名 `pending` 单名查询（不再探测 `待处理`）
    #[tokio::test]
    async fn test_pending_tag_resolved_by_system_name() {
        let db = setup_db().await;
        let service = ImportService::new(db.clone());

        let csv_data = concat!(
            "交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,收/付款方式,交易状态,交易订单号,商家订单号,备注,\n",
            "2024-01-15 12:30:00,餐饮美食,美团外卖,mei***@tuan.com,美团外卖-午餐,支出,35.00,蚂蚁宝藏信用卡,交易成功,2024011522001470000001\t,MO20240101\t,,\n",
        );

        let result = service
            .import(csv_data.as_bytes(), "alipay", MemberId(1))
            .await
            .unwrap();

        // pending_tag_name 固定为系统名
        assert_eq!(result.pending_tag_name.as_deref(), Some("pending"));

        // 交易挂上了 pending 系统标签；显示名按语言回退链解析
        let names_en = db
            .tag_names_by_transactions(&result.transaction_ids, "en")
            .await
            .unwrap();
        assert!(
            names_en
                .values()
                .all(|names| names.iter().any(|n| n == "pending")),
            "en 显示名应为 pending: {:?}",
            names_en
        );
        let names_zh = db
            .tag_names_by_transactions(&result.transaction_ids, "zh-CN")
            .await
            .unwrap();
        assert!(
            names_zh
                .values()
                .all(|names| names.iter().any(|n| n == "待处理")),
            "zh-CN 显示名应为 待处理: {:?}",
            names_zh
        );
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
            result.errors.iter().all(|e| matches!(
                e,
                AdaptError::Row {
                    detail: crate::import::RowErrorDetail::ClosedTransaction,
                    ..
                }
            )),
            "测试账本出现非预期的跳过错误：{:?}",
            result.errors
        );
    }
}
