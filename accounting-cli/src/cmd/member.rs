use crate::cmd::MemberRow;
use crate::output::{OutputFormat, print_line, print_vec};
use accounting::id::MemberId;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

#[derive(Subcommand)]
pub enum MemberCmd {
    /// 列出成员
    List(MemberListArgs),
    /// 添加成员
    Add(MemberAddArgs),
    /// 删除成员
    Delete(MemberDeleteArgs),
}

#[derive(Args)]
pub struct MemberListArgs {
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
}

#[derive(Args)]
pub struct MemberAddArgs {
    pub name: String,
}

#[derive(Args)]
pub struct MemberDeleteArgs {
    pub id: i64,
}

impl MemberCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), accounting::error::AccountingError> {
        match self {
            MemberCmd::List(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                let members = service.list(args.limit, args.offset).await?;
                let rows: Vec<MemberRow> = members.iter().map(|m| m.into()).collect();
                print_vec(&rows, format);
            }
            MemberCmd::Add(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                let id = service.add(args.name).await?;
                print_line(&format!("{}", t!("member_created", id = id.0)), format);
            }
            MemberCmd::Delete(args) => {
                let service = accounting_service::member_service::MemberService::new(db);
                service.delete(MemberId(args.id)).await?;
                print_line(&format!("{}", t!("member_deleted", id = args.id)), format);
            }
        }
        Ok(())
    }
}
