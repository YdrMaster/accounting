# import

从外部账单文件导入交易。导入的交易会自动打上系统标签 `待处理` / `pending`，便于后续审查。

## 用法

```bash
accounting <DB_PATH> import \
  --file <FILE> \
  --source <SOURCE> \
  --member <NAME>
```

## 选项

| 选项 | 说明 |
|------|------|
| `--file` | 账单文件路径 |
| `--source` | 账单来源，如 `alipay`。来源值同时对应一个渠道名称。 |
| `--member` | 成员名称 |

## 示例

```bash
accounting my.db import \
  --file /path/to/alipay_bill.csv \
  --source alipay \
  --member alice
```

## 输出

导入完成后会输出成功条数、跳过条数和交易 ID 列表：

```
导入完成：15 条交易，2 条跳过
已添加 "待处理" 标签，使用 `tx list --tag 待处理` 查看导入的交易
交易 ID: [101, 102, ...]
```

## 说明

- 导入前需要确保：
  1. 成员已存在（`member add`）。
  2. 对应来源的渠道已存在，或通过 `mapping set` 等方式创建。
  3. 已经配置好账户映射（`mapping set`），否则交易会进入 `Assets:Import:<channel>` / `Income:Import:<channel>` / `Expenses:Import:<channel>` 等 fallback 账户。
- 目前内置适配器支持 `alipay`。
- 跳过的记录通常是因为文件格式不匹配或金额解析失败，具体原因会在输出中列出。
