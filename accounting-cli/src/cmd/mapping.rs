use crate::cmd::resolver::{resolve_channel, resolve_member};
use crate::output::OutputFormat;
use accounting::error::AccountingError;
use accounting_service::mapping_service::MappingService;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use serde::Serialize;
use tabled::Tabled;

/// 账户映射管理
#[derive(Subcommand)]
pub enum MappingCmd {
    /// 设置映射
    Set(MappingSetArgs),
    /// 列出映射
    List(MappingListArgs),
    /// 删除映射
    Delete(MappingDeleteArgs),
}

#[derive(Args)]
pub struct MappingSetArgs {
    /// 成员名称
    #[arg(long)]
    pub member: String,
    /// 渠道名称
    #[arg(long)]
    pub channel: String,
    /// 映射 key（如 "收支:餐饮美食" 或 "资产:信用卡"）
    #[arg(long)]
    pub category: String,
    /// 目标账户路径（如 "Expenses:餐饮"）
    #[arg(long)]
    pub account: String,
}

#[derive(Args)]
pub struct MappingListArgs {
    /// 成员名称
    #[arg(long)]
    pub member: String,
    /// 渠道名称
    #[arg(long)]
    pub channel: String,
}

#[derive(Args)]
pub struct MappingDeleteArgs {
    /// 成员名称
    #[arg(long)]
    pub member: String,
    /// 渠道名称
    #[arg(long)]
    pub channel: String,
    /// 映射 key（如 "收支:餐饮美食"）
    #[arg(long)]
    pub category: String,
}

impl MappingCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        match self {
            MappingCmd::Set(args) => args.run(db, format).await,
            MappingCmd::List(args) => args.run(db, format).await,
            MappingCmd::Delete(args) => args.run(db, format).await,
        }
    }
}

impl MappingSetArgs {
    pub async fn run(
        self,
        db: SqliteDatabase,
        _format: OutputFormat,
    ) -> Result<(), AccountingError> {
        let member_id = resolve_member(&db, &self.member).await?;
        let channel_id = resolve_channel(&db, &self.channel).await?;
        let service = MappingService::new(db);

        service
            .set(member_id, channel_id, &self.category, &self.account)
            .await?;

        println!(
            "映射已设置：成员 {} 渠道 '{}' {} → {}",
            self.member, self.channel, self.category, self.account
        );
        Ok(())
    }
}

impl MappingListArgs {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        let member_id = resolve_member(&db, &self.member).await?;
        let channel_id = resolve_channel(&db, &self.channel).await?;
        let service = MappingService::new(db);
        let list = service.list(member_id, channel_id).await?;

        match format {
            OutputFormat::Table => {
                if list.is_empty() {
                    println!("无映射记录");
                } else {
                    let rows: Vec<MappingRow> = list.iter().map(MappingRow::from_mapping).collect();
                    let table = tabled::Table::new(&rows).to_string();
                    println!("{table}");
                }
            }
            OutputFormat::Json => {
                let rows: Vec<MappingRow> = list.iter().map(MappingRow::from_mapping).collect();
                println!("{}", serde_json::to_string_pretty(&rows).unwrap());
            }
        }

        Ok(())
    }
}

impl MappingDeleteArgs {
    pub async fn run(
        self,
        db: SqliteDatabase,
        _format: OutputFormat,
    ) -> Result<(), AccountingError> {
        let member_id = resolve_member(&db, &self.member).await?;
        let channel_id = resolve_channel(&db, &self.channel).await?;
        let service = MappingService::new(db);

        service
            .delete(member_id, channel_id, &self.category)
            .await?;

        println!(
            "映射已删除：成员 {} 渠道 '{}' {}",
            self.member, self.channel, self.category
        );
        Ok(())
    }
}

/// 映射表格行
#[derive(Tabled, Serialize)]
pub struct MappingRow {
    pub member_id: i64,
    pub channel_id: i64,
    pub category: String,
    pub account_id: i64,
}

impl MappingRow {
    fn from_mapping(m: &accounting::account_mapping::AccountMapping) -> Self {
        Self {
            member_id: m.member_id.0,
            channel_id: m.channel_id.0,
            category: m.category.clone(),
            account_id: m.account_id.0,
        }
    }
}
