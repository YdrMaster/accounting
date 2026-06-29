# tx

交易管理。交易由一组分录（posting）组成，分录金额之和必须为零。

## 子命令

```bash
accounting <DB_PATH> tx add      --date <DATE> --description <DESC> --posting <P> ... [OPTIONS]
accounting <DB_PATH> tx list     [OPTIONS]
accounting <DB_PATH> tx show     <ID>
accounting <DB_PATH> tx delete   <ID>
accounting <DB_PATH> tx update   <ID> --date <DATE> --description <DESC> --posting <P> ... [OPTIONS]
accounting <DB_PATH> tx reconcile <PATH_ID> [--set <BOOL>]
```

## 分录格式

```
<账户路径>:<商品符号>:<金额>
```

示例：

```bash
--posting "Assets:Cash:CNY:-50"
--posting "Expenses:Food:CNY:50"
```

### 多币种 / 成本

支持 5 段格式记录成本：

```
<账户路径>:<商品符号>:<金额>:<成本商品符号>:<成本金额>
```

示例：用 700 CNY 换取 100 USD：

```bash
--posting "Assets:Bank:USD:100:CNY:700" \
--posting "Assets:Cash:CNY:-700"
```

### 多次指定

`--posting` 和 `--tag` 都可以多次使用，也可以用分隔符写在同一个参数里：

```bash
--posting "Assets:Cash:CNY:-50" --posting "Expenses:Food:CNY:50"
--posting "Assets:Cash:CNY:-50;Expenses:Food:CNY:50"
--tag work,travel
--tag work --tag travel
```

## tx add

添加一笔交易。

| 选项 | 说明 |
|------|------|
| `--date` | 交易日期，格式 `YYYY-MM-DD` 或 `YYYY-MM-DD HH:MM:SS` |
| `--description` | 交易描述 |
| `--posting` | 分录，可多次指定 |
| `--tag` | 标签，可多次指定 |
| `--member` | 成员名称 |
| `--channel` | 渠道链路，语法见下 |

```bash
accounting my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:CNY:-50" \
  --posting "Expenses:Food:CNY:50"
```

### 渠道链路语法

```
一级 -> 二级 -> 三级&三级并行
```

- `->` 表示链路下一级，前后可以有空格。
- `&` 只能出现在最后一级，表示多个并行渠道。
- 因为 `&` 是 shell 元字符，整条表达式需要加引号。

示例：

```bash
--channel "淘宝 -> 支付宝 -> 花呗 & 建行卡"
```

## tx list

列出交易，支持多种自然键过滤。

| 选项 | 说明 |
|------|------|
| `--from` | 起始日期 `YYYY-MM-DD` |
| `--to` | 结束日期 `YYYY-MM-DD` |
| `--account` | 账户路径，可多次指定 |
| `--member` | 成员名称，可多次指定 |
| `--channel` | 渠道名称，可多次指定 |
| `--tag` | 标签名称，可多次指定 |
| `--keyword` | 描述关键字 |
| `--limit` | 最大条数 |
| `--offset` | 跳过条数 |

```bash
accounting my.db tx list --from 2024-06-01 --to 2024-06-30 --member alice
accounting my.db tx list --account Expenses:Food --tag travel
```

## tx show

查看交易详情，包括交易本身、渠道链路节点和分录。

| 参数 | 说明 |
|------|------|
| `ID` | 交易 ID |

```bash
accounting my.db tx show 42
```

## tx delete

删除交易。

```bash
accounting my.db tx delete 42
```

## tx update

全量更新交易。所有字段都会替换，未提供的字段会被清空或重置。

```bash
accounting my.db tx update 42 \
  --date 2024-06-02 \
  --description "晚餐" \
  --posting "Assets:Cash:CNY:-80" \
  --posting "Expenses:Food:CNY:80"
```

## tx reconcile

标记某条渠道链路记录为已对账或未对账。

| 参数 | 说明 |
|------|------|
| `PATH_ID` | 渠道链路记录 ID（`channel_path` 表 ID） |

| 选项 | 说明 |
|------|------|
| `--set` | `true` 标记已对账，`false` 取消对账；默认 `true` |

```bash
accounting my.db tx reconcile 123 --set true
accounting my.db tx reconcile 123 --set false
```

## 说明

- 交易和分录没有稳定的自然键，因此 `show / delete / update / reconcile` 仍使用数据库 ID。
- `tx add` 成功后会在输出中打印交易 ID，便于后续操作。
- 分录借贷金额之和必须为零，否则提交会失败。
