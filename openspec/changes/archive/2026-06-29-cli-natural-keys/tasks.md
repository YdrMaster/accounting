## 1. Schema & Data Integrity

- [x] 1.1 Add `UNIQUE` constraint to `members.name` in `accounting-sql/src/schema.rs`
- [x] 1.2 Add `UNIQUE` constraint to `budgets.name` in `accounting-sql/src/schema.rs`
- [x] 1.3 Add duplicate-name detection for existing databases during initialization/startup
- [x] 1.4 Add SQLx repo function `member_get_by_name` in `accounting-sql/src/repo/member.rs`
- [x] 1.5 Add SQLx repo function `budget_get_by_name` in `accounting-sql/src/repo/budget.rs`
- [x] 1.6 Expose `member_get_by_name` through `SqliteDatabase`
- [x] 1.7 Expose `budget_get_by_name` through `SqliteDatabase`
- [x] 1.8 Add `MemberService::get_by_name` wrapper in `accounting-service`
- [x] 1.9 Add `BudgetService::get_by_name` wrapper in `accounting-service`

## 2. CLI Natural Key Resolver

- [x] 2.1 Create `accounting-cli/src/cmd/resolver.rs`
- [x] 2.2 Implement `resolve_member(db, name)` with human-readable errors
- [x] 2.3 Implement `resolve_account(db, path)` reusing `account_get_by_name`
- [x] 2.4 Implement `resolve_channel(db, name)` reusing `channel_get_by_name`
- [x] 2.5 Implement `resolve_commodity(db, symbol)` reusing `commodity_get_by_symbol`
- [x] 2.6 Implement `resolve_budget(db, name)`
- [x] 2.7 Add unit tests for resolver edge cases (not found, empty, duplicate safety)

## 3. Member Commands

- [x] 3.1 Change `MemberDeleteArgs` from `id: i64` to `name: String`
- [x] 3.2 Use `resolve_member` in `member delete`
- [x] 3.3 Update success/error messages to use member name instead of ID
- [x] 3.4 Add CLI tests for member delete by name

## 4. Account Commands

- [x] 4.1 Change `AccountShowArgs/CloseArgs/ReopenArgs/BalanceArgs` from `id: i64` to `path: String`
- [x] 4.2 Use `resolve_account` in account show/close/reopen/balance
- [x] 4.3 Change `AccountAddArgs` to use full path and remove `--parent-id`
- [x] 4.4 Use `AccountService::create_cascading` for `account add`
- [x] 4.5 Update `AccountListArgs` `--type` to accept account type name (or keep ID? confirm)
- [x] 4.6 Add CLI tests for account commands using paths

## 5. Transaction Commands

- [x] 5.1 Change `TxAddArgs.member` from `Option<i64>` to `Option<String>`
- [x] 5.2 Change `TxAddArgs.channel` from `Vec<String>` to `Option<String>`
- [x] 5.3 Rewrite `parse_channel_paths` to use `->` and `&` syntax with regex
- [x] 5.4 Use `resolve_member` and `resolve_channel` in `tx add`
- [x] 5.5 Apply same changes to `TxUpdateArgs`
- [x] 5.6 Change `TxListArgs` `--account`/`--member`/`--channel` from `Vec<i64>` to natural key strings
- [x] 5.7 Update `build_filter` to resolve natural keys to IDs
- [x] 5.8 Update `tx show` output to display channel names instead of channel IDs
- [x] 5.9 Add CLI tests for new channel path syntax and natural key filters
- [x] 5.10 Verify `--posting` supports repeated arguments (`--posting a --posting b`)
- [x] 5.11 Update README examples to demonstrate repeated `--posting` usage
- [x] 5.12 Add CLI tests for multiple `--posting` arguments

## 6. Mapping Commands

- [x] 6.1 Change `MappingSetArgs/ListArgs/DeleteArgs` `--member` from `i64` to `String`
- [x] 6.2 Use `resolve_member` in mapping commands
- [x] 6.3 Add CLI tests for mapping by member name

## 7. Budget Commands

- [x] 7.1 Change `BudgetShowArgs/UpdateArgs/DeleteArgs` from `budget_id: i64` to `name: String`
- [x] 7.2 Change `BudgetCreateArgs`/`BudgetUpdateArgs` `--commodity` from `i64` to `String`
- [x] 7.3 Use `resolve_budget` and `resolve_commodity` in budget commands
- [x] 7.4 Add CLI tests for budget commands using names and commodity symbols

## 8. Import & Report Commands

- [x] 8.1 Change `ImportArgs.member` from `i64` to `String`
- [x] 8.2 Use `resolve_member` in `import` command
- [x] 8.3 Change `CashFlowArgs.commodity` from `Option<i64>` to `Option<String>`
- [x] 8.4 Use `resolve_commodity` in `report cashflow`
- [x] 8.5 Add CLI tests for import and report cashflow with natural keys

## 9. Channel Name Validation

- [x] 9.1 Add validation in channel creation/update to reject names containing `->` or `&`
- [x] 9.2 Add repo/service tests for invalid channel names

## 10. Documentation

- [x] 10.1 Update `accounting-cli/README.md` command examples to use natural keys
- [x] 10.2 Update `plan/cli-design.md` to reflect new parameter formats
- [x] 10.3 Document new `->`/`&` channel path syntax in README

## 11. Integration & Verification

- [x] 11.1 Run `cargo build` for the workspace
- [x] 11.2 Run `cargo test` for affected crates
- [x] 11.3 Run CLI smoke tests manually with natural keys
- [x] 11.4 Validate OpenSpec change with `openspec validate cli-natural-keys`
