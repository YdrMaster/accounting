# 示例 3：导入账单并对账

本示例演示如何导入支付宝账单、查看导入的交易、修正分类，最后标记对账。

## 涉及命令

`member add`、`account add`、`mapping set`、`import`、`tx list`、`tx update`、`tx reconcile`

## 步骤

```bash
# 1. 初始化并创建成员
accounting my.db initialize --lang zh-CN
accounting my.db member add alice

# 2. 创建常用支出账户
accounting my.db account add Expenses:Food
accounting my.db account add Expenses:Transport
accounting my.db account add Expenses:Shopping

# 3. 配置导入映射，让支付宝分类自动落入本地账户
accounting my.db mapping set \
  --member alice \
  --channel alipay \
  --category "收支:餐饮美食" \
  --account "Expenses:Food"

accounting my.db mapping set \
  --member alice \
  --channel alipay \
  --category "收支:交通出行" \
  --account "Expenses:Transport"

# 4. 导入账单
accounting my.db import \
  --file /path/to/alipay_bill.csv \
  --source alipay \
  --member alice

# 5. 查看导入结果（会打印交易 ID）
accounting my.db tx list --tag 待处理 --member alice

# 6. 假设交易 ID 101 的分类不对，更新它
accounting my.db tx update 101 \
  --date 2024-06-01 \
  --description "修正后的描述" \
  --posting "Assets:Cash:CNY:-35" \
  --posting "Expenses:Shopping:CNY:35"

# 7. 对账确认（假设 path_id 为 202）
accounting my.db tx reconcile 202 --set true
```

## 说明

- 导入的交易会自动打上 `待处理` 标签，方便批量审查。
- `mapping set` 需要渠道存在；如果 `alipay` 渠道不存在，可以先通过其他方式创建，或检查初始化语言对应的渠道名称。
- `tx reconcile` 的 `PATH_ID` 是渠道链路记录 ID，不是交易 ID，通常从 `tx show <ID>` 的输出中获取。
