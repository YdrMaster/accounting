# accounting-cli 文档

`accounting-cli` 是一个基于 SQLite 的命令行记账工具。所有数据保存在一个数据库文件中，命令通过自然键（名称、路径、符号）操作实体，用户通常不需要记住数据库里的实体 ID。

## 基本用法

```bash
accounting <DB_PATH> <COMMAND> [OPTIONS]
```

- `DB_PATH`：SQLite 数据库文件路径。
- 如果数据库不存在，必须先执行 `initialize`。
- 其他命令要求数据库文件已经存在，否则直接报错退出。

## 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--format` | 输出格式：`table` 或 `json` | `table` |
| `--lang` | 当前命令的界面语言，如 `zh-CN`、`en` | 数据库中保存的语言配置 |

`initialize` 时指定的 `--lang` 会持久化到数据库。后续命令默认沿用该语言；用 `--lang` 可临时覆盖当前命令的提示语言，但不会改写已初始化好的内置账户名。

## 输出格式

- `table`：使用表格输出，适合人工阅读。
- `json`：输出 JSON，便于脚本处理。

示例：

```bash
accounting my.db --format json tx list --from 2024-01-01
```

## 命令索引

- [`initialize`](commands/initialize.md)：初始化数据库
- [`config`](commands/config.md)：配置导入导出
- [`member`](commands/member.md)：成员管理
- [`account`](commands/account.md)：账户管理
- [`commodity`](commands/commodity.md)：商品/币种管理
- [`tag`](commands/tag.md)：标签管理
- [`tx`](commands/tx.md)：交易管理
- [`mapping`](commands/mapping.md)：账户映射管理
- [`budget`](commands/budget.md)：预算管理
- [`import`](commands/import.md)：账单导入
- [`report`](commands/report.md)：财务报表
- [`beancount`](commands/beancount.md)：Beancount 导入导出

## 操作示例

- [示例 1：建立第一本账簿](examples/01-first-book.md)
- [示例 2：预算跟踪](examples/02-budget-tracking.md)
- [示例 3：导入账单并对账](examples/03-import-and-reconcile.md)
- [示例 4：多渠道支出与映射](examples/04-multi-channel-expense.md)
- [示例 5：Beancount 互导](examples/05-beancount-roundtrip.md)
