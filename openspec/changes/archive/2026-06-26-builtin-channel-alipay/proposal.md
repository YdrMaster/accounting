## Why

导入功能已支持支付宝账单，但渠道表没有对应的内置记录。用户必须手动创建"支付宝"渠道才能导入，这不合理——能导入的源必然是合法渠道。需要为渠道增加系统内置标记，并预置支付宝渠道。

## What Changes

- 为 `channels` 表增加 `is_system` 布尔字段，标识系统内置渠道
- 种子数据中按语言插入内置渠道：中文"支付宝"，英文"Alipay"
- 所有适配器（当前仅 Alipay）对应的渠道均为内置渠道，导入时自动校验渠道存在性
- 内置渠道不可删除（与现有 system tag/account 行为一致）

## Capabilities

### New Capabilities
- `builtin-channel`: 系统内置渠道机制——is_system 字段、种子数据、删除保护

### Modified Capabilities
- `bill-import`: 导入服务依赖内置渠道存在，适配器与内置渠道一一对应

## Impact

- **数据库**: `channels` 表新增 `is_system` 列，schema 初始化需更新
- **种子数据**: `insert_seed_data` 按语言插入内置渠道
- **accounting-service**: `ImportService` 的渠道校验逻辑可简化（内置渠道已预置）
- **accounting-api**: 渠道列表/删除接口需考虑 `is_system` 标记
- **accounting-cli**: 无直接影响
