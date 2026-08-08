# Tasks: fix-saving-plan-badge

## 1. 测试先行(RED)

- [x] 1.1 在 `accounting-web/src/views/__tests__/SavingPlanView.spec.ts` 新增失败用例:两计划 `met` 均为 true、计划 1 `satisfaction=100`、计划 2 `satisfaction=25` 时,计划 1 卡片显示「已达标」徽标,计划 2 显示缺口徽标而非「已达标」

## 2. 实现(GREEN)

- [x] 2.1 `SavingPlanView.vue` 卡片徽标判定从 `status.met` 改为 `Number(status.satisfaction) >= 100`,与 `ringClass`/`ringColor` 口径一致;详情展开区的账面 `met` 展示保持不变
- [x] 2.2 缺口徽标金额从账面 `status.gap` 改为分配口径 `target_amount − allocated`(`allocationGap` 函数),测试断言共享账户场景显示 150 而非 -50

## 3. 验证

- [x] 3.1 运行前端全量测试与 `vue-tsc`,确认无回归
- [x] 3.2 目视确认:共享账户场景下各计划徽标独立(环形 25% 的卡片不再显示「已达标」)
