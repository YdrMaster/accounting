import { describe, expect, it } from 'vitest'
import { fromCents, toCents } from '../money'

describe('toCents', () => {
  it('converts precision 2 strings', () => {
    expect(toCents('678.01')).toBe(67801)
    expect(toCents('0')).toBe(0)
    expect(toCents('0.00')).toBe(0)
    expect(toCents('75')).toBe(7500)
    expect(toCents('100.5')).toBe(10050)
  })

  it('converts negative amounts without negative zero', () => {
    expect(toCents('-678.01')).toBe(-67801)
    expect(toCents('-0.004')).toBe(0)
    expect(toCents('-0.005')).toBe(-1)
  })

  it('supports precision 0 and 8', () => {
    expect(toCents('12', 0)).toBe(12)
    expect(toCents('12.6', 0)).toBe(13)
    expect(toCents('0.0001', 8)).toBe(10000)
    expect(toCents('0.000000001', 8)).toBe(0)
  })

  it('rounds half up on overflowing digits', () => {
    expect(toCents('0.005')).toBe(1)
    expect(toCents('0.004')).toBe(0)
    expect(toCents('1.999')).toBe(200)
    expect(toCents('9.995')).toBe(1000)
  })

  it('throws on invalid input', () => {
    expect(() => toCents('')).toThrow()
    expect(() => toCents('abc')).toThrow()
    expect(() => toCents('1.2.3')).toThrow()
    expect(() => toCents('12.')).toThrow()
    expect(() => toCents('.5')).toThrow()
    expect(() => toCents(' 1')).toThrow()
  })
})

describe('fromCents', () => {
  it('renders fixed precision strings', () => {
    expect(fromCents(67801)).toBe('678.01')
    expect(fromCents(0)).toBe('0.00')
    expect(fromCents(5)).toBe('0.05')
    expect(fromCents(-5)).toBe('-0.05')
    expect(fromCents(100)).toBe('1.00')
    expect(fromCents(-67801)).toBe('-678.01')
  })

  it('supports precision 0 and 8', () => {
    expect(fromCents(12, 0)).toBe('12')
    expect(fromCents(-12, 0)).toBe('-12')
    expect(fromCents(100, 8)).toBe('0.00000100')
    expect(fromCents(5, 8)).toBe('0.00000005')
  })
})

describe('roundtrip', () => {
  it('fromCents(toCents(v)) reproduces fixed 2dp strings', () => {
    for (const v of ['678.01', '0.00', '75.00', '100.50', '-33.33']) {
      expect(fromCents(toCents(v))).toBe(v)
    }
  })

  it('normalizes variable scale to fixed 2dp', () => {
    expect(fromCents(toCents('75'))).toBe('75.00')
    expect(fromCents(toCents('100.5'))).toBe('100.50')
  })
})
