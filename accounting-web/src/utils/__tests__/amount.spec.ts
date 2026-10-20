import { describe, expect, it } from 'vitest'
import { formatCalendarAmount, sumAssetAmount } from '../amount'

describe('formatCalendarAmount', () => {
  describe('zh locale', () => {
    it('shows full amount below threshold', () => {
      expect(formatCalendarAmount('340.92', 'zh-CN')).toBe('340.92')
    })

    it('shows full amount at threshold boundary', () => {
      expect(formatCalendarAmount('99999', 'zh-CN')).toBe('99999')
    })

    it('switches to 万 above threshold', () => {
      expect(formatCalendarAmount('100000', 'zh-CN')).toBe('10万')
      expect(formatCalendarAmount('173692.17', 'zh-CN')).toBe('17.4万')
    })
  })

  describe('en locale', () => {
    it('shows full amount below threshold', () => {
      expect(formatCalendarAmount('340.92', 'en')).toBe('340.92')
    })

    it('shows full amount at threshold boundary', () => {
      expect(formatCalendarAmount('9999', 'en')).toBe('9999')
    })

    it('switches to k above threshold', () => {
      expect(formatCalendarAmount('10000', 'en')).toBe('10k')
      expect(formatCalendarAmount('173692.17', 'en')).toBe('173.7k')
    })
  })

  it('passes through variable-scale strings below threshold unchanged', () => {
    expect(formatCalendarAmount('75', 'zh-CN')).toBe('75')
    expect(formatCalendarAmount('100.5', 'en')).toBe('100.5')
    expect(formatCalendarAmount('-50.25', 'zh-CN')).toBe('-50.25')
  })

  it('abbreviates negative amounts above threshold by absolute value', () => {
    expect(formatCalendarAmount('-173692.17', 'zh-CN')).toBe('-17.4万')
  })

  it('throws on invalid decimal string', () => {
    expect(() => formatCalendarAmount('abc', 'zh-CN')).toThrow()
  })
})

describe('sumAssetAmount', () => {
  const p = (account_type: string, amount: string) => ({ account_type, amount })

  it('sums asset postings for an expense', () => {
    expect(sumAssetAmount([p('asset', '-35.00'), p('expense', '35.00')]).toFixed(2)).toBe('-35.00')
  })

  it('sums asset postings for income', () => {
    expect(sumAssetAmount([p('asset', '8000.00'), p('income', '-8000.00')]).toFixed(2)).toBe(
      '8000.00'
    )
  })

  it('returns zero for a zero-net asset transaction (no positive fallback)', () => {
    expect(
      sumAssetAmount([p('asset', '-100.00'), p('asset', '100.00'), p('expense', '0.00')]).isZero()
    ).toBe(true)
  })

  it('returns zero when there are no asset postings', () => {
    expect(sumAssetAmount([p('income', '-10.00'), p('expense', '10.00')]).isZero()).toBe(true)
  })
})
