use crate::error::BeancountError;
use crate::model::*;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse(input: &str) -> Result<BeancountData, BeancountError> {
    let mut data = BeancountData {
        commodities: vec![],
        accounts: vec![],
        members: vec![],
        channels: vec![],
        transactions: vec![],
        documents: vec![],
    };

    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_num = i + 1;

        if line.trim().is_empty() || line.starts_with(';') || line.starts_with('*') {
            i += 1;
            continue;
        }

        if line.starts_with("option") {
            i += 1;
            continue;
        }

        if let Some(date_str) = line.split_whitespace().next()
            && date_str.len() == 10
            && date_str.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 2 {
                i += 1;
                continue;
            }

            let directive = parts[1];

            // If directive looks like a time (contains ':'), this is a transaction
            if directive.contains(':') {
                let (tx, next_i) = parse_transaction(&lines, i, line_num)?;
                data.transactions.push(tx);
                i = next_i;
                continue;
            }

            match directive {
                "commodity" => {
                    let (commodity, next_i) = parse_commodity(&lines, i, line_num)?;
                    data.commodities.push(commodity);
                    i = next_i;
                }
                "open" => {
                    let (account, next_i) = parse_account_open(&lines, i, line_num)?;
                    data.accounts.push(account);
                    i = next_i;
                }
                "close" => {
                    parse_account_close(&lines, i, line_num, &mut data.accounts)?;
                    i = skip_metadata(&lines, i + 1);
                }
                "custom" => {
                    let custom_type = extract_quoted_after(line, "custom");
                    match custom_type.as_deref() {
                        Some("member") => {
                            let (member, next_i) = parse_member(&lines, i, line_num)?;
                            data.members.push(member);
                            i = next_i;
                        }
                        Some("channel") => {
                            let (channel, next_i) = parse_channel(&lines, i, line_num)?;
                            data.channels.push(channel);
                            i = next_i;
                        }
                        _ => {
                            i = skip_metadata(&lines, i + 1);
                        }
                    }
                }
                "document" => {
                    let doc = parse_document(&lines, i, line_num)?;
                    data.documents.push(doc);
                    i = skip_metadata(&lines, i + 1);
                }
                "*" | "!" => {
                    let (tx, next_i) = parse_transaction(&lines, i, line_num)?;
                    data.transactions.push(tx);
                    i = next_i;
                }
                _ => {
                    if NaiveDate::from_str(date_str).is_ok() {
                        let (tx, next_i) = parse_transaction(&lines, i, line_num)?;
                        data.transactions.push(tx);
                        i = next_i;
                    } else {
                        i += 1;
                    }
                }
            }
            continue;
        }

        i += 1;
    }

    Ok(data)
}

fn parse_commodity(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BCommodity, usize), BeancountError> {
    let line = lines[start];
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid commodity directive".to_string(),
        });
    }

    let symbol = parts[2].to_string();
    let metadata = collect_metadata(lines, start + 1);
    let next_i = start + 1 + metadata.len();

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let name = metadata.get("name").map(|v| unquote(v)).unwrap_or_default();
    let precision = metadata
        .get("precision")
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);

    Ok((
        BCommodity {
            internal_id,
            symbol,
            name,
            precision,
            created_at: None,
        },
        next_i,
    ))
}

fn parse_account_open(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BAccount, usize), BeancountError> {
    let line = lines[start];
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid open directive".to_string(),
        });
    }

    let path = parts[2].to_string();
    let metadata = collect_metadata(lines, start + 1);
    let next_i = start + 1 + metadata.len();

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let account_type = metadata
        .get("account_type")
        .map(|v| unquote(v))
        .unwrap_or_else(|| infer_account_type(&path));
    let billing_day = metadata.get("billing_day").and_then(|v| v.parse().ok());
    let repayment_day = metadata.get("repayment_day").and_then(|v| v.parse().ok());

    Ok((
        BAccount {
            internal_id,
            path,
            account_type,
            created_at: None,
            closed_at: None,
            billing_day,
            repayment_day,
        },
        next_i,
    ))
}

fn parse_account_close(
    lines: &[&str],
    start: usize,
    line_num: usize,
    accounts: &mut [BAccount],
) -> Result<(), BeancountError> {
    let line = lines[start];
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid close directive".to_string(),
        });
    }

    let date_str = parts[0];
    let path = parts[2].to_string();

    let closed_at = NaiveDate::from_str(date_str).map_err(|_| BeancountError::ParseError {
        line: line_num,
        message: format!("invalid date: {}", date_str),
    })?;

    if let Some(account) = accounts.iter_mut().find(|a| a.path == path) {
        account.closed_at = Some(closed_at);
    }

    Ok(())
}

fn parse_member(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BMember, usize), BeancountError> {
    let line = lines[start];
    let quoted_values = extract_all_quoted(line);
    if quoted_values.is_empty() {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid member custom directive".to_string(),
        });
    }

    let name = if quoted_values.len() >= 2 {
        quoted_values[1].clone()
    } else {
        quoted_values[0].clone()
    };
    let metadata = collect_metadata(lines, start + 1);
    let next_i = start + 1 + metadata.len();

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok((BMember { internal_id, name }, next_i))
}

fn parse_channel(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BChannel, usize), BeancountError> {
    let line = lines[start];
    let quoted_values = extract_all_quoted(line);
    if quoted_values.is_empty() {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid channel custom directive".to_string(),
        });
    }

    let name = if quoted_values.len() >= 2 {
        quoted_values[1].clone()
    } else {
        quoted_values[0].clone()
    };
    let metadata = collect_metadata(lines, start + 1);
    let next_i = start + 1 + metadata.len();

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let description = metadata.get("description").map(|v| unquote(v));

    Ok((
        BChannel {
            internal_id,
            name,
            description,
        },
        next_i,
    ))
}

fn parse_document(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<BDocument, BeancountError> {
    let line = lines[start];
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid document directive".to_string(),
        });
    }

    let date = NaiveDate::from_str(parts[0]).map_err(|_| BeancountError::ParseError {
        line: line_num,
        message: format!("invalid date: {}", parts[0]),
    })?;
    let account = parts[2].to_string();
    let quoted_values = extract_all_quoted(line);
    let filename = quoted_values
        .last()
        .cloned()
        .unwrap_or_else(|| parts[3].to_string());

    let metadata = collect_metadata(lines, start + 1);
    let transaction_internal_id = metadata.get("transaction_id").and_then(|v| v.parse().ok());

    Ok(BDocument {
        date,
        account,
        filename,
        transaction_internal_id,
    })
}

fn parse_transaction(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BTransaction, usize), BeancountError> {
    let line = lines[start];

    let (date_time, rest) = split_datetime(line, line_num)?;

    let (description, tags) = parse_txn_header(rest);

    let mut i = start + 1;
    let mut metadata = std::collections::HashMap::new();
    let mut postings = Vec::new();

    while i < lines.len() {
        let next_line = lines[i];
        if next_line.trim().is_empty() {
            break;
        }
        if !next_line.starts_with(' ') {
            break;
        }
        if is_metadata_line(next_line) {
            let (key, value) = parse_metadata_line(next_line);
            metadata.insert(key, value);
            i += 1;
        } else if is_posting_line(next_line) {
            let (posting, next_i) = parse_posting(lines, i, line_num)?;
            postings.push(posting);
            i = next_i;
        } else {
            i += 1;
        }
    }

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let kind = metadata
        .get("kind")
        .map(|v| unquote(v))
        .unwrap_or_else(|| "normal".to_string());
    let member = metadata.get("member").map(|v| unquote(v));

    let channel_path = metadata
        .get("channel_path")
        .map(|v| parse_channel_path(&unquote(v)))
        .unwrap_or_default();

    let reversal_of = metadata
        .get("reversal_of")
        .and_then(|v| parse_reversal(&unquote(v)));

    Ok((
        BTransaction {
            internal_id,
            date_time,
            description,
            kind,
            member,
            tags,
            channel_path,
            postings,
            reversal_of,
        },
        i,
    ))
}

fn parse_posting(
    lines: &[&str],
    start: usize,
    line_num: usize,
) -> Result<(BPosting, usize), BeancountError> {
    let line = lines[start].trim();

    let (account, rest) = line
        .split_once(|c: char| c.is_whitespace())
        .ok_or_else(|| BeancountError::ParseError {
            line: line_num,
            message: "invalid posting line".to_string(),
        })?;

    let rest = rest.trim();

    let (amount, commodity, cost, cost_commodity) = parse_amount_cost(rest)?;

    let mut i = start + 1;
    let mut metadata = std::collections::HashMap::new();

    while i < lines.len() {
        let next_line = lines[i];
        if is_metadata_line(next_line) {
            let (key, value) = parse_metadata_line(next_line);
            metadata.insert(key, value);
            i += 1;
        } else {
            break;
        }
    }

    let internal_id = metadata
        .get("internal_id")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let reimbursable = metadata
        .get("reimbursable")
        .map(|v| v == "TRUE")
        .unwrap_or(false);

    Ok((
        BPosting {
            internal_id,
            account: account.to_string(),
            amount,
            commodity,
            cost,
            cost_commodity,
            reimbursable,
        },
        i,
    ))
}

fn parse_amount_cost(
    s: &str,
) -> Result<(Decimal, String, Option<Decimal>, Option<String>), BeancountError> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(BeancountError::ParseError {
            line: 0,
            message: format!("invalid amount: {}", s),
        });
    }

    let amount = Decimal::from_str(parts[0]).map_err(|_| BeancountError::ParseError {
        line: 0,
        message: format!("invalid amount: {}", parts[0]),
    })?;
    let commodity = parts[1].to_string();

    let (cost, cost_commodity) = if parts.len() >= 4 && parts[2] == "{" && parts[4] == "}" {
        let cost = Decimal::from_str(parts[3]).ok();
        let cost_comm = if parts.len() > 5 {
            Some(parts[5].trim_end_matches('}').to_string())
        } else {
            None
        };
        (cost, cost_comm)
    } else if parts.len() >= 4 && parts[2].starts_with('{') {
        let cost_str = parts[2].trim_start_matches('{');
        let cost = Decimal::from_str(cost_str).ok();
        let cost_comm = if parts.len() > 3 {
            Some(parts[3].trim_end_matches('}').to_string())
        } else {
            None
        };
        (cost, cost_comm)
    } else {
        (None, None)
    };

    Ok((amount, commodity, cost, cost_commodity))
}

fn split_datetime(line: &str, line_num: usize) -> Result<(NaiveDateTime, &str), BeancountError> {
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 3 {
        return Err(BeancountError::ParseError {
            line: line_num,
            message: "invalid transaction line".to_string(),
        });
    }

    let date_str = parts[0];
    let time_or_flag = parts[1];

    if time_or_flag.contains(':') {
        let datetime_str = format!("{} {}", date_str, time_or_flag);
        let dt = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M"))
            .map_err(|_| BeancountError::ParseError {
                line: line_num,
                message: format!("invalid datetime: {}", datetime_str),
            })?;
        let rest = if parts.len() > 3 { parts[3] } else { "" };
        Ok((dt, rest))
    } else {
        let date = NaiveDate::from_str(date_str).map_err(|_| BeancountError::ParseError {
            line: line_num,
            message: format!("invalid date: {}", date_str),
        })?;
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
        let rest = if parts.len() > 2 {
            line[date_str.len()..].trim()
        } else {
            ""
        };
        Ok((dt, rest))
    }
}

fn parse_txn_header(rest: &str) -> (String, Vec<String>) {
    let rest = rest.trim();

    let rest = if rest.starts_with("* ") || rest.starts_with("! ") {
        &rest[2..]
    } else {
        rest
    };

    let mut tags = Vec::new();
    let mut description = String::new();

    let mut remaining = rest;

    if remaining.starts_with('"')
        && let Some(end) = find_closing_quote(remaining, 1)
    {
        let payee = unquote(&remaining[..=end]);
        remaining = remaining[end + 1..].trim();

        if remaining.starts_with('"') {
            if let Some(end2) = find_closing_quote(remaining, 1) {
                let narration = unquote(&remaining[..=end2]);
                remaining = remaining[end2 + 1..].trim();
                description = if payee.is_empty() {
                    narration
                } else {
                    format!("{} - {}", payee, narration)
                };
            }
        } else {
            description = payee;
        }
    }

    for part in remaining.split_whitespace() {
        if let Some(tag) = part.strip_prefix('#') {
            tags.push(tag.to_string());
        }
    }

    (description, tags)
}

fn find_closing_quote(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_metadata_line(line: &str) -> bool {
    if line.len() < 4 {
        return false;
    }
    let leading_spaces = line.len() - line.trim_start().len();
    if leading_spaces < 4 {
        return false;
    }
    let trimmed = line.trim();
    !trimmed.is_empty() && !trimmed.starts_with(';') && trimmed.contains(':')
}

fn is_posting_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(';') {
        return false;
    }
    let leading_spaces = line.len() - line.trim_start().len();
    if leading_spaces < 2 {
        return false;
    }
    if leading_spaces >= 4 && is_metadata_line(line) {
        return false;
    }
    let first = trimmed.split_whitespace().next().unwrap_or("");
    first.contains(':')
}

fn parse_metadata_line(line: &str) -> (String, String) {
    let trimmed = line.trim();
    if let Some((key, rest)) = trimmed.split_once(':') {
        let key = key.trim().to_string();
        let rest = rest.trim();
        // 如果值是 JSON 字符串（以 [ 或 { 开头），则把冒号后整段作为值，
        // 避免后续冒号被 split_once 截断。
        if rest.starts_with('"') && (rest.contains('[') || rest.contains('{')) {
            // 找到第一个 " 开始的位置，取从该位置到行尾的内容
            if let Some(start) = trimmed.find(':') {
                let value = trimmed[start + 1..].trim().to_string();
                return (key, value);
            }
        }
        (key, rest.to_string())
    } else {
        (trimmed.to_string(), String::new())
    }
}

fn collect_metadata(lines: &[&str], start: usize) -> std::collections::HashMap<String, String> {
    let mut metadata = std::collections::HashMap::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if is_metadata_line(line) {
            let (key, value) = parse_metadata_line(line);
            metadata.insert(key, value);
            i += 1;
        } else {
            break;
        }
    }
    metadata
}

fn skip_metadata(lines: &[&str], start: usize) -> usize {
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            return i + 1;
        }
        if is_metadata_line(line) {
            i += 1;
        } else {
            break;
        }
    }
    i
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        s.to_string()
    }
}

fn extract_quoted_after(line: &str, keyword: &str) -> Option<String> {
    let pos = line.find(keyword)?;
    let after = &line[pos + keyword.len()..];
    let after = after.trim();
    if after.starts_with('"') {
        let end = find_closing_quote(after, 1)?;
        Some(unquote(&after[..=end]))
    } else {
        None
    }
}

fn extract_all_quoted(line: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_from = 0;
    let bytes = line.as_bytes();
    while search_from < bytes.len() {
        if bytes[search_from] == b'"' {
            if let Some(end) = find_closing_quote(line, search_from + 1) {
                results.push(unquote(&line[search_from..=end]));
                search_from = end + 1;
            } else {
                break;
            }
        } else {
            search_from += 1;
        }
    }
    results
}

fn infer_account_type(path: &str) -> String {
    let root = path.split(':').next().unwrap_or("");
    match root.to_lowercase().as_str() {
        "assets" | "资产" => "Asset".to_string(),
        "liabilities" | "负债" => "Equity".to_string(),
        "equity" | "权益" => "Equity".to_string(),
        "income" | "收入" => "Income".to_string(),
        "expenses" | "支出" => "Expense".to_string(),
        _ => "Asset".to_string(),
    }
}

fn parse_channel_path(value: &str) -> Vec<ChannelPathEntry> {
    // 优先尝试新的文本格式
    if let Some(entries) = parse_channel_path_text(value) {
        return entries;
    }

    // 兼容旧 JSON 备份
    serde_json::from_str::<Vec<serde_json::Value>>(value)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            let status = if v.get("reconciled")?.as_bool().unwrap_or(false) {
                accounting::channel_path::ChannelPathStatus::Verified
            } else {
                accounting::channel_path::ChannelPathStatus::Default
            };
            Some(ChannelPathEntry {
                position: v.get("position")?.as_i64()? as i32,
                channel: v.get("channel")?.as_str()?.to_string(),
                status,
            })
        })
        .collect()
}

/// 解析 CLI 文本格式的渠道链路。
///
/// 示例：`淘宝 -> 支付宝* -> 花呗 & 建行卡√`
fn parse_channel_path_text(value: &str) -> Option<Vec<ChannelPathEntry>> {
    let value = value.trim();
    // JSON 旧格式以 [ 或 { 开头，交给 JSON fallback 处理
    if value.starts_with('[') || value.starts_with('{') {
        return None;
    }

    let mut entries = Vec::new();

    for (pos, part) in value.split("->").enumerate() {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }

        for name in part.split('&') {
            let name = name.trim();
            if name.is_empty() {
                return None;
            }

            let (name, status) = parse_channel_status(name);
            if name.is_empty() {
                return None;
            }

            entries.push(ChannelPathEntry {
                position: pos as i32,
                channel: name.to_string(),
                status,
            });
        }
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// 从渠道名末尾剥离状态后缀。
fn parse_channel_status(name: &str) -> (&str, accounting::channel_path::ChannelPathStatus) {
    if let Some(prefix) = name.strip_suffix('*') {
        return (
            prefix.trim(),
            accounting::channel_path::ChannelPathStatus::Pending,
        );
    }
    if let Some(prefix) = name.strip_suffix('√') {
        return (
            prefix.trim(),
            accounting::channel_path::ChannelPathStatus::Verified,
        );
    }
    (name, accounting::channel_path::ChannelPathStatus::Default)
}

fn parse_reversal(json_str: &str) -> Option<ReversalInfo> {
    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
    Some(ReversalInfo {
        posting_id: v.get("posting_id")?.as_i64()?,
        target_posting_id: v.get("target_posting_id")?.as_i64()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commodity() {
        let input = "1970-01-01 commodity CNY\n    internal_id: 1\n    name: \"人民币\"\n    precision: 2\n";
        let data = parse(input).unwrap();
        assert_eq!(data.commodities.len(), 1);
        assert_eq!(data.commodities[0].symbol, "CNY");
        assert_eq!(data.commodities[0].name, "人民币");
        assert_eq!(data.commodities[0].precision, 2);
        assert_eq!(data.commodities[0].internal_id, 1);
    }

    #[test]
    fn test_parse_account_open() {
        let input = "1970-01-01 open 资产:现金\n    internal_id: 1\n    account_type: \"Asset\"\n";
        let data = parse(input).unwrap();
        assert_eq!(data.accounts.len(), 1);
        assert_eq!(data.accounts[0].path, "资产:现金");
        assert_eq!(data.accounts[0].account_type, "Asset");
    }

    #[test]
    fn test_parse_account_close() {
        let input = "1970-01-01 open 支出:食品\n    internal_id: 2\n    account_type: \"Expense\"\n\n2024-12-31 close 支出:食品\n";
        let data = parse(input).unwrap();
        assert_eq!(data.accounts.len(), 1);
        assert_eq!(
            data.accounts[0].closed_at,
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        );
    }

    #[test]
    fn test_parse_member() {
        let input = "1970-01-01 custom \"member\" \"张三\"\n    internal_id: 1\n";
        let data = parse(input).unwrap();
        assert_eq!(data.members.len(), 1);
        assert_eq!(data.members[0].name, "张三");
    }

    #[test]
    fn test_parse_channel() {
        let input = "1970-01-01 custom \"channel\" \"微信\"\n    internal_id: 1\n    description: \"微信支付\"\n";
        let data = parse(input).unwrap();
        assert_eq!(data.channels.len(), 1);
        assert_eq!(data.channels[0].name, "微信");
        assert_eq!(data.channels[0].description, Some("微信支付".to_string()));
    }

    #[test]
    fn test_parse_transaction() {
        let input = "2024-03-15 10:30:00 * \"\" \"盒马买菜\" #餐饮\n    internal_id: 100\n    kind: \"normal\"\n    member: \"张三\"\n  支出:食品 150.00 CNY\n        internal_id: 200\n        reimbursable: FALSE\n  资产:现金 -150.00 CNY\n        internal_id: 201\n        reimbursable: FALSE\n";
        let data = parse(input).unwrap();
        assert_eq!(data.transactions.len(), 1);
        let tx = &data.transactions[0];
        assert_eq!(tx.description, "盒马买菜");
        assert_eq!(tx.kind, "normal");
        assert_eq!(tx.member, Some("张三".to_string()));
        assert_eq!(tx.tags, vec!["餐饮"]);
        assert_eq!(tx.postings.len(), 2);
        assert_eq!(tx.postings[0].account, "支出:食品");
        assert_eq!(tx.postings[0].amount, Decimal::from_str("150.00").unwrap());
        assert_eq!(tx.postings[0].commodity, "CNY");
    }

    #[test]
    fn test_parse_posting_with_cost() {
        let input = "2024-01-01 * \"\" \"test\"\n    internal_id: 1\n    kind: \"normal\"\n  Assets:USD 100 USD {720 CNY}\n        internal_id: 1\n        reimbursable: FALSE\n";
        let data = parse(input).unwrap();
        let tx = &data.transactions[0];
        assert_eq!(tx.postings[0].cost, Some(Decimal::from_str("720").unwrap()));
        assert_eq!(tx.postings[0].cost_commodity, Some("CNY".to_string()));
    }

    #[test]
    fn test_parse_document() {
        let input = "2024-03-15 document 支出:食品 \"attachments/5_receipt.jpg\"\n";
        let data = parse(input).unwrap();
        assert_eq!(data.documents.len(), 1);
        assert_eq!(data.documents[0].account, "支出:食品");
        assert_eq!(data.documents[0].filename, "attachments/5_receipt.jpg");
    }

    #[test]
    fn test_parse_channel_path_metadata_json() {
        let input = "2024-01-01 * \"\" \"test\"\n    internal_id: 1\n    kind: \"normal\"\n    channel_path: \"[{\\\"position\\\":0,\\\"channel\\\":\\\"微信\\\",\\\"reconciled\\\":true}]\"\n  支出:食品 100 CNY\n        internal_id: 1\n        reimbursable: FALSE\n";
        let data = parse(input).unwrap();
        let tx = &data.transactions[0];
        assert_eq!(tx.channel_path.len(), 1);
        assert_eq!(tx.channel_path[0].channel, "微信");
        assert_eq!(
            tx.channel_path[0].status,
            accounting::channel_path::ChannelPathStatus::Verified
        );
    }

    #[test]
    fn test_parse_channel_path_metadata_text() {
        let input = "2024-01-01 * \"\" \"test\"\n    internal_id: 1\n    kind: \"normal\"\n    channel_path: \"淘宝 -> 支付宝* -> 花呗 & 建行卡√\"\n  支出:食品 100 CNY\n        internal_id: 1\n        reimbursable: FALSE\n";
        let data = parse(input).unwrap();
        let tx = &data.transactions[0];
        assert_eq!(tx.channel_path.len(), 4);
        assert_eq!(tx.channel_path[0].channel, "淘宝");
        assert_eq!(
            tx.channel_path[0].status,
            accounting::channel_path::ChannelPathStatus::Default
        );
        assert_eq!(tx.channel_path[1].channel, "支付宝");
        assert_eq!(
            tx.channel_path[1].status,
            accounting::channel_path::ChannelPathStatus::Pending
        );
        assert_eq!(tx.channel_path[2].channel, "花呗");
        assert_eq!(
            tx.channel_path[2].status,
            accounting::channel_path::ChannelPathStatus::Default
        );
        assert_eq!(tx.channel_path[3].channel, "建行卡");
        assert_eq!(
            tx.channel_path[3].status,
            accounting::channel_path::ChannelPathStatus::Verified
        );
    }
}
