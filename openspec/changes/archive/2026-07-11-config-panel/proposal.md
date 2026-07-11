## 为什么

应用缺少集中管理基础数据（成员、渠道、标签）的界面。用户目前无法通过 UI 对这些实体进行增删改操作。需要一个配置面板来提供这些基础数据类型的 CRUD 功能。

## 变更内容

- 在 PageSwitcher 中添加齿轮图标按钮，点击打开配置面板
- 实现底部抽屉覆盖层（最大高度 66vh），包含三个 tab：成员、渠道、标签
- 成员 tab：列表形式，支持行内新增/改名/删除
- 标签 tab：列表形式，支持行内新增/改名/删除
- 渠道 tab：可展开卡片列表，显示名称、描述和关联账户，支持编辑所有字段，关联账户通过 AccountPickerOverlay 选择
- 后端 API 补全：
  - 成员增加改名接口：`PUT /api/members/{id}`
  - 标签增加更新接口：`PUT /api/tags/{id}`（名称 + 描述）
  - 渠道更新接口扩展：`PUT /api/channels/{id}` 支持名称和描述字段

## 能力

### 新增能力
- `config-panel`：底部抽屉 UI 组件，带 tab 界面管理成员、渠道、标签
- `member-api`：成员 CRUD REST API（列表、创建、改名、删除）
- `channel-api`：渠道 CRUD REST API（列表、创建、更新名称/描述/关联账户、删除）
- `tag-api`：标签 CRUD REST API（列表、创建、更新名称/描述、删除）

### 修改的能力
<!-- 无需修改现有规格 -->

## 影响

- **前端**：新增 ConfigPanel.vue 组件，修改 PageSwitcher.vue，集成现有 AccountPickerOverlay
- **后端**：member.rs、channel.rs、tag.rs handler 中新增/扩展端点
- **API**：三个新增或扩展的 REST 端点
- **依赖**：无新增依赖（使用现有 Vue、Pinia、Axum 技术栈）
- **系统**：Web UI 和 API 服务
