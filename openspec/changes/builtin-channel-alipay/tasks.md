## 1. 领域模型与 Schema

- [ ] 1.1 `accounting/src/channel.rs` 增加 `is_system: bool` 字段
- [ ] 1.2 `accounting-sql/src/schema.rs` 的 `channels` 表 DDL 增加 `is_system INTEGER NOT NULL DEFAULT 0`
- [ ] 1.3 新增 `SEED_CHANNELS_EN` 和 `SEED_CHANNELS_ZH` 种子数据常量
- [ ] 1.4 `insert_seed_data` 函数中按语言选择并执行渠道种子数据

## 2. Repo 层适配

- [ ] 2.1 `accounting-sql/src/repo/channel.rs` 的 `ChannelRow` 增加 `is_system` 字段
- [ ] 2.2 `into_channel` 映射 `is_system` 到领域模型
- [ ] 2.3 所有 SELECT 查询增加 `is_system` 列
- [ ] 2.4 `channel_create` INSERT 语句增加 `is_system` 列绑定
- [ ] 2.5 `channel_force_delete_by_id` 增加 `is_system` 检查，系统渠道不可删除

## 3. Database 层方法

- [ ] 3.1 更新 `SqliteDatabase` 中所有 channel 相关方法的返回类型（已在 repo 层处理，此处验证）

## 4. API 层适配

- [ ] 4.1 `accounting-api/src/dto.rs` 的 `ChannelDto` 增加 `is_system: bool` 字段
- [ ] 4.2 `accounting-api/src/handlers/channel.rs` 的 handler 填充 `is_system` 字段
- [ ] 4.3 渠道删除 handler 返回系统渠道不可删除的错误

## 5. 测试

- [ ] 5.1 `accounting-sql/src/schema.rs` 测试验证内置渠道种子数据
- [ ] 5.2 `accounting-sql/src/repo/channel.rs` 测试验证 `is_system` 读写和删除保护
- [ ] 5.3 运行全量测试确认无回归
