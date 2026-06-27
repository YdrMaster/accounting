#[cfg(test)]
mod tests {
    use accounting::account::Account;
    use accounting::attachment::Attachment;
    use accounting::channel::Channel;
    use accounting::channel_path::ChannelPathNode;
    use accounting::commodity::Commodity;
    use accounting::id::*;
    use accounting::member::Member;
    use accounting::posting::Posting;
    use accounting::tag::Tag;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting::transaction_filter::TransactionFilter;
    use accounting_beancount::export::export;
    use accounting_beancount::import::import;
    use accounting_sql::SqliteDatabase;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use tempfile::TempDir;

    async fn setup_source_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("zh-CN")).await.unwrap();
        db
    }

    async fn setup_target_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize(Some("zh-CN")).await.unwrap();
        db
    }

    async fn create_test_data(db: &SqliteDatabase) -> TransactionId {
        // 创建商品
        let _usd_id = db
            .commodity_create(&Commodity {
                id: CommodityId(0),
                symbol: "USD".to_string(),
                name: "美元".to_string(),
                precision: 2,
            })
            .await
            .unwrap();

        // 创建成员
        let member_id = db
            .member_create(&Member {
                id: MemberId(0),
                name: "张三".to_string(),
            })
            .await
            .unwrap();

        // 创建渠道
        let channel_id = db
            .channel_create(&Channel {
                id: ChannelId(0),
                name: "微信".to_string(),
                description: Some("微信支付".to_string()),
                account_id: None,
                is_system: false,
            })
            .await
            .unwrap();

        // 创建账户
        let assets_id = db.account_get_by_name("资产").await.unwrap().unwrap().id;
        let bank_id = db
            .account_create_with_closure(&Account {
                id: AccountId(0),
                name: "工商银行".to_string(),
                parent_id: Some(assets_id),
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            })
            .await
            .unwrap();

        let expense_id = db.account_get_by_name("支出").await.unwrap().unwrap().id;
        let food_id = db
            .account_create_with_closure(&Account {
                id: AccountId(0),
                name: "餐饮".to_string(),
                parent_id: Some(expense_id),
                closed_at: None,
                is_system: false,
                billing_day: None,
                repayment_day: None,
            })
            .await
            .unwrap();

        // 创建标签
        let tag_id = db
            .tag_create(&Tag {
                id: TagId(0),
                name: "日常".to_string(),
                description: Some("日常开支".to_string()),
                is_system: false,
            })
            .await
            .unwrap();

        // 创建交易
        let tx = Transaction {
            id: TransactionId(0),
            date_time: chrono::NaiveDateTime::parse_from_str(
                "2024-03-15 10:30:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            description: "午餐".to_string(),
            kind: TransactionKind::Normal,
            member_id: Some(member_id),
        };
        let tag_ids = vec![tag_id];
        let tx_id = db.transaction_insert(&tx, &tag_ids).await.unwrap();

        // 创建分录
        let posting1 = Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id: food_id,
            commodity_id: CommodityId(1), // CNY
            amount: Decimal::from_str("-50.00").unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        db.posting_insert(&posting1).await.unwrap();

        let posting2 = Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id: bank_id,
            commodity_id: CommodityId(1), // CNY
            amount: Decimal::from_str("50.00").unwrap(),
            cost: None,
            cost_commodity_id: None,
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        };
        db.posting_insert(&posting2).await.unwrap();

        // 创建渠道路径
        let nodes = vec![ChannelPathNode {
            position: 0,
            channel_id,
            reconciled: true,
        }];
        db.channel_path_create_batch(tx_id, &nodes).await.unwrap();

        tx_id
    }

    #[tokio::test]
    async fn test_round_trip_basic() {
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // 导出
        let beancount_text = export(&source_db, &output_dir).await.unwrap();
        assert!(!beancount_text.is_empty());

        // 写入 beancount 文件
        let beancount_file = output_dir.join("backup.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 导入到新库
        let target_db = setup_target_db().await;
        let import_result = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();

        // 验证导入统计
        assert_eq!(import_result.commodities, 2); // CNY + USD
        assert!(import_result.accounts >= 7); // 5 根账户 + 工商银行 + 餐饮
        assert_eq!(import_result.members, 1);
        assert_eq!(import_result.channels, 2); // 支付宝 (seed) + 微信
        assert_eq!(import_result.transactions, 1);

        // 验证数据一致性
        let members = target_db.member_list().await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "张三");

        let channels = target_db.channel_list().await.unwrap();
        assert_eq!(channels.len(), 2);
        let wechat = channels.iter().find(|c| c.name == "微信").unwrap();
        assert_eq!(wechat.description, Some("微信支付".to_string()));

        let commodities = target_db.commodity_list().await.unwrap();
        assert_eq!(commodities.len(), 2);
        let usd = commodities.iter().find(|c| c.symbol == "USD").unwrap();
        assert_eq!(usd.name, "美元");

        let transactions = target_db
            .transaction_list(&TransactionFilter::default(), 100, 0)
            .await
            .unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].description, "午餐");
        assert_eq!(transactions[0].kind, TransactionKind::Normal);

        let postings = target_db
            .posting_list_by_transaction(transactions[0].id)
            .await
            .unwrap();
        assert_eq!(postings.len(), 2);

        let channel_paths = target_db
            .channel_path_list_by_transaction(transactions[0].id)
            .await
            .unwrap();
        assert_eq!(channel_paths.len(), 1);
        assert_eq!(channel_paths[0].position, 0);
        assert!(channel_paths[0].reconciled);
    }

    #[tokio::test]
    async fn test_round_trip_with_attachments() {
        let source_db = setup_source_db().await;
        let tx_id = create_test_data(&source_db).await;

        // 创建附件
        let attachment_data = b"test receipt content";
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "receipt.txt".to_string(),
            data: attachment_data.to_vec(),
        };
        source_db.attachment_create(&attachment).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // 导出
        let beancount_text = export(&source_db, &output_dir).await.unwrap();

        // 验证附件文件已创建
        let attachments_dir = output_dir.join("attachments");
        assert!(attachments_dir.exists());
        let attachment_files: Vec<_> = std::fs::read_dir(&attachments_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(attachment_files.len(), 1);

        // 验证附件文件内容
        let exported_attachment_path = attachment_files[0].path();
        let exported_attachment_content = std::fs::read(&exported_attachment_path).unwrap();
        assert_eq!(exported_attachment_content, attachment_data);

        // 写入 beancount 文件
        let beancount_file = output_dir.join("backup.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 导入到新库
        let target_db = setup_target_db().await;
        let import_result = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();

        assert_eq!(import_result.attachments, 1);

        // 验证附件已导入
        let target_transactions = target_db
            .transaction_list(&TransactionFilter::default(), 100, 0)
            .await
            .unwrap();
        let target_tx_id = target_transactions[0].id;

        let target_attachments = target_db
            .attachment_list_by_transaction(target_tx_id)
            .await
            .unwrap();
        assert_eq!(target_attachments.len(), 1);
        assert_eq!(target_attachments[0].filename, "receipt.txt");
        assert_eq!(target_attachments[0].data, attachment_data);
    }

    #[tokio::test]
    async fn test_import_invalid_beancount() {
        let target_db = setup_target_db().await;
        let temp_dir = TempDir::new().unwrap();

        // 测试无效的商品声明 - 缺少 symbol
        let invalid_commodity = "2024-01-01 commodity";
        let _result = import(&target_db, invalid_commodity, temp_dir.path()).await;
        // 这应该能解析但可能创建空商品，或者报错
        // 实际行为取决于 parser 的容错性

        // 测试完全无效的行
        let invalid_line = "this is not valid beancount";
        let result = import(&target_db, invalid_line, temp_dir.path()).await;
        // parser 应该忽略无法识别的行，所以这应该成功但导入 0 条数据
        assert!(result.is_ok());
        let import_result = result.unwrap();
        assert_eq!(import_result.transactions, 0);
    }

    #[tokio::test]
    async fn test_import_missing_attachment() {
        let source_db = setup_source_db().await;
        let tx_id = create_test_data(&source_db).await;

        // 创建附件
        let attachment = Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "missing.txt".to_string(),
            data: b"test".to_vec(),
        };
        source_db.attachment_create(&attachment).await.unwrap();

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // 导出
        let beancount_text = export(&source_db, &output_dir).await.unwrap();

        // 删除附件文件
        let attachments_dir = output_dir.join("attachments");
        let attachment_files: Vec<_> = std::fs::read_dir(&attachments_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        if !attachment_files.is_empty() {
            std::fs::remove_file(attachment_files[0].path()).unwrap();
        }

        // 写入 beancount 文件
        let beancount_file = output_dir.join("backup.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 导入应该成功但附件数为 0（因为文件不存在）
        let target_db = setup_target_db().await;
        let import_result = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();

        assert_eq!(import_result.attachments, 0);
    }

    #[tokio::test]
    async fn test_import_duplicate_transaction() {
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // 导出
        let beancount_text = export(&source_db, &output_dir).await.unwrap();

        // 写入 beancount 文件
        let beancount_file = output_dir.join("backup.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 第一次导入
        let target_db = setup_target_db().await;
        let import_result1 = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();
        assert_eq!(import_result1.transactions, 1);
        assert_eq!(import_result1.skipped, 0);

        // 第二次导入（应该跳过）
        let import_result2 = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();
        assert_eq!(import_result2.transactions, 0);
        assert_eq!(import_result2.skipped, 1);
    }
}
