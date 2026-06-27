use crate::cmd::{PostingRow, TransactionRow};
use crate::output::{OutputFormat, print as output_print, print_line, print_vec};
use accounting::channel_path::ChannelPathNode;
use accounting::error::AccountingError;
use accounting::id::{AccountId, ChannelId, MemberId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::SqliteDatabase;
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
    /// 对账标记
    Reconcile(TxReconcileArgs),
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
    /// 渠道 ID，可多次指定（有序，从 position 0 开始）；同位置多渠道用 "pos:channelId" 语法
    #[arg(long)]
    pub channel: Vec<String>,
}

#[derive(Args)]
pub struct TxListArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub account: Vec<i64>,
    #[arg(long)]
    pub member: Vec<i64>,
    #[arg(long)]
    pub tag: Vec<String>,
    #[arg(long)]
    pub channel: Vec<i64>,
    #[arg(long)]
    pub keyword: Option<String>,
    #[arg(long)]
    pub limit: Option<i64>,
    #[arg(long)]
    pub offset: Option<i64>,
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
    /// 渠道 ID，可多次指定（有序，从 position 0 开始）；同位置多渠道用 "pos:channelId" 语法
    #[arg(long)]
    pub channel: Vec<String>,
}

#[derive(Args)]
pub struct TxReconcileArgs {
    /// channel_path 记录 ID
    pub path_id: i64,
    /// 标记为已对账或未对账
    #[arg(long)]
    pub set: Option<bool>,
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
                let (tx, postings, tag_ids, channel_path_nodes) = parse_tx_args(args, &db).await?;
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let id = service
                    .submit(tx, postings, tag_ids, channel_path_nodes)
                    .await?;
                print_line(&format!("{}", t!("tx_created", id = id.0)), format);
            }
            TxCmd::List(args) => {
                // 构建过滤条件并查询交易列表
                let limit = args.limit;
                let offset = args.offset;
                let filter = build_filter(&args, &db).await?;
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let results = service.list(filter, limit, offset).await?;
                let rows: Vec<TransactionRow> = results
                    .iter()
                    .map(|(t, _, channel_paths)| {
                        let mut row: TransactionRow = t.into();
                        row.channel_paths = channel_paths
                            .iter()
                            .map(|n| ChannelPathRow {
                                position: n.position,
                                channel_id: n.channel_id.0,
                                reconciled: n.reconciled,
                            })
                            .collect();
                        row
                    })
                    .collect();
                print_vec(&rows, format);
            }
            TxCmd::Show(args) => {
                // 查询单笔交易详情并打印交易与分录及链路
                let service = accounting_service::transaction_service::TransactionService::new(db);
                let result = service.get(TransactionId(args.id)).await?;
                match result {
                    Some((tx, postings, channel_paths)) => {
                        let mut tx_row: TransactionRow = (&tx).into();
                        tx_row.channel_paths = channel_paths
                            .iter()
                            .map(|n| ChannelPathRow {
                                position: n.position,
                                channel_id: n.channel_id.0,
                                reconciled: n.reconciled,
                            })
                            .collect();
                        output_print(&tx_row, format);
                        if !channel_paths.is_empty() {
                            print_vec(
                                &channel_paths
                                    .iter()
                                    .map(|n| ChannelPathRow {
                                        position: n.position,
                                        channel_id: n.channel_id.0,
                                        reconciled: n.reconciled,
                                    })
                                    .collect::<Vec<_>>(),
                                format,
                            );
                        }
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
                let (mut tx, postings, tag_ids, channel_path_nodes) =
                    parse_tx_args_for_update(&args, &db).await?;
                tx.id = TransactionId(id);
                let service = accounting_service::transaction_service::TransactionService::new(db);
                service
                    .update(tx, postings, tag_ids, channel_path_nodes)
                    .await?;
                print_line(&format!("{}", t!("tx_updated", id = id)), format);
            }
            TxCmd::Reconcile(args) => {
                // 对账标记
                let reconciled = args.set.unwrap_or(true);
                let service = accounting_service::transaction_service::TransactionService::new(db);
                service
                    .update_reconciled(accounting::id::ChannelPathId(args.path_id), reconciled)
                    .await?;
                let msg = if reconciled {
                    t!("reconciled_marked", id = args.path_id)
                } else {
                    t!("reconciled_unmarked", id = args.path_id)
                };
                print_line(msg.as_ref(), format);
            }
        }
        Ok(())
    }
}

/// 链路节点行（CLI 显示用）
#[derive(tabled::Tabled, serde::Serialize)]
pub struct ChannelPathRow {
    pub position: i32,
    pub channel_id: i64,
    pub reconciled: bool,
}

async fn parse_tx_args(
    args: TxAddArgs,
    db: &SqliteDatabase,
) -> Result<(Transaction, Vec<Posting>, Vec<TagId>, Vec<ChannelPathNode>), AccountingError> {
    let date_time = parse_date_time(&args.date)?;
    let postings = parse_postings(&args.posting, db).await?;
    let tag_ids = resolve_tags(&args.tag, db).await?;
    let channel_path_nodes = parse_channel_paths(&args.channel)?;
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description,
        kind: accounting::transaction::TransactionKind::Normal,
        member_id: args.member.map(MemberId),
    };
    Ok((tx, postings, tag_ids, channel_path_nodes))
}

async fn parse_tx_args_for_update(
    args: &TxUpdateArgs,
    db: &SqliteDatabase,
) -> Result<(Transaction, Vec<Posting>, Vec<TagId>, Vec<ChannelPathNode>), AccountingError> {
    let date_time = parse_date_time(&args.date)?;
    let postings = parse_postings(&args.posting, db).await?;
    let tag_ids = resolve_tags(&args.tag, db).await?;
    let channel_path_nodes = parse_channel_paths(&args.channel)?;
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description.clone(),
        kind: accounting::transaction::TransactionKind::Normal,
        member_id: args.member.map(MemberId),
    };
    Ok((tx, postings, tag_ids, channel_path_nodes))
}

/// 解析渠道参数列表。
///
/// 支持两种语法:
/// - 直接传渠道 ID: `--channel 1 --channel 2` → position 依次为 0, 1
/// - 指定位置: `--channel "0:1" --channel "1:2" --channel "2:3"` → position 显式指定
fn parse_channel_paths(channels: &[String]) -> Result<Vec<ChannelPathNode>, AccountingError> {
    if channels.is_empty() {
        return Ok(Vec::new());
    }

    // 检测是否使用了 "pos:channelId" 语法
    let uses_explicit_position = channels.iter().any(|c| c.contains(':'));

    if uses_explicit_position {
        let mut nodes = Vec::new();
        for ch in channels {
            let parts: Vec<&str> = ch.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(AccountingError::InvalidTransaction(format!(
                    "Invalid channel path format: {}, expected 'position:channelId'",
                    ch
                )));
            }
            let position: i32 = parts[0].parse().map_err(|_| {
                AccountingError::InvalidTransaction(format!(
                    "Invalid position in channel path: {}",
                    parts[0]
                ))
            })?;
            let channel_id: i64 = parts[1].parse().map_err(|_| {
                AccountingError::InvalidTransaction(format!(
                    "Invalid channelId in channel path: {}",
                    parts[1]
                ))
            })?;
            nodes.push(ChannelPathNode {
                position,
                channel_id: ChannelId(channel_id),
                reconciled: false,
            });
        }
        Ok(nodes)
    } else {
        // 顺序分配 position
        Ok(channels
            .iter()
            .enumerate()
            .map(|(i, ch)| {
                let channel_id: i64 = ch
                    .parse()
                    .unwrap_or_else(|_| panic!("Invalid channelId: {}", ch));
                ChannelPathNode {
                    position: i as i32,
                    channel_id: ChannelId(channel_id),
                    reconciled: false,
                }
            })
            .collect())
    }
}

fn parse_date_time(s: &str) -> Result<chrono::NaiveDateTime, AccountingError> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .map_err(|_| {
            AccountingError::InvalidDate(format!("{}", t!("invalid_date_format", value = s)))
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
                "{}",
                t!("invalid_posting_format", value = s)
            )));
        }

        // 解析策略：最后一项必须是金额；如果倒数第三项也是金额，则按 5 段解析（支持账户名含冒号）
        let last_is_decimal = Decimal::from_str(parts.last().unwrap()).is_ok();
        if !last_is_decimal {
            return Err(AccountingError::InvalidTransaction(format!(
                "{}",
                t!("invalid_amount_format", value = parts.last().unwrap())
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
                "{}",
                t!("account_name_empty", value = s)
            )));
        }

        // 查询数据库验证账户与商品是否存在
        let account = db
            .account_get_by_name(&account_name)
            .await
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let account_id = account
            .ok_or_else(|| {
                AccountingError::AccountNotFound(format!(
                    "{}",
                    t!("account_name_not_found", name = account_name)
                ))
            })?
            .id;

        let commodity = db
            .commodity_get_by_symbol(commodity_symbol)
            .await
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let commodity_id = commodity
            .ok_or_else(|| {
                AccountingError::CommodityNotFound(format!(
                    "{}",
                    t!("commodity_symbol_not_found", symbol = commodity_symbol)
                ))
            })?
            .id;

        // 若存在 cost，则进一步查询 cost_commodity 并构造 Posting
        let (cost, cost_commodity_id) = if let Some(cost_commodity_symbol) = cost_commodity {
            let cost_commodity = db
                .commodity_get_by_symbol(cost_commodity_symbol)
                .await
                .map_err(|e| AccountingError::Unknown(e.to_string()))?;
            let cost_commodity_id = cost_commodity
                .ok_or_else(|| {
                    AccountingError::CommodityNotFound(format!(
                        "{}",
                        t!("cost_commodity_not_found", symbol = cost_commodity_symbol)
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
            is_reimbursable: false,
            linked_posting_id: None,
            reversal_total: Decimal::ZERO,
        });
    }
    Ok(postings)
}

async fn resolve_tags(
    tag_names: &[String],
    db: &SqliteDatabase,
) -> Result<Vec<TagId>, AccountingError> {
    let mut tag_ids = Vec::new();
    for name in tag_names {
        let tag = db
            .tag_get_by_name(name)
            .await
            .map_err(|e| AccountingError::Unknown(e.to_string()))?;
        let tag_id = tag
            .ok_or_else(|| {
                AccountingError::Unknown(format!("{}", t!("tag_name_not_found", name = name)))
            })?
            .id;
        tag_ids.push(tag_id);
    }
    Ok(tag_ids)
}

async fn build_filter(
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
    filter.account_ids = args.account.iter().map(|&id| AccountId(id)).collect();
    filter.member_ids = args.member.iter().map(|&id| MemberId(id)).collect();
    filter.channel_ids = args.channel.iter().map(|&id| ChannelId(id)).collect();
    for tag_name in &args.tag {
        let tag = db
            .tag_get_by_name(tag_name)
            .await
            .map_err(|e| AccountingError::Unknown(e.to_string()))?
            .ok_or_else(|| {
                AccountingError::Unknown(format!("{}", t!("tag_name_not_found", name = tag_name)))
            })?;
        filter.tag_ids.push(tag.id);
    }
    if let Some(ref keyword) = args.keyword {
        filter.keyword = Some(keyword.clone());
    }
    Ok(filter)
}
