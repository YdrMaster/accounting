# 示例 2：预算跟踪

本示例演示如何创建月度预算、记录若干笔支出，并查看预算执行情况。

## 涉及命令

`budget create`、`tx add`、`budget show`

## 步骤

```bash
# 1. 初始化数据库
accounting my.db initialize --lang zh-CN

# 2. 创建支出账户
accounting my.db account add Expenses:Food
accounting my.db account add Expenses:Transport

# 3. 创建本月预算
accounting my.db budget create \
  --name "6 月生活预算" \
  --period monthly \
  --commodity CNY \
  --limit "Expenses:Food:2000" \
  --limit "Expenses:Transport:500"

# 4. 记录一笔餐饮支出
accounting my.db tx add \
  --date 2024-06-02 \
  --description "超市采购" \
  --posting "Assets:Cash:CNY:-120" \
  --posting "Expenses:Food:CNY:120"

# 5. 记录一笔交通支出
accounting my.db tx add \
  --date 2024-06-03 \
  --description "地铁充值" \
  --posting "Assets:Cash:CNY:-200" \
  --posting "Expenses:Transport:CNY:200"

# 6. 查看预算执行
accounting my.db budget show "6 月生活预算"
```

## 预期结果

`budget show` 会输出：

```
预算表：6 月生活预算
周期：2024-06-01 ~ 2024-06-30 (Monthly)

Account                            Limit        Actual    Remaining        %
Expenses:Food                       2000           120         1880      6.00%
Expenses:Transport                   500           200          300     40.00%
```

## 说明

- `budget create` 的 `--limit` 可多次使用，每次指定一个账户路径和限额。
- 实际金额按周期自动汇总，超支的账户会显示 `⚠ 超支`。
