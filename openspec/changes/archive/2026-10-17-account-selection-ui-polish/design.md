# Design: account-selection-ui-polish

## Context

容纳框模型上线后(见归档 `2026-10-17-account-container-drag-and-create`)的三个问题:① 选择器复用 `AccountGrid` 时不传容器 props,`selectedContainerId === rootAccount?.id` 在两侧同为 `undefined` 时判真,所有分区常显选中/拖落线框;② 容器选中用内部线框(`outline`),且与卡片选中各自独立状态(`selectedContainerId` vs `selectedAccountId`),同一语义(在子集末尾新建)却不同步;③ 详情抽屉关闭后重复点击已选中账户只做展开切换,抽屉无法重开。视觉效果已经用户以静态预览(`/tmp/ui-preview/container-selection-preview.html`)确认。

## Goals / Non-Goals

**Goals:**

- 容纳框视觉/交互仅在账户页(`rootAccount` 存在)生效;选择器零容器视觉。
- 单一选中源:容纳框选中态 = `selectedAccountId === owner`,卡片与容纳框联动。
- 选中容纳框整体高亮(accent 半透明铺色,无线框)。
- 抽屉关闭后重复点击已选中非根账户卡片重新拉起抽屉。

**Non-Goals:**

- 不改拖放落点的虚线框(拖拽暂态反馈,保留)。
- 不改选择器的单选语义(仍只能选一个账户)。
- 不动后端与交易保存路径。

## Decisions

### D1 开关 = `rootAccount` 存在性,null-safe 判等

`isSelectedContainer`/`isDropContainer` 改为先判 `rootAccount != null && owner != null` 再比较 id。选择器不传 `rootAccount` → 两个 computed 恒 false → 零容器视觉,`containerClick` 也不会发出(现有 `onGridClick` 守卫已如此)。被否决备选:给选择器显式传 `selectedContainerId: null`——依赖调用方记得传,不如源头开关可靠。

### D2 删除 `selectedContainerId`,选中态单源化

容纳框选中态改为 `props.selectedAccountId === owner.id`。AccountsView 删除 `selectedContainerId` ref 及其传递;点容器空白只设 `selectedAccountId = owner`,点卡片只设 `selectedAccountId = 卡片 id`——有可见容器的卡片被选中时其容器自动同亮,反向同理。行为与既有测试中的「点击卡片后容器选中标识清除」场景相反,该场景按新规格(联动)改写。

### D3 整体高亮实现:伪元素铺色,z-index 叠层

`.account-grid`/`.children-container` 加 `position: relative` 与 `::after` 伪元素(`inset` 外扩少许、`border-radius: .75rem`、`background: hsla(237, 76%, 65%, 0.17)`、`z-index: -1`、`pointer-events: none`)。层级:卡片(z auto)→ 高亮层(-1)→ 纸片背景层(-2),高亮叠在纸片 tint 之上、卡片之下;嵌套选中时父容器与子容器的高亮自然叠深。空容纳框占位区选中态:虚线边透明化 + 同款铺色。被否决备选:整盒 `background` 直涂——会盖住 `::before` 纸片层且嵌套叠加失真。

### D4 抽屉重开分支

`handleAccountClick`:现逻辑「点已选中卡片 → 切换展开」前加分支——若 `clickedId === selectedAccountId`:抽屉打开时维持切换;抽屉关闭且卡片非根、非系统时 `drawerVisible = true`(展开状态不动)。容器点击不弹抽屉的守卫、未选中首点开抽屉的行为不变。被否决备选:点击始终重开抽屉再切换展开——一次点击双义,不可预期。

## Risks / Trade-offs

- [AccountRowGroup 子容纳框伪元素与既有 `::before` 纸片层叠] → 伪元素用 `::after` 且 `z-index: -1`(纸片 -2),层级确定;`pointer-events: none` 防挡点击。
- [选中视觉从线框改铺色后,选中态与展开路径的视觉区分度] → 卡片选中仍是 accent 实底,容器铺色叠深,语义为「新建目标容器」,与拖放虚线暂态不冲突。
- [删除 `selectedContainerId` 影响既有测试断言] → `AccountGrid.spec` 与 `AccountsViewRefresh.spec` 中相关断言按联动语义改写(预览已确认该行为)。

## Migration Plan

1. `AccountGrid`/`AccountRowGroup`:null-safe、删 prop、整体高亮。
2. `AccountsView`:删状态、抽屉重开分支。
3. 更新两组件测试 + 视图测试。
4. `vitest` + `vue-tsc` + 手测(与预览一致)。

纯前端,回滚即 revert。

## Open Questions

（无）
