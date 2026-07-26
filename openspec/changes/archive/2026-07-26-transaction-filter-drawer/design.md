## Context

交易页（TransactionView）当前仅有时间倒序无限滚动，无任何筛选 UI。后端 `GET /transactions` 已完整支持 `from`/`to`/`account`/`member`/`tag`/`channel`/`keyword`/`reimbursable` 参数。前端 store 的 `loadInitial`/`loadMore` 硬编码只传 `{ to, limit }`。

ViewPanel header 通过 `provide/inject`（panelAction）渲染单个 action 按钮，硬编码 `+` 前缀。五个 panel 永不卸载（carousel 位移切换），组件状态天然跨 panel 保持。

## Goals / Non-Goals

**Goals:**
- 交易页提供全维度筛选能力，条件变化即时反映到列表和 Hero 汇总
- 筛选状态跨 panel 切换保持，刷新后重置
- 复用现有 store/组件模式，不引入新依赖

**Non-Goals:**
- 筛选条件持久化（URL params / localStorage）
- 筛选条件保存为"快捷方式"
- 后端 API 变更
- 其他页面（日历、资产等）的筛选能力
- 筛选结果的导出

## Decisions

### 1. 筛选状态放 Pinia store

`activeFilter: TxFilters | null` 存于 transaction store。loadInitial/loadMore 序列化时读取。

**替代方案**: 组件 ref + props 传递 — 被否，loadMore 由滚动事件触发，需要访问筛选参数，store 是唯一自然归属地。

### 2. 半屏底部抽屉（bottom sheet）

抽屉 absolute 定位于 panel 内部底部，高度约 60%，向上滑入。列表上半部分保持可见，用户边调边看。抽屉内容独立滚动。

**替代方案**: full overlay（TransactionFormOverlay 模式）— 被否，遮挡列表无法实时预览；内联折叠 — 空间不足，移动端体验差。

### 3. 即时应用 + debounce

条件变化 → 300ms debounce → `resetList()` + `loadInitial()`。文本输入和选择器统一走同一 debounce 通道。"完成"按钮仅收起抽屉。

**替代方案**: 手动"应用" — 被否，用户明确选择即时模式。

请求竞态处理：每次 loadInitial 递增 requestId，响应回来时比对，丢弃过期响应。

### 4. panelAction 扩展为多按钮 + icon

`PanelAction` 增加 `icon?: string` 字段；ViewPanel 的注入类型改为 `Ref<PanelAction[]>`（空数组 = 无按钮）。按钮按数组顺序从右到左排列（第一个最右）。现有各 view 改为注册数组。

**替代方案**: 新增 panelSecondaryAction 注入点 — 被否，两个注入 key 增加认知负担，数组更通用且改动量相当。

### 5. 多值参数序列化

`fetchTransactions` 改为接受 `URLSearchParams` 或新增 `buildTxQuery(filter)` 工具函数，用 `params.append()` 处理重复 key（`account=1&account=2`）。后端 `deserialize_vec_from_single_or_list` 已兼容。

### 6. expandSameDay 加上限

筛选激活时结果稀疏，翻倍膨胀可能失控。加 cap：最多翻倍 3 次（limit ≤ 800），超过则接受当前范围。无筛选时保持原有无限膨胀行为不变。

### 7. 抽屉与表单覆盖层互斥

表单 overlay 打开时：抽屉自动收起（showFilterDrawer = false）。抽屉打开时点击新建：先收起抽屉再打开表单。

### 8. 时间筛选与滚动加载的交互

- 无 `from`：行为不变，从 `to`（默认今天）向过去翻页
- 有 `from`：loadMore 到 `loadedRange.from <= filter.from` 时停止
- 时间预设：本月 / 上月 / 近三月 / 今年 / 全部（清除 from/to）
- 自定义范围：两个 date input

### 9. Hero 跟随筛选

Hero 从 `txStore.transactions`（已筛选）客户端计算，数字自然跟随。筛选激活时月份标签旁显示"已筛选"标记，提示数字非整月汇总。

## Risks / Trade-offs

- **即时模式请求量** → 300ms debounce + requestId 竞态丢弃；用户快速连续调整时只有最后一次生效
- **expandSameDay cap 导致单日数据截断** → 仅在筛选激活时 cap，且 800 笔/页远超正常场景；无筛选保持原行为
- **Hero 数字被误读为月度汇总** → "已筛选"标记提示
- **panelAction 类型变更影响所有 view** → 改动机械（ref 赋值改为数组），逐 view 适配即可
- **tag 参数传名称而非 ID** → 后端按名称解析，tag 重命名后旧筛选失效；可接受（筛选本身不持久化）
