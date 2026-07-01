use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BCommodity {
    pub internal_id: i64,
    pub symbol: String,
    pub name: String,
    pub precision: u8,
    pub created_at: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct BAccount {
    pub internal_id: i64,
    pub path: String,
    pub account_type: String,
    pub created_at: Option<NaiveDate>,
    pub closed_at: Option<NaiveDate>,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct BMember {
    pub internal_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct BChannel {
    pub internal_id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BPosting {
    pub internal_id: i64,
    pub account: String,
    pub amount: Decimal,
    pub commodity: String,
    pub cost: Option<Decimal>,
    pub cost_commodity: Option<String>,
    pub reimbursable: bool,
}

#[derive(Debug, Clone)]
pub struct BTransaction {
    pub internal_id: i64,
    pub date_time: NaiveDateTime,
    pub description: String,
    pub kind: String,
    pub member: Option<String>,
    pub tags: Vec<String>,
    pub channel_path: Vec<ChannelPathEntry>,
    pub postings: Vec<BPosting>,
    pub reversal_of: Option<ReversalInfo>,
}

#[derive(Debug, Clone)]
pub struct ChannelPathEntry {
    pub position: i32,
    pub channel: String,
    pub status: accounting::channel_path::ChannelPathStatus,
}

#[derive(Debug, Clone)]
pub struct ReversalInfo {
    pub posting_id: i64,
    pub target_posting_id: i64,
}

#[derive(Debug, Clone)]
pub struct BDocument {
    pub date: NaiveDate,
    pub account: String,
    pub filename: String,
    pub transaction_internal_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct BeancountData {
    pub commodities: Vec<BCommodity>,
    pub accounts: Vec<BAccount>,
    pub members: Vec<BMember>,
    pub channels: Vec<BChannel>,
    pub transactions: Vec<BTransaction>,
    pub documents: Vec<BDocument>,
}

impl BeancountData {
    pub fn account_map(&self) -> HashMap<String, &BAccount> {
        self.accounts.iter().map(|a| (a.path.clone(), a)).collect()
    }
}
