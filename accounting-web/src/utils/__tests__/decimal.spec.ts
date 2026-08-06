import { describe, expect, it } from 'vitest'
import { formatDecimal } from '../decimal'

describe('formatDecimal', () => {
  it('strips trailing zeros', () => {
    expect(formatDecimal('75.00')).toBe('75')
    expect(formatDecimal('500.00')).toBe('500')
    expect(formatDecimal('-1000.00')).toBe('-1000')
  })

  it('rounds recurring decimals to dp places', () => {
    expect(formatDecimal('33.33333333333333333333333333')).toBe('33.33')
    expect(formatDecimal('66.66666666666666666666666667')).toBe('66.67')
  })

  it('keeps integers and short decimals as-is', () => {
    expect(formatDecimal('100')).toBe('100')
    expect(formatDecimal('2.5')).toBe('2.5')
  })

  it('returns non-numeric input unchanged', () => {
    expect(formatDecimal('abc')).toBe('abc')
  })
})
