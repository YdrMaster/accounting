/**
 * TS 侧语义色板：供 echarts 配置、SVG 属性等无法使用 CSS var() 的场景。
 * 值必须与 style.css 中同名 CSS 变量保持一致。
 */
export const PALETTE = {
  expense: '#e74c3c',
  expenseSoft: '#ff7b7b',
  income: '#27ae60',
  incomeSoft: '#4ade80',
  warning: '#f39c12',
  attention: '#f1c40f',
  info: '#3498db',
  infoSoft: '#60a5fa',
  neutral: '#7f8c8d',
} as const
