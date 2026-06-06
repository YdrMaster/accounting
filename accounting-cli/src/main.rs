//! CLI 工具：基于 clap 的命令行记账工具

rust_i18n::i18n!("locales", fallback = "en");

mod cmd;
mod output;

use accounting_sql::database::Database;
use clap::Parser;
use cmd::{Cli, Commands};
use std::process;

fn detect_system_language() -> String {
    if let Ok(lang) = std::env::var("LANG") {
        return lang.split('.').next().unwrap_or("en").to_string();
    }
    "en".to_string()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Initialize => initialize(&cli.db, cli.lang.as_deref()).await,
        _ => run_command(cli).await,
    };

    if let Err(e) = result {
        eprintln!("错误: {}", e);
        process::exit(1);
    }
}

async fn initialize(
    db_path: &std::path::Path,
    cli_lang: Option<&str>,
) -> Result<(), accounting::error::AccountingError> {
    if db_path.exists() {
        return Err(accounting::error::AccountingError::DbAlreadyExists);
    }
    let db = accounting_sql::impls::sqlite::SqliteDatabase::open(db_path.to_str().unwrap())
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
    let lang = cli_lang.map(|s| s.to_string()).unwrap_or_else(detect_system_language);
    rust_i18n::set_locale(&lang);
    db.initialize(&lang)
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
    println!("{}", rust_i18n::t!("db_initialized"));
    Ok(())
}

async fn run_command(cli: Cli) -> Result<(), accounting::error::AccountingError> {
    if !cli.db.exists() {
        return Err(accounting::error::AccountingError::DbNotInitialized);
    }
    let db = accounting_sql::impls::sqlite::SqliteDatabase::open(cli.db.to_str().unwrap())
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;

    // 从数据库读取语言配置，优先级：--lang > 数据库配置 > 系统默认
    let db_lang: Option<String> = db
        .connection()
        .query_row(
            "SELECT value FROM settings WHERE key = 'language'",
            [],
            |row| row.get(0),
        )
        .ok();

    let lang = cli
        .lang
        .clone()
        .or(db_lang)
        .unwrap_or_else(detect_system_language);
    rust_i18n::set_locale(&lang);

    match cli.command {
        Commands::Initialize => unreachable!(),
        Commands::Member(cmd) => cmd.run(db, cli.format).await,
        Commands::Account(cmd) => cmd.run(db, cli.format).await,
        Commands::Commodity(cmd) => cmd.run(db, cli.format).await,
        Commands::Tx(cmd) => cmd.run(db, cli.format).await,
        Commands::Tag(cmd) => cmd.run(db, cli.format).await,
        Commands::Report(cmd) => cmd.run(db, cli.format).await,
    }
}
