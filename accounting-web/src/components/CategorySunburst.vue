<script setup lang="ts">
import { useResizeObserver } from '@vueuse/core'
import Decimal from 'decimal.js'
import type { EChartsType } from 'echarts/core'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CategoryAmountItemDto } from '../types/api'
import { formatAmount } from '../utils/amount'
import { echarts } from '../utils/echarts'
import { buildSunburstTree, type SunburstNode } from '../utils/sunburst'

const props = defineProps<{
  title: string
  items: CategoryAmountItemDto[]
}>()

const { t } = useI18n()

// ECharts 官方 sunburst-drink 示例的一级分类配色；只需给一级节点显式着色，
// ECharts 会自动将后代提亮为同色系浅色（sunburstVisual 的 lift）
const PALETTE = [
  '#da0d68',
  '#da1d23',
  '#ebb40f',
  '#187a2f',
  '#0aa3b5',
  '#c94930',
  '#ad213e',
  '#a87b64',
  '#e65832',
]

const containerRef = ref<HTMLDivElement>()
let chart: EChartsType | null = null

const tree = computed(() => buildSunburstTree(props.items))

function maxDepthOf(nodes: { children?: unknown[] }[]): number {
  let max = 1
  for (const n of nodes) {
    if (n.children?.length) max = Math.max(max, 1 + maxDepthOf(n.children as { children?: unknown[] }[]))
  }
  return max
}

function render() {
  if (!chart) return
  const { children, total } = tree.value
  const maxDepth = maxDepthOf(children)
  // 里圈（非最外环）标签落在扇区上且颜色跟随扇区，加白色描边保证可辨；
  // 最外环标签在图外深色背景上，保持原样
  const colorNode = (node: SunburstNode, depth: number, color?: string): SunburstNode => ({
    ...node,
    ...(color ? { itemStyle: { color } } : {}),
    ...(depth < maxDepth - 1
      ? { label: { textBorderColor: 'rgba(255,255,255,0.9)', textBorderWidth: 2 } }
      : {}),
    children: node.children?.map((c) => colorNode(c, depth + 1)),
  })
  const colored = children.map((node, i) => colorNode(node, 0, PALETTE[i % PALETTE.length]))

  chart.setOption({
    tooltip: {
      // 中间节点的 value 是可见子节点之和，真实总额取 data.total
      formatter: (p: { name: string; value: number; data: { total?: number } }) =>
        `${p.name}: ${formatAmount(new Decimal(p.data?.total ?? p.value ?? 0))}`,
    },
    series: [
      {
        type: 'sunburst',
        data: colored,
        // 外圈留边距给外侧标签
        radius: ['12%', '75%'],
        sort: undefined,
        // ECharts 6 运行时的下钻取值是 'rootToNode'（类型声明里的 'zoomToNode' 实际无效）
        nodeClick: 'rootToNode',
        label: {
          show: true,
          // 外侧标签 + 始终水平（rotate: 0），过小扇区（minAngle）不显示避免拥挤
          position: 'outside',
          rotate: 0,
          minAngle: 5,
          fontSize: 11,
          // 标签颜色跟随扇区；显式禁用文字描边与阴影（'inherit' 时 ECharts 默认加白色描边，
          // 仅设 textBorderWidth: 0 不够，需 textBorderColor: 'transparent'）
          color: 'inherit',
          textBorderColor: 'transparent',
          textBorderWidth: 0,
          textShadowBlur: 0,
          formatter: (p: { name: string; value: number; data: { total?: number } }) => {
            if (!total) return p.name
            const amount = p.data?.total ?? p.value ?? 0
            return `${p.name} ${((amount / total) * 100).toFixed(1)}%`
          },
        },
        labelLine: { show: true, length: 8, lineStyle: { color: '#666' } },
        itemStyle: { borderRadius: 7, borderWidth: 2, borderColor: 'rgba(0,0,0,0.2)' },
      },
    ],
  })
}

function initChart(el: HTMLDivElement) {
  chart = echarts.init(el)
  render()
}

onMounted(() => {
  if (containerRef.value) initChart(containerRef.value)
})

watch(containerRef, (el, oldEl) => {
  if (!el) {
    chart?.dispose()
    chart = null
    return
  }
  if (el !== oldEl) {
    chart?.dispose()
    chart = null
    nextTick(() => initChart(el))
  }
})

useResizeObserver(containerRef, () => {
  chart?.resize()
})

watch(() => [props.items, props.title], () => render())

onBeforeUnmount(() => {
  chart?.dispose()
  chart = null
})
</script>

<template>
  <div class="sunburst">
    <h4 class="title">{{ title }}</h4>
    <div v-if="tree.children.length" ref="containerRef" class="chart"></div>
    <div v-else class="empty">{{ t('assets.category.empty') }}</div>
  </div>
</template>

<style scoped>
.sunburst {
  flex: 1;
  min-width: 0;
}

.title {
  margin: 0 0 0.25rem;
  text-align: center;
  font-size: 0.875rem;
  color: var(--text-heading);
}

.chart {
  width: 100%;
  height: 360px;
}

.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}
</style>
