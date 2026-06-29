# beancount

与 Beancount 格式互导。可用于备份、迁移或与 Beancount 生态集成。

## 子命令

```bash
accounting <DB_PATH> beancount export <OUTPUT_DIR>
accounting <DB_PATH> beancount import <INPUT_FILE>
```

## beancount export

将当前数据库导出为 Beancount 文件，写入 `<OUTPUT_DIR>/backup.beancount`。

| 参数 | 说明 |
|------|------|
| `OUTPUT_DIR` | 输出目录路径 |

```bash
accounting my.db beancount export ./beancount-backup
```

导出后文件路径为 `./beancount-backup/backup.beancount`。

## beancount import

从 Beancount 文件导入账目，包括账户、商品、成员、渠道、交易和附件。

| 参数 | 说明 |
|------|------|
| `INPUT_FILE` | 输入 Beancount 文件路径 |

```bash
accounting my.db beancount import ./ledger.beancount
```

## 输出

导入完成后会输出统计：

```
导入完成: 100 笔交易, 0 跳过, 20 账户, 2 商品, 1 成员, 3 渠道, 5 附件
```

## 说明

- 导出时若输出目录不存在会自动创建。
- Beancount 的 `include` 语句会被解析，附件路径相对于输入文件所在目录解析。
- 导入会尽量复用已存在的账户和商品；遇到冲突时通常以数据库中已存在的数据为准。
