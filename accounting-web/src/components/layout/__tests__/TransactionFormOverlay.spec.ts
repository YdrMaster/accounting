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

describe('TransactionFormOverlay 金额合法性', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  async function mountEdit() {
    const wrapper = mountOverlay(1)
    await flushPromises()
    return wrapper
  }

  function amountInput(wrapper: ReturnType<typeof mountOverlay>, index: number) {
    return wrapper.findAll('.amount-input')[index]
  }

  it('清空后输入 - 不崩溃，标红且确认/添加分录按钮均禁用', async () => {
    const wrapper = await mountEdit()
    await amountInput(wrapper, 0).setValue('-')

    expect(wrapper.find('.form-overlay').exists()).toBe(true)
    const input = amountInput(wrapper, 0)
    expect((input.element as HTMLInputElement).value).toBe('-')
    expect(input.classes()).toContain('invalid')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()
    expect(wrapper.find('.add-posting-btn').attributes('disabled')).toBeDefined()
  })

  it('其他非法中间态（. 和 -.）同样标红且不崩溃', async () => {
    const wrapper = await mountEdit()
    for (const v of ['.', '-.']) {
      await amountInput(wrapper, 0).setValue(v)
      expect(wrapper.find('.form-overlay').exists()).toBe(true)
      expect(amountInput(wrapper, 0).classes()).toContain('invalid')
    }
  })

  it('合法金额与空串都不标红', async () => {
    const wrapper = await mountEdit()
    for (const v of ['-5', '5.', '.5', '0']) {
      await amountInput(wrapper, 0).setValue(v)
      expect(amountInput(wrapper, 0).classes()).not.toContain('invalid')
    }
    await amountInput(wrapper, 0).setValue('')
    expect(amountInput(wrapper, 0).classes()).not.toContain('invalid')
  })

  it('所有金额非空合法且配平、日期与成员已填时确认可用；非法或不平时禁用', async () => {
    const wrapper = await mountEdit()
    // 初始：10.00 / -10.00 平衡且合法
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeUndefined()

    // 不平 -> 禁用
    await amountInput(wrapper, 0).setValue('5')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()

    // 重新配平且合法 -> 可用
    await amountInput(wrapper, 0).setValue('10')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeUndefined()

    // 非法 -> 禁用
    await amountInput(wrapper, 0).setValue('-')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()
  })

  it('缺少日期时间或成员时确认禁用', async () => {
    const wrapper = await mountEdit()

    // 缺日期时间
    await wrapper.find('input[type="datetime-local"]').setValue('')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()
    await wrapper.find('input[type="datetime-local"]').setValue('2026-01-01T12:00:00')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeUndefined()

    // 缺成员：选中值为 null 的占位项
    const memberSelect = wrapper.find('.field-member select')
    const placeholder = memberSelect.findAll('option')[0]
    ;(placeholder.element as HTMLOptionElement).selected = true
    await memberSelect.trigger('change')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()
  })

  it('存在空金额时确认禁用', async () => {
    const wrapper = await mountEdit()
    await amountInput(wrapper, 0).setValue('')
    expect(wrapper.find('.submit-btn').attributes('disabled')).toBeDefined()
  })
})
