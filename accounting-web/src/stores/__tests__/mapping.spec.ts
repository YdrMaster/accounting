import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MappingDto } from '../../types/api'

vi.mock('../../api/client', () => ({
  fetchMappings: vi.fn(),
  upsertMapping: vi.fn(),
  deleteMapping: vi.fn(),
}))

import { deleteMapping, fetchMappings, upsertMapping } from '../../api/client'
import { useMappingStore } from '../mapping'

const mockedFetch = vi.mocked(fetchMappings)
const mockedUpsert = vi.mocked(upsertMapping)
const mockedDelete = vi.mocked(deleteMapping)

const sample: MappingDto[] = [
  { member_id: 1, channel_id: 2, category: 'Expenses:餐饮美食', account_id: 10 },
  { member_id: 1, channel_id: 2, category: 'Assets:余额宝', account_id: 11 },
]

describe('mapping store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('load fetches mappings and caches them under member:channel key', async () => {
    mockedFetch.mockResolvedValue(sample)
    const store = useMappingStore()

    await store.load(1, 2)

    expect(mockedFetch).toHaveBeenCalledWith(1, 2)
    expect(store.forKey(1, 2)).toEqual(sample)
    expect(store.error).toBeNull()
  })

  it('forKey returns empty array for unknown key', () => {
    const store = useMappingStore()
    expect(store.forKey(9, 9)).toEqual([])
  })

  it('load sets error and keeps cache empty on failure', async () => {
    mockedFetch.mockRejectedValue(new Error('network down'))
    const store = useMappingStore()

    await store.load(1, 2)

    expect(store.error).toBe('network down')
    expect(store.forKey(1, 2)).toEqual([])
  })

  it('set upserts via api and appends a new mapping to the cache', async () => {
    mockedFetch.mockResolvedValue(sample)
    mockedUpsert.mockResolvedValue('ok')
    const store = useMappingStore()
    await store.load(1, 2)

    const dto: MappingDto = {
      member_id: 1,
      channel_id: 2,
      category: 'Income:工资',
      account_id: 12,
    }
    await store.set(dto)

    expect(mockedUpsert).toHaveBeenCalledWith(dto)
    expect(store.forKey(1, 2)).toHaveLength(3)
    expect(store.forKey(1, 2)[2]).toEqual(dto)
  })

  it('set replaces an existing mapping with the same category', async () => {
    mockedFetch.mockResolvedValue(sample)
    mockedUpsert.mockResolvedValue('ok')
    const store = useMappingStore()
    await store.load(1, 2)

    const dto: MappingDto = {
      member_id: 1,
      channel_id: 2,
      category: 'Expenses:餐饮美食',
      account_id: 99,
    }
    await store.set(dto)

    expect(store.forKey(1, 2)).toHaveLength(2)
    expect(store.forKey(1, 2)[0].account_id).toBe(99)
  })

  it('set sets error and leaves cache untouched on failure', async () => {
    mockedFetch.mockResolvedValue(sample)
    mockedUpsert.mockRejectedValue(new Error('upsert failed'))
    const store = useMappingStore()
    await store.load(1, 2)

    await store.set({
      member_id: 1,
      channel_id: 2,
      category: 'Income:工资',
      account_id: 12,
    })

    expect(store.error).toBe('upsert failed')
    expect(store.forKey(1, 2)).toEqual(sample)
  })

  it('remove deletes via api and drops the mapping from the cache', async () => {
    mockedFetch.mockResolvedValue(sample)
    mockedDelete.mockResolvedValue(undefined)
    const store = useMappingStore()
    await store.load(1, 2)

    await store.remove(1, 2, 'Expenses:餐饮美食')

    expect(mockedDelete).toHaveBeenCalledWith(1, 2, 'Expenses:餐饮美食')
    expect(store.forKey(1, 2)).toEqual([sample[1]])
  })

  it('remove sets error and keeps cache on failure', async () => {
    mockedFetch.mockResolvedValue(sample)
    mockedDelete.mockRejectedValue(new Error('delete failed'))
    const store = useMappingStore()
    await store.load(1, 2)

    await store.remove(1, 2, 'Expenses:餐饮美食')

    expect(store.error).toBe('delete failed')
    expect(store.forKey(1, 2)).toEqual(sample)
  })
})
