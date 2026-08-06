# Tasks: account-type-resolution

## 1. SQL 层：批量根名查询 + 改名保护

- [x] 1.1 `accounting-sql/src/repo/account.rs`：新增 `account_root_names_by_ids(account_ids, lang) -> Vec<(AccountId, String)>`（闭包表取 depth 最大祖先 + join account_names 系统名，单条 SQL）
- [x] 1.2 `database.rs` + `transaction.rs`：对应包装方法
- [x] 1.3 `database.rs`/`transaction.rs` 的 `account_rename`：执行前检查目标账户 `parent_id IS NULL AND is_system=1`，是则返回语义明确错误（含 "system root account" 固定文案）
- [x] 1.4 SQL 层测试：批量根名（多层级/多根/无对应语言名/空输入）、根账户改名拒绝（en/zh-CN）、非根账户改名正常

## 2. Service 层：批量解析切换

- [x] 2.1 `report/mod.rs` 的 `load_account_types`：改为一次 `account_root_names_by_ids` 调用 + 内存 `AccountType::from_str` 映射（无法推导的账户仍不出现于结果，语义不变）
- [x] 2.2 Service 层测试：load_account_types 结果与逐账户解析等价（多类型混合、无根名账户）；预算/攒钱计划/账户关闭既有测试原样通过（回归）

## 3. CLI 与 API：错误呈现

- [x] 3.1 `accounting-cli`：account rename 对系统根账户的错误映射为本地化词条（zh-CN/en）
- [x] 3.2 `accounting-api`：账户改名端点对系统根账户返回 400 + 本地化错误信息；补集成测试

## 4. 端到端验证

- [x] 4.1 `cargo fmt` + `cargo clippy --workspace --all-targets` 无警告 + `cargo test --workspace` 全绿
- [x] 4.2 CLI 冒烟：`account rename Assets --lang en` 报错且根名不变；预算/攒钱计划创建回归正常
