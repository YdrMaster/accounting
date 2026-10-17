import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { AccountDto } from '../../types/api'
import {
  pickContainer,
  pickInsertSlot,
  useAccountDrag,
  type AccountDropResult,
} from '../useAccountDrag'

const rect = (left: number, top: number, width: number, height: number): DOMRect =>
  ({
    left,
    top,
    right: left + width,
    bottom: top + height,
    width,
    height,
    x: left,
    y: top,
    toJSON: () => ({}),
  }) as DOMRect

describe('pickContainer', () => {
  const containers = [
    { id: 1, rect: rect(0, 0, 400, 300) },
    { id: 2, rect: rect(10, 10, 200, 100) },
  ]

  it('返回包含指针的容器', () => {
    expect(pickContainer(containers, 350, 250)?.id).toBe(1)
  })

  it('嵌套时取面积最小的最内层容器', () => {
    expect(pickContainer(containers, 50, 50)?.id).toBe(2)
  })

  it('指针在所有容器外返回 null', () => {
    expect(pickContainer(containers, 500, 500)).toBeNull()
  })
})

describe('pickInsertSlot', () => {
  it('无卡片返回 null（由调用方回落到整框落点）', () => {
    expect(pickInsertSlot([], 10, 10)).toBeNull()
  })

  it('单卡片左侧取「插到其前」', () => {
    const slot = pickInsertSlot([{ id: 7, rect: rect(100, 0, 80, 40) }], 60, 20)
    expect(slot?.beforeId).toBe(7)
    expect(slot?.left).toBe(97)
    expect(slot?.top).toBe(0)
    expect(slot?.height).toBe(40)
  })

  it('同行两卡片之间的空隙取「插到后者之前」', () => {
    const cards = [
      { id: 1, rect: rect(0, 0, 80, 40) },
      { id: 2, rect: rect(90, 0, 80, 40) },
    ]
    const slot = pickInsertSlot(cards, 85, 20)
    expect(slot?.beforeId).toBe(2)
  })

  it('最后一卡片右侧取末位插槽（beforeId 为 null）', () => {
    const slot = pickInsertSlot([{ id: 7, rect: rect(100, 0, 80, 40) }], 250, 20)
    expect(slot?.beforeId).toBeNull()
    expect(slot?.left).toBe(181)
  })

  it('指针位于行上方时归并到最近行的首位插槽', () => {
    const slot = pickInsertSlot([{ id: 7, rect: rect(100, 40, 80, 40) }], 100, 10)
    expect(slot?.beforeId).toBe(7)
  })

  it('指针位于行下方时归并到末位插槽', () => {
    const slot = pickInsertSlot([{ id: 7, rect: rect(100, 0, 80, 40) }], 150, 80)
    expect(slot?.beforeId).toBeNull()
  })

  it('两行之间归并到更近的一行', () => {
    const cards = [
      { id: 1, rect: rect(0, 0, 80, 40) },
      { id: 2, rect: rect(0, 100, 80, 40) },
    ]
    // y=60 距第一行底(40) 20、距第二行顶(100) 40 → 归并第一行末位
    expect(pickInsertSlot(cards, 120, 60)?.beforeId).toBeNull()
    expect(pickInsertSlot(cards, 120, 60)?.left).toBe(81)
    // y=80 距第一行底 40、距第二行顶 20 → 归并第二行首位
    expect(pickInsertSlot(cards, 120, 80)?.beforeId).toBe(2)
  })
})

describe('useAccountDrag.updateHitTest', () => {
  const account = (id: number, parent_id: number | null, name = `acct-${id}`): AccountDto => ({
    id,
    name,
    account_type: 'Asset',
    parent_id,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  })

  let dropped: AccountDropResult[]

  function createDrag(accounts: AccountDto[]) {
    dropped = []
    return useAccountDrag({
      getAccount: id => accounts.find(a => a.id === id),
      getChildren: id => accounts.filter(a => a.parent_id === id),
      isDescendant: (accountId, ancestorId) => {
        let cur = accounts.find(a => a.id === accountId)
        while (cur) {
          if (cur.id === ancestorId) return true
          if (cur.parent_id === null) return false
          cur = accounts.find(a => a.id === cur!.parent_id)
        }
        return false
      },
      isExpanded: () => true,
      expand: () => {},
      onDrop: result => dropped.push(result),
    })
  }

  function installDom(nodes: { selector: string; id: string; rect: DOMRect }[]) {
    const cleanups: (() => void)[] = []
    for (const node of nodes) {
      const el = document.createElement('div')
      if (node.selector === 'container') {
        el.setAttribute('data-container-account-id', node.id)
      } else {
        el.className = 'account-card'
        el.setAttribute('data-account-id', node.id)
      }
      vi.spyOn(el, 'getBoundingClientRect').mockReturnValue(node.rect)
      document.body.appendChild(el)
      cleanups.push(() => el.remove())
    }
    return () => cleanups.forEach(fn => fn())
  }

  function pointer(type: string, x: number, y: number): Event {
    const event = new Event(type)
    Object.assign(event, { clientX: x, clientY: y })
    return event
  }

  beforeEach(() => {
    vi.spyOn(document.body.classList, 'add').mockImplementation(() => {})
    vi.spyOn(document.body.classList, 'remove').mockImplementation(() => {})
  })

  afterEach(() => {
    vi.restoreAllMocks()
    window.dispatchEvent(pointer('pointerup', 0, 0))
  })

  it('卡片命中成为放置目标，松手触发 child 落点', () => {
    const accounts = [account(1, null), account(2, 1), account(3, 1)]
    const drag = createDrag(accounts)
    const cleanup = installDom([
      { selector: 'container', id: '1', rect: rect(0, 0, 400, 300) },
      { selector: 'card', id: '3', rect: rect(100, 50, 80, 40) },
    ])

    drag.onCardPointerDown(accounts[1], {
      pointerType: 'mouse',
      button: 0,
      clientX: 10,
      clientY: 10,
    } as PointerEvent)
    window.dispatchEvent(pointer('pointermove', 20, 20))
    window.dispatchEvent(pointer('pointermove', 120, 60))

    expect(drag.dropTargetId.value).toBe(3)
    expect(drag.insertion.value).toBeNull()

    window.dispatchEvent(pointer('pointerup', 120, 60))
    expect(dropped).toEqual([{ kind: 'child', draggedId: 2, targetId: 3 }])
    cleanup()
  })

  it('拖入空容器整框落点（parentId 为容器 owner）', () => {
    const accounts = [account(1, null), account(2, 1)]
    const drag = createDrag(accounts)
    const cleanup = installDom([{ selector: 'container', id: '1', rect: rect(0, 0, 400, 300) }])

    drag.onCardPointerDown(accounts[1], {
      pointerType: 'mouse',
      button: 0,
      clientX: 10,
      clientY: 10,
    } as PointerEvent)
    window.dispatchEvent(pointer('pointermove', 20, 20))
    window.dispatchEvent(pointer('pointermove', 200, 150))

    expect(drag.dropTargetId.value).toBeNull()
    expect(drag.insertion.value).toMatchObject({ parentId: 1, beforeId: null })

    window.dispatchEvent(pointer('pointerup', 200, 150))
    expect(dropped).toEqual([{ kind: 'insert', draggedId: 2, parentId: 1, beforeId: null }])
    cleanup()
  })

  it('跨层级：深层卡片拖到一级卡片间隙，插入目标为该容器 owner', () => {
    const accounts = [account(1, null), account(2, 1), account(3, 2)]
    const drag = createDrag(accounts)
    const cleanup = installDom([
      { selector: 'container', id: '1', rect: rect(0, 0, 400, 300) },
      { selector: 'card', id: '2', rect: rect(100, 0, 80, 40) },
    ])

    drag.onCardPointerDown(accounts[2], {
      pointerType: 'mouse',
      button: 0,
      clientX: 300,
      clientY: 290,
    } as PointerEvent)
    window.dispatchEvent(pointer('pointermove', 310, 280))
    window.dispatchEvent(pointer('pointermove', 85, 20))

    expect(drag.insertion.value).toMatchObject({ parentId: 1, beforeId: 2 })
    cleanup()
  })

  it('自身后代卡片与其容器均拒绝作为落点', () => {
    const accounts = [account(1, null), account(2, 1), account(3, 2)]
    const drag = createDrag(accounts)
    // 容器 3（账户 3 的容纳框）与卡片 3 都是被拖账户 2 的后代，
    // 且该区域不与任何允许容器重叠
    const cleanup = installDom([
      { selector: 'container', id: '1', rect: rect(0, 0, 40, 40) },
      { selector: 'container', id: '3', rect: rect(50, 50, 200, 100) },
      { selector: 'card', id: '3', rect: rect(60, 60, 80, 40) },
    ])

    drag.onCardPointerDown(accounts[1], {
      pointerType: 'mouse',
      button: 0,
      clientX: 10,
      clientY: 10,
    } as PointerEvent)
    window.dispatchEvent(pointer('pointermove', 20, 20))
    window.dispatchEvent(pointer('pointermove', 70, 70))

    expect(drag.dropTargetId.value).toBeNull()
    expect(drag.insertion.value).toBeNull()
    cleanup()
  })
})
