# web-channel-import-rules 设计

## Context

配置抽屉（`ConfigPanel.vue`）渠道 tab 目前对所有渠道显示删除按钮，点击系统内置渠道时由后端 `channel_force_delete_by_id`（`accounting-sql/src/repo/channel.rs:163`）报错兜底。标签 tab 已有正确模式：`v-if="!tag.is_system"` 隐藏删除按钮。

导入时的"分类→账户"映射（`account_mappings` 表）已有完整后端：数据模型、`MappingService`（`accounting-service/src/mapping_service.rs`）、CLI `mapping` 子命令。Web 端无 API、无 UI。映射维度为 (member_id, channel_id)，category 格式 `"<role>:<原始分类>"`（见 `openspec/specs/account-mapping/spec.md`）。

## Goals / Non-Goals

**Goals:**
- 渠道卡片对 `is_system=true` 的渠道隐藏删除按钮
- 新增 mapping REST API（list / set / delete），供 Web 使用
- 渠道卡片展开区新增"导入规则"区块：成员切换 + 映射 CRUD

**Non-Goals:**
- 不改数据模型、不改 `MappingService` 既有方法签名、不动 CLI
- 不做 Web 端 CSV 导入入口
- 不做渠道→适配器的持久化绑定
- 不限制系统渠道改名（后端 `channel_rename` 本就是安全设计）

## Decisions

### 1. 删除保护只做前端隐藏

后端防线已存在且有测试覆盖，无需叠加 API 层校验。渠道卡片套用标签卡片模式：删除按钮加 `v-if="!channel.is_system"`。

### 2. API 以 account_id 而非账户路径为输入

`MappingService.set` 接受账户路径字符串（CLI 场景）。Web 端 `AccountPicker` 产出的是 account_id，前端已有账户全量数据。因此 API 请求体直接携带 `account_id`，handler 校验账户存在后调用 `db.account_mapping_upsert`（或在 `MappingService` 增加 `set_by_id` 薄封装）。避免 Web 场景无谓的"路径拼接→再解析"往返。

### 3. API 形状

复用现有 handler 风格（`Arc<AppState>` + `Lang` extractor + `Result<_, String>`）：

```
GET    /api/mappings?member_id=1&channel_id=2   → [MappingDto]
PUT    /api/mappings   { member_id, channel_id, category, account_id }  → upsert
DELETE /api/mappings?member_id=1&channel_id=2&category=<urlencoded>     → 删除
```

`MappingDto { member_id, channel_id, category, account_id }`。账户显示名由前端 account store 按 `account_id` 本地解析（account store 已加载全量账户），API 不做名字拼接。

### 4. 导入规则区块内嵌渠道卡片，成员切换用简单下拉

展开渠道卡片后在现有字段下方追加"导入规则"区块：

```
┌─ 渠道卡片（展开）──────────────────┐
│ 名称 / 描述 / 关联账户（现有字段）    │
│ ── 导入规则 ──────────────────    │
│ 成员: [自己 ▾]                     │
│ Expenses:餐饮美食 → 餐饮        [×] │
│ Assets:余额宝    → 余额宝       [×] │
│ [分类输入.......] [账户选择] [添加]   │
└──────────────────────────────────┘
```

- 成员切换：简单 `<select>`，数据来自已有 `memberStore`，默认选中第一个成员；切换成员重新拉取该 (成员, 渠道) 的映射
- 分类输入：自由文本（如 `Expenses:餐饮美食`），placeholder 提示格式；账户用现有 `AccountPicker`
- 新区块抽为独立组件 `ChannelMappingSection.vue`，避免 `ConfigPanel.vue` 继续膨胀
- **仅对关联了导入适配器的渠道显示**：`GET /api/channels` 的 ChannelDto 新增 `has_import_adapter`——渠道的任一语言名字能匹配某个内置适配器（`find_adapter` 语义）即为 true。前端按此字段条件渲染区块。适配器关联判定放后端，因为适配器注册表（`builtin_adapters()`）只在 Rust 侧可见

### 5. 新增 mapping store

`stores/mapping.ts`：按 `(member_id, channel_id)` 缓存映射列表，提供 `load / set / remove`；沿用 channel store 的错误处理模式（`error` ref + 页面内展示）。

## Risks / Trade-offs

- [DELETE 的 category 含中文与冒号，需 URL 编码] → 前端用 `URLSearchParams` 构造 query；后端 axum query extractor 自动解码
- [渠道卡片信息量变多，抽屉 `max-height: 66vh` 内滚动变长] → 可接受，drawer-body 本就可滚动；不引入新交互层级
- [用户随意输入 category 导致映射永不命中] → 属既有 CLI 同款风险；placeholder 与描述文案提示 `"<role>:<分类>"` 格式，后续可考虑从已导入交易回读分类候选（本变更不做）
