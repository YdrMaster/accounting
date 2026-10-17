# Proposal: account-selection-ui-polish

## Why

容纳框模型上线后暴露一个判等缺陷与两处交互/视觉问题:账户选择页(`AccountPickerOverlay` 复用 `AccountGrid`)不传容器 props,`undefined === undefined` 判真导致每个分区常显紫色线框且穿过卡片;选中容纳框的内部线框视觉不够醒目,且选中账户与其子容纳框的选中态彼此脱节(两者本是同一语义——在该子集末尾新建);此外详情抽屉关闭后重复点击已选中账户无法重新打开抽屉,必须先选中别的账户再选回来。

## What Changes

- 修复判等缺陷:容纳框选中/拖放视觉只在 `rootAccount` 存在时(账户页)生效;账户选择页回归纯展示——零容器视觉、点击空白无效果。
- 删除独立的 `selectedContainerId` 状态,容纳框选中态直接由 `selectedAccountId === owner` 驱动:选中一个拥有可见容纳框的账户时其容纳框同步进入选中态,点选容纳框时 owner 卡片同步进入选中态(两者共同指向同一新建目标容器)。
- 选中容纳框的视觉从内部线框改为整体高亮(容器盒 accent 半透明铺色,无线框);空容纳框占位区选中时同样铺色替代虚线边;拖放落点的虚线框保留(拖拽暂态反馈)。视觉效果以已确认的静态预览(`/tmp/ui-preview/container-selection-preview.html`)为准。
- 抽屉重开:详情抽屉打开时,重复点击已选中账户维持现有展开/折叠切换;抽屉已关闭时,重复点击已选中的非根账户卡片重新拉起其详情抽屉,不再需要先选中其他账户。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `account-page`: 选中容纳框的视觉改为整体高亮并增加卡片/容纳框选中联动,容纳框视觉仅存在于账户页面;选中账户的重复点击在抽屉关闭时重新拉起详情抽屉。

## Impact

- 前端（accounting-web）:
  - `src/components/layout/AccountGrid.vue` —— null-safe 判等、删 `selectedContainerId` prop、选中视觉改整体高亮伪元素、占位区选中铺色。
  - `src/components/layout/AccountRowGroup.vue` —— 同步删除 prop、子容纳框整体高亮。
  - `src/views/AccountsView.vue` —— 删 `selectedContainerId` 状态;`handleAccountClick` 增加抽屉重开分支。
- 后端无改动。
- 涉及测试:`AccountGrid.spec.ts`(选择器模式零容器类、高亮类断言)、`AccountsViewRefresh.spec.ts`(联动与抽屉重开)。
- 待办关联:上一修复(交易日期格式与错误映射)尚未提交发布,本变更实施时可一并处理。
