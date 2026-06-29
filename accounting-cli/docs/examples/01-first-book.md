# 示例 1：建立第一本账簿

本示例演示如何从零初始化数据库、创建账户、记录第一笔交易，并查看余额和资产负债表。

## 涉及命令

`initialize`、`account add`、`commodity add`、`tx add`、`account balance`、`report bs`

## 步骤

```bash
# 1. 初始化数据库（中文内置账户）
accounting my.db initialize --lang zh-CN

# 2. 添加常用账户
accounting my.db account add Assets:Cash
accounting my.db account add Assets:Bank
accounting my.db account add Expenses:Food
accounting my.db account add Expenses:Transport

# 3. 添加一个外币商品（如需多币种记账）
accounting my.db commodity add USD --name "美元" --precision 2

# 4. 记录一笔午餐支出
accounting my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:CNY:-50" \
  --posting "Expenses:Food:CNY:50"

# 5. 查看现金账户余额
accounting my.db account balance Assets:Cash

# 6. 查看资产负债表
accounting my.db report bs
```

## 预期结果

- `Assets:Cash` 余额为 `-50 CNY`（现金流出）。
- `Expenses:Food` 余额为 `50 CNY`。
- 资产负债表列出资产类账户及其余额。

## 说明

- 初始化后默认已存在 `CNY` 商品，但每笔分录仍需显式写出商品符号：`账户路径:商品符号:金额`。
- `account add` 会自动创建不存在的父级账户。
