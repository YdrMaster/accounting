# Tasks: account-selection-ui-polish

## 1. AccountGrid / AccountRowGroup:开关修复与整体高亮

- [x] 1.1 `AccountGrid.vue`:`isSelectedContainer`/`isDropContainer` 改为 null-safe(`rootAccount != null` 前置),删除 `selectedContainerId` prop
- [x] 1.2 `AccountGrid.vue`:选中容纳框视觉改为 `::after` 整体铺色(`position: relative` + `z-index: -1` + `pointer-events: none`),移除内部线框样式;`.grid-empty-placeholder` 选中态虚线边透明化 + 同款铺色
- [x] 1.3 `AccountRowGroup.vue`:同步删除 `selectedContainerId` prop,子容纳框选中态绑 `selectedAccountId === expandedAccountId`,视觉改 `::after` 整体铺色
- [x] 1.4 拖放落点 `container-drop-target` 虚线样式保持不变(仅拖拽暂态)

## 2. AccountsView:状态收敛与抽屉重开

- [x] 2.1 删除 `selectedContainerId` ref 及其向 `AccountGrid` 的传递;`handleAccountClick`/`handleContainerClick`/`onAccountDeleted` 清理相关赋值
- [x] 2.2 `handleAccountClick`:点已选中卡片时,抽屉打开 → 维持展开/折叠切换;抽屉关闭且非根非系统 → `drawerVisible = true` 重开抽屉(展开状态不动)

## 3. 测试更新与回归

- [x] 3.1 `AccountGrid.spec.ts`:选择器模式(不传 rootAccount)断言零容器类;账户页模式断言整体高亮类随 `selectedAccountId` 联动;删 `selectedContainerId` 相关断言
- [x] 3.2 `AccountsViewRefresh.spec.ts`:容器选中由 `selectedAccountId` 驱动;新增「抽屉关闭后重复点击已选中账户重开抽屉」「选中卡片联动其容纳框」用例
- [x] 3.3 回归:`npm run test` 与 `vue-tsc` 全绿;浏览器对照预览验证联动与高亮效果

## 4. 配置按钮图标扁平化(实施中新增)

- [x] 4.1 `PageSwitcher.vue`:配置按钮的 `⚙` 字符替换为内联扁平齿轮 SVG(currentColor,随按钮 muted/hover 变色),消除 3D emoji 渲染
- [x] 4.2 渲染验证:重建 dist 后截图页头确认扁平效果,全量测试与类型检查通过
