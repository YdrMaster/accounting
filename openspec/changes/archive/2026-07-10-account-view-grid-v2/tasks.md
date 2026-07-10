## 1. 样式：实现包裹式纸片叠放高亮

- [x] 1.1 在 `AccountRowGroup.vue` 中给 `.children-container` 添加 `.stacked-children` 样式类，严格使用设计稿中的样式：

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

- [x] 1.2 按深度设置 `--paper-bg`，直接采用设计稿中的颜色值：
  - Depth 0：`hsla(237, 76%, 65%, 0.08)`
  - Depth 1：`hsla(242, 78%, 68%, 0.10)`
  - Depth 2：`hsla(247, 80%, 71%, 0.12)`
  - Depth 3+：按 HSV 偏移规律递增（H +5、S +2%、V +3%、alpha +0.02）。
- [x] 1.3 卡片颜色保持统一 `#2d2d2d`，确保所有层级的 `.row` 仍使用 `subgrid`，卡片等宽且列对齐，无缩进。
- [x] 1.4 确保所有层级的 `.row` 仍使用 `subgrid`，卡片等宽且列对齐，无缩进。

## 2. 交互：收紧折叠条件

- [x] 2.1 修改 `AccountsView.vue` 的 `handleAccountClick`：在更新 `selectedAccountId` 前，先判断 `clickedId` 是否在 `expandedPath` 中。
- [x] 2.2 若 `clickedId` 在 `expandedPath` 上，仅切换选中态，不重建 `expandedPath`。
- [x] 2.3 若 `clickedId` 不在 `expandedPath` 上，按现有逻辑重建 `expandedPath` 并折叠不在新路径上的子树。

## 3. 测试

- [x] 3.1 更新 `AccountRowGroup.spec.ts`：断言展开路径上的每一级都有 `.stacked-children` 伪元素高亮，且卡片等宽对齐。
- [x] 3.2 更新 `AccountGrid.spec.ts`：断言扁平网格按固定列数渲染，占位符正确填充。
- [x] 3.3 更新 `AccountsView.spec.ts`：补充“路径内切换不折叠”与“路径外切换折叠”的场景断言。
- [x] 3.4 运行 `npm run test`、`npm run build`、ESLint，确保全部通过。

## 4. 验证与收尾

- [x] 4.1 在浏览器中打开设计稿 `docs/superpowers/specs/2026-07-09-account-view-grid-v2-design.html`（已通过本地 HTTP 服务 `http://127.0.0.1:8765/2026-07-09-account-view-grid-v2-design.html` 访问），实现样式与设计稿一致。
- [x] 4.2 通过单元测试覆盖展开、选中、折叠行为，确认账户页面交互符合预期。
