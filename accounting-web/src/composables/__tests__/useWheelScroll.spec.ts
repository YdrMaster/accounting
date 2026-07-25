import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ringDist, useWheelScroll } from '../useWheelScroll'

const N = 5

describe('ringDist', () => {
  it('returns 0 for the page at scrollPos', () => {
    expect(ringDist(2, 2)).toBe(0)
  })

  it('returns signed distance within [-N/2, N/2)', () => {
    expect(ringDist(1, 0)).toBe(1)
    expect(ringDist(2, 0)).toBe(2)
    expect(ringDist(3, 0)).toBe(-2)
    expect(ringDist(4, 0)).toBe(-1)
  })

  it('wraps across the ring boundary', () => {
    expect(ringDist(0, 4)).toBe(1)
    expect(ringDist(4, 0)).toBe(-1)
    expect(ringDist(1, 4)).toBe(2)
    expect(ringDist(4, 1)).toBe(-2)
  })

  it('handles fractional scrollPos continuously', () => {
    expect(ringDist(0, 4.5)).toBeCloseTo(0.5)
    expect(ringDist(4, 0.25)).toBeCloseTo(-1.25)
    expect(ringDist(0, 0.7)).toBeCloseTo(-0.7)
  })

  it('stays in range for negative scrollPos', () => {
    for (let i = 0; i < N; i++) {
      const d = ringDist(i, -1.3)
      expect(d).toBeGreaterThanOrEqual(-N / 2)
      expect(d).toBeLessThan(N / 2)
    }
  })
})

describe('useWheelScroll drag', () => {
  const wheel = useWheelScroll()

  beforeEach(() => {
    wheel.scrollPos.value = 0
  })

  it('moves scrollPos proportionally while dragging left', () => {
    wheel.beginDrag(200, 50)
    wheel.updateDrag(150)
    expect(wheel.scrollPos.value).toBeCloseTo(1)
    wheel.updateDrag(175)
    expect(wheel.scrollPos.value).toBeCloseTo(0.5)
    expect(wheel.isDragging.value).toBe(true)
  })

  it('moves scrollPos in the opposite direction while dragging right', () => {
    wheel.beginDrag(200, 50)
    wheel.updateDrag(250)
    expect(wheel.scrollPos.value).toBeCloseTo(-1)
  })

  it('marks dragMoved only after more than 5px of movement', () => {
    wheel.beginDrag(200, 50)
    wheel.updateDrag(203)
    expect(wheel.dragMoved.value).toBe(false)
    wheel.updateDrag(210)
    expect(wheel.dragMoved.value).toBe(true)
  })

  it('snaps to the nearest page on release', async () => {
    wheel.beginDrag(200, 100)
    wheel.updateDrag(130)
    wheel.endDrag()
    expect(wheel.isDragging.value).toBe(false)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(1), { timeout: 1000 })
  })

  it('springs back when released before half a unit', async () => {
    wheel.beginDrag(200, 100)
    wheel.updateDrag(160)
    wheel.endDrag()
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(0), { timeout: 1000 })
  })

  it('normalizes scrollPos into [0, N) after settling', async () => {
    wheel.beginDrag(200, 100)
    wheel.updateDrag(260)
    wheel.endDrag()
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(N - 1), { timeout: 1000 })
  })
})

describe('useWheelScroll navigation', () => {
  const wheel = useWheelScroll()

  beforeEach(() => {
    wheel.scrollPos.value = 0
  })

  it('stepBy moves one page in the given direction', async () => {
    wheel.stepBy(1)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(1), { timeout: 1000 })
    wheel.stepBy(-1)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(0), { timeout: 1000 })
  })

  it('spinTo takes the shortest ring path', async () => {
    wheel.spinTo(4)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(4), { timeout: 1000 })
    wheel.spinTo(1)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(1), { timeout: 1000 })
  })

  it('rapid stepBy accumulates instead of retargeting the same page', async () => {
    wheel.stepBy(1)
    wheel.stepBy(1)
    await vi.waitFor(() => expect(wheel.scrollPos.value).toBe(2), { timeout: 1000 })
  })
})
