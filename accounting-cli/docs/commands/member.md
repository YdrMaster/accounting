# member

成员管理。每个成员对应一个独立的记账主体，交易和账单导入都可以归属到具体成员。

## 子命令

```bash
accounting <DB_PATH> member list   [--limit <N>] [--offset <N>]
accounting <DB_PATH> member add    <NAME>
accounting <DB_PATH> member delete <NAME>
```

## member list

列出所有成员。

| 选项 | 说明 |
|------|------|
| `--limit` | 返回的最大条数 |
| `--offset` | 跳过的条数 |

```bash
accounting my.db member list
accounting my.db member list --limit 10 --offset 20
```

## member add

添加新成员。成员名称必须唯一。

| 参数 | 说明 |
|------|------|
| `NAME` | 成员名称 |

```bash
accounting my.db member add alice
```

## member delete

删除指定成员。名称不存在时会报错。

| 参数 | 说明 |
|------|------|
| `NAME` | 成员名称 |

```bash
accounting my.db member delete alice
```

## 说明

- 成员名称在数据库层面有 `UNIQUE` 约束，添加重复名称会失败。
- 删除成员前，请确保没有交易归属到该成员，否则可能触发外键约束错误。
