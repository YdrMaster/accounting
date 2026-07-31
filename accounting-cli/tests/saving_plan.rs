use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn db_path() -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/tmp/accounting_cli_saving_plan_test_{}.db", id)
}

fn run(db: &str, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_accounting-cli"))
        .arg(db)
        .args(args)
        .args(["--lang", "zh-CN"])
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

fn run_fail(db: &str, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_accounting-cli"))
        .arg(db)
        .args(args)
        .args(["--lang", "zh-CN"])
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
    run(&db, &["initialize"]);
    run(&db, &["account", "add", "Assets:Alipay"]);
    run(&db, &["account", "add", "Assets:WeChat"]);
    run(&db, &["account", "add", "Expenses:Food"]);
    db
}

#[test]
fn test_saving_plan_create_one_off_and_list() {
    let db = setup();

    let out = run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--deadline",
            "2026-09-30",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
            "--account",
            "Assets:WeChat",
        ],
    );
    assert!(out.contains("攒钱计划已创建"));

    // list 表格：ID/名称/周期/截止日期/目标金额/币种（币种显示符号而非内部 ID）
    let out = run(&db, &["saving-plan", "list"]);
    assert!(out.contains("旅行基金"));
    assert!(out.contains("2026-09-30"));
    assert!(out.contains("5000"));
    assert!(out.contains("CNY"));
}

#[test]
fn test_saving_plan_create_recurring() {
    let db = setup();

    let out = run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "房租备用金",
            "--period",
            "monthly",
            "--commodity",
            "CNY",
            "--target",
            "6000",
            "--account",
            "Assets:Alipay",
        ],
    );
    assert!(out.contains("攒钱计划已创建"));

    let out = run(&db, &["saving-plan", "list"]);
    assert!(out.contains("房租备用金"));
    assert!(out.contains("Monthly"));
}

#[test]
fn test_saving_plan_create_invalid_period() {
    let db = setup();
    let err = run_fail(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "测试",
            "--period",
            "biweekly",
            "--commodity",
            "CNY",
            "--target",
            "100",
            "--account",
            "Assets:Alipay",
        ],
    );
    assert!(err.contains("未知周期类型"));
    assert!(err.contains("daily"));
}

#[test]
fn test_saving_plan_create_invalid_deadline() {
    let db = setup();
    let err = run_fail(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "测试",
            "--deadline",
            "2026/09/30",
            "--commodity",
            "CNY",
            "--target",
            "100",
            "--account",
            "Assets:Alipay",
        ],
    );
    assert!(err.contains("日期格式应为 YYYY-MM-DD"));
}

#[test]
fn test_saving_plan_create_non_asset_account_rejected() {
    let db = setup();
    let err = run_fail(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "测试",
            "--commodity",
            "CNY",
            "--target",
            "100",
            "--account",
            "Expenses:Food",
        ],
    );
    assert!(err.contains("账户必须位于资产根账户子树内"));
}

#[test]
fn test_saving_plan_create_account_not_found() {
    let db = setup();
    let err = run_fail(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "测试",
            "--commodity",
            "CNY",
            "--target",
            "100",
            "--account",
            "Assets:NotExist",
        ],
    );
    assert!(err.contains("账户不存在"));
}

#[test]
fn test_saving_plan_show_one_off_status() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--deadline",
            "2026-09-30",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
        ],
    );

    let out = run(
        &db,
        &["saving-plan", "show", "旅行基金", "--date", "2026-08-15"],
    );
    assert!(out.contains("旅行基金"));
    assert!(out.contains("目标金额：5000"));
    assert!(out.contains("当前余额：0"));
    assert!(out.contains("缺口：5000"));
    assert!(out.contains("是否达标：否"));
    // 未达标提醒
    assert!(out.contains("未达标提醒"));
    // 一次性计划不显示周期区间
    assert!(!out.contains("周期："));
}

#[test]
fn test_saving_plan_show_recurring_period_range() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "房租备用金",
            "--period",
            "monthly",
            "--commodity",
            "CNY",
            "--target",
            "6000",
            "--account",
            "Assets:Alipay",
        ],
    );

    let out = run(
        &db,
        &["saving-plan", "show", "房租备用金", "--date", "2026-08-15"],
    );
    assert!(out.contains("周期：2026-08-01 ~ 2026-08-31"));
}

#[test]
fn test_saving_plan_show_expired() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--deadline",
            "2020-01-01",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
        ],
    );

    let out = run(
        &db,
        &["saving-plan", "show", "旅行基金", "--date", "2026-08-15"],
    );
    assert!(out.contains("已失效"));
}

#[test]
fn test_saving_plan_show_met() {
    let db = setup();
    run(&db, &["member", "add", "Alice"]);
    run(&db, &["account", "add", "Income:Salary"]);
    run(
        &db,
        &[
            "tx",
            "add",
            "--date",
            "2026-06-01",
            "--description",
            "工资",
            "--posting",
            "Income:Salary:CNY:-6000",
            "--posting",
            "Assets:Alipay:CNY:6000",
            "--member",
            "Alice",
        ],
    );
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
        ],
    );

    let out = run(
        &db,
        &["saving-plan", "show", "旅行基金", "--date", "2026-08-15"],
    );
    assert!(out.contains("当前余额：6000"));
    assert!(out.contains("是否达标：是"));
    assert!(!out.contains("未达标提醒"));
}

#[test]
fn test_saving_plan_show_not_found() {
    let db = setup();
    let err = run_fail(&db, &["saving-plan", "show", "不存在的计划"]);
    assert!(err.contains("攒钱计划 '不存在的计划' 不存在"));
}

#[test]
fn test_saving_plan_update() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
            "--account",
            "Assets:WeChat",
        ],
    );

    // 更新名称，其余字段保持不变
    let out = run(
        &db,
        &[
            "saving-plan",
            "update",
            "旅行基金",
            "--name",
            "欧洲旅行基金",
        ],
    );
    assert!(out.contains("攒钱计划已更新"));
    let out = run(&db, &["saving-plan", "list"]);
    assert!(out.contains("欧洲旅行基金"));
    assert!(!out.contains("旅行基金 \""));

    // 更新目标金额
    run(
        &db,
        &["saving-plan", "update", "欧洲旅行基金", "--target", "8000"],
    );
    let out = run(
        &db,
        &[
            "saving-plan",
            "show",
            "欧洲旅行基金",
            "--date",
            "2026-08-15",
        ],
    );
    assert!(out.contains("目标金额：8000"));

    // 替换账户集合（仅剩 Assets:WeChat）
    run(
        &db,
        &[
            "saving-plan",
            "update",
            "欧洲旅行基金",
            "--account",
            "Assets:WeChat",
        ],
    );

    // 设置 deadline → 过期标注出现
    run(
        &db,
        &[
            "saving-plan",
            "update",
            "欧洲旅行基金",
            "--deadline",
            "2020-01-01",
        ],
    );
    let out = run(
        &db,
        &[
            "saving-plan",
            "show",
            "欧洲旅行基金",
            "--date",
            "2026-08-15",
        ],
    );
    assert!(out.contains("已失效"));

    // --deadline none 清除 → 过期标注消失
    run(
        &db,
        &[
            "saving-plan",
            "update",
            "欧洲旅行基金",
            "--deadline",
            "none",
        ],
    );
    let out = run(
        &db,
        &[
            "saving-plan",
            "show",
            "欧洲旅行基金",
            "--date",
            "2026-08-15",
        ],
    );
    assert!(!out.contains("已失效"));
}

#[test]
fn test_saving_plan_update_not_found() {
    let db = setup();
    let err = run_fail(
        &db,
        &["saving-plan", "update", "不存在的计划", "--name", "新名称"],
    );
    assert!(err.contains("攒钱计划 '不存在的计划' 不存在"));
}

#[test]
fn test_saving_plan_delete() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
        ],
    );

    let out = run(&db, &["saving-plan", "delete", "旅行基金"]);
    assert!(out.contains("攒钱计划已删除"));

    let out = run(&db, &["saving-plan", "list"]);
    assert!(out.contains("暂无攒钱计划"));

    let err = run_fail(&db, &["saving-plan", "delete", "旅行基金"]);
    assert!(err.contains("攒钱计划 '旅行基金' 不存在"));
}

#[test]
fn test_budget_one_off_and_deadline() {
    let db = setup();

    // 一次性预算（不提供 --period）+ deadline
    let out = run(
        &db,
        &[
            "budget",
            "create",
            "--name",
            "旅行预算",
            "--deadline",
            "2026-09-30",
            "--commodity",
            "CNY",
            "--limit",
            "Expenses:Food:8000",
        ],
    );
    assert!(out.contains("预算表已创建"));

    // 一次性预算不显示周期区间
    let out = run(&db, &["budget", "show", "旅行预算", "--date", "2026-08-15"]);
    assert!(out.contains("旅行预算"));
    assert!(!out.contains("周期："));
    assert!(!out.contains("已失效"));

    // 设置过去的 deadline → 已失效标注
    run(
        &db,
        &["budget", "update", "旅行预算", "--deadline", "2020-01-01"],
    );
    let out = run(&db, &["budget", "show", "旅行预算", "--date", "2026-08-15"]);
    assert!(out.contains("已失效"));

    // --deadline none 清除 → 已失效标注消失
    run(&db, &["budget", "update", "旅行预算", "--deadline", "none"]);
    let out = run(&db, &["budget", "show", "旅行预算", "--date", "2026-08-15"]);
    assert!(!out.contains("已失效"));
}

#[test]
fn test_budget_update_one_off_keeps_period() {
    let db = setup();

    // 一次性预算 update 时未指定 --period 沿用旧值（仍为一次性），不再报错
    run(
        &db,
        &[
            "budget",
            "create",
            "--name",
            "旅行预算",
            "--commodity",
            "CNY",
            "--limit",
            "Expenses:Food:8000",
        ],
    );
    let out = run(
        &db,
        &[
            "budget",
            "update",
            "旅行预算",
            "--limit",
            "Expenses:Food:9000",
        ],
    );
    assert!(out.contains("预算表已更新"));

    let out = run(&db, &["budget", "show", "旅行预算", "--date", "2026-08-15"]);
    assert!(!out.contains("周期："));
    assert!(out.contains("9000"));
}

#[test]
fn test_saving_plan_create_period_once_is_one_off() {
    let db = setup();

    // --period once 与不提供 --period 等价：一次性计划
    let out = run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "旅行基金",
            "--period",
            "once",
            "--commodity",
            "CNY",
            "--target",
            "5000",
            "--account",
            "Assets:Alipay",
        ],
    );
    assert!(out.contains("攒钱计划已创建"));

    let out = run(
        &db,
        &["saving-plan", "show", "旅行基金", "--date", "2026-08-15"],
    );
    assert!(!out.contains("周期："));
}

#[test]
fn test_saving_plan_update_period_once_clears_recurrence() {
    let db = setup();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "房租备用金",
            "--period",
            "monthly",
            "--commodity",
            "CNY",
            "--target",
            "6000",
            "--account",
            "Assets:Alipay",
        ],
    );
    let out = run(
        &db,
        &["saving-plan", "show", "房租备用金", "--date", "2026-08-15"],
    );
    assert!(out.contains("周期："));

    // --period once 置为一次性（清除循环）
    run(
        &db,
        &["saving-plan", "update", "房租备用金", "--period", "once"],
    );
    let out = run(
        &db,
        &["saving-plan", "show", "房租备用金", "--date", "2026-08-15"],
    );
    assert!(!out.contains("周期："));
}

#[test]
fn test_budget_create_period_once_is_one_off() {
    let db = setup();

    let out = run(
        &db,
        &[
            "budget",
            "create",
            "--name",
            "旅行预算",
            "--period",
            "once",
            "--commodity",
            "CNY",
            "--limit",
            "Expenses:Food:8000",
        ],
    );
    assert!(out.contains("预算表已创建"));

    let out = run(&db, &["budget", "show", "旅行预算", "--date", "2026-08-15"]);
    assert!(!out.contains("周期："));
}

#[test]
fn test_budget_update_period_once_clears_recurrence() {
    let db = setup();
    run(
        &db,
        &[
            "budget",
            "create",
            "--name",
            "月度生活",
            "--period",
            "monthly",
            "--commodity",
            "CNY",
            "--limit",
            "Expenses:Food:2000",
        ],
    );
    let out = run(&db, &["budget", "show", "月度生活", "--date", "2026-08-15"]);
    assert!(out.contains("周期："));

    // --period once 置为一次性（清除循环）
    run(&db, &["budget", "update", "月度生活", "--period", "once"]);
    let out = run(&db, &["budget", "show", "月度生活", "--date", "2026-08-15"]);
    assert!(!out.contains("周期："));
}
