# 示例 4：多渠道支出与映射

本示例演示如何记录经过多个支付渠道的交易，并通过渠道筛选交易。

## 涉及命令

`member add`、`tx add`、`tx list`、`account balance`、`mapping set`

## 步骤

```bash
# 1. 初始化并创建成员
accounting my.db initialize --lang zh-CN
accounting my.db member add alice

# 2. 创建账户
accounting my.db account add Assets:Cash
accounting my.db account add Assets:Alipay
accounting my.db account add Assets:BankCard
accounting my.db account add Expenses:Food

# 3. 设置映射：支付宝餐饮类 -> Expenses:Food
accounting my.db mapping set \
  --member alice \
  --channel 支付宝 \
  --category "Expenses:餐饮美食" \
  --account "Expenses:Food"

# 4. 记录一笔多渠道支出：淘宝 -> 支付宝 -> 花呗 & 建行卡
#    注意：& 是 shell 元字符，整个 --channel 参数需要加引号
accounting my.db tx add \
  --date 2024-06-10 \
  --description "网购零食" \
  --member alice \
  --channel "淘宝 -> 支付宝 -> 花呗 & 建行卡" \
  --posting "Assets:BankCard:CNY:-30" \
  --posting "Assets:Alipay:CNY:-40" \
  --posting "Expenses:Food:CNY:70"

# 5. 按渠道筛选交易
accounting my.db tx list --channel 支付宝 --member alice

# 6. 查看支付宝账户余额
accounting my.db account balance Assets:Alipay
```

## 说明

- 渠道链路语法：`->` 表示链路层级，`&` 仅允许在最后一级表示并行渠道。
- 因为 `&` 在 shell 中有特殊含义，必须给 `--channel` 的参数加引号。
- `tx list --channel` 可以按单个渠道名称过滤交易。
