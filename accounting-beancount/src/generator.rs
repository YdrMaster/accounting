use crate::model::*;
use chrono::NaiveDate;
use std::fmt::Write;

pub fn generate(data: &BeancountData) -> String {
    let mut out = String::new();

    for c in &data.commodities {
        generate_commodity(&mut out, c);
    }

    for a in &data.accounts {
        generate_account_open(&mut out, a);
    }

    for m in &data.members {
        generate_member(&mut out, m);
    }

    for ch in &data.channels {
        generate_channel(&mut out, ch);
    }

    for tx in &data.transactions {
        generate_transaction(&mut out, tx);
    }

    for a in &data.accounts {
        if let Some(closed_at) = a.closed_at {
            generate_account_close(&mut out, a, closed_at);
        }
    }

    for doc in &data.documents {
        generate_document(&mut out, doc);
    }

    out
}

fn write_metadata(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {}: {}", key, value);
}

fn escape_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn generate_commodity(out: &mut String, c: &BCommodity) {
    let _ = writeln!(out, "1970-01-01 commodity {}", c.symbol);
    write_metadata(out, "internal_id", &c.internal_id.to_string());
    write_metadata(out, "name", &escape_string(&c.name));
    write_metadata(out, "precision", &c.precision.to_string());
    let _ = writeln!(out);
}

fn generate_account_open(out: &mut String, a: &BAccount) {
    let open_date = a
        .created_at
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let _ = writeln!(out, "{} open {}", open_date, a.path);
    write_metadata(out, "internal_id", &a.internal_id.to_string());
    write_metadata(out, "account_type", &escape_string(&a.account_type));
    if let Some(bd) = a.billing_day {
        write_metadata(out, "billing_day", &bd.to_string());
    }
    if let Some(rd) = a.repayment_day {
        write_metadata(out, "repayment_day", &rd.to_string());
    }
    let _ = writeln!(out);
}

fn generate_account_close(out: &mut String, a: &BAccount, closed_at: chrono::NaiveDate) {
    let _ = writeln!(out, "{} close {}", closed_at, a.path);
    let _ = writeln!(out);
}

fn generate_member(out: &mut String, m: &BMember) {
    let _ = writeln!(
        out,
        "1970-01-01 custom \"member\" {}",
        escape_string(&m.name)
    );
    write_metadata(out, "internal_id", &m.internal_id.to_string());
    let _ = writeln!(out);
}

fn generate_channel(out: &mut String, ch: &BChannel) {
    let _ = writeln!(
        out,
        "1970-01-01 custom \"channel\" {}",
        escape_string(&ch.name)
    );
    write_metadata(out, "internal_id", &ch.internal_id.to_string());
    if let Some(ref desc) = ch.description {
        write_metadata(out, "description", &escape_string(desc));
    }
    let _ = writeln!(out);
}

fn generate_transaction(out: &mut String, tx: &BTransaction) {
    let date_time_str = tx.date_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let tags_str = if tx.tags.is_empty() {
        String::new()
    } else {
        let tags: Vec<String> = tx.tags.iter().map(|t| format!("#{}", t)).collect();
        format!(" {}", tags.join(" "))
    };

    let _ = writeln!(
        out,
        "{} * \"\" {}{}",
        date_time_str,
        escape_string(&tx.description),
        tags_str,
    );

    write_metadata(out, "internal_id", &tx.internal_id.to_string());
    write_metadata(out, "kind", &escape_string(&tx.kind));

    if let Some(ref member) = tx.member {
        write_metadata(out, "member", &escape_string(member));
    }

    if !tx.channel_path.is_empty() {
        let cp_json = serde_json::to_string(
            &tx.channel_path
                .iter()
                .map(|cp| {
                    serde_json::json!({
                        "position": cp.position,
                        "channel": cp.channel,
                        "reconciled": cp.reconciled,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        write_metadata(out, "channel_path", &escape_string(&cp_json));
    }

    if let Some(ref rev) = tx.reversal_of {
        let rev_json = serde_json::to_string(&serde_json::json!({
            "posting_id": rev.posting_id,
            "target_posting_id": rev.target_posting_id,
        }))
        .unwrap();
        write_metadata(out, "reversal_of", &escape_string(&rev_json));
    }

    for p in &tx.postings {
        generate_posting(out, p);
    }

    let _ = writeln!(out);
}

fn generate_posting(out: &mut String, p: &BPosting) {
    let amount_str = if let (Some(cost), Some(cost_comm)) = (p.cost, &p.cost_commodity) {
        format!("{} {} {{{} {}}}", p.amount, p.commodity, cost, cost_comm)
    } else {
        format!("{} {}", p.amount, p.commodity)
    };

    let _ = writeln!(out, "  {} {}", p.account, amount_str);
    let _ = writeln!(out, "        internal_id: {}", p.internal_id);
    let _ = writeln!(
        out,
        "        reimbursable: {}",
        if p.reimbursable { "TRUE" } else { "FALSE" },
    );
}

fn generate_document(out: &mut String, doc: &BDocument) {
    let _ = writeln!(
        out,
        "{} document {} {}",
        doc.date,
        doc.account,
        escape_string(&doc.filename),
    );
    if let Some(tx_id) = doc.transaction_internal_id {
        let _ = writeln!(out, "    transaction_id: {}", tx_id);
    }
    let _ = writeln!(out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn sample_data() -> BeancountData {
        BeancountData {
            commodities: vec![BCommodity {
                internal_id: 1,
                symbol: "CNY".to_string(),
                name: "人民币".to_string(),
                precision: 2,
            }],
            accounts: vec![
                BAccount {
                    internal_id: 1,
                    path: "资产:现金".to_string(),
                    account_type: "Asset".to_string(),
                    created_at: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                    closed_at: None,
                    billing_day: None,
                    repayment_day: None,
                },
                BAccount {
                    internal_id: 2,
                    path: "支出:食品".to_string(),
                    account_type: "Expense".to_string(),
                    created_at: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                    closed_at: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
                    billing_day: None,
                    repayment_day: None,
                },
            ],
            members: vec![BMember {
                internal_id: 1,
                name: "张三".to_string(),
            }],
            channels: vec![BChannel {
                internal_id: 1,
                name: "微信".to_string(),
                description: Some("微信支付".to_string()),
            }],
            transactions: vec![BTransaction {
                internal_id: 100,
                date_time: chrono::NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                    chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
                ),
                description: "盒马买菜".to_string(),
                kind: "normal".to_string(),
                member: Some("张三".to_string()),
                tags: vec!["餐饮".to_string()],
                channel_path: vec![ChannelPathEntry {
                    position: 0,
                    channel: "微信".to_string(),
                    reconciled: true,
                }],
                postings: vec![
                    BPosting {
                        internal_id: 200,
                        account: "支出:食品".to_string(),
                        amount: Decimal::from_str("150.00").unwrap(),
                        commodity: "CNY".to_string(),
                        cost: None,
                        cost_commodity: None,
                        reimbursable: false,
                    },
                    BPosting {
                        internal_id: 201,
                        account: "资产:现金".to_string(),
                        amount: Decimal::from_str("-150.00").unwrap(),
                        commodity: "CNY".to_string(),
                        cost: None,
                        cost_commodity: None,
                        reimbursable: false,
                    },
                ],
                reversal_of: None,
            }],
            documents: vec![],
        }
    }

    #[test]
    fn test_generate_commodity() {
        let data = sample_data();
        let output = generate(&data);
        assert!(output.contains("1970-01-01 commodity CNY"));
        assert!(output.contains("internal_id: 1"));
        assert!(output.contains("name: \"人民币\""));
        assert!(output.contains("precision: 2"));
    }

    #[test]
    fn test_generate_account_open_and_close() {
        let data = sample_data();
        let output = generate(&data);
        assert!(output.contains("2024-01-01 open 资产:现金"));
        assert!(output.contains("account_type: \"Asset\""));
        assert!(output.contains("2024-12-31 close 支出:食品"));
    }

    #[test]
    fn test_generate_member() {
        let data = sample_data();
        let output = generate(&data);
        assert!(output.contains("custom \"member\" \"张三\""));
    }

    #[test]
    fn test_generate_channel() {
        let data = sample_data();
        let output = generate(&data);
        assert!(output.contains("custom \"channel\" \"微信\""));
        assert!(output.contains("description: \"微信支付\""));
    }

    #[test]
    fn test_generate_transaction() {
        let data = sample_data();
        let output = generate(&data);
        assert!(output.contains("2024-03-15 10:30:00 * \"\" \"盒马买菜\" #餐饮"));
        assert!(output.contains("kind: \"normal\""));
        assert!(output.contains("member: \"张三\""));
        assert!(output.contains("支出:食品 150.00 CNY"));
        assert!(output.contains("资产:现金 -150.00 CNY"));
    }

    #[test]
    fn test_generate_posting_with_cost() {
        let data = BeancountData {
            commodities: vec![],
            accounts: vec![],
            members: vec![],
            channels: vec![],
            transactions: vec![BTransaction {
                internal_id: 1,
                date_time: chrono::NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                ),
                description: "test".to_string(),
                kind: "normal".to_string(),
                member: None,
                tags: vec![],
                channel_path: vec![],
                postings: vec![BPosting {
                    internal_id: 1,
                    account: "Assets:USD".to_string(),
                    amount: Decimal::from_str("100").unwrap(),
                    commodity: "USD".to_string(),
                    cost: Some(Decimal::from_str("720").unwrap()),
                    cost_commodity: Some("CNY".to_string()),
                    reimbursable: false,
                }],
                reversal_of: None,
            }],
            documents: vec![],
        };
        let output = generate(&data);
        assert!(output.contains("Assets:USD 100 USD {720 CNY}"));
    }

    #[test]
    fn test_generate_document() {
        let data = BeancountData {
            commodities: vec![],
            accounts: vec![],
            members: vec![],
            channels: vec![],
            transactions: vec![],
            documents: vec![BDocument {
                date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                account: "支出:食品".to_string(),
                filename: "attachments/5_receipt.jpg".to_string(),
                transaction_internal_id: None,
            }],
        };
        let output = generate(&data);
        assert!(output.contains("2024-03-15 document 支出:食品 \"attachments/5_receipt.jpg\""));
    }
}
