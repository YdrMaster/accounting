## Context

当前 `channels` 表没有 `is_system` 字段，所有渠道都是用户手动创建的。导入功能要求渠道已存在，但用户必须手动创建"支付宝"渠道才能导入——这不合理，因为能导入的源必然是合法渠道。

现有 `accounts` 和 `tags` 表已有 `is_system` 字段和分语言种子数据的成熟模式，渠道应复用同样的模式。

## Goals / Non-Goals

**Goals:**
- 为 `channels` 表增加 `is_system` 字段
- 在种子数据中按语言插入内置渠道（中文"支付宝"，英文"Alipay"）
- 内置渠道不可删除（与 accounts/tags 的 system 保护一致）
- 所有适配器对应的渠道均为内置渠道

**Non-Goals:**
- 不改变导入流程本身（`ImportService` 逻辑不变）
- 不实现渠道的国际化显示名（仅种子数据名称分语言）
- 不实现内置渠道的编辑保护（仅保护删除）

## Decisions

### 1. `is_system` 字段加在 `channels` 表

与 `accounts.is_system` 和 `tags.is_system` 保持一致，使用 `INTEGER NOT NULL DEFAULT 0`。

### 2. 种子数据分语言

复用现有模式：`SEED_CHANNELS_EN` 插入 "Alipay"，`SEED_CHANNELS_ZH` 插入 "支付宝"，均标记 `is_system=1`。

### 3. `Channel` 领域模型增加 `is_system: bool`

与 `Tag.is_system` 和 `Account.is_system` 对齐。repo 层 `ChannelRow` 增加 `is_system` 字段映射。

### 4. 删除保护

`channel_force_delete_by_id` 增加 `is_system` 检查，系统渠道不允许删除。

## Risks / Trade-offs

- [已有数据库无 is_system 列] → 当前项目处于开发阶段，数据库可随时重建，不需要 migration
- [适配器与渠道名称映射] → 适配器 `names()` 返回的名称列表中的第一个作为渠道种子数据的英文名，中文名由种子数据决定
