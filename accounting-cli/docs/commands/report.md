# report

财务报表查询。

## 子命令

```bash
accounting <DB_PATH> report bs
accounting <DB_PATH> report cashflow [--date <DATE>] [--period <PERIOD>] [--commodity <SYMBOL>]
```

## report bs

资产负债表，列出所有资产类账户的余额。

```bash
accounting my.db report bs
```

## report cashflow

资金流量表，统计指定周期内各账户的流入、流出和净流入。

| 选项 | 说明 |
|------|------|
| `--date` | 查询日期，默认今天 |
| `--period` | 周期类型，默认 `monthly` |
| `--commodity` | 币种符号，默认 `CNY` |

```bash
accounting my.db report cashflow
accounting my.db report cashflow --date 2024-06-01 --period monthly --commodity CNY
```

## 说明

- `bs` 只输出资产类账户余额。
- `cashflow` 的周期类型与 `budget` 相同：`daily`、`weekly-sun`、`weekly-mon`、`monthly`、`yearly`。
- 若不指定 `--commodity`，默认使用 `CNY`。
