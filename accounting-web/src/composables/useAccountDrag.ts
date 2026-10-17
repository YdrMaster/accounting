import { onScopeDispose, ref } from 'vue'
import type { AccountDto } from '../types/api'

const DRAG_THRESHOLD = 6
const EXPAND_DELAY = 600
const ROW_Y_TOLERANCE = 4

export type AccountDropResult =
  | { kind: 'child'; draggedId: number; targetId: number }
  | { kind: 'insert'; draggedId: number; parentId: number; beforeId: number | null }

export interface InsertionIndicator {
  parentId: number
  beforeId: number | null
  left: number
  top: number
  height: number
}

interface UseAccountDragOptions {
  getAccount: (id: number) => AccountDto | undefined
  getChildren: (id: number) => AccountDto[]
  isDescendant: (accountId: number, ancestorId: number) => boolean
  isExpanded: (id: number) => boolean
  expand: (id: number) => void
  onDrop: (result: AccountDropResult) => void
}

interface RectLike {
  left: number
  right: number
  top: number
  bottom: number
  width: number
  height: number
}

interface CardRect {
  id: number
  parentId: number | null
  rect: RectLike
}

interface ContainerRect {
  id: number
  rect: RectLike
}

export interface InsertSlot {
  beforeId: number | null
  left: number
  top: number
  height: number
}

/** 取包含指针的最内层（面积最小）容器；无命中返回 null。 */
export function pickContainer(
  containers: ContainerRect[],
  x: number,
  y: number
): ContainerRect | null {
  let best: ContainerRect | null = null
  let bestArea = Infinity
  for (const c of containers) {
    const r = c.rect
    if (x < r.left || x > r.right || y < r.top || y > r.bottom) continue
    const area = r.width * r.height
    if (area < bestArea) {
      bestArea = area
      best = c
    }
  }
  return best
}

/** 在容器直接子卡片的几何中取最近插槽；卡片为空返回 null（由调用方回落到整框落点）。 */
export function pickInsertSlot(
  cards: { id: number; rect: RectLike }[],
  x: number,
  y: number
): InsertSlot | null {
  if (cards.length === 0) return null

  const rows: { top: number; bottom: number; cards: { id: number; rect: RectLike }[] }[] = []
  for (const card of cards) {
    const row = rows.find(r => Math.abs(r.top - card.rect.top) <= ROW_Y_TOLERANCE)
    if (row) {
      row.cards.push(card)
      row.bottom = Math.max(row.bottom, card.rect.bottom)
    } else {
      rows.push({ top: card.rect.top, bottom: card.rect.bottom, cards: [card] })
    }
  }
  rows.sort((a, b) => a.top - b.top)
  for (const row of rows) row.cards.sort((a, b) => a.rect.left - b.rect.left)

  const distToRow = (row: { top: number; bottom: number }) =>
    y < row.top ? row.top - y : y > row.bottom ? y - row.bottom : 0
  const row = rows.reduce((nearest, r) => (distToRow(r) < distToRow(nearest) ? r : nearest))

  const first = row.cards[0]
  const last = row.cards[row.cards.length - 1]
  const slotBefore = (card: { id: number; rect: RectLike }): InsertSlot => ({
    beforeId: card.id,
    left: card.rect.left - 3,
    top: card.rect.top,
    height: card.rect.height,
  })
  const slotAfter = (card: { id: number; rect: RectLike }): InsertSlot => ({
    beforeId: null,
    left: card.rect.right + 1,
    top: card.rect.top,
    height: card.rect.height,
  })

  // 垂直方向在行带之外时，归并到该行的首位/末位插槽
  if (y < row.top) return slotBefore(first)
  if (y > row.bottom) return slotAfter(last)

  if (x < first.rect.left) return slotBefore(first)
  for (const card of row.cards) {
    if (x <= card.rect.right) return slotBefore(card)
  }
  return slotAfter(last)
}

export function useAccountDrag(options: UseAccountDragOptions) {
  const draggingId = ref<number | null>(null)
  const dragPosition = ref({ x: 0, y: 0 })
  const dropTargetId = ref<number | null>(null)
  const insertion = ref<InsertionIndicator | null>(null)

  let pendingId: number | null = null
  let startX = 0
  let startY = 0
  let expandTimer: number | null = null
  let suppressClick = false

  function onCardPointerDown(account: AccountDto, event: PointerEvent) {
    if (account.is_system || account.parent_id === null) return
    if (event.pointerType === 'mouse' && event.button !== 0) return
    pendingId = account.id
    startX = event.clientX
    startY = event.clientY
    window.addEventListener('pointermove', onPointerMove)
    window.addEventListener('pointerup', onPointerUp)
    window.addEventListener('pointercancel', onPointerCancel)
  }

  function onPointerMove(event: PointerEvent) {
    if (pendingId === null && draggingId.value === null) return
    if (draggingId.value === null) {
      if (Math.hypot(event.clientX - startX, event.clientY - startY) < DRAG_THRESHOLD) return
      draggingId.value = pendingId
      pendingId = null
      document.body.classList.add('account-drag-active')
    }
    dragPosition.value = { x: event.clientX, y: event.clientY }
    updateHitTest(event.clientX, event.clientY)
  }

  function onPointerUp() {
    if (draggingId.value !== null) {
      suppressClick = true
      const draggedId = draggingId.value
      if (dropTargetId.value !== null) {
        options.onDrop({ kind: 'child', draggedId, targetId: dropTargetId.value })
      } else if (insertion.value !== null) {
        const ins = insertion.value
        options.onDrop({
          kind: 'insert',
          draggedId,
          parentId: ins.parentId,
          beforeId: ins.beforeId,
        })
      }
    }
    cleanup()
  }

  function onPointerCancel() {
    cleanup()
  }

  function cleanup() {
    clearExpandTimer()
    draggingId.value = null
    pendingId = null
    dropTargetId.value = null
    insertion.value = null
    document.body.classList.remove('account-drag-active')
    window.removeEventListener('pointermove', onPointerMove)
    window.removeEventListener('pointerup', onPointerUp)
    window.removeEventListener('pointercancel', onPointerCancel)
  }

  function clearExpandTimer() {
    if (expandTimer !== null) {
      window.clearTimeout(expandTimer)
      expandTimer = null
    }
  }

  function scheduleExpand(id: number) {
    clearExpandTimer()
    if (options.getChildren(id).length === 0 || options.isExpanded(id)) return
    expandTimer = window.setTimeout(() => {
      expandTimer = null
      options.expand(id)
    }, EXPAND_DELAY)
  }

  function collectCards(): CardRect[] {
    const els = document.querySelectorAll<HTMLElement>('.account-card[data-account-id]')
    const result: CardRect[] = []
    for (const el of els) {
      const id = Number(el.dataset.accountId)
      if (Number.isNaN(id)) continue
      if (id === draggingId.value) continue
      const account = options.getAccount(id)
      if (!account) continue
      result.push({ id, parentId: account.parent_id, rect: el.getBoundingClientRect() })
    }
    return result
  }

  function collectContainers(): ContainerRect[] {
    const els = document.querySelectorAll<HTMLElement>('[data-container-account-id]')
    const result: ContainerRect[] = []
    for (const el of els) {
      const id = Number(el.dataset.containerAccountId)
      if (Number.isNaN(id)) continue
      result.push({ id, rect: el.getBoundingClientRect() })
    }
    return result
  }

  function updateHitTest(x: number, y: number) {
    clearExpandTimer()
    dropTargetId.value = null
    insertion.value = null
    const draggedId = draggingId.value
    if (draggedId === null) return
    if (!options.getAccount(draggedId)) return

    const cards = collectCards()

    // 1) 卡片直接命中 => 成为该卡片的最后一个子账户
    for (const card of cards) {
      const r = card.rect
      if (x >= r.left && x <= r.right && y >= r.top && y <= r.bottom) {
        if (!options.isDescendant(card.id, draggedId)) {
          dropTargetId.value = card.id
          scheduleExpand(card.id)
        }
        return
      }
    }

    // 2) 最内层包含容器（自身及其子树容器不可作为落点，防成环）
    const containers = collectContainers().filter(c => !options.isDescendant(c.id, draggedId))
    const container = pickContainer(containers, x, y)
    if (!container) return

    // 3) 容器内最近插槽；空容器整框即落点
    const childCards = cards.filter(c => c.parentId === container.id)
    const slot = pickInsertSlot(childCards, x, y)
    if (slot) {
      insertion.value = {
        parentId: container.id,
        beforeId: slot.beforeId,
        left: slot.left,
        top: slot.top,
        height: slot.height,
      }
    } else {
      insertion.value = {
        parentId: container.id,
        beforeId: null,
        left: container.rect.left - 3,
        top: container.rect.top,
        height: container.rect.height,
      }
    }
  }

  function onWindowClick(event: MouseEvent) {
    if (!suppressClick) return
    suppressClick = false
    event.stopPropagation()
    event.preventDefault()
  }

  window.addEventListener('click', onWindowClick, true)
  onScopeDispose(() => {
    window.removeEventListener('click', onWindowClick, true)
    cleanup()
  })

  return {
    draggingId,
    dragPosition,
    dropTargetId,
    insertion,
    onCardPointerDown,
  }
}
