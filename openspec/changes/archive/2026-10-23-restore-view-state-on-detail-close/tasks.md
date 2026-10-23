# Tasks: restore-view-state-on-detail-close

## 1. 交易页：覆盖层往返保留列表状态

- [x] 1.1 在 `accounting-web/src/views/TransactionView.vue` 中，覆盖层打开时不再卸载列表内容（将 `<template v-if="!showFormOverlay">` 改为保留挂载的隐藏方式），并保留 `no-scroll` 防滚动；验证：手动打开覆盖层后滚动容器内仍有列表节点，关闭后 `scrollTop` 与打开前一致
- [x] 1.2 覆盖层打开时对底层滚动容器施加交互屏蔽（优先 `inert`，不支持时退化 `aria-hidden="true"`）；验证：单元测试断言覆盖层打开时底层容器带 `inert`（或兜底属性），关闭后属性移除
- [x] 1.3 在 `accounting-web/src/views/__tests__/TransactionView.spec.ts` 补充「滚动后打开→关闭，滚动位置保持」的用例（编辑与新建两条路径）；验证：`npm test -- src/views/__tests__/TransactionView.spec.ts` 通过

## 2. 日历页：覆盖层往返保留列表与日历状态

- [x] 2.1 在 `accounting-web/src/views/CalendarView.vue` 中，覆盖层打开时保持 `CalendarGrid` 与 `TransactionList` 挂载（不再 `v-if` 卸载），并保留 `no-scroll`；验证：单元测试断言覆盖层打开后 `CalendarGrid` 仍存在于 DOM
- [x] 2.2 覆盖层打开时对底层容器施加交互屏蔽（与 1.2 同一策略）；验证：单元测试断言覆盖层打开时底层容器带 `inert`（或兜底属性）
- [x] 2.3 在 `accounting-web/src/views/__tests__/CalendarViewRefresh.spec.ts` 补充「翻月并选中某日 → 打开 → 关闭后显示月份与选中日期不变」的用例；验证：`npm test -- src/views/__tests__/CalendarViewRefresh.spec.ts` 通过
- [x] 2.4 补充日历页下方交易列表滚动位置在往返后保持的用例；验证：该用例通过

## 3. 集成与回归验证

- [x] 3.1 运行前端全量测试 `npm test`（在 `accounting-web/` 下），确认无回归；验证：测试套件全绿
- [x] 3.2 运行 `npm run lint` 与 `npm run build`，确认类型检查与构建通过
- [ ] 3.3 手工核验关键路径：交易页滚动后双击某笔→关闭仍在原位；日历页翻月→点某日→打开详情→关闭仍显示该月且该日选中；新建/保存/删除后关闭同样回到原位置；确认筛选条件变化后仍按既有行为重置到顶部

## Workflow follow-up

- 实现完成并通过评审后归档本变更。
