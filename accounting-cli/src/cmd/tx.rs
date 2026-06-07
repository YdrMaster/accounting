use crate::cmd::{PostingRow, TransactionRow};
use crate::output::{OutputFormat, print as output_print, print_line, print_vec};
use accounting::error::AccountingError;
use accounting::id::{AccountId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::database::Database;
use accounting_sql::impls::sqlite::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use rust_i18n::t;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum TxCmd {
    /// 添加交易
    Add(TxAddArgs),
    /// 列出交易
    List(TxListArgs),
    /// 查看交易
    Show(TxShowArgs),
    /// 删除交易
    Delete(TxDeleteArgs),
    /// 更新交易
    Update(TxUpdateArgs),
}

#[derive(Args)]
pub struct TxAddArgs {
    #[arg(long)]
    pub date: String,
    #[arg(long)]
    pub description: String,
    #[arg(long, value_delimiter = ';')]
    pub posting: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub tag: Vec<String>,
    #[arg(long)]
    pub member: Option<i64>,
    #[arg(long)]
    pub channel: Option<i64>,
}

#[derive(Args)]
pub struct TxListArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub account: Option<i64>,
    #[arg(long)]
    pub member: Option<i64>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long)]
    pub keyword: Option<String>,
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
    /// 是否只显示模板交易
    #[arg(long)]
    pub template: bool,
    /// 是否只显示分期交易
    #[arg(long)]
    pub installment: bool,
}

#[derive(Args)]
pub struct TxShowArgs {
    pub id: i64,
}

#[derive(Args)]
pub struct TxDeleteArgs {
    pub id: i64,
}

#[derive(Args)]
pub struct TxUpdateArgs {
    pub id: i64,
    #[arg(long)]
    pub date: String,
    #[arg(long)]
    pub description: String,
    #[arg(long, value_delimiter = ';')]
    pub posting: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub tag: Vec<String>,
    #[arg(long)]
    pub member: Option<i64>,
    #[arg(long)]
    pub channel: Option<i64>,
}

impl TxCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
    ) -> Result<(), AccountingError> {
        match self {
            TxCmd::Add(args) => {
                // 解析参数并提交新交易
                let (tx, postings, tag_ids) = parse_tx_args(args, &db).await?;
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let id = service.submit(tx, postings, tag_ids).await?;
                print_line(&format!("{}", t!("tx_created", id = id.0)), format);
            }
            TxCmd::List(args) => {
                // 构建过滤条件并查询交易列表
                let limit = args.limit;
                let offset = args.offset;
                let filter = build_filter(&args, &db)?;
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let results = service.list(filter, limit, offset).await?;
                let rows: Vec<TransactionRow> = results.iter().map(|(t, _)| t.into()).collect();
                print_vec(&rows, format);
            }
            TxCmd::Show(args) => {
                // 查询单笔交易详情并打印交易与分录
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let result = service.get(TransactionId(args.id)).await?;
                match result {
                    Some((tx, postings)) => {
                        let tx_row: TransactionRow = (&tx).into();
                        output_print(&tx_row, format);
                        if !postings.is_empty() {
                            let posting_rows: Vec<PostingRow> =
                                postings.iter().map(|p| p.into()).collect();
                            print_vec(&posting_rows, format);
                        }
                    }
                    None => print_line(&format!("{}", t!("tx_not_found", id = args.id)), format),
                }
            }
            TxCmd::Delete(args) => {
                // 删除指定交易
                let service = accounting_service::transaction_service::TransactionService::new(db);
                service.delete(TransactionId(args.id)).await?;
                print_line(&format!("{}", t!("tx_deleted", id = args.id)), format);
            }
            TxCmd::Update(args) => {
                // 解析更新参数并执行全量替换
                let id = args.id;
                let (mut tx, postings, tag_ids) = parse_tx_args_for_update(&args, &db).await?;
                tx.id = TransactionId(id);
                let service = accounting_service::transaction_service::TransactionService::new(db);
                service.update(tx, postings, tag_ids).await?;
                print_line(&format!("{}", t!("tx_updated", id = id)), format);
            }
        }
        Ok(())
    }
}

async fn parse_tx_args(
    args: TxAddArgs,
    db: &SqliteDatabase,
) -> Result<(Transaction, Vec<Posting>, Vec<TagId>), AccountingError> {
    let date_time = parse_date_time(&args.date)?;
    let postings = parse_postings(&args.posting, db).await?;
    let tag_ids = resolve_tags(&args.tag, db).await?;
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description,
        member_id: args.member.map(MemberId),
        channel_id: None,
        is_template: false,
    };
    Ok((tx, postings, tag_ids))
}

async fn parse_tx_args_for_update(
    args: &TxUpdateArgs,
    db: &SqliteDatabase,
) -> Result<(Transaction, Vec<Posting>, Vec<TagId>), AccountingError> {
    let date_time = parse_date_time(&args.date)?;
    let postings = parse_postings(&args.posting, db).await?;
    let tag_ids = resolve_tags(&args.tag, db).await?;
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description.clone(),
        member_id: args.member.map(MemberId),
        channel_id: None,
        is_template: false,
    };
    Ok((tx, postings, tag_ids))
}

fn parse_date_time(s: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .map_err(|_| {
            AccountingError::InvalidDate(format!(
                "时间格式应为 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS: {}",
                s
            ))
        })
}

async fn parse_postings(
    posting_strs: &[String],
    db: &SqliteDatabase,
) -> Result<Vec<Posting>, AccountingError> {
    let mut postings = Vec::new();
    for s in posting_strs {
        // 按冒号拆分，至少要有 account:commodity:amount 三段
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 3 {
            return Err(AccountingError::InvalidTransaction(format!(
                "分录格式错误: {} (应为 account:commodity:amount 或 account:commodity:amount:cost_commodity:cost)",
                s
            )));
        }

        // 解析策略：最后一项必须是金额；如果倒数第三项也是金额，则按 5 段解析（支持账户名含冒号）
        let last_is_decimal = Decimal::from_str(parts.last().unwrap()).is_ok();
        if !last_is_decimal {
            return Err(AccountingError::InvalidTransaction(format!(
                "金额格式错误: {}",
                parts.last().unwrap()
            )));
        }

        let could_be_five_part =
            parts.len() >= 5 && Decimal::from_str(parts[parts.len() - 3]).is_ok();

        // 根据段数提取账户名、商品符号、金额及可选的 cost 信息
        let (account_name, commodity_symbol, amount, cost_commodity, cost_amount) =
            if could_be_five_part {
                let account_name = parts[..parts.len() - 4].join(":");
                let commodity_symbol = parts[parts.len() - 4];
                let amount = Decimal::from_str(parts[parts.len() - 3]).unwrap();
                let cost_commodity = parts[parts.len() - 2];
                let cost_amount = Decimal::from_str(parts[parts.len() - 1]).unwrap();
                (
                    account_name,
                    commodity_symbol,
                    amount,
                    Some(cost_commodity),
                    Some(cost_amount),
                )
            } else {
                let account_name = parts[..parts.len() - 2].join(":");
                let commodity_symbol = parts[parts.len() - 2];
                let amount = Decimal::from_str(parts[parts.len() - 1]).unwrap();
                (account_name, commodity_symbol, amount, None, None)
            };

        if account_name.is_empty() {
            return Err(AccountingError::InvalidTransaction(format!(
                "账户名不能为空: {}",
                s
            )));
        }

        // 查询数据库验证账户与商品是否存在
        let conn = db.connection();
        let account = db
            .account_repo()
            .get_by_name(&conn, &account_name)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let account_id = account
            .ok_or_else(|| {
                AccountingError::AccountNotFound(format!("账户不存在: {}", account_name))
            })?
            .id;

        let commodity = db
            .commodity_repo()
            .get_by_symbol(&conn, commodity_symbol)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let commodity_id = commodity
            .ok_or_else(|| {
                AccountingError::CommodityNotFound(format!("商品不存在: {}", commodity_symbol))
            })?
            .id;

        // 若存在 cost，则进一步查询 cost_commodity 并构造 Posting
        let (cost, cost_commodity_id) = if let Some(cost_commodity_symbol) = cost_commodity {
            let cost_commodity = db
                .commodity_repo()
                .get_by_symbol(&conn, cost_commodity_symbol)
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
            let cost_commodity_id = cost_commodity
                .ok_or_else(|| {
                    AccountingError::CommodityNotFound(format!(
                        "成本商品不存在: {}",
                        cost_commodity_symbol
                    ))
                })?
                .id;
            (cost_amount, Some(cost_commodity_id))
        } else {
            (None, None)
        };

        postings.push(Posting {
            id: PostingId(0),
            transaction_id: TransactionId(0),
            account_id,
            commodity_id,
            amount,
            cost,
            cost_commodity_id,
            description: None,
            member_id: None,
            channel_id: None,
        });
    }
    Ok(postings)
}

async fn resolve_tags(
    tag_names: &[String],
    db: &SqliteDatabase,
) -> Result<Vec<TagId>, AccountingError> {
    let mut tag_ids = Vec::new();
    let conn = db.connection();
    for name in tag_names {
        let tag = db
            .tag_repo()
            .get_by_name(&conn, name)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let tag_id = tag
            .ok_or_else(|| AccountingError::Unknown(format!("标签不存在: {}", name)))?
            .id;
        tag_ids.push(tag_id);
    }
    Ok(tag_ids)
}

fn build_filter(
    args: &TxListArgs,
    db: &SqliteDatabase,
) -> Result<TransactionFilter, AccountingError> {
    let mut filter = TransactionFilter::default();
    if let Some(ref from) = args.from {
        filter.start_date = Some(parse_date_time(from)?.date());
    }
    if let Some(ref to) = args.to {
        filter.end_date = Some(parse_date_time(to)?.date());
    }
    if let Some(account) = args.account {
        filter.account_id = Some(AccountId(account));
    }
    if let Some(member) = args.member {
        filter.member_id = Some(MemberId(member));
    }
    if let Some(ref tag_name) = args.tag {
        let conn = db.connection();
        let tag = db
            .tag_repo()
            .get_by_name(&conn, tag_name)
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        if let Some(tag) = tag {
            filter.tag_id = Some(tag.id);
        } else {
            return Err(AccountingError::Unknown(format!(
                "标签不存在: {}",
                tag_name
            )));
        }
    }
    if let Some(ref keyword) = args.keyword {
        filter.keyword = Some(keyword.clone());
    }
    filter.is_template = if args.template { Some(true) } else { None };
    filter.has_installment = if args.installment { Some(true) } else { None };
    Ok(filter)
}
