import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ImportResultDto } from '../../types/api'

vi.mock('../../api/client', () => ({
  fetchChannels: vi.fn(),
  createChannel: vi.fn(),
  updateChannel: vi.fn(),
  deleteChannel: vi.fn(),
  importBill: vi.fn(),
}))

import { importBill } from '../../api/client'
import { useChannelStore } from '../channel'

const mockedImport = vi.mocked(importBill)

const sampleResult: ImportResultDto = {
  imported: 320,
  skipped: 2,
  pending_tag_name: 'pending',
  errors: [
    { row: 15, detail: 'transaction closed' },
    { row: 88, detail: 'transaction closed' },
  ],
}

describe('channel store importFile', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('returns import result on success', async () => {
    mockedImport.mockResolvedValue(sampleResult)
    const store = useChannelStore()
    const file = new File(['csv'], 'bill.csv')

    const result = await store.importFile(1, file, 7)

    expect(mockedImport).toHaveBeenCalledWith(1, 7, file)
    expect(result).toEqual(sampleResult)
    expect(store.error).toBeNull()
    expect(store.importingChannelId).toBeNull()
  })

  it('tracks importingChannelId while request is in flight', async () => {
    let resolve: (v: ImportResultDto) => void = () => {}
    mockedImport.mockImplementation(
      () =>
        new Promise<ImportResultDto>(r => {
          resolve = r
        })
    )
    const store = useChannelStore()
    const file = new File(['csv'], 'bill.csv')

    const pending = store.importFile(1, file, 7)
    expect(store.importingChannelId).toBe(1)
    resolve(sampleResult)
    await pending
    expect(store.importingChannelId).toBeNull()
  })

  it('ignores a second import while one is in flight', async () => {
    let resolve: (v: ImportResultDto) => void = () => {}
    mockedImport.mockImplementation(
      () =>
        new Promise<ImportResultDto>(r => {
          resolve = r
        })
    )
    const store = useChannelStore()
    const file = new File(['csv'], 'bill.csv')

    const first = store.importFile(1, file, 7)
    const second = await store.importFile(2, file, 7)

    expect(second).toBeUndefined()
    expect(mockedImport).toHaveBeenCalledTimes(1)
    resolve(sampleResult)
    await first
  })

  it('sets error and returns undefined on failure', async () => {
    mockedImport.mockRejectedValue(new Error('解析失败'))
    const store = useChannelStore()
    const file = new File(['csv'], 'bill.csv')

    const result = await store.importFile(1, file, 7)

    expect(result).toBeUndefined()
    expect(store.error).toBe('解析失败')
    expect(store.importingChannelId).toBeNull()
  })
})
