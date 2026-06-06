use crate::cmd::CommodityRow;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum CommodityCmd {
    /// 列出商品
    List,
    /// 添加商品
    Add(CommodityAddArgs),
}

#[derive(Args)]
pub struct CommodityAddArgs {
    pub symbol: String,
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "2")]
    pub precision: u8,
}

impl CommodityCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            CommodityCmd::List => {
                let service = accounting_service::commodity_service::CommodityService::new(db);
                let commodities = service.list().await?;
                let rows: Vec<CommodityRow> = commodities.iter().map(|c| c.into()).collect();
                print_vec(&rows, format);
            }
            CommodityCmd::Add(args) => {
                let service = accounting_service::commodity_service::CommodityService::new(db);
                let id = service.add(args.symbol, args.name, args.precision).await?;
                print_line(&format!("商品已创建，ID: {}", id.0), format);
            }
        }
        Ok(())
    }
}
