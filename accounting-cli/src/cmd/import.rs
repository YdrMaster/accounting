use crate::cmd::resolver::resolve_member;
use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting_service::import::{AdaptError, RowErrorDetail};
use accounting_service::import_service::{ImportError, ImportService};
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
    /// 成员名称
    #[arg(long)]
    pub member: String,
}

fn format_import_error(err: ImportError) -> AccountingError {
    match err {
        ImportError::UnsupportedSource { source } => AccountingError::InvalidTransaction(format!(
            "{}",
            rust_i18n::t!("import_unsupported_source", source = source)
        )),
        ImportError::ChannelNotFound { source } => AccountingError::AccountNotFound(format!(
            "{}",
            rust_i18n::t!("import_channel_not_found", source = source)
        )),
        ImportError::CnyCommodityNotFound => AccountingError::CommodityNotFound(format!(
            "{}",
            rust_i18n::t!("import_cny_commodity_not_found")
        )),
        ImportError::Parse { source } => AccountingError::InvalidTransaction(format!(
            "{}",
            rust_i18n::t!("import_parse_failed", source = source)
        )),
        ImportError::Database { source } => AccountingError::DatabaseError(format!(
            "{}",
            rust_i18n::t!("import_database_error", source = source)
        )),
    }
}

fn format_adapt_error(err: &AdaptError) -> String {
    match err {
        AdaptError::Encoding { source } => {
            rust_i18n::t!("adapt_encoding_error", source = source).to_string()
        }
        AdaptError::Row { row, detail } => {
            let detail_str = match detail {
                RowErrorDetail::MissingColumn { index, name } => {
                    rust_i18n::t!("adapt_missing_column", index = index, name = name).to_string()
                }
                RowErrorDetail::AmountParse { value, source } => {
                    rust_i18n::t!("adapt_amount_parse_failed", value = value, source = source)
                        .to_string()
                }
                RowErrorDetail::DateParse { value } => {
                    rust_i18n::t!("adapt_date_parse_failed", value = value).to_string()
                }
                RowErrorDetail::ClosedTransaction => {
                    rust_i18n::t!("adapt_transaction_closed").to_string()
                }
                RowErrorDetail::Other { message } => message.clone(),
            };
            rust_i18n::t!("import_skipped_row", row = row, detail = detail_str).to_string()
        }
    }
}

impl ImportArgs {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        // 读取文件
        let data = std::fs::read(&self.file).map_err(|e| {
            AccountingError::DatabaseError(format!(
                "{}",
                rust_i18n::t!(
                    "import_file_read_failed",
                    file = self.file.display(),
                    source = e
                )
            ))
        })?;

        let member_id = resolve_member(&db, &self.member).await?;
        let service = ImportService::new(db);
        let result = service
            .import(&data, &self.source, member_id)
            .await
            .map_err(format_import_error)?;

        // 输出摘要
        match format {
            OutputFormat::Table => {
                println!(
                    "{}",
                    rust_i18n::t!(
                        "import_summary",
                        imported = result.imported,
                        skipped = result.skipped
                    )
                );

                if !result.errors.is_empty() {
                    println!();
                    for err in &result.errors {
                        println!("{}", format_adapt_error(err));
                    }
                }

                if !result.transaction_ids.is_empty() {
                    println!();
                    if let Some(tag) = &result.pending_tag_name {
                        println!("{}", rust_i18n::t!("import_pending_tag_hint", tag = tag));
                    }
                    let ids: Vec<String> = result
                        .transaction_ids
                        .iter()
                        .map(|id| id.0.to_string())
                        .collect();
                    println!(
                        "{}",
                        rust_i18n::t!("import_transaction_ids", ids = ids.join(", "))
                    );
                }
            }
            OutputFormat::Json => {
                let ids: Vec<i64> = result.transaction_ids.iter().map(|id| id.0).collect();
                let output = serde_json::json!({
                    "imported": result.imported,
                    "skipped": result.skipped,
                    "errors": result.errors.iter().map(format_adapt_error).collect::<Vec<_>>(),
                    "transaction_ids": ids,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
        }

        Ok(())
    }
}
