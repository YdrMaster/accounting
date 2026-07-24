<script setup lang="ts">
import { useResizeObserver } from '@vueuse/core'
import Decimal from 'decimal.js'
import type { EChartsType } from 'echarts/core'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ChartPeriod, NetWorthTrendPointDto } from '../types/api'
import { formatAmount } from '../utils/amount'
import { echarts } from '../utils/echarts'

const props = defineProps<{
  points: NetWorthTrendPointDto[]
  period: ChartPeriod
}>()

const { t, locale } = useI18n()

const MIN_POINT_WIDTH = 48

const containerRef = ref<HTMLDivElement>()
let chart: EChartsType | null = null

function axisLabel(date: string): string {
  if (props.period === 'weekly') return date.slice(5)
  if (props.period === 'monthly') return date.slice(0, 7)
  return date.slice(0, 4)
}

function yLabel(v: number): string {
  if (locale.value.startsWith('zh')) {
    return Math.abs(v) >= 10000 ? `${Math.round(v / 10000)}万` : String(v)
  }
  return Math.abs(v) >= 1000 ? `${Math.round(v / 1000)}k` : String(v)
}

function render() {
  if (!chart || !containerRef.value) return
  const width = containerRef.value.clientWidth
  const n = Math.max(2, Math.floor(width / MIN_POINT_WIDTH))
  const visible = props.points.slice(-n)

  const dates = visible.map(p => axisLabel(p.date))
  const liabilities = visible.map(p => Number(p.liabilities))
  const netWorth = visible.map(p => Number(p.assets) - Number(p.liabilities))

  chart.setOption({
    animation: false,
    grid: { left: 8, right: 12, top: 12, bottom: 4, containLabel: true },
    tooltip: {
      trigger: 'axis',
      formatter: (params: { dataIndex: number }[]) => {
        const p = visible[params[0].dataIndex]
        if (!p) return ''
        const assets = new Decimal(p.assets)
        const liab = new Decimal(p.liabilities)
        const net = assets.minus(liab)
        return [
          `<b>${p.date}</b>`,
          `${t('assets.totalAssets')}: ${formatAmount(assets)}`,
          `${t('assets.totalLiabilities')}: ${formatAmount(liab)}`,
          `${t('assets.netWorth')}: ${formatAmount(net)}`,
        ].join('<br/>')
      },
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: { fontSize: 10 },
    },
    yAxis: {
      type: 'value',
      axisLabel: { fontSize: 10, formatter: yLabel },
      splitLine: { lineStyle: { opacity: 0.3 } },
    },
    series: [
      {
        name: t('assets.totalLiabilities'),
        type: 'line',
        stack: 'net-worth',
        stackStrategy: 'all',
        data: liabilities,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: { width: 1.5, color: '#ff7b7b' },
        itemStyle: { color: '#ff7b7b' },
      },
      {
        name: t('assets.netWorth'),
        type: 'line',
        stack: 'net-worth',
        stackStrategy: 'all',
        data: netWorth,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: { width: 1.5, color: '#4ade80' },
        itemStyle: { color: '#4ade80' },
        areaStyle: { color: 'rgba(74, 222, 128, 0.15)' },
      },
    ],
  })
}

onMounted(() => {
  if (containerRef.value) {
    chart = echarts.init(containerRef.value)
    render()
  }
})

watch(containerRef, el => {
  if (el && !chart) {
    chart = echarts.init(el)
    nextTick(render)
  }
})

useResizeObserver(containerRef, () => {
  chart?.resize()
  render()
})

watch(
  () => [props.points, props.period, locale.value],
  () => render()
)

onBeforeUnmount(() => {
  chart?.dispose()
  chart = null
})
</script>

<template>
  <div v-if="points.length" ref="containerRef" class="trend-chart"></div>
  <div v-else class="empty">{{ t('assets.trend.empty') }}</div>
</template>

<style scoped>
.trend-chart {
  width: 100%;
  height: 220px;
}

.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}
</style>
