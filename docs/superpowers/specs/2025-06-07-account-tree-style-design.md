# 账户树样式设计

## 参考
Semi Design Tree: https://semi.design/zh-CN/navigation/tree

## 设计要点

### 节点行
- 行高 `40px`，垂直居中
- `padding: 0 16px`，整行占满容器宽度
- **无分隔线**

### 交互状态（覆盖整行）
- `hover`：明亮 `#f5f5f5`，暗色 `#2a2a2a`
- `选中`：明亮 `#e6f7ff`，暗色 `#15325b`
- 选中文字色：明亮 `#1890ff`，暗色 `#177ddc`

### 内容
- 字号 `16px`
- 系统账户：**下划线**标识
- `+` 按钮：hover 时淡入，字号放大（视作图标）

## 实现范围
仅修改 `accounting-web/src/views/AccountTree.vue` 的样式部分。
