import { describe, expect, it } from 'vitest'
import { formatCalendarAmount } from '../amount'

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
})
