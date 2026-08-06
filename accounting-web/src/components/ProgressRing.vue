<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    /** 0-100，超过 100 时弧按 100 绘制，中心内容由调用方决定 */
    percentage: number
    /** 进度弧颜色 */
    color: string
    /** 圆环直径（px） */
    size?: number
  }>(),
  { size: 64 }
)

const strokeWidth = computed(() => props.size / 8)
const radius = computed(() => props.size / 2 - strokeWidth.value / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)
const arcLength = computed(
  () => (Math.min(Math.max(props.percentage, 0), 100) / 100) * circumference.value
)
</script>

<template>
  <div class="progress-ring" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
      <circle
        class="progress-ring-track"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        fill="none"
        :stroke-width="strokeWidth"
      />
      <circle
        v-if="arcLength > 0"
        class="progress-ring-arc"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        fill="none"
        :stroke="color"
        :stroke-width="strokeWidth"
        :stroke-dasharray="`${arcLength} ${circumference}`"
        stroke-linecap="round"
        :transform="`rotate(-90 ${size / 2} ${size / 2})`"
      />
    </svg>
    <div class="progress-ring-center">
      <slot>{{ percentage }}%</slot>
    </div>
  </div>
</template>

<style scoped>
.progress-ring {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.progress-ring-track {
  stroke: var(--border, #333333);
}

.progress-ring-center {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-heading);
}
</style>
