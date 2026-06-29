# budget

预算管理。按周期为指定账户设置限额，并跟踪实际支出。

## 子命令

```bash
accounting <DB_PATH> budget create --name <NAME> --period <PERIOD> --commodity <SYMBOL> --limit <PATH:AMOUNT> ...
accounting <DB_PATH> budget list
accounting <DB_PATH> budget show   <NAME> [--date <DATE>]
accounting <DB_PATH> budget update <NAME> [--new-name <NAME>] [--period <PERIOD>] [--commodity <SYMBOL>] [--limit <PATH:AMOUNT> ...]
accounting <DB_PATH> budget delete <NAME>
```

## 周期类型

| 值 | 说明 |
|------|------|
| `daily` | 按日 |
| `weekly-sun` | 按周，周日起算 |
| `weekly-mon` | 按周，周一起算 |
| `monthly` | 按月 |
| `yearly` | 按年 |

## budget create

创建预算表。

| 选项 | 说明 |
|------|------|
| `--name` | 预算表名称，必须唯一 |
| `--period` | 周期类型 |
| `--commodity` | 币种符号 |
| `--limit` | 限额，格式 `账户路径:金额`，可多次指定 |

```bash
accounting my.db budget create \
  --name "月度餐饮预算" \
  --period monthly \
  --commodity CNY \
  --limit "Expenses:Food:2000" \
  --limit "Expenses:Transport:500"
```

## budget list

列出所有预算表。

```bash
accounting my.db budget list
```

## budget show

查看某预算表在指定日期的执行情况。默认使用今天。

```bash
accounting my.db budget show "月度餐饮预算"
accounting my.db budget show "月度餐饮预算" --date 2024-06-15
```

## budget update

更新预算表。若指定 `--limit`，则会替换所有限额；否则保留原有限额。

```bash
accounting my.db budget update "月度餐饮预算" --commodity CNY
accounting my.db budget update "月度餐饮预算" --limit "Expenses:Food:2500"
```

## budget delete

删除预算表。

```bash
accounting my.db budget delete "月度餐饮预算"
```

## 说明

- 预算表名称有 `UNIQUE` 约束，不能重复。
- 实际金额按周期统计归属到限额账户的交易分录。
- 超支时 `show` 输出会带 `⚠ 超支` 提示。
