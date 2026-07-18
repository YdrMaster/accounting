use crate::error::BeancountError;
use crate::generator;
use crate::model::*;
use accounting::account::Account;
use accounting::channel::Channel;
use accounting::id::*;
use accounting::member::Member;
use accounting::transaction::TransactionKind;
use accounting_sql::SqliteDatabase;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;

/// 按显示语言解析出的实体显示名（批量回退链：所选 → en → zh-CN → rowid）
struct DisplayNames {
    accounts: HashMap<AccountId, String>,
    members: HashMap<MemberId, String>,
    channels: HashMap<ChannelId, String>,
}

/// 导出数据库为 beancount 文本。
///
/// - `db`：数据库句柄（内存库亦可导出）
/// - `lang`：当前显示语言，账户/成员/渠道等名字按回退链取该语言的显示名；
///   币种只导出 symbol，不导出名字
pub async fn export(
    db: &SqliteDatabase,
    lang: &str,
    output_dir: &Path,
) -> Result<String, BeancountError> {
    let attachments_dir = output_dir.join("attachments");
    std::fs::create_dir_all(&attachments_dir)?;

    let accounts = db
        .account_list()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let members = db
        .member_list()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let channels = db
        .channel_list()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    let names = resolve_display_names(db, &accounts, &members, &channels, lang).await?;

    let mut data = BeancountData {
        commodities: vec![],
        accounts: vec![],
        members: vec![],
        channels: vec![],
        transactions: vec![],
        documents: vec![],
    };

    export_commodities(db, &mut data).await?;
    export_accounts(db, &accounts, &names.accounts, &mut data).await?;
    export_members(&members, &names.members, &mut data);
    export_channels(&channels, &names.channels, &mut data);
    export_transactions(db, &accounts, &names, &mut data, &attachments_dir, lang).await?;

    let text = generator::generate(&data);
    Ok(text)
}

async fn resolve_display_names(
    db: &SqliteDatabase,
    accounts: &[Account],
    members: &[Member],
    channels: &[Channel],
    lang: &str,
) -> Result<DisplayNames, BeancountError> {
    let account_ids: Vec<AccountId> = accounts.iter().map(|a| a.id).collect();
    let member_ids: Vec<MemberId> = members.iter().map(|m| m.id).collect();
    let channel_ids: Vec<ChannelId> = channels.iter().map(|c| c.id).collect();

    let accounts = db
        .account_display_names(&account_ids, lang)
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let members = db
        .member_display_names(&member_ids, lang)
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let channels = db
        .channel_display_names(&channel_ids, lang)
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    Ok(DisplayNames {
        accounts,
        members,
        channels,
    })
}

async fn export_commodities(
    db: &SqliteDatabase,
    data: &mut BeancountData,
) -> Result<(), BeancountError> {
    let commodities = db
        .commodity_list()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let created_at_map = db
        .commodity_created_at_map()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    for c in &commodities {
        data.commodities.push(BCommodity {
            internal_id: c.id.0,
            symbol: c.symbol.clone(),
            precision: c.precision,
            created_at: created_at_map.get(&c.id).copied(),
        });
    }
    Ok(())
}

async fn export_accounts(
    db: &SqliteDatabase,
    accounts: &[Account],
    account_names: &HashMap<AccountId, String>,
    data: &mut BeancountData,
) -> Result<(), BeancountError> {
    let created_at_by_id: HashMap<AccountId, NaiveDate> = db
        .account_created_at_map()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    let accounts_by_id: HashMap<AccountId, Account> =
        accounts.iter().map(|a| (a.id, a.clone())).collect();

    for a in accounts {
        let path = a.display_path(&accounts_by_id, account_names);
        let root_name = root_account_name(a, &accounts_by_id, account_names);

        let account_type = match root_name.as_str() {
            "Asset" | "Assets" | "资产" => "Asset",
            "Equity" | "权益" => "Equity",
            "Income" | "收入" => "Income",
            "Expense" | "Expenses" | "支出" => "Expense",
            _ => "Asset",
        };

        data.accounts.push(BAccount {
            internal_id: a.id.0,
            path,
            account_type: account_type.to_string(),
            created_at: created_at_by_id.get(&a.id).copied(),
            closed_at: a.closed_at,
            billing_day: a.billing_day,
            repayment_day: a.repayment_day,
        });
    }
    Ok(())
}

/// 沿父链找到根账户，返回其显示名（无根或无名时返回空串）
fn root_account_name(
    account: &Account,
    accounts_by_id: &HashMap<AccountId, Account>,
    account_names: &HashMap<AccountId, String>,
) -> String {
    let mut current = account;
    while let Some(pid) = current.parent_id {
        match accounts_by_id.get(&pid) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    account_names.get(&current.id).cloned().unwrap_or_default()
}

fn export_members(
    members: &[Member],
    member_names: &HashMap<MemberId, String>,
    data: &mut BeancountData,
) {
    for m in members {
        data.members.push(BMember {
            internal_id: m.id.0,
            name: member_names.get(&m.id).cloned().unwrap_or_default(),
        });
    }
}

fn export_channels(
    channels: &[Channel],
    channel_names: &HashMap<ChannelId, String>,
    data: &mut BeancountData,
) {
    for ch in channels {
        data.channels.push(BChannel {
            internal_id: ch.id.0,
            name: channel_names.get(&ch.id).cloned().unwrap_or_default(),
            description: ch.description.clone(),
        });
    }
}

async fn export_transactions(
    db: &SqliteDatabase,
    accounts: &[Account],
    names: &DisplayNames,
    data: &mut BeancountData,
    attachments_dir: &Path,
    lang: &str,
) -> Result<(), BeancountError> {
    use accounting::transaction_filter::TransactionFilter;

    let filter = TransactionFilter::default();
    let transactions = db
        .transaction_list(&filter, usize::MAX, 0)
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    let accounts_by_id: HashMap<AccountId, Account> =
        accounts.iter().map(|a| (a.id, a.clone())).collect();

    let commodities = db
        .commodity_list()
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;
    let comm_by_id: HashMap<CommodityId, String> = commodities
        .iter()
        .map(|c| (c.id, c.symbol.clone()))
        .collect();

    let tx_ids: Vec<TransactionId> = transactions.iter().map(|t| t.id).collect();
    let tags_by_tx = db
        .tag_names_by_transactions(&tx_ids, lang)
        .await
        .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

    for tx in &transactions {
        let postings = db
            .posting_list_by_transaction(tx.id)
            .await
            .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

        let channel_paths = db
            .channel_path_list_by_transaction(tx.id)
            .await
            .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

        let tags = tags_by_tx.get(&tx.id).cloned().unwrap_or_default();

        let kind_str = match tx.kind {
            TransactionKind::Normal => "normal",
            TransactionKind::Refund => "refund",
            TransactionKind::Reimbursement => "reimbursement",
        };

        let member_name = Some(names.members.get(&tx.member_id).cloned().ok_or_else(|| {
            BeancountError::DatabaseError(format!("member id {} not found", tx.member_id.0))
        })?);

        let cp_entries: Vec<ChannelPathEntry> = channel_paths
            .iter()
            .map(|cp| ChannelPathEntry {
                position: cp.position,
                channel: names
                    .channels
                    .get(&cp.channel_id)
                    .cloned()
                    .unwrap_or_default(),
                status: cp.status,
            })
            .collect();

        let mut b_postings = Vec::new();
        let mut reversal_info = None;

        for p in &postings {
            let account_path = {
                let acct = accounts_by_id.get(&p.account_id);
                acct.map(|a| a.display_path(&accounts_by_id, &names.accounts))
                    .unwrap_or_default()
            };

            let commodity_symbol = comm_by_id.get(&p.commodity_id).cloned().unwrap_or_default();

            let (cost, cost_commodity) =
                if let (Some(cost), Some(cc_id)) = (p.cost, p.cost_commodity_id) {
                    (Some(cost), comm_by_id.get(&cc_id).cloned())
                } else {
                    (None, None)
                };

            if let Some(linked_id) = p.linked_posting_id {
                reversal_info = Some(ReversalInfo {
                    posting_id: p.id.0,
                    target_posting_id: linked_id.0,
                });
            }

            b_postings.push(BPosting {
                internal_id: p.id.0,
                account: account_path,
                amount: p.amount,
                commodity: commodity_symbol,
                cost,
                cost_commodity,
                reimbursable: p.is_reimbursable,
            });
        }

        data.transactions.push(BTransaction {
            internal_id: tx.id.0,
            date_time: tx.date_time,
            description: tx.description.clone(),
            kind: kind_str.to_string(),
            member: member_name,
            tags,
            channel_path: cp_entries,
            postings: b_postings,
            reversal_of: reversal_info,
        });

        let attachments = db
            .attachment_list_by_transaction(tx.id)
            .await
            .map_err(|e| BeancountError::DatabaseError(e.to_string()))?;

        for att in &attachments {
            let file_name = format!("{}_{}", att.id.0, att.filename);
            let file_path = attachments_dir.join(&file_name);
            std::fs::write(&file_path, &att.data)?;

            let account_path = postings
                .first()
                .and_then(|p| accounts_by_id.get(&p.account_id))
                .map(|a| a.display_path(&accounts_by_id, &names.accounts))
                .unwrap_or_default();

            data.documents.push(BDocument {
                date: tx.date_time.date(),
                account: account_path,
                filename: format!("attachments/{}", file_name),
                transaction_internal_id: Some(tx.id.0),
            });
        }
    }

    Ok(())
}
