# 示例 5：Beancount 互导

本示例演示如何将数据库导出为 Beancount 文件，再导入到另一个数据库，完成 round-trip 备份。

## 涉及命令

`initialize`、`account add`、`tx add`、`beancount export`、`beancount import`、`tx list`

## 步骤

```bash
# 1. 初始化源数据库
accounting source.db initialize --lang zh-CN

# 2. 创建账户并记录交易
accounting source.db account add Assets:Cash
accounting source.db account add Expenses:Food
accounting source.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:CNY:-50" \
  --posting "Expenses:Food:CNY:50"

# 3. 导出为 Beancount
accounting source.db beancount export ./beancount-backup

# 4. 初始化目标数据库
accounting target.db initialize --lang zh-CN

# 5. 从 Beancount 文件导入
accounting target.db beancount import ./beancount-backup/backup.beancount

# 6. 验证目标数据库中的交易
accounting target.db tx list
accounting target.db account balance Assets:Cash
```

## 预期结果

- `./beancount-backup/backup.beancount` 包含所有账目。
- `target.db` 导入后包含与 `source.db` 相同的账户和交易。
- `account balance Assets:Cash` 在目标库中显示 `-50 CNY`。

## 说明

- 导出目录不存在时会自动创建。
- Beancount 的 `include` 语句会被解析，相对路径基于输入文件所在目录。
- 导入时若账户/商品已存在，会尽量复用，避免重复创建。
