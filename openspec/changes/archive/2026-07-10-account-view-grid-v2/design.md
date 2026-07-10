## Context

账户页面当前实现已使用 `compileRows` 与 `buildRowTree` 将多级账户树展平为固定列数的卡片网格，并通过 `AccountGrid` / `AccountRowGroup` 递归渲染。设计稿 `docs/superpowers/specs/2026-07-09-account-view-grid-v2-design.html` 已经确定了本次视觉与交互方向：

- 视觉：HSV 深度渐变 + 包裹式纸片叠放高亮。
- 交互：仅当新选中卡片不在当前 `expandedPath` 上时才折叠。

## Goals / Non-Goals

**Goals：**
- 子账户以等宽卡片网格展示，所有深度列对齐、卡片等宽。
- 高亮区域使用 HSV 深度渐变，形成包裹式纸片叠放效果。
- 折叠条件收紧，避免在展开路径上切换选中时误折叠子树。
- 保持现有组件结构（`AccountGrid` → `AccountRowGroup` → `AccountCard`），修改范围最小化。

**Non-Goals：**
- 不改动账户 API、数据模型或抽屉编辑逻辑。
- 不引入新的依赖库。
- 不改变卡片仅显示账户名称的约束。

## Decisions

1. **高亮用 `::before` 伪元素实现，容器保持 subgrid 对齐**
   - 原因：若用 padding/margin 让子级容器回叠，会导致卡片逐层错位。伪元素只负责背景绘制，不影响容器内容布局，可保证所有深度卡片严格对齐。
   - 实现时直接采用设计稿中的样式（摘抄自 `docs/superpowers/specs/2026-07-09-account-view-grid-v2-design.html`）：

     ```css
     .children-container.stacked-children {
       position: relative;
       padding: 0.5rem 0;
       margin: -0.25rem 0;
       background: transparent;
       box-shadow: none;
     }
     .children-container.stacked-children::before {
       content: '';
       position: absolute;
       top: 0;
       bottom: 0;
       left: -0.5rem;
       right: -0.5rem;
       background: var(--paper-bg);
       border-radius: 0.75rem;
       z-index: -1;
       box-shadow: 0 -2px 8px rgba(0,0,0,0.25), 0 -1px 0 rgba(255,255,255,0.04) inset;
     }
     ```

2. **背景色使用 HSV 深度渐变**
   - 原因：HSL 的色相、饱和度、明度随深度同步变化，能在不刺眼的前提下让深层展开区自然变亮、变冷，叠加后产生纵深。
   - 卡片颜色保持统一 `#2d2d2d`。
   - 高亮背景按深度使用设计稿中的下列值（通过行内 `--paper-bg` 或动态 class 设置）：
     - Depth 0：`hsla(237, 76%, 65%, 0.08)`
     - Depth 1：`hsla(242, 78%, 68%, 0.10)`
     - Depth 2：`hsla(247, 80%, 71%, 0.12)`
     - Depth 3+：继续按 HSV 偏移规律递增（H +5、S +2%、V +3%、alpha +0.02）。

3. **折叠条件：仅当新选中不在 `expandedPath` 上时才折叠**
   - 原因：当前实现会在任意切换时重新计算路径，导致在路径上切换兄弟子树时父级被折叠。改为“路径内切换只改选中态、路径外切换才重建路径”后，浏览更稳定。
   - 实现点位于 `AccountsView.handleAccountClick`。

4. **保留嵌套组件结构，只改样式与事件处理**
   - 原因：`AccountRowGroup` 的递归结构已能表达层级，最小改动可降低回归风险。

## Risks / Trade-offs

- **[Risk]** 伪元素 `z-index: -1` 在部分旧浏览器或复杂 stacking context 中可能被错误裁剪。  
  → **Mitigation**：容器已设置 `position: relative` 与 `z-index: 1`，伪元素位于容器 stacking context 内部；实现后在目标浏览器验证。

- **[Risk]** HSV 渐变颜色在不同显示器上可能显得过亮或过冷。  
  → **Mitigation**：颜色值已写入设计稿并内联渲染，可直接用浏览器查看效果；如不满意可在实现前微调 H/S/V 步长。

- **[Risk]** 折叠条件变更可能影响现有测试用例。  
  → **Mitigation**：补充针对“路径内不折叠 / 路径外折叠”的单元测试，并运行现有端到端测试。

## Migration Plan

- 无数据迁移或部署特殊步骤。本次为纯前端样式与交互变更，合并后直接部署即可。
- 回滚：恢复相关组件与测试文件到变更前版本。

## Open Questions

- 无。
