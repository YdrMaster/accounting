//! CLI 工具：基于 clap 的命令行记账工具

rust_i18n::i18n!("locales", fallback = "en");

mod cmd;
mod output;

use clap::Parser;
use cmd::{Cli, Commands};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Initialize => initialize(&cli.db, cli.lang.as_deref()).await,
        _ => run_command(cli).await,
    };

    if let Err(e) = result {
        eprintln!("{}", rust_i18n::t!("error_prefix", msg = e));
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
    let db = accounting_sql::SqliteDatabase::open(db_path.to_str().unwrap())
        .await
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
    let lang = db
        .initialize(cli_lang)
        .await
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
    rust_i18n::set_locale(&lang);
    println!("{}", rust_i18n::t!("db_initialized"));
    Ok(())
}

async fn run_command(cli: Cli) -> Result<(), accounting::error::AccountingError> {
    if !cli.db.exists() {
        return Err(accounting::error::AccountingError::DbNotInitialized);
    }
    let db = accounting_sql::SqliteDatabase::open(cli.db.to_str().unwrap())
        .await
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;

    // 从数据库读取语言配置，优先级：--lang > 数据库配置
    let db_lang = db
        .get_setting("language")
        .await
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?
        .ok_or(accounting::error::AccountingError::DbNotInitialized)?;

    let lang = cli.lang.clone().unwrap_or(db_lang);
    rust_i18n::set_locale(&lang);

    match cli.command {
        Commands::Initialize => unreachable!(),
        Commands::Config(cmd) => cmd.run(db).await,
        Commands::Member(cmd) => cmd.run(db, cli.format).await,
        Commands::Account(cmd) => cmd.run(db, cli.format).await,
        Commands::Commodity(cmd) => cmd.run(db, cli.format).await,
        Commands::Tx(cmd) => cmd.run(db, cli.format).await,
        Commands::Tag(cmd) => cmd.run(db, cli.format).await,
        Commands::Report(cmd) => cmd.run(db, cli.format).await,
        Commands::Import(cmd) => cmd.run(db, cli.format).await,
        Commands::Mapping(cmd) => cmd.run(db, cli.format).await,
        Commands::Budget(cmd) => cmd.run(&db, &cli.format).await,
        Commands::Beancount(cmd) => cmd.run(db, cli.format).await,
    }
}
