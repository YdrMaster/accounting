use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn db_path() -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/tmp/accounting_cli_account_test_{id}.db")
}

fn run(db: &str, args: &[&str], lang: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_accounting-cli"))
        .arg(db)
        .args(args)
        .args(["--lang", lang])
        .output()
        .expect("failed to execute accounting-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!("accounting-cli failed: db={db} args={args:?}\nstdout={stdout}\nstderr={stderr}");
    }
    stdout.to_string()
}

fn run_fail(db: &str, args: &[&str], lang: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_accounting-cli"))
        .arg(db)
        .args(args)
        .args(["--lang", lang])
        .output()
        .expect("failed to execute accounting-cli");

    assert!(
        !output.status.success(),
        "expected failure: db={} args={:?}\nstdout={}",
        db,
        args,
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn setup() -> String {
    let db = db_path();
    let _ = std::fs::remove_file(&db);
    run(&db, &["initialize"], "en");
    db
}

#[test]
fn rename_system_root_account_rejected_en() {
    let db = setup();
    let stderr = run_fail(&db, &["account", "rename", "Assets", "MyAssets"], "en");
    assert!(
        stderr.contains("System root account cannot be renamed"),
        "unexpected stderr: {stderr}"
    );
    // 根账户名保持不变
    let out = run(&db, &["account", "show", "Assets"], "en");
    assert!(out.contains("Assets"), "unexpected output: {out}");
}

#[test]
fn rename_system_root_account_rejected_zh() {
    let db = setup();
    let stderr = run_fail(&db, &["account", "rename", "支出", "花销"], "zh-CN");
    assert!(
        stderr.contains("系统根账户不可改名"),
        "unexpected stderr: {stderr}"
    );
    // 根账户名保持不变
    let out = run(&db, &["account", "show", "支出"], "zh-CN");
    assert!(out.contains("支出"), "unexpected output: {out}");
}

#[test]
fn rename_non_root_account_ok() {
    let db = setup();
    run(&db, &["account", "add", "Assets:Bank"], "en");
    let out = run(&db, &["account", "rename", "Assets:Bank", "NewBank"], "en");
    assert!(out.contains("NewBank"), "unexpected output: {out}");
    // 新名字可命中
    let out = run(&db, &["account", "show", "Assets:NewBank"], "en");
    assert!(out.contains("NewBank"), "unexpected output: {out}");
}
