use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn db_path() -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/tmp/accounting_cli_saving_plan_test_{id}.db")
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
            "accounting-cli failed: db={db} args={args:?}\nstdout={stdout}\nstderr={stderr}"
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

/// 建立含五个资金账户的库：A 3000 / B 1000 / C 2000 / D 1000 / E 500
fn setup_funded() -> String {
    let db = db_path();
    let _ = std::fs::remove_file(&db);
    run(&db, &["initialize"]);
    run(&db, &["member", "add", "Alice"]);
    run(&db, &["account", "add", "Income:Salary"]);
    for (path, amount) in [
        ("Assets:A", "3000"),
        ("Assets:B", "1000"),
        ("Assets:C", "2000"),
        ("Assets:D", "1000"),
        ("Assets:E", "500"),
    ] {
        run(&db, &["account", "add", path]);
        let neg = format!("Income:Salary:CNY:-{amount}");
        let pos = format!("{path}:CNY:{amount}");
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
                &neg,
                "--posting",
                &pos,
                "--member",
                "Alice",
            ],
        );
    }
    db
}

#[test]
fn test_saving_plan_list_satisfaction_column() {
    // spec：计划 1（{A,B} 目标 3000）检查点早于计划 2（{A,E} 目标 2000），
    // A 3000、B 1000、E 500 → 计划 1 满足率 100，计划 2 满足率 75
    let db = setup_funded();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划1",
            "--deadline",
            "2099-08-31",
            "--commodity",
            "CNY",
            "--target",
            "3000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:B",
        ],
    );
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划2",
            "--deadline",
            "2099-10-31",
            "--commodity",
            "CNY",
            "--target",
            "2000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:E",
        ],
    );

    let out = run(&db, &["saving-plan", "list"]);
    assert!(out.contains("Satisfaction"), "list 输出: {out}");
    let line1 = out
        .lines()
        .find(|l| l.contains("计划1"))
        .expect("list 应包含计划1");
    assert!(line1.contains("100"), "计划1 满足率应为 100: {line1}");
    let line2 = out
        .lines()
        .find(|l| l.contains("计划2"))
        .expect("list 应包含计划2");
    assert!(line2.contains("75"), "计划2 满足率应为 75: {line2}");
    // 未归一化的 "100.00"/"75.00" 不应出现
    assert!(!line1.contains("100.00"), "满足率应归一化: {line1}");
    assert!(!line2.contains("75.00"), "满足率应归一化: {line2}");
}

#[test]
fn test_saving_plan_show_allocation_detail() {
    // spec：计划 1（{A,B} 目标 3000）先于计划 2（{A,E} 目标 2000），
    // show 计划1 → A（余额 3000、被占用 0、分配 2000）、B（余额 1000、被占用 0、分配 1000）
    let db = setup_funded();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划1",
            "--deadline",
            "2099-08-31",
            "--commodity",
            "CNY",
            "--target",
            "3000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:B",
        ],
    );
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划2",
            "--deadline",
            "2099-10-31",
            "--commodity",
            "CNY",
            "--target",
            "2000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:E",
        ],
    );

    let out = run(&db, &["saving-plan", "show", "计划1"]);
    assert!(out.contains("满足率：100%"), "show 输出: {out}");
    assert!(out.contains("分配明细"), "show 输出: {out}");
    assert!(
        out.contains("资产:A：余额 3000，被占用 0，本计划分配 2000"),
        "show 输出: {out}"
    );
    assert!(
        out.contains("资产:B：余额 1000，被占用 0，本计划分配 1000"),
        "show 输出: {out}"
    );
}

#[test]
fn test_saving_plan_show_allocation_classic_three_plans() {
    // 经典三计划例：计划1{A,B}3000、计划2{C,D}4000、计划3{A,E}2000，
    // 余额 A3000/B1000/C2000/D1000/E500 → 计划3 满足率 75，
    // A（余额 3000、被占用 2000、分配 1000）、E（余额 500、被占用 0、分配 500）
    let db = setup_funded();
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划1",
            "--deadline",
            "2099-08-31",
            "--commodity",
            "CNY",
            "--target",
            "3000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:B",
        ],
    );
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划2",
            "--deadline",
            "2099-09-30",
            "--commodity",
            "CNY",
            "--target",
            "4000",
            "--account",
            "Assets:C",
            "--account",
            "Assets:D",
        ],
    );
    run(
        &db,
        &[
            "saving-plan",
            "create",
            "--name",
            "计划3",
            "--deadline",
            "2099-10-31",
            "--commodity",
            "CNY",
            "--target",
            "2000",
            "--account",
            "Assets:A",
            "--account",
            "Assets:E",
        ],
    );

    let out = run(&db, &["saving-plan", "show", "计划3"]);
    assert!(out.contains("满足率：75%"), "show 输出: {out}");
    assert!(
        out.contains("资产:A：余额 3000，被占用 2000，本计划分配 1000"),
        "show 输出: {out}"
    );
    assert!(
        out.contains("资产:E：余额 500，被占用 0，本计划分配 500"),
        "show 输出: {out}"
    );
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
