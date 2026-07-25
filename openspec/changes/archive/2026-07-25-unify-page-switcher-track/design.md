## Context

页面切换由三个文件协作：`useResponsiveLayout.ts`（startIndex 状态与 shift/goTo 操作）、`PageSwitcher.vue`（标签条，自持拖拽逻辑，40px 阈值，释放后重排 rotatedLabels 数组）、`ResponsiveShell.vue`（面板轨道，旋转+复制 10 个 pane，自持触摸拖拽，15% paneWidth 阈值，transitionend 时提交新索引）。

两套拖拽互不感知：拖标签条时页面不动，释放后才跳变；面板轨道在提交时重排 DOM 数组。"数组排列 = 视觉位置"的耦合是所有跳变的根源。

## Goals / Non-Goals

**Goals:**
- 单一连续 scrollPos 驱动标签条与面板轨道，拖动任一面另一面逐帧联动
- 标签从固定高亮窗口下滑过，边缘标签实时淡出并带纵深（scale + translateY）
- 释放/提交时视觉零跳变（提交仅为簿记）
- 桌面与移动端统一为全环形拓扑

**Non-Goals:**
- 惯性/动量减速（明确不做）
- 桌面端内容区鼠标拖拽翻页（保持禁用，避免与列表滚动/文本选择冲突）
- 键盘导航、无障碍滚轮语义
- 页面内容本身的任何改动

## Decisions

### 1. 共享 composable `useWheelScroll` 持有唯一 scrollPos

scrollPos: Ref\<number\>，浮点，单位为"页"。整数值 = 静止对齐位置。

- committedIndex = round(scrollPos)（静止时）
- 拖动中: scrollPos = committedIndex - dragDelta / unitWidth（负号：手指左移 → scrollPos 增大 → 显示下一页）
- 释放: snapToNearest() — 过半个单元则动画到相邻整数，否则弹回当前整数
- 动画结束后 committedIndex 同步，scrollPos 归一化（mod N）— 纯数值操作，渲染不变

**替代方案**：保持两套状态通过事件同步 — 拒绝：事件同步必然有帧延迟，无法实现"逐帧联动"，且阈值语义无法统一。

### 2. ringDist 渲染模型，彻底移除数组旋转

```
ringDist(i, scrollPos) = ((i - scrollPos + N/2) % N) - N/2    ∈ [-N/2, N/2)
```

任何元素的屏幕位置 = f(ringDist)，与数组顺序无关。scrollPos 跨越整数时 ringDist 连续变化（±N/2 处的折叠发生在完全不可见区域），因此提交/归一化永远不产生视觉变化。

**替代方案**：保留 flex 行 + 旋转数组，在提交时做"无缝重排" — 拒绝：需要精确验证重排前后像素一致，goTo 跨多页时缓冲区不够（需 3N+ 副本），复杂度高且脆弱。

### 3. 面板轨道：5 个 pane 绝对定位，各自由 ringDist 驱动 transform

- 每个 pane: `position: absolute; width: paneWidth; height: 100%`
- transform: `translateX(ringDist(i, scrollPos) * paneWidth)`
- 删除 `[...rotated, ...rotated]` 10 pane 复制（内存反而更优：5 < 10）
- 每帧更新 5 个 transform — GPU 合成层操作，开销可忽略

**替代方案**：flex 单轨道 + 单个 translateX — 性能略优（1 个 transform），但必须在提交时重排 DOM 且 goTo 动画需要超大缓冲区。5 个 transform 的代价换来模型纯粹性，值得。

### 4. 标签条：uniform slots + 固定高亮窗口

- slotWidth = max(所有标签宽度) + gap；每个标签居中于自己的 slot
- 标签 i 的屏幕 X = trackCenter + (ringDist(i, scrollPos) - (columns-1)/2) × slotWidth
- 高亮窗口：固定居中，宽度 = columns × slotWidth - gap，拖动全程不动
- "可见/激活"判定：标签中心落在窗口内（静止时恰好 columns 个）

**为什么 uniform slots**：固定窗口要求标签以均匀节奏通过窗口边缘，否则窗口宽度随内容抖动。中文标签均为两字，宽度几乎相同，视觉上无感知差异。

**替代方案**：CSS mask-image 做边缘淡出 — 拒绝：mask 只能做透明度渐变，无法实现 scale/translateY 纵深；且淡出曲线绑定在容器边缘而非高亮窗口边缘。

### 5. 纵深效果：逐标签 opacity + scale + translateY

以标签中心到窗口边缘的距离计算进度 d（0 = 窗口边缘，1 = 完全淡出位置）：

| 属性 | d=0 | d=1 | 曲线 |
|------|-----|-----|------|
| opacity | 1 | 0 | ease-out |
| scale | 1 | 0.85 | linear |
| translateY | 0 | +4px | linear |

淡出距离 = (trackWidth - windowWidth) / 2（窗口边缘到 track 边缘的可用空间）。d > 1 的标签 visibility: hidden。

### 6. scrollPos 动画用 rAF 插值，不用 CSS transition

snap/spinTo 时对 scrollPos 本身做 300ms ease 插值（requestAnimationFrame），所有派生视觉（标签位置/透明度/纵深、面板位置）自然同步。

**替代方案**：CSS transition 加在 transform 上 — 拒绝：opacity/scale 由 scrollPos 派生，CSS 只动画 transform 会导致透明度跳变不同步。

### 7. 统一翻页语义：snap-to-nearest

释放时 scrollPos 距最近整数 > 0.5 则翻到该整数，否则弹回。替代现有 40px / 15% 两套阈值。等价于"半个单元"阈值，是滚轮的自然物理语义。

### 8. spinTo 最短路径

点击标签：`delta = ringDist(targetIndex, scrollPos)`，动画 scrollPos += delta。环形结构保证 |delta| ≤ N/2，永远沿最短方向旋转。

### 9. useResponsiveLayout 的归宿

columns/isMobile/paneWidth 等响应式计算保留在 useResponsiveLayout；startIndex/shiftLeft/shiftRight/goTo/maxStart 全部删除，由 useWheelScroll 的 scrollPos/stepBy/spinTo 替代。useWheelScroll 内部消费 useResponsiveLayout 的 columns（窗口大小变化时重新对齐 scrollPos）。

## Risks / Trade-offs

- [桌面全环拓扑是用户可感知的行为变化（出现跨边界混合窗口）] → 已与需求方确认为期望行为；移动端不受影响
- [每帧响应式更新 scrollPos 触发 Vue 重渲染（10 个元素的 style）] → 当前规模微不足道；如未来 pane 数量增长可改为直接 DOM style 写入，接口不变
- [uniform slot 对宽度差异大的标签（如英文长词）会留出空白] → 标签居中于 slot，视觉可接受；slotWidth 取最大值保证不裁切
- [移动端 track 很窄时淡出距离短，标签可能未完全透明就被 overflow:hidden 裁切] → 裁切发生在 d≈1 附近，与淡出叠加后不可感知
- [触摸手势冲突：标签条水平拖动 vs 页面垂直滚动] → 沿用现有 touch-action: pan-y 策略，水平手势由 touchmove preventDefault 接管
