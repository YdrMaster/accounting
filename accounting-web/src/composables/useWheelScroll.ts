import { ref, watch } from 'vue'
import { paneNames, useResponsiveLayout } from './useResponsiveLayout'

const N = paneNames.length

export function ringDist(index: number, scrollPos: number, count: number = N): number {
  const raw = (((index - scrollPos + count / 2) % count) + count) % count
  return raw - count / 2
}

const scrollPos = ref(0)
const isDragging = ref(false)
const isAnimating = ref(false)
const dragMoved = ref(false)

let dragStartX = 0
let dragAnchor = 0
let dragUnit = 1
let animFrame = 0
let animTarget: number | null = null

function normalize(pos: number): number {
  return ((pos % N) + N) % N
}

function cancelAnimation() {
  if (animFrame) {
    cancelAnimationFrame(animFrame)
    animFrame = 0
  }
  animTarget = null
  isAnimating.value = false
}

function animateTo(target: number, duration = 300) {
  cancelAnimation()
  const from = scrollPos.value
  const delta = target - from
  if (Math.abs(delta) < 1e-6) {
    scrollPos.value = normalize(target)
    return
  }
  animTarget = target
  isAnimating.value = true
  const startTime = performance.now()
  const ease = (t: number) => 1 - Math.pow(1 - t, 3)

  const step = (now: number) => {
    const t = Math.min(1, (now - startTime) / duration)
    scrollPos.value = from + delta * ease(t)
    if (t < 1) {
      animFrame = requestAnimationFrame(step)
    } else {
      animFrame = 0
      animTarget = null
      isAnimating.value = false
      scrollPos.value = normalize(target)
    }
  }
  animFrame = requestAnimationFrame(step)
}

function beginDrag(startX: number, unitWidth: number) {
  cancelAnimation()
  isDragging.value = true
  dragMoved.value = false
  dragStartX = startX
  dragAnchor = scrollPos.value
  dragUnit = Math.max(1, unitWidth)
}

function updateDrag(currentX: number) {
  if (!isDragging.value) return
  if (Math.abs(currentX - dragStartX) > 5) dragMoved.value = true
  scrollPos.value = dragAnchor - (currentX - dragStartX) / dragUnit
}

function endDrag() {
  if (!isDragging.value) return
  isDragging.value = false
  animateTo(Math.round(scrollPos.value))
}

function stepBy(delta: number) {
  const base = animTarget ?? Math.round(scrollPos.value)
  animateTo(base + delta)
}

function spinTo(index: number) {
  animateTo(scrollPos.value + ringDist(index, scrollPos.value))
}

const { columns } = useResponsiveLayout()
watch(columns, () => {
  if (!isDragging.value) animateTo(Math.round(scrollPos.value))
})

export function useWheelScroll() {
  return {
    scrollPos,
    isDragging,
    isAnimating,
    dragMoved,
    beginDrag,
    updateDrag,
    endDrag,
    stepBy,
    spinTo,
    animateTo,
  }
}
