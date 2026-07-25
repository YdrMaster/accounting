## 1. 共享滚动核心 useWheelScroll

- [x] 1.1 新建 `accounting-web/src/composables/useWheelScroll.ts`，定义 scrollPos: Ref\<number\>、committedIndex、isDragging、dragDelta 状态，N（页面数）由 paneNames.length 提供
- [x] 1.2 实现 ringDist(i, scrollPos) 纯函数：`((i - scrollPos + N/2) % N) - N/2`，导出供两个组件复用
- [x] 1.3 实现拖拽手势：beginDrag(startX, unitWidth)、updateDrag(currentX)、endDrag() —— endDrag 按 snap-to-nearest（|小数部分| > 0.5 则翻到相邻整数，否则弹回）
- [x] 1.4 实现 stepBy(±1)（箭头）与 spinTo(index)（最短路径 ringDist 增量）
- [x] 1.5 实现 rAF 动画 animateTo(target)：300ms ease 插值 scrollPos，结束后同步 committedIndex 并将 scrollPos 归一化到 [0, N)
- [x] 1.6 消费 useResponsiveLayout 的 columns，窗口大小变化时把 scrollPos 对齐到最近合法整数

## 2. useResponsiveLayout 收敛

- [x] 2.1 从 useResponsiveLayout 删除 startIndex、activeIndex、maxStart、shiftLeft、shiftRight、goTo，保留 width/height/ratio/columns/isMobile/paneNames/paneLabels
- [x] 2.2 更新所有现有调用点（ResponsiveShell 等）改用 useWheelScroll 的 scrollPos/stepBy/spinTo

## 3. PageSwitcher 重写为连续滚轮

- [x] 3.1 改为接收 scrollPos、columns、paneWidth 相关 props（或直接消费 useWheelScroll），移除 rotatedLabels/visibleIndices/rowOffset/highlightStyle 旧逻辑
- [x] 3.2 计算 slotWidth = max(标签宽度) + gap，标签 i 屏幕 X = trackCenter + (ringDist(i, scrollPos) - (columns-1)/2) × slotWidth，标签绝对定位
- [x] 3.3 高亮窗口改为固定居中，宽度 = columns × slotWidth - gap，拖动全程不动
- [x] 3.4 逐标签计算纵深样式：进度 d = 标签中心到窗口边缘距离 / 淡出距离，opacity 1→0(ease-out)、scale 1→0.85、translateY 0→4px，d>1 时 visibility:hidden
- [x] 3.5 接入拖拽手势：track 上的 pointer/touch 事件调用 useWheelScroll 的 beginDrag/updateDrag/endDrag（unitWidth = slotWidth）
- [x] 3.6 标签点击调用 spinTo(originalIndex)；箭头按钮调用 stepBy(±1)；config 按钮不变

## 4. ResponsiveShell 面板轨道重写

- [x] 4.1 面板改为绝对定位：每个 pane `position:absolute; width:paneWidth; height:100%`，transform = translateX(ringDist(i, scrollPos) × paneWidth)，删除 orderedPanes 旋转复制与 trackBase/targetOffset/pendingNewIndex/onTransitionEnd
- [x] 4.2 移动端 viewport 触摸拖拽接入 useWheelScroll（unitWidth = paneWidth），删除本地 isDragging/dragOffset/shiftLeft/shiftRight/moveTo 逻辑
- [x] 4.3 onSwitcherGoTo 改为 spinTo；箭头 left/right 改为 stepBy(±1)
- [x] 4.4 桌面端内容区保持禁用鼠标拖拽（不绑定 mouse 事件，仅移动端 touch）

## 5. 验证与收尾

- [x] 5.1 运行类型检查与现有测试（`npm run typecheck` / 测试脚本），确保无编译错误
- [x] 5.2 启动 dev server，移动端视口验证：拖标签条页面实时跟手、边缘标签淡入淡出+纵深、释放 snap/弹回、内容区拖拽双向联动
- [x] 5.3 桌面端宽屏验证：多列固定高亮窗口、连续翻页出现跨边界混合窗口无跳变、箭头与点击最短路径旋转
- [x] 5.4 确认提交/归一化瞬间无视觉跳变（快速连续翻页、点击远端标签）
