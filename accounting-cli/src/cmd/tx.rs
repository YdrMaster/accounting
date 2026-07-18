use crate::cmd::resolver::{resolve_account, resolve_channel, resolve_member};
use crate::cmd::{PostingRow, TransactionRow};
use crate::output::{OutputFormat, print as output_print, print_line, print_vec};
use accounting::channel_path::{ChannelPathNode, ChannelPathStatus};
use accounting::error::AccountingError;
use accounting::id::{ChannelId, PostingId, TagId, TransactionId};
use accounting::posting::Posting;
use accounting::transaction::Transaction;
use accounting::transaction_filter::TransactionFilter;
use accounting_sql::SqliteDatabase;
use clap::{Args, Subcommand};
use rust_decimal::Decimal;
use rust_i18n::t;
use std::collections::HashMap;
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
    pub member: String,
    /// 渠道链路，如 "淘宝 -> 支付宝* -> 花呗 & 建行卡√"
    #[arg(long)]
    pub channel: Option<String>,
}

#[derive(Args)]
pub struct TxListArgs {
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub account: Vec<String>,
    #[arg(long)]
    pub member: Vec<String>,
    #[arg(long)]
    pub tag: Vec<String>,
    #[arg(long)]
    pub channel: Vec<String>,
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
    pub member: Option<String>,
    /// 渠道链路，如 "淘宝 -> 支付宝* -> 花呗 & 建行卡√"
    #[arg(long)]
    pub channel: Option<String>,
}

#[derive(Args)]
pub struct TxReconcileArgs {
    /// 交易 ID
    pub tx_id: i64,
    /// 要标记的渠道名称（支持 * / √ 后缀与别名）
    #[arg(long)]
    pub channel: String,
    /// 取消已校验标记，回到 default
    #[arg(long)]
    pub unset: bool,
}

impl TxCmd {
    pub async fn run(
        self,
        db: SqliteDatabase,
        format: OutputFormat,
        lang: &str,
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
                let service =
                    accounting_service::transaction_service::TransactionService::new(db.clone());
                let results = service.list(filter, limit, offset).await?;
                let channel_names = channel_name_map(&db, lang).await?;
                let rows: Vec<TransactionRow> = results
                    .iter()
                    .map(|(t, _, channel_paths)| {
                        let mut row: TransactionRow = t.into();
                        row.channel_paths = channel_paths
                            .iter()
                            .map(|n| ChannelPathRow {
                                position: n.position,
                                channel: channel_names
                                    .get(&n.channel_id)
                                    .cloned()
                                    .unwrap_or_else(|| n.channel_id.0.to_string()),
                                status: n.status.as_str().to_string(),
                            })
                            .collect();
                        row
                    })
                    .collect();
                print_vec(&rows, format);
            }
            TxCmd::Show(args) => {
                // 查询单笔交易详情并打印交易与分录及链路
                let service =
                    accounting_service::transaction_service::TransactionService::new(db.clone());
                let result = service.get(TransactionId(args.id)).await?;
                match result {
                    Some((tx, postings, channel_paths)) => {
                        let channel_names = channel_name_map(&db, lang).await?;
                        let mut tx_row: TransactionRow = (&tx).into();
                        tx_row.channel_paths = channel_paths
                            .iter()
                            .map(|n| ChannelPathRow {
                                position: n.position,
                                channel: channel_names
                                    .get(&n.channel_id)
                                    .cloned()
                                    .unwrap_or_else(|| n.channel_id.0.to_string()),
                                status: n.status.as_str().to_string(),
                            })
                            .collect();
                        output_print(&tx_row, format);
                        if !channel_paths.is_empty() {
                            print_vec(
                                &channel_paths
                                    .iter()
                                    .map(|n| ChannelPathRow {
                                        position: n.position,
                                        channel: channel_names
                                            .get(&n.channel_id)
                                            .cloned()
                                            .unwrap_or_else(|| n.channel_id.0.to_string()),
                                        status: n.status.as_str().to_string(),
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
                // 对账标记：默认设为 verified，--unset 时回到 default
                let status = if args.unset {
                    ChannelPathStatus::Default
                } else {
                    ChannelPathStatus::Verified
                };
                let (channel_name, _) = parse_channel_status(&args.channel);
                let channel_id = resolve_channel(&db, channel_name).await?;
                let tx_id = TransactionId(args.tx_id);

                // 确认交易存在
                let service =
                    accounting_service::transaction_service::TransactionService::new(db.clone());
                if service.get(tx_id).await?.is_none() {
                    return Err(AccountingError::InvalidTransaction(format!(
                        "{}",
                        t!("tx_not_found", id = args.tx_id)
                    )));
                }

                let updated = service
                    .update_channel_status_by_tx_and_channel(tx_id, channel_id, status)
                    .await?;
                if updated == 0 {
                    print_line(
                        &format!(
                            "{}",
                            t!(
                                "reconcile_no_channel",
                                tx_id = args.tx_id,
                                channel = channel_name
                            )
                        ),
                        format,
                    );
                } else {
                    let msg = if args.unset {
                        t!(
                            "reconcile_unset",
                            tx_id = args.tx_id,
                            channel = channel_name
                        )
                    } else {
                        t!(
                            "reconcile_marked",
                            tx_id = args.tx_id,
                            channel = channel_name
                        )
                    };
                    print_line(msg.as_ref(), format);
                }
            }
        }
        Ok(())
    }
}

/// 链路节点行（CLI 显示用）
#[derive(tabled::Tabled, serde::Serialize)]
pub struct ChannelPathRow {
    pub position: i32,
    pub channel: String,
    pub status: String,
}

async fn parse_tx_args(
    args: TxAddArgs,
    db: &SqliteDatabase,
) -> Result<(Transaction, Vec<Posting>, Vec<TagId>, Vec<ChannelPathNode>), AccountingError> {
    let date_time = parse_date_time(&args.date)?;
    let postings = parse_postings(&args.posting, db).await?;
    let tag_ids = resolve_tags(&args.tag, db).await?;
    let channel_path_nodes = parse_channel_paths(args.channel.as_deref(), db).await?;
    let member_id = resolve_member(db, &args.member).await?;
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description,
        kind: accounting::transaction::TransactionKind::Normal,
        member_id,
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
    let channel_path_nodes = parse_channel_paths(args.channel.as_deref(), db).await?;
    let member_id = match args.member {
        Some(ref name) => resolve_member(db, name).await?,
        None => {
            let existing = db
                .transaction_get(TransactionId(args.id))
                .await
                .map_err(|e| AccountingError::DatabaseError(e.to_string()))?
                .ok_or_else(|| {
                    AccountingError::InvalidTransaction(format!(
                        "{}",
                        t!("tx_not_found", id = args.id)
                    ))
                })?;
            existing.member_id
        }
    };
    let tx = Transaction {
        id: TransactionId(0),
        date_time,
        description: args.description.clone(),
        kind: accounting::transaction::TransactionKind::Normal,
        member_id,
    };
    Ok((tx, postings, tag_ids, channel_path_nodes))
}

/// 解析渠道链路表达式。
///
/// 语法：`渠道1 -> 渠道2* -> 渠道3&渠道4√`
/// - `->` 表示链的下一级，前后可有空格
/// - `&` 仅允许在最后一级使用，表示末级多个并行渠道
/// - 渠道名后可追加 `*` 表示 pending，`√` 表示 verified，无后缀为 default
async fn parse_channel_paths(
    expr: Option<&str>,
    db: &SqliteDatabase,
) -> Result<Vec<ChannelPathNode>, AccountingError> {
    let expr = match expr {
        Some(e) if !e.trim().is_empty() => e.trim(),
        _ => return Ok(Vec::new()),
    };

    let segments: Vec<&str> = expr.split("->").map(|s| s.trim()).collect();
    if segments.iter().any(|s| s.is_empty()) {
        return Err(AccountingError::InvalidTransaction(format!(
            "{}",
            t!("tx_channel_path_empty_node")
        )));
    }

    let last_idx = segments.len() - 1;
    let mut nodes = Vec::new();

    for (i, seg) in segments.iter().enumerate() {
        let is_last = i == last_idx;
        let names: Vec<&str> = if is_last {
            seg.split('&').map(|s| s.trim()).collect()
        } else {
            if seg.contains('&') {
                return Err(AccountingError::InvalidTransaction(format!(
                    "{}",
                    t!("tx_ampersand_only_last")
                )));
            }
            vec![*seg]
        };

        if names.iter().any(|n| n.is_empty()) {
            return Err(AccountingError::InvalidTransaction(format!(
                "{}",
                t!("tx_channel_name_empty")
            )));
        }

        for name in names {
            let (name, status) = parse_channel_status(name);
            let channel_id = resolve_channel(db, name).await?;
            nodes.push(ChannelPathNode {
                position: i as i32,
                channel_id,
                status,
            });
        }
    }

    Ok(nodes)
}

/// 从渠道名末尾剥离状态后缀。
///
/// `*` → pending，`√` → verified，无后缀 → default。
fn parse_channel_status(name: &str) -> (&str, ChannelPathStatus) {
    if let Some(prefix) = name.strip_suffix('*') {
        return (prefix.trim(), ChannelPathStatus::Pending);
    }
    if let Some(prefix) = name.strip_suffix('√') {
        return (prefix.trim(), ChannelPathStatus::Verified);
    }
    (name, ChannelPathStatus::Default)
}

async fn channel_name_map(
    db: &SqliteDatabase,
    lang: &str,
) -> Result<HashMap<ChannelId, String>, AccountingError> {
    let channels = db
        .channel_list()
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))?;
    let ids: Vec<ChannelId> = channels.iter().map(|c| c.id).collect();
    db.channel_display_names(&ids, lang)
        .await
        .map_err(|e| AccountingError::DatabaseError(e.to_string()))
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
    for account_path in &args.account {
        filter
            .account_ids
            .push(resolve_account(db, account_path).await?);
    }
    for member_name in &args.member {
        filter
            .member_ids
            .push(resolve_member(db, member_name).await?);
    }
    for channel_name in &args.channel {
        filter
            .channel_ids
            .push(resolve_channel(db, channel_name).await?);
    }
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
