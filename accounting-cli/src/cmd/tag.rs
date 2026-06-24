use crate::cmd::TagRow;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

#[derive(Subcommand)]
pub enum TagCmd {
    /// 列出标签
    List,
    /// 添加标签
    Add(TagAddArgs),
    /// 删除标签
    Delete(TagDeleteArgs),
}

#[derive(Args)]
pub struct TagAddArgs {
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args)]
pub struct TagDeleteArgs {
    pub name: String,
}

impl TagCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            TagCmd::List => {
                let service = accounting_service::tag_service::TagService::new(db);
                let tags = service.list().await?;
                let rows: Vec<TagRow> = tags.iter().map(|t| t.into()).collect();
                print_vec(&rows, format);
            }
            TagCmd::Add(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                let id = service.add(args.name.clone(), args.description).await?;
                print_line(&format!("{}", t!("tag_created", id = id.0)), format);
            }
            TagCmd::Delete(args) => {
                let service = accounting_service::tag_service::TagService::new(db);
                service.delete(&args.name).await?;
                print_line(&format!("{}", t!("tag_deleted", name = args.name)), format);
            }
        }
        Ok(())
    }
}
