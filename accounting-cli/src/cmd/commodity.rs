use crate::cmd::CommodityRow;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

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
        lang: &str,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            CommodityCmd::List => {
                let service =
                    accounting_service::commodity_service::CommodityService::new(db.clone());
                let commodities = service.list().await?;
                let ids: Vec<accounting::id::CommodityId> =
                    commodities.iter().map(|c| c.id).collect();
                let names = db.commodity_display_names(&ids, lang).await.map_err(|e| {
                    accounting::error::AccountingError::DatabaseError(e.to_string())
                })?;
                let rows: Vec<CommodityRow> = commodities
                    .iter()
                    .map(|c| CommodityRow::new(c, names.get(&c.id).cloned().unwrap_or_default()))
                    .collect();
                print_vec(&rows, format);
            }
            CommodityCmd::Add(args) => {
                let service = accounting_service::commodity_service::CommodityService::new(db);
                let id = service
                    .add(args.symbol, args.name, args.precision, lang)
                    .await?;
                print_line(&format!("{}", t!("commodity_created", id = id.0)), format);
            }
        }
        Ok(())
    }
}
