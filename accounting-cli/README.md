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

## 渠道链路

`tx add` / `tx update` 的 `--channel` 参数支持描述交易经过的渠道链路：

- `->` 表示链路下一级，前后可有空格。
- `&` 仅允许在最后一级使用，表示多个并行渠道。
- 渠道名后可加 `*` 表示 `pending`（待校验），加 `√` 表示 `verified`（已校验），无后缀为 `default`。

示例：

```bash
accounting my.db tx add \
  --date 2024-06-01 \
  --description "盒马买菜" \
  --posting "Expenses:Food:CNY:120" \
  --posting "Assets:Cash:CNY:-120" \
  --member Alice \
  --channel "淘宝 -> 支付宝* -> 花呗 & 信用卡√"
```

第三方账单导入（如支付宝 CSV）的交易，渠道状态会自动设为 `pending`。导入时 `--source` 支持别名与大小写不敏感匹配：中文环境下创建的 `支付宝` 渠道可用 `--source alipay`，英文环境下创建的 `Alipay` 渠道可用 `--source 支付宝`。

可通过 `tx reconcile <tx_id> --channel <channel>` 将交易中的指定渠道标记为 `verified`，使用 `--unset` 回到 `default`：

```bash
accounting my.db tx reconcile 42 --channel 支付宝
accounting my.db tx reconcile 42 --channel 支付宝 --unset
```

## Beancount 导出/导入

```bash
# 导出到目录，生成 transactions.beancount 与 attachments/
accounting my.db beancount export ./output

# 从 beancount 文件导入
accounting my.db beancount import ./output/transactions.beancount
```

导出的 `channel_path` metadata 使用上述文本格式；`commodity` 指令日期使用数据库中的 `created_at`，缺失时回退到 `1970-01-01`。导入时同时兼容新文本格式与旧 JSON 格式备份。

## 完整文档

详细命令说明和操作示例请见 [`docs/`](docs/README.md)。
