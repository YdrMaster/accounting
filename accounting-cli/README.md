# accounting-cli

基于 clap + tokio 的命令行记账工具。

## 命令格式

```plaintext
accounting <DB_PATH> <COMMAND> [OPTIONS]
```

## 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--format` | 输出格式：`table`（表格）或 `json`（JSON） | `table` |
| `--lang` | 界面语言，如 `zh-CN`、`en` | 数据库配置 / 系统环境 |

## 初始化

```bash
# 创建新的数据库文件并初始化 schema
accounting my.db initialize

# 指定初始语言（影响内置账户名和界面语言）
accounting my.db initialize --lang zh-CN
```

> 数据库文件必须已存在才能执行其他命令。若文件不存在且命令不是 `initialize`，CLI 会直接报错退出。
>
> `initialize` 时指定的语言会持久化到数据库配置中，后续命令默认沿用该语言。`--lang` 临时覆盖仅影响当前命令的界面提示，不会修改已写入的内置账户名。

## 成员管理

```bash
# 列出成员
accounting my.db member list [--limit <N>] [--offset <N>]

# 添加成员
accounting my.db member add <NAME>

# 删除成员
accounting my.db member delete <ID>
```

## 账户管理

```bash
# 列出账户
accounting my.db account list [--type <TYPE>] [--limit <N>] [--offset <N>]

# 添加账户
accounting my.db account add <FULL_NAME> --type <TYPE> \
  [--parent <ID>] [--billing-day <D>] [--repayment-day <D>]

# 查看账户详情
accounting my.db account show <ID>

# 关闭账户（Asset/Liability 需余额为零）
accounting my.db account close <ID>

# 重新开启账户
accounting my.db account reopen <ID>

# 查询账户余额（含子账户聚合）
accounting my.db account balance <ID>
```

**账户类型**：`asset`, `liability`, `equity`, `income`, `expense`

## 商品/货币管理

```bash
# 列出商品
accounting my.db commodity list

# 添加商品
accounting my.db commodity add <SYMBOL> --name <NAME> --precision <N>
```

## 交易管理

### 添加交易

```bash
accounting my.db tx add \
  --date <YYYY-MM-DD> \
  --description <DESC> \
  --posting "<ACCOUNT>:<COMMODITY>:<AMOUNT>;<ACCOUNT>:<COMMODITY>:<AMOUNT>" \
  [--tag <TAG1>,<TAG2>] \
  [--member <ID>] \
  [--channel <ID>]
```

**posting 格式**：`账户全名:商品符号:金额`

多币种交易（换汇）需指定 cost：

```bash
--posting "Assets:USD:100;Assets:Cash:CNY:-700"
```

### 列出交易

```bash
accounting my.db tx list \
  [--from <YYYY-MM-DD>] [--to <YYYY-MM-DD>] \
  [--account <ID>] [--member <ID>] [--tag <TAG>] [--keyword <TEXT>] \
  [--limit <N>] [--offset <N>]
```

### 查看/删除/更新

```bash
# 查看交易详情（含分录）
accounting my.db tx show <ID>

# 删除交易
accounting my.db tx delete <ID>

# 更新交易（全量替换）
accounting my.db tx update <ID> \
  --date <YYYY-MM-DD> \
  --description <DESC> \
  --posting "..."
```

## 标签管理

```bash
# 列出标签
accounting my.db tag list

# 添加标签
accounting my.db tag add <NAME>

# 删除标签
accounting my.db tag delete <NAME>
```

## 报告查询

```bash
# 查询指定账户余额（含子账户）
accounting my.db report balance <ACCOUNT_ID>

# 资产负债表
accounting my.db report bs

# 损益表
accounting my.db report is
```

## 示例

### 初始化并创建账户

```bash
accounting my.db initialize
accounting my.db account add Assets:Cash --type asset
accounting my.db account add Expenses:Food --type expense
```

### 记录一笔支出

```bash
accounting my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:-50;Expenses:Food:50"
```

### 查看余额

```bash
accounting my.db account balance 1
accounting my.db report bs
```

### 导出为 JSON

```bash
accounting my.db --format json tx list --from 2024-01-01
```
