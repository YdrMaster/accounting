use serde::Serialize;
use tabled::{Table, Tabled};

/// 输出格式
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// 表格格式
    #[default]
    Table,
    /// JSON 格式
    Json,
}

/// 打印单个对象
pub fn print<T: Tabled + Serialize>(value: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value).unwrap()),
        OutputFormat::Table => println!("{}", Table::new([value])),
    }
}

/// 打印对象列表
pub fn print_vec<T: Tabled + Serialize>(values: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(values).unwrap()),
        OutputFormat::Table => println!("{}", Table::new(values)),
    }
}

/// 打印单行消息
pub fn print_line(msg: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{{\"result\":\"{}\"}}", msg),
        OutputFormat::Table => println!("{}", msg),
    }
}
