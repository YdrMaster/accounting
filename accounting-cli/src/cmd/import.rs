use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting::id::MemberId;
use accounting_service::import_service::ImportService;
use accounting_sql::SqliteDatabase;
use clap::Args;
use std::path::PathBuf;

/// 导入账单
#[derive(Args)]
pub struct ImportArgs {
    /// 账单文件路径
    #[arg(long)]
    pub file: PathBuf,
    /// 账单来源（如 alipay）
    #[arg(long)]
    pub source: String,
    /// 成员 ID
    #[arg(long)]
    pub member: i64,
}

impl ImportArgs {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        // 读取文件
        let data = std::fs::read(&self.file).map_err(|e| {
            AccountingError::DatabaseError(format!("读取文件 '{}' 失败：{e}", self.file.display()))
        })?;

        let service = ImportService::new(db);
        let result = service
            .import(&data, &self.source, MemberId(self.member))
            .await?;

        // 输出摘要
        match format {
            OutputFormat::Table => {
                println!(
                    "导入完成：{} 条交易，{} 条跳过",
                    result.imported, result.skipped
                );

                if !result.errors.is_empty() {
                    println!();
                    for err in &result.errors {
                        println!("  跳过：{err}");
                    }
                }

                if !result.transaction_ids.is_empty() {
                    println!();
                    println!("已添加 \"待处理\" 标签，使用 `tx list --tag 待处理` 查看导入的交易");
                    let ids: Vec<String> = result
                        .transaction_ids
                        .iter()
                        .map(|id| id.0.to_string())
                        .collect();
                    println!("交易 ID: [{}]", ids.join(", "));
                }
            }
            OutputFormat::Json => {
                let ids: Vec<i64> = result.transaction_ids.iter().map(|id| id.0).collect();
                let output = serde_json::json!({
                    "imported": result.imported,
                    "skipped": result.skipped,
                    "errors": result.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                    "transaction_ids": ids,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
        }

        Ok(())
    }
}
