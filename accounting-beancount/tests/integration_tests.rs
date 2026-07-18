#[cfg(test)]
mod tests {
    use accounting::channel_path::{ChannelPathNode, ChannelPathStatus};
    use accounting::commodity::Commodity;
    use accounting::id::*;
    use accounting::posting::Posting;
    use accounting::transaction::{Transaction, TransactionKind};
    use accounting::transaction_filter::TransactionFilter;
    use accounting_beancount::export::export;
    use accounting_beancount::import::import;
    use accounting_sql::SqliteDatabase;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use tempfile::TempDir;

    /// 导出接口接收数据库句柄，源库可用内存库
    async fn setup_source_db() -> SqliteDatabase {
        let db = SqliteDatabase::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    async fn setup_target_db() -> SqliteDatabase {
        setup_source_db().await
    }

    async fn create_test_data(db: &SqliteDatabase) -> TransactionId {
        // 创建商品（币种只有 symbol，无名字）
        db.commodity_create(&Commodity {
            id: CommodityId(0),
            symbol: "USD".to_string(),
            precision: 2,
            created_at: None,
        })
        .await
        .unwrap();

        // 创建成员
        let member_id = db
            .member_get_or_create_by_name("张三", "zh-CN")
            .await
            .unwrap();

        // 创建渠道
        let channel_id = db
            .channel_upsert_by_name("微信", Some("微信支付"), None, "zh-CN")
            .await
            .unwrap();

        // 创建账户：中文路径、英文路径各一例
        let bank_id = db
            .account_get_or_create_by_path("资产:工商银行", "zh-CN")
            .await
            .unwrap();
        let dining_id = db
            .account_get_or_create_by_path("Expenses:Dining", "en")
            .await
            .unwrap();

        // 创建标签
        let tag_id = db.tag_upsert_by_name("日常", None, "zh-CN").await.unwrap();

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
            member_id,
        };
        let tag_ids = vec![tag_id];
        let tx_id = db.transaction_insert(&tx, &tag_ids).await.unwrap();

        let cny_id = db.commodity_get_by_symbol("CNY").await.unwrap().unwrap().id;

        // 创建分录
        let posting1 = Posting {
            id: PostingId(0),
            transaction_id: tx_id,
            account_id: dining_id,
            commodity_id: cny_id,
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
            commodity_id: cny_id,
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
            status: ChannelPathStatus::Verified,
        }];
        db.channel_path_create_batch(tx_id, &nodes).await.unwrap();

        tx_id
    }

    #[tokio::test]
    async fn test_round_trip_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let output_dir = temp_dir.path().join("out");

        // 导出（中文显示语言）
        let beancount_text = export(&source_db, "zh-CN", &output_dir).await.unwrap();
        assert!(!beancount_text.is_empty());
        assert!(
            beancount_text.contains("资产:工商银行"),
            "zh-CN 导出应含中文账户路径，got:\n{}",
            beancount_text
        );

        // 写入 beancount 文件
        let beancount_file = output_dir.join("transactions.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 导入到新库
        let target_db = setup_target_db().await;
        let import_result = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();

        // 验证导入统计
        assert_eq!(import_result.commodities, 2); // CNY + USD
        assert!(import_result.accounts >= 6); // 4 根账户 + 工商银行 + Dining
        assert_eq!(import_result.members, 1);
        assert_eq!(import_result.channels, 2); // Alipay (seed) + 微信
        assert_eq!(import_result.transactions, 1);

        // 验证数据一致性（名字走名字表命中查询）
        let members = target_db.member_list().await.unwrap();
        assert_eq!(members.len(), 1);
        assert!(
            target_db
                .member_get_by_name("张三")
                .await
                .unwrap()
                .is_some()
        );

        let channels = target_db.channel_list().await.unwrap();
        assert_eq!(channels.len(), 2);
        let wechat = target_db
            .channel_get_by_name("微信")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(wechat.description, Some("微信支付".to_string()));

        let commodities = target_db.commodity_list().await.unwrap();
        assert_eq!(commodities.len(), 2);
        assert!(
            target_db
                .commodity_get_by_symbol("USD")
                .await
                .unwrap()
                .is_some()
        );

        // 中英文账户名均按名字表命中既有账户（不重复创建；账户按路径逐段命中）
        let bank = target_db
            .account_get_by_name("资产:工商银行")
            .await
            .unwrap();
        assert!(bank.is_some(), "中文账户路径应命中导入后的账户");
        let dining = target_db
            .account_get_by_name("Expenses:Dining")
            .await
            .unwrap();
        assert!(dining.is_some(), "英文账户路径应命中导入后的账户");

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
        assert_eq!(channel_paths[0].status, ChannelPathStatus::Verified);
    }

    #[tokio::test]
    async fn test_export_lang_switches_account_paths() {
        let temp_dir = TempDir::new().unwrap();
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let en_dir = temp_dir.path().join("out_en");
        let zh_dir = temp_dir.path().join("out_zh");

        let en_text = export(&source_db, "en", &en_dir).await.unwrap();
        let zh_text = export(&source_db, "zh-CN", &zh_dir).await.unwrap();

        // 同一库，系统根账户路径随 lang 变化
        assert!(
            en_text.contains("open Assets:"),
            "en 导出应含英文系统账户路径，got:\n{}",
            en_text
        );
        assert!(
            zh_text.contains("open 资产:"),
            "zh-CN 导出应含中文系统账户路径，got:\n{}",
            zh_text
        );
        assert_ne!(en_text, zh_text);

        // 英文账户名（en 名字）在两种语言下按回退链解析
        assert!(en_text.contains("Expenses:Dining"));
        assert!(zh_text.contains("支出:Dining"));
        // 中文账户名（zh-CN 名字）在 en 下回退到中文名
        assert!(en_text.contains("Assets:工商银行"));
        assert!(zh_text.contains("资产:工商银行"));
    }

    #[tokio::test]
    async fn test_round_trip_with_attachments() {
        let temp_dir = TempDir::new().unwrap();
        let source_db = setup_source_db().await;
        let tx_id = create_test_data(&source_db).await;

        // 创建附件
        let attachment_data = b"test receipt content";
        let attachment = accounting::attachment::Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "receipt.txt".to_string(),
            data: attachment_data.to_vec(),
        };
        source_db.attachment_create(&attachment).await.unwrap();

        let output_dir = temp_dir.path().join("out");

        // 导出
        let beancount_text = export(&source_db, "zh-CN", &output_dir).await.unwrap();

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
        let beancount_file = output_dir.join("transactions.beancount");
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
        let temp_dir = TempDir::new().unwrap();
        let source_db = setup_source_db().await;
        let tx_id = create_test_data(&source_db).await;

        // 创建附件
        let attachment = accounting::attachment::Attachment {
            id: AttachmentId(0),
            transaction_id: tx_id,
            filename: "missing.txt".to_string(),
            data: b"test".to_vec(),
        };
        source_db.attachment_create(&attachment).await.unwrap();

        let output_dir = temp_dir.path().join("out");

        // 导出
        let beancount_text = export(&source_db, "zh-CN", &output_dir).await.unwrap();

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
        let beancount_file = output_dir.join("transactions.beancount");
        std::fs::write(&beancount_file, &beancount_text).unwrap();

        // 导入应该成功但附件数为 0（因为文件不存在）
        let target_db = setup_target_db().await;
        let import_result = import(&target_db, &beancount_text, &output_dir)
            .await
            .unwrap();

        assert_eq!(import_result.attachments, 0);
    }

    /// 导出接口接收 `&SqliteDatabase`，内存库可直接导出（不再要求文件路径）
    #[tokio::test]
    async fn test_export_from_in_memory_db() {
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("out");

        let text = export(&source_db, "zh-CN", &output_dir).await.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("资产:工商银行"));
    }

    #[tokio::test]
    async fn test_import_duplicate_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let source_db = setup_source_db().await;
        let _tx_id = create_test_data(&source_db).await;

        let output_dir = temp_dir.path().join("out");

        // 导出
        let beancount_text = export(&source_db, "zh-CN", &output_dir).await.unwrap();

        // 写入 beancount 文件
        let beancount_file = output_dir.join("transactions.beancount");
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
