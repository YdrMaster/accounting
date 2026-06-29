# initialize

初始化新的数据库文件并写入 schema、内置账户、商品、标签和渠道。

## 用法

```bash
accounting <DB_PATH> initialize [--lang <LANG>]
```

## 参数

| 参数 | 说明 |
|------|------|
| `DB_PATH` | 数据库文件路径。文件必须不存在，否则会报错。 |

## 选项

| 选项 | 说明 |
|------|------|
| `--lang` | 初始化语言，如 `zh-CN`、`en`。决定内置账户、标签、渠道的名称。 |

## 示例

```bash
# 使用中文内置账户初始化
accounting my.db initialize --lang zh-CN

# 使用英文内置账户初始化
accounting my.db initialize --lang en
```

## 说明

- `initialize` 是唯一可以在数据库不存在时执行的命令。
- 初始化成功后会创建默认商品 `CNY`、系统标签（如 `待处理` / `pending`）和系统渠道（如 `支付宝` / `Alipay`）。
- 已初始化的数据库再次执行 `initialize` 会报错。
