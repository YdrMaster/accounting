//! 日期时间工具函数

use chrono::{NaiveDate, NaiveDateTime};

/// 将日期转换为当天 00:00:00。
///
/// # Panics
///
/// 此函数在语义上不可能 panic：0 时 0 分 0 秒是合法时间，
/// [`NaiveDate::and_hms_opt`] 必定返回 `Some`。使用 `expect` 而非 `unwrap`
/// 以明确表达这一安全断言。
pub fn start_of_day(date: NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(0, 0, 0)
        .expect("00:00:00 is always a valid time of day")
}

/// 将日期转换为当天 23:59:59。
///
/// # Panics
///
/// 此函数在语义上不可能 panic：23 时 59 分 59 秒是合法时间，
/// [`NaiveDate::and_hms_opt`] 必定返回 `Some`。使用 `expect` 而非 `unwrap`
/// 以明确表达这一安全断言。
pub fn end_of_day(date: NaiveDate) -> NaiveDateTime {
    date.and_hms_opt(23, 59, 59)
        .expect("23:59:59 is always a valid time of day")
}
