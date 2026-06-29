# config

导入/导出数据库的配置信息（如账户映射、预算等）到 YAML 文件，用于备份或迁移。

## 子命令

```bash
accounting <DB_PATH> config export <FILE>
accounting <DB_PATH> config import <FILE>
```

## config export

将当前数据库的配置导出为 YAML 文件。

| 参数 | 说明 |
|------|------|
| `FILE` | 输出 YAML 文件路径。 |

```bash
accounting my.db config export config.yaml
```

## config import

从 YAML 文件导入配置到当前数据库。

| 参数 | 说明 |
|------|------|
| `FILE` | 输入 YAML 文件路径。 |

```bash
accounting my.db config import config.yaml
```

## 说明

- 配置导入导出不影响交易流水，只导出可重复使用的结构型配置。
- 导入时若配置格式不合法会报错，已存在的数据不会被清空。
