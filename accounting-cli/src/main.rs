mod cmd;
mod output;

use clap::Parser;
use cmd::{Cli, Commands};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Initialize => initialize(&cli.db).await,
        _ => run_command(cli).await,
    };

    if let Err(e) = result {
        eprintln!("错误: {:?}", e);
        process::exit(1);
    }
}

async fn initialize(db_path: &std::path::Path) -> Result<(), accounting::error::AccountingError> {
    if db_path.exists() {
        return Err(accounting::error::AccountingError::Unknown(
            "数据库文件已存在".to_string(),
        ));
    }
    let _db = accounting_sql::impls::sqlite::SqliteDatabase::open(db_path.to_str().unwrap())
        .map_err(|e| accounting::error::AccountingError::Unknown(e.to_string()))?;
    println!("数据库已初始化");
    Ok(())
}

async fn run_command(cli: Cli) -> Result<(), accounting::error::AccountingError> {
    if !cli.db.exists() {
        return Err(accounting::error::AccountingError::Unknown(
            "数据库文件不存在，请先运行 initialize 命令".to_string(),
        ));
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
