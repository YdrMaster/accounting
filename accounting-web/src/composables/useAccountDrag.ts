import { onScopeDispose, ref } from 'vue'
import type { AccountDto } from '../types/api'

const DRAG_THRESHOLD = 6
const EXPAND_DELAY = 600
const ROW_Y_TOLERANCE = 4
const GAP_X_TOLERANCE = 8

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

interface CardRect {
  id: number
  parentId: number | null
  rect: DOMRect
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
      if (id === draggingId.value) continue
      const account = options.getAccount(id)
      if (!account) continue
      result.push({ id, parentId: account.parent_id, rect: el.getBoundingClientRect() })
    }
    return result
  }

  function updateHitTest(x: number, y: number) {
    clearExpandTimer()
    dropTargetId.value = null
    insertion.value = null
    const draggedId = draggingId.value
    if (draggedId === null) return

    const cards = collectCards()

    // Direct hit on a card => drop as its child.
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

    // Gap between sibling cards of the dragged account => sibling reorder.
    const dragged = options.getAccount(draggedId)
    if (!dragged) return
    const rows: { top: number; bottom: number; cards: CardRect[] }[] = []
    for (const card of cards) {
      if (card.parentId !== dragged.parent_id) continue
      const row = rows.find(r => Math.abs(r.top - card.rect.top) <= ROW_Y_TOLERANCE)
      if (row) {
        row.cards.push(card)
        row.bottom = Math.max(row.bottom, card.rect.bottom)
      } else {
        rows.push({ top: card.rect.top, bottom: card.rect.bottom, cards: [card] })
      }
    }

    for (const row of rows) {
      if (y < row.top - ROW_Y_TOLERANCE || y > row.bottom + ROW_Y_TOLERANCE) continue
      row.cards.sort((a, b) => a.rect.left - b.rect.left)
      const first = row.cards[0]
      const last = row.cards[row.cards.length - 1]

      if (x >= first.rect.left - GAP_X_TOLERANCE && x < first.rect.left) {
        insertion.value = {
          parentId: dragged.parent_id!,
          beforeId: first.id,
          left: first.rect.left - 3,
          top: first.rect.top,
          height: first.rect.height,
        }
        return
      }
      for (let i = 0; i < row.cards.length - 1; i++) {
        const a = row.cards[i]
        const b = row.cards[i + 1]
        if (x >= a.rect.right && x <= b.rect.left) {
          insertion.value = {
            parentId: dragged.parent_id!,
            beforeId: b.id,
            left: (a.rect.right + b.rect.left) / 2 - 1.5,
            top: a.rect.top,
            height: a.rect.height,
          }
          return
        }
      }
      if (x > last.rect.right && x <= last.rect.right + GAP_X_TOLERANCE) {
        insertion.value = {
          parentId: dragged.parent_id!,
          beforeId: null,
          left: last.rect.right + 1,
          top: last.rect.top,
          height: last.rect.height,
        }
        return
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
