## Why

当前页面切换条的拖动体验不自然：标签行整体跟随手指平移，释放后才根据阈值重排数组并跳变到新排列，视觉上像"拖动 → 洗牌"而非滚轮。标签条与内容区是两套独立的拖拽逻辑（40px 阈值 vs 15% 面板宽度），拖动标签条时页面不跟手，缺乏"连续滚轮"的物理感。

## What Changes

- 引入统一连续滚动模型：单一浮点 `scrollPos`（单位：页）同时驱动标签条和内容面板轨道，拖动任一面另一面实时联动
- 标签条改为固定高亮窗口 + 标签从窗口下滑过：高亮框位置与宽度固定，标签实时滑入滑出
- 标签边缘淡出与纵深效果：滑出高亮窗口的标签 opacity 渐降至 0，同时缩小（scale）并微微下沉（translateY），模拟圆柱滚轮背面
- 拖动过程中实时变换：标签的淡入淡出、页面轨道的位移全部逐帧跟随手指，释放时仅做簿记提交（视觉零跳变）
- **BREAKING**（行为层面）：桌面多列模式改为全环形导航——所有旋转位置均合法（如 5 页 3 列时出现 [日历,预算,交易] 这类跨边界窗口），不再有钳制跳变；移动端行为不变（已是环形）
- 点击标签改为最短路径旋转：沿环形距离最短的方向转动到目标页面
- 统一翻页阈值语义：拖动超过半个单元宽度则翻到下一页，否则弹回原位
- 移除现有的数组旋转 + 复制轨道 + 提交时重排机制

## Capabilities

### New Capabilities

（无——所有行为变化均在现有 page-switcher 能力范围内）

### Modified Capabilities

- `page-switcher`: 拖动切换从"释放后跳变重排"改为"实时连续滚轮"（标签淡入淡出 + 纵深 + 固定高亮窗口）；标签条与内容区拖拽合并为同一滚动位置的联动；桌面多列导航从钳制窗口改为全环形；点击切换改为最短路径旋转

## Impact

- `accounting-web/src/components/layout/PageSwitcher.vue` — 重写渲染模型（uniform slots、逐标签 opacity/scale/translateY、固定高亮框）
- `accounting-web/src/components/layout/ResponsiveShell.vue` — 面板轨道从旋转复制数组改为 ringDist 定位，拖拽逻辑移至共享 composable
- `accounting-web/src/composables/useResponsiveLayout.ts` — startIndex/shiftLeft/shiftRight/goTo 被统一 scrollPos 模型替代
- 新增 composable（如 `useWheelScroll.ts`）承载共享滚动状态与拖拽手势
- 无 API / 依赖变化，纯前端交互层改动
