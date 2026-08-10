# accounting-beancount

Beancount 格式的导入 / 导出与互转。纯库 crate（无二进制），依赖 [`accounting`](../accounting) 与 [`accounting-sql`](../accounting-sql)，由 [`accounting-cli`](../accounting-cli) 与 [`accounting-api`](../accounting-api) 调用。

## 职责

- `parser`：解析 `.beancount` 文本为内部模型。
- `model`：Beancount 指令/账户/金额的内部表达。
- `import`：把 beancount 文件导入账簿（建账、账户映射、交易落库），兼容新文本格式与旧 JSON 格式备份。
- `export`：把账簿表数据导出为 beancount 文本（`transactions.beancount` + `attachments/`）。
- `generator`：导出文本的生成器（`channel_path` metadata 文本格式、`commodity` 指令日期等）。

## 用法（经 CLI）

```bash
# 导出到目录，生成 transactions.beancount 与 attachments/
accounting my.db beancount export ./output

# 从 beancount 文件导入
accounting my.db beancount import ./output/transactions.beancount
```

导出的 `channel_path` metadata 使用 CLI 渠道链路文本格式（`->`/`&`/`*`/`√`）；`commodity` 指令日期用数据库 `created_at`，缺失时回退 `1970-01-01`。详见 [`accounting-cli/README.md`](../accounting-cli/README.md) 的"Beancount 导出/导入"。

## 相关规格

活规格见 `beancount-export`、`beancount-import`；导入导出能力的决策来源见归档 `openspec/changes/archive/2026-06-27-beancount-import-export/`。

## 分层上下文

见根 [`README.md`](../README.md) 的"各 crate 文档"。
