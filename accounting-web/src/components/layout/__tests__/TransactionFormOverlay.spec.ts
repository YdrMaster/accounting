import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n } from '../../../i18n'
import { useTransactionStore } from '../../../stores/transaction'
import { dialogState, resolveDialog } from '../../../utils/dialog'
import TransactionFormOverlay from '../TransactionFormOverlay.vue'

vi.mock('../../../api/client', () => {
  const tx = {
    id: 1,
    date_time: '2026-01-01T12:00:00',
    description: '午饭',
    kind: 'normal',
    member_id: 1,
    member_name: 'm',
    tags: [],
    pending: false,
    channel_paths: [],
    postings: [
      {
        id: 1,
        transaction_id: 1,
        account_id: 1,
        account: 'Expenses:Fees',
        account_type: 'expense',
        commodity: 'CNY',
        amount: '10.00',
        is_reimbursable: false,
        linked_posting_id: null,
        reversal_total: '0',
      },
      {
        id: 2,
        transaction_id: 1,
        account_id: 2,
        account: 'Assets:Cash',
        account_type: 'asset',
        commodity: 'CNY',
        amount: '-10.00',
        is_reimbursable: false,
        linked_posting_id: null,
        reversal_total: '0',
      },
    ],
    is_transfer: false,
  }
  return {
    fetchTransaction: vi.fn(async (id: number) => ({ ...tx, id })),
    fetchMembers: vi.fn(async () => []),
    fetchCommodities: vi.fn(async () => [{ id: 1, symbol: 'CNY', name: 'CNY' }]),
    fetchChannels: vi.fn(async () => []),
    fetchTags: vi.fn(async () => []),
    fetchAccounts: vi.fn(async () => []),
  }
})

function mountOverlay(editId?: number) {
  return mount(TransactionFormOverlay, {
    props: { editId },
    global: {
      plugins: [i18n],
      stubs: { AccountPicker: true, ChannelPathInput: true },
    },
  })
}

describe('TransactionFormOverlay 删除操作', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('编辑态显示删除按钮，新建态不显示', async () => {
    const editWrapper = mountOverlay(1)
    await flushPromises()
    expect(editWrapper.find('.delete-btn').exists()).toBe(true)

    const createWrapper = mountOverlay()
    await flushPromises()
    expect(createWrapper.find('.delete-btn').exists()).toBe(false)
  })

  it('确认删除后调用 store.remove 并关闭表单', async () => {
    const store = useTransactionStore()
    const removeSpy = vi.spyOn(store, 'remove').mockResolvedValue(undefined)
    const wrapper = mountOverlay(1)
    await flushPromises()

    await wrapper.find('.delete-btn').trigger('click')
    expect(dialogState.visible).toBe(true)
    expect(dialogState.kind).toBe('confirm')

    resolveDialog(true)
    await flushPromises()

    expect(removeSpy).toHaveBeenCalledWith(1)
    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('取消删除不调用 store.remove 且表单保留', async () => {
    const store = useTransactionStore()
    const removeSpy = vi.spyOn(store, 'remove').mockResolvedValue(undefined)
    const wrapper = mountOverlay(1)
    await flushPromises()

    await wrapper.find('.delete-btn').trigger('click')
    resolveDialog(false)
    await flushPromises()

    expect(removeSpy).not.toHaveBeenCalled()
    expect(wrapper.emitted('close')).toBeFalsy()
    expect(wrapper.find('.delete-btn').exists()).toBe(true)
  })
})
