# accounting-cli

基于 clap + tokio 的命令行记账工具。

## 基本用法

```bash
accounting <DB_PATH> <COMMAND> [OPTIONS]
```

## 全局选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--format` | 输出格式：`table` 或 `json` | `table` |
| `--lang` | 当前命令的界面语言 | 数据库配置 |

## 快速开始

```bash
# 初始化数据库
accounting my.db initialize --lang zh-CN

# 添加账户并记录第一笔支出
accounting my.db account add Assets:Cash
accounting my.db account add Expenses:Food
accounting my.db tx add \
  --date 2024-06-01 \
  --description "午餐" \
  --posting "Assets:Cash:CNY:-50" \
  --posting "Expenses:Food:CNY:50"

# 查看余额和报表
accounting my.db account balance Assets:Cash
accounting my.db report bs
```

## 完整文档

详细命令说明和操作示例请见 [`docs/`](docs/README.md)。
