use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum BeancountCmd {
    /// 导出账目为 beancount 格式
    Export(BeancountExportArgs),
    /// 从 beancount 文件导入账目
    Import(BeancountImportArgs),
}

#[derive(Args)]
pub struct BeancountExportArgs {
    /// 输出目录
    pub output_dir: PathBuf,
}

#[derive(Args)]
pub struct BeancountImportArgs {
    /// 输入文件路径
    pub input_file: PathBuf,
}

impl BeancountCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        _format: OutputFormat,
        lang: &str,
    ) -> Result<(), AccountingError> {
        match self {
            BeancountCmd::Export(args) => run_export(db, args, lang).await,
            BeancountCmd::Import(args) => run_import(db, args).await,
        }
    }
}

async fn run_export(
    db: SqliteDatabase,
    args: BeancountExportArgs,
    lang: &str,
) -> Result<(), AccountingError> {
    std::fs::create_dir_all(&args.output_dir).map_err(|e| {
        AccountingError::Unknown(format!("{}", t!("beancount_create_dir_failed", error = e)))
    })?;

    let text = accounting_beancount::export::export(&db, lang, &args.output_dir)
        .await
        .map_err(|e| AccountingError::Unknown(e.to_string()))?;

    let output_file = args.output_dir.join("transactions.beancount");
    std::fs::write(&output_file, &text).map_err(|e| {
        AccountingError::Unknown(format!("{}", t!("beancount_write_file_failed", error = e)))
    })?;

    println!(
        "{}",
        rust_i18n::t!("beancount_exported", file = output_file.display())
    );
    Ok(())
}

async fn run_import(db: SqliteDatabase, args: BeancountImportArgs) -> Result<(), AccountingError> {
    let input = std::fs::read_to_string(&args.input_file).map_err(|e| {
        AccountingError::Unknown(format!(
            "{}",
            t!(
                "beancount_read_file_failed",
                file = args.input_file.display(),
                error = e
            )
        ))
    })?;

    let base_dir = args
        .input_file
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let result = accounting_beancount::import::import(&db, &input, base_dir)
        .await
        .map_err(|e| AccountingError::Unknown(e.to_string()))?;

    println!(
        "{}",
        t!(
            "beancount_import_summary",
            transactions = result.transactions,
            skipped = result.skipped,
            accounts = result.accounts,
            commodities = result.commodities,
            members = result.members,
            channels = result.channels,
            attachments = result.attachments
        )
    );
    Ok(())
}
