# mapping

账户映射管理。用于把账单导入时的分类（如支付宝的 "餐饮美食"）映射到本地账户路径。

## 子命令

```bash
accounting <DB_PATH> mapping set    --member <NAME> --channel <NAME> --category <KEY> --account <PATH>
accounting <DB_PATH> mapping list   --member <NAME> --channel <NAME>
accounting <DB_PATH> mapping delete --member <NAME> --channel <NAME> --category <KEY>
```

## mapping set

设置一条映射规则。

| 选项 | 说明 |
|------|------|
| `--member` | 成员名称 |
| `--channel` | 渠道名称 |
| `--category` | 导入源中的分类 key，如 `Expenses:餐饮美食` 或 `Assets:蚂蚁宝藏信用卡` |
| `--account` | 目标账户路径，如 `Expenses:Food` |

```bash
accounting my.db mapping set \
  --member alice \
  --channel 支付宝 \
  --category "Expenses:餐饮美食" \
  --account "Expenses:Food"

accounting my.db mapping set \
  --member alice \
  --channel 支付宝 \
  --category "Assets:蚂蚁宝藏信用卡" \
  --account "Assets:CreditCard"
```

## mapping list

列出某个成员和渠道下的所有映射。

```bash
accounting my.db mapping list --member alice --channel 支付宝
```

## mapping delete

删除指定映射。

```bash
accounting my.db mapping delete \
  --member alice \
  --channel 支付宝 \
  --category "Expenses:餐饮美食"
```

## 说明

- 映射以 `(member, channel, category)` 为唯一键。
- 设置映射时如果渠道不存在，部分实现会自动创建渠道。
- 导入账单前配置好映射，可以让交易自动记入正确的账户。
