# Tasks

## 1. Data model & core types

- [x] 1.1 Define `ChannelPathStatus` enum in `accounting/src/channel_path.rs` and replace `reconciled: bool` with `status: ChannelPathStatus` on `ChannelPath` and `ChannelPathNode`
- [x] 1.2 Update accounting API DTOs in `accounting-api/src/transaction.rs` to expose `status: String` for channel path nodes
- [x] 1.3 Update `accounting-sql/src/repo/channel_path.rs` to read/write `channel_paths.status` instead of `reconciled`

## 2. Service layer

- [x] 2.1 Update `accounting-service/src/transaction_service.rs` to accept channel path status and provide status update methods
- [x] 2.2 Set third-party import channel path status to `pending` in `accounting-service/src/import_service.rs`

## 3. CLI

- [x] 3.1 Parse `*` / `√` suffixes in `accounting-cli/src/cmd/tx.rs` for `tx add` / `tx update --channel`
- [x] 3.2 Adjust `tx reconcile` semantics to set channel path status to `verified` (with optional `--unset` to `default`)

## 4. Beancount

- [x] 4.1 Use `commodities.created_at` for commodity directive dates in `accounting-beancount/src/generator.rs`
- [x] 4.2 Export `channel_path` metadata as CLI text format with `*` / `√` suffixes
- [x] 4.3 Import `channel_path` text format and keep legacy JSON compatibility in `accounting-beancount/src/parser.rs`

## 5. Web / tests

- [x] 5.1 Update frontend channel path display from `reconciled` to `status`
- [x] 5.2 Update Rust unit/integration tests for new status semantics and commodity dates

## 6. Documentation & verification

- [x] 6.1 Update README / CLI docs for new channel syntax and `tx reconcile` usage
- [x] 6.2 Run end-to-end verification: alipay import → export → re-import → reconcile

## 7. Post-verification fixes

- [x] 7.1 Add channel alias/case-insensitive resolution so `--source alipay` works in zh-CN locale and `--source 支付宝` works in en locale
- [x] 7.2 Change `tx reconcile` from `<path_id>` to `<tx_id> --channel <channel>` with `--unset` support
