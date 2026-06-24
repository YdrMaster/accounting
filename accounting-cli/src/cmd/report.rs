use crate::cmd::{BalanceRow, ReportBalanceRow, StatRow};
use crate::output::{OutputFormat, print_line, print_vec};
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId};
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::database::Database;
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_i18n::t;

#[derive(Subcommand)]
pub enum ReportCmd {
    /// 查询账户余额
    Balance(ReportBalanceArgs),
    /// 资产负债表
    Bs,
    /// 损益表
    Is,
    /// 按维度统计
    Stat(ReportStatArgs),
}

#[derive(Args)]
pub struct ReportBalanceArgs {
    pub account_id: i64,
}

#[derive(Args)]
pub struct ReportStatArgs {
    /// 按标签统计
    #[arg(long, group = "dimension")]
    pub by_tag: bool,
    /// 按成员统计
    #[arg(long, group = "dimension")]
    pub by_member: bool,
    /// 按渠道统计
    #[arg(long, group = "dimension")]
    pub by_channel: bool,
    /// 起始日期
    #[arg(long)]
    pub from: Option<String>,
    /// 结束日期
    #[arg(long)]
    pub to: Option<String>,
    /// 指定账户（可多次指定）
    #[arg(long)]
    pub account: Vec<i64>,
    /// 指定成员（可多次指定）
    #[arg(long)]
    pub member: Vec<i64>,
    /// 指定标签名称（可多次指定）
    #[arg(long)]
    pub tag: Vec<String>,
    /// 指定渠道（可多次指定）
    #[arg(long)]
    pub channel: Vec<i64>,
    /// 关键词
    #[arg(long)]
    pub keyword: Option<String>,
}

impl ReportCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        match self {
            ReportCmd::Balance(args) => {
                // 查询指定账户余额并转换为表格行
                let service = accounting_service::report_service::ReportService::new(db);
                let balances = service.get_balance(AccountId(args.account_id)).await?;
                let rows: Vec<BalanceRow> = balances
                    .iter()
                    .map(|(cid, amount)| BalanceRow {
                        commodity_id: cid.0,
                        amount: amount.to_string(),
                    })
                    .collect();
                if rows.is_empty() {
                    print_line(t!("balance_zero").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::Bs => {
                // 生成资产负债表并按资产/权益分类输出
                let service = accounting_service::report_service::ReportService::new(db);
                let bs = service.balance_sheet().await?;
                let mut rows = Vec::new();
                for item in &bs.assets {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[资产] {}", item.account.name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &bs.equity {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[权益] {}", item.account.name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line(t!("no_data").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::Is => {
                // 生成损益表并按收入/费用分类输出
                let service = accounting_service::report_service::ReportService::new(db);
                let is = service.income_statement().await?;
                let mut rows = Vec::new();
                for item in &is.income {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[收入] {}", item.account.name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                for item in &is.expenses {
                    for (cid, amount) in &item.balances {
                        rows.push(ReportBalanceRow {
                            account_id: item.account.id.0,
                            account_name: format!("[费用] {}", item.account.name),
                            commodity_id: cid.0,
                            amount: amount.to_string(),
                        });
                    }
                }
                if rows.is_empty() {
                    print_line(t!("no_data").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
            ReportCmd::Stat(args) => {
                if !args.by_tag && !args.by_member && !args.by_channel {
                    return Err(AccountingError::Unknown(
                        t!("report_no_group_by").to_string(),
                    ));
                }

                let mut filter = TransactionFilter::default();
                if let Some(ref from) = args.from {
                    filter.start_date = Some(parse_date(from)?);
                }
                if let Some(ref to) = args.to {
                    filter.end_date = Some(parse_date(to)?);
                }
                filter.account_ids = args.account.iter().map(|&id| AccountId(id)).collect();
                filter.member_ids = args.member.iter().map(|&id| MemberId(id)).collect();
                for tag_name in &args.tag {
                    let conn = db.connection();
                    let tag = db
                        .tag_repo()
                        .get_by_name(&conn, tag_name)
                        .map_err(|e| AccountingError::Unknown(e.to_string()))?
                        .ok_or_else(|| {
                            AccountingError::Unknown(format!(
                                "{}",
                                t!("tag_name_not_found", name = tag_name)
                            ))
                        })?;
                    filter.tag_ids.push(tag.id);
                }
                filter.channel_ids = args.channel.iter().map(|&id| ChannelId(id)).collect();
                if let Some(ref keyword) = args.keyword {
                    filter.keyword = Some(keyword.clone());
                }

                let service = accounting_service::report_service::ReportService::new(db);
                let mut rows = Vec::new();

                if args.by_tag {
                    let stats = service.stats_by_tag(&filter).await?;
                    for stat in &stats {
                        for (cid, amount) in &stat.income {
                            rows.push(StatRow {
                                dimension_name: stat.tag.name.clone(),
                                stat_type: "收入".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                        for (cid, amount) in &stat.expense {
                            rows.push(StatRow {
                                dimension_name: stat.tag.name.clone(),
                                stat_type: "支出".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                    }
                } else if args.by_member {
                    let stats = service.stats_by_member(&filter).await?;
                    for stat in &stats {
                        for (cid, amount) in &stat.income {
                            rows.push(StatRow {
                                dimension_name: stat.member.name.clone(),
                                stat_type: "收入".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                        for (cid, amount) in &stat.expense {
                            rows.push(StatRow {
                                dimension_name: stat.member.name.clone(),
                                stat_type: "支出".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                    }
                } else if args.by_channel {
                    let stats = service.stats_by_channel(&filter).await?;
                    for stat in &stats {
                        for (cid, amount) in &stat.income {
                            rows.push(StatRow {
                                dimension_name: stat.channel.name.clone(),
                                stat_type: "收入".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                        for (cid, amount) in &stat.expense {
                            rows.push(StatRow {
                                dimension_name: stat.channel.name.clone(),
                                stat_type: "支出".to_string(),
                                commodity_id: cid.0,
                                amount: amount.to_string(),
                            });
                        }
                    }
                }

                if rows.is_empty() {
                    print_line(t!("no_data").as_ref(), format);
                } else {
                    print_vec(&rows, format);
                }
            }
        }
        Ok(())
    }
}

fn parse_date(s: &str) -> Result<chrono::NaiveDate, AccountingError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        AccountingError::InvalidDate(format!("{}", t!("invalid_date_only_format", value = s)))
    })
}
