## 1. 纯导入账单检测与条件显示

- [ ] 1.1 添加 `isPureImport(tx)` 函数：检查所有 postings 的 account_type 是否都不在 ['asset', 'income', 'expense'] 中
- [ ] 1.2 在 tx-top 区域，当 `isPureImport(tx)` 为 true 时隐藏收支账户名称（ie-accounts）
- [ ] 1.3 在 tx-middle 区域，当 `isPureImport(tx)` 为 true 时隐藏金额显示（tx-amount）
- [ ] 1.4 在 tx-bottom 区域，当 `isPureImport(tx)` 为 true 时隐藏资产账户名称（asset-accounts）

## 2. 分录展开功能

- [ ] 2.1 添加 `expandedIds` ref（Set<number>）用于存储展开的账单 ID
- [ ] 2.2 添加 `toggleExpand(id)` 函数，切换 ID 在 Set 中的状态
- [ ] 2.3 在 tx-card 上添加点击事件，调用 toggleExpand
- [ ] 2.4 添加展开指示器（▼/▲ 箭头），显示在 tx-top 右侧
- [ ] 2.5 创建分录显示区域，使用 Vue `<Transition>` 组件包裹
- [ ] 2.6 实现分录列表渲染：遍历 tx.postings，显示账户短名、商品代码和金额
- [ ] 2.7 金额颜色：正值绿色（#27ae60），负值红色（#e74c3c）

## 3. 动画与样式

- [ ] 3.1 添加分录区域的 CSS 样式（背景、内边距、分隔线等）
- [ ] 3.2 实现 max-height 过渡动画（展开：0 → 500px，折叠：500px → 0）
- [ ] 3.3 设置过渡持续时间 0.3s，ease-in-out 缓动
- [ ] 3.4 添加 overflow: hidden 确保折叠时内容不溢出

## 4. 验证

- [ ] 4.1 验证纯导入账单不显示金额和账户信息，仅显示备注、成员和标签
- [ ] 4.2 验证正常账单展开后显示完整分录列表
- [ ] 4.3 验证多个账单可以独立展开/折叠
- [ ] 4.4 验证展开/折叠动画平滑
