//! CLI 工具：基于 clap 的命令行记账工具

rust_i18n::i18n!("locales", fallback = "en");

mod cmd;
mod output;

use clap::Parser;
use cmd::{Cli, Commands};
use std::process;

fn detect_language(cli_lang: Option<&str>) -> String {
    if let Some(lang) = cli_lang {
        return lang.to_string();
    }
    if let Ok(lang) = std::env::var("LANG") {
        return lang.split('.').next().unwrap_or("en").to_string();
    }
    "en".to_string()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let lang = detect_language(cli.lang.as_deref());
    rust_i18n::set_locale(&lang);

    let result = match &cli.command {
        Commands::Initialize => initialize(&cli.db).await,
        _ => run_command(cli).await,
    };

    if let Err(e) = result {
        eprintln!("错误: {}", e);
        process::exit(1);
    }
}

async fn initialize(db_path: &std::path::Path) -> Result<(), accounting::error::AccountingError> {
    if db_path.exists() {
        return Err(accounting::error::AccountingError::DbAlreadyExists);
    }
    let _db = accounting_sql::impls::sqlite::SqliteDatabase::open(db_path.to_str().unwrap())
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
