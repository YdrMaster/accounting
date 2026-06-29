# tag

标签管理。标签可用于标记交易，便于后续筛选和批量处理。

## 子命令

```bash
accounting <DB_PATH> tag list
accounting <DB_PATH> tag add <NAME> [--description <DESC>]
accounting <DB_PATH> tag delete <NAME>
```

## tag list

列出所有标签，包括系统内置标签。

```bash
accounting my.db tag list
```

## tag add

添加标签。

| 参数 | 说明 |
|------|------|
| `NAME` | 标签名称 |

| 选项 | 说明 |
|------|------|
| `--description` | 标签描述 |

```bash
accounting my.db tag add travel --description "差旅相关"
```

## tag delete

删除标签。

```bash
accounting my.db tag delete travel
```

## 说明

- 初始化时会创建系统标签，如 `待处理` / `pending`、`还款`、`不计收支`、`不计预算`。
- 系统标签不建议删除，删除可能导致导入等功能异常。
