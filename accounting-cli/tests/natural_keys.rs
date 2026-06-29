use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn db_path() -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/tmp/accounting_cli_natural_keys_test_{}.db", id)
}

fn run(db: &str, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_accounting-cli"))
        .arg(db)
        .args(args)
        .output()
        .expect("failed to execute accounting-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!(
            "accounting-cli failed: db={} args={:?}\nstdout={}\nstderr={}",
            db, args, stdout, stderr
        );
    }
    stdout.to_string()
}

fn setup() -> String {
    let db = db_path();
    let _ = std::fs::remove_file(&db);
    run(&db, &["initialize", "--lang", "zh-CN"]);
    db
}

#[test]
fn test_member_natural_keys() {
    let db = setup();
    let out = run(&db, &["member", "add", "Alice"]);
    assert!(out.contains("成员已创建"));

    let out = run(&db, &["member", "list"]);
    assert!(out.contains("Alice"));

    let out = run(&db, &["member", "delete", "Alice"]);
    assert!(out.contains("成员已删除: Alice"));

    let out = run(&db, &["member", "list"]);
    assert!(!out.contains("Alice"));
}

#[test]
fn test_account_natural_keys() {
    let db = setup();
    let out = run(&db, &["account", "add", "Assets:Cash"]);
    assert!(out.contains("账户已创建"));

    let out = run(&db, &["account", "show", "Assets:Cash"]);
    assert!(out.contains("Cash"));

    let out = run(&db, &["account", "balance", "Assets:Cash"]);
    assert!(out.contains("余额为零"));
}

#[test]
fn test_tx_natural_keys_and_channel_path() {
    let db = setup();
    run(&db, &["member", "add", "Alice"]);
    run(&db, &["account", "add", "Assets:Cash"]);
    run(&db, &["account", "add", "Expenses:Food"]);

    let out = run(&db, &[
        "tx",
        "add",
        "--date",
        "2024-06-01",
        "--description",
        "午餐",
        "--posting",
        "Assets:Cash:CNY:-50",
        "--posting",
        "Expenses:Food:CNY:50",
        "--member",
        "Alice",
        "--channel",
        "支付宝",
    ]);
    assert!(out.contains("交易已创建"));

    let out = run(&db, &["tx", "list", "--member", "Alice", "--channel", "支付宝"]);
    assert!(out.contains("午餐"));

    let out = run(&db, &["tx", "show", "1"]);
    assert!(out.contains("支付宝"));
}

#[test]
fn test_budget_natural_keys() {
    let db = setup();
    run(&db, &["account", "add", "Expenses:Food"]);

    let out = run(&db, &[
        "budget",
        "create",
        "--name",
        "月度生活",
        "--period",
        "monthly",
        "--commodity",
        "CNY",
        "--limit",
        "Expenses:Food:1000",
    ]);
    assert!(out.contains("预算表已创建"));

    let out = run(&db, &["budget", "show", "月度生活"]);
    assert!(out.contains("月度生活"));

    let out = run(&db, &[
        "budget",
        "update",
        "月度生活",
        "--new-name",
        "月度餐饮",
        "--limit",
        "Expenses:Food:2000",
    ]);
    assert!(out.contains("预算表已更新"));

    let out = run(&db, &["budget", "delete", "月度餐饮"]);
    assert!(out.contains("预算表已删除"));
}

#[test]
fn test_mapping_natural_keys() {
    let db = setup();
    run(&db, &["member", "add", "Alice"]);
    run(&db, &["account", "add", "Expenses:Food"]);

    let out = run(&db, &[
        "mapping",
        "set",
        "--member",
        "Alice",
        "--channel",
        "支付宝",
        "--category",
        "收支:餐饮美食",
        "--account",
        "Expenses:Food",
    ]);
    assert!(out.contains("映射已设置"));

    let out = run(&db, &["mapping", "list", "--member", "Alice", "--channel", "支付宝"]);
    assert!(out.contains("收支:餐饮美食"));

    let out = run(&db, &[
        "mapping",
        "delete",
        "--member",
        "Alice",
        "--channel",
        "支付宝",
        "--category",
        "收支:餐饮美食",
    ]);
    assert!(out.contains("映射已删除"));
}

#[test]
fn test_report_cashflow_commodity_symbol() {
    let db = setup();
    let out = run(&db, &["report", "cash-flow", "--commodity", "CNY"]);
    assert!(out.contains("资金流量表"));
}
