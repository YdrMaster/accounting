# account

账户管理。账户采用层级路径组织，支持级联创建父级。

## 子命令

```bash
accounting <DB_PATH> account list    [--type <ROOT>] [--limit <N>] [--offset <N>]
accounting <DB_PATH> account add     <PATH> [--billing-day <D>] [--repayment-day <D>]
accounting <DB_PATH> account show    <PATH>
accounting <DB_PATH> account close   <PATH>
accounting <DB_PATH> account reopen  <PATH>
accounting <DB_PATH> account balance <PATH>
```

## 账户类型

账户类型由根节点名称决定，支持以下类型（大小写不敏感）：

| 类型 | 英文名 | 中文名 |
|------|--------|--------|
| 资产 | `asset` / `assets` | `资产` |
| 权益 | `equity` | `权益` |
| 收入 | `income` | `收入` |
| 支出 | `expense` / `expenses` | `支出` |
| 导入 | `import` / `imports` | `导入` |

## account list

列出账户。

| 选项 | 说明 |
|------|------|
| `--type` | 只列出某个根账户下的子树，如 `Assets`、`支出` |
| `--limit` | 最大条数 |
| `--offset` | 跳过条数 |

```bash
accounting my.db account list
accounting my.db account list --type Assets
```

## account add

添加账户。`PATH` 为完整路径，父级不存在时会自动级联创建。

| 参数 | 说明 |
|------|------|
| `PATH` | 账户完整路径，如 `Assets:Cash`、`支出:餐饮:午餐` |

| 选项 | 说明 |
|------|------|
| `--billing-day` | 账单日（常用于信用卡类账户） |
| `--repayment-day` | 还款日（常用于信用卡类账户） |

```bash
accounting my.db account add Assets:Cash
accounting my.db account add Assets:CreditCard --billing-day 5 --repayment-day 25
```

## account show

查看单个账户详情。

```bash
accounting my.db account show Assets:Cash
```

## account close

关闭账户。资产类账户要求余额为零，其他类型账户可随时关闭。

```bash
accounting my.db account close Assets:Cash
```

## account reopen

重新开启已关闭的账户。

```bash
accounting my.db account reopen Assets:Cash
```

## account balance

查询账户余额，结果会聚合所有子账户的余额。

```bash
accounting my.db account balance Assets
accounting my.db account balance Expenses:Food
```

## 说明

- 账户路径分隔符为 `:`，路径中可包含多层父级。
- 关闭资产类账户时若余额不为零会报错。
- `account balance` 对无余额的账户会显示为零余额提示。
