# 账户分类选择器设计

## 概述
将 TransactionForm 中的树形下拉选择器替换为分层级点击的弹出面板选择器。

## 交互流程
1. 点击"选择账户"按钮 → 弹出面板
2. 显示四个大类（资产/权益/支出/收入）
3. 点击分类 → 面包屑更新，显示子分类
4. 点击叶子节点 → 直接选中并关闭面板
5. 面包屑可点击回退到任意层级

## 组件接口
```typescript
interface AccountPicker {
  modelValue?: number  // 选中的账户 ID
  accountType?: string // 可选：限定账户类型
}
```

## 状态管理
- `currentPath: number[]` — 当前路径
- `currentLevel: Account[]` — 当前层级子账户
- 面包屑点击 → 截断路径，重新计算层级

## 样式
- 网格布局：`repeat(auto-fill, minmax(80px, 1fr))`
- 按钮高度：40px
- 叶子节点：绿色边框
- 非叶子节点：灰色边框

## 特殊模式
- 退款/报销模式：只显示资产账户
