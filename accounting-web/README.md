# try-frontend

基于 Vue 3.6 + Vite + TypeScript 的最小单页应用（SPA）。

## 技术栈

- Node.js 24（LTS）
- Vue 3.6（当前使用 3.6.0-beta）
- Vue Router 5
- Vite 8（Vite 9 尚未发布，当前最新可用版本为 8.1.0）
- TypeScript 6.0.3
- Vitest 5（beta）+ @vue/test-utils + happy-dom
- ESLint 10 + eslint-plugin-vue + typescript-eslint
- Prettier 3 + prettier-plugin-organize-imports

## 常用命令

```bash
# 安装依赖（Vue 3.6 为 beta，需要跳过严格的 peer dependency 检查）
npm install --legacy-peer-deps

# 启动开发服务器
npm run dev

# 生产构建
npm run build

# 预览生产构建
npm run preview

# 运行单元测试
npm run test

# 代码检查
npm run lint
npm run lint:fix

# 格式化
npm run format
npm run format:check
```

## 环境说明

项目使用 Node.js 24 LTS。仓库根目录已添加 `.nvmrc`，如果使用 nvm，进入目录后执行：

```bash
nvm use
```

即可切换到正确版本。

## 代码质量说明

项目使用 **ESLint + Prettier** 统一代码风格与质量，并已覆盖 `.vue` 单文件组件：

- 2 个空格缩进
- 单引号、无分号
- 100 列换行
- 自动排序 import
- Vue 官方推荐的 `eslint-plugin-vue` 规则
- TypeScript 推荐规则

## 目录结构

```plaintext
src/
  components/            # 可复用组件
  components/__tests__/  # 组件测试
  router/                # Vue Router 配置
  views/                 # 页面级组件
  App.vue                # 根组件
  main.ts                # 应用入口
  style.css              # 全局样式
```
