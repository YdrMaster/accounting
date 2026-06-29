# commodity

商品/币种管理。商品用于计量分录金额，常见用法是把人民币、美元等币种作为商品。

## 子命令

```bash
accounting <DB_PATH> commodity list
accounting <DB_PATH> commodity add <SYMBOL> --name <NAME> [--precision <N>]
```

## commodity list

列出所有商品。

```bash
accounting my.db commodity list
```

## commodity add

添加商品。

| 参数 | 说明 |
|------|------|
| `SYMBOL` | 商品符号，如 `CNY`、`USD`、`BTC` |

| 选项 | 说明 |
|------|------|
| `--name` | 商品名称，如 `人民币` |
| `--precision` | 小数位精度，默认 `2` |

```bash
accounting my.db commodity add USD --name "US Dollar" --precision 2
accounting my.db commodity add BTC --name "Bitcoin" --precision 8
```

## 说明

- 初始化数据库时会自动创建默认商品 `CNY`，一般不需要手动添加人民币。
- 商品符号在分录中直接使用，如 `--posting "Assets:Bank:USD:100"`。
