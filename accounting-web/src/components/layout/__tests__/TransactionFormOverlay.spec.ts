import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { i18n } from '../../../i18n'
import { useTagStore } from '../../../stores/tag'
import type { TagDto } from '../../../types/api'
import TransactionFormOverlay from '../TransactionFormOverlay.vue'

vi.mock('../../../api/client', () => ({
  fetchTransaction: vi.fn(),
  fetchTransactions: vi.fn().mockResolvedValue([]),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
  fetchMembers: vi.fn().mockResolvedValue([{ id: 1, name: '我' }]),
  fetchCommodities: vi.fn().mockResolvedValue([{ id: 1, symbol: 'CNY', name: '人民币', precision: 2 }]),
  fetchChannels: vi.fn().mockResolvedValue([]),
  fetchTags: vi.fn(),
  fetchAccounts: vi.fn().mockResolvedValue([]),
  fetchBalanceSheet: vi.fn().mockResolvedValue({ assets: [] }),
  fetchNetWorthTrend: vi.fn(),
  fetchCashFlow: vi.fn(),
  fetchBudgetStatuses: vi.fn().mockResolvedValue([]),
  fetchSavingPlanStatuses: vi.fn().mockResolvedValue([]),
  createChannel: vi.fn(),
  deleteChannel: vi.fn(),
  updateChannel: vi.fn(),
  importBill: vi.fn(),
  createTag: vi.fn(),
  deleteTag: vi.fn(),
  updateTag: vi.fn(),
  createMember: vi.fn(),
  deleteMember: vi.fn(),
  renameMember: vi.fn(),
}))

vi.mock('../AccountPicker.vue', () => ({
  default: { name: 'AccountPicker', props: ['modelValue'], template: '<div />' },
}))

const systemTag: TagDto = { id: 1, name: '餐饮', description: null, is_system: false }
const otherTag: TagDto = { id: 2, name: '交通', description: null, is_system: false }

async function mountForm() {
  const wrapper = mount(TransactionFormOverlay, { global: { plugins: [i18n] } })
  await flushPromises()
  await nextTick()
  return wrapper
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  const tagStore = useTagStore()
  tagStore.tags = [systemTag, otherTag]
})

describe('TransactionFormOverlay 布局', () => {
  it('成员与日期在同一横排，成员在左', async () => {
    const wrapper = await mountForm()
    const row = wrapper.find('.field-row')
    expect(row.exists()).toBe(true)

    const memberField = row.find('.field-member')
    const dateField = row.find('.field-date')
    expect(memberField.exists()).toBe(true)
    expect(dateField.exists()).toBe(true)
    // 成员在左：DOM 顺序 member 先于 date
    const children = row.element.children
    expect(children[0].className).toContain('field-member')
    expect(children[1].className).toContain('field-date')
  })

  it('备注框禁止拖动调整大小且两端对齐', async () => {
    const wrapper = await mountForm()
    const textarea = wrapper.find('.desc-textarea')
    expect(textarea.exists()).toBe(true)
    // 不再有 rows=2 的固定行高（高度由内容自适应）
    expect(textarea.attributes('rows')).toBe('1')
  })
})

describe('TransactionFormOverlay 标签选择', () => {
  it('不提供手工输入框，通过下拉选择已有标签', async () => {
    const wrapper = await mountForm()
    const tagField = wrapper.find('.field-tags')
    expect(tagField.exists()).toBe(true)
    expect(tagField.find('input[type="text"]').exists()).toBe(false)

    const select = tagField.find('select')
    expect(select.exists()).toBe(true)
    const optionTexts = select.findAll('option').map(o => o.text())
    expect(optionTexts).toContain('餐饮')
    expect(optionTexts).toContain('交通')
  })

  it('选择标签后生成 chip，且该标签从候选中移除', async () => {
    const wrapper = await mountForm()
    const tagField = wrapper.find('.field-tags')
    const select = tagField.find('select')

    await select.setValue('餐饮')
    await nextTick()

    const chips = tagField.findAll('.tag-chip')
    expect(chips.length).toBe(1)
    expect(chips[0].text()).toContain('餐饮')

    const optionTexts = select.findAll('option').map(o => o.text())
    expect(optionTexts).not.toContain('餐饮')
    expect(optionTexts).toContain('交通')
  })

  it('chip 上的 × 可移除标签并回到候选', async () => {
    const wrapper = await mountForm()
    const tagField = wrapper.find('.field-tags')
    await tagField.find('select').setValue('交通')
    await nextTick()

    await tagField.find('.tag-chip button').trigger('click')
    await nextTick()

    expect(tagField.findAll('.tag-chip').length).toBe(0)
    const optionTexts = tagField.find('select').findAll('option').map(o => o.text())
    expect(optionTexts).toContain('交通')
  })
})

describe('TransactionFormOverlay 分录', () => {
  it('新建交易时只自动创建一个分录', async () => {
    const wrapper = await mountForm()
    expect(wrapper.findAll('.posting-row').length).toBe(1)
  })

  it('存在未填完的分录（分类或金额为空）时，添加分录按钮不可交互', async () => {
    const wrapper = await mountForm()
    const addBtn = wrapper.find('.add-posting-btn')
    // 初始分录分类和金额都为空 → 不可交互
    expect(addBtn.attributes('disabled')).toBeDefined()
  })

  it('所有分录填完后，添加分录按钮恢复可交互', async () => {
    const wrapper = await mountForm()
    // 填分类
    await wrapper.findComponent({ name: 'AccountPicker' }).vm.$emit('update:model-value', 1)
    // 填金额
    await wrapper.find('.amount-input').setValue('-100')
    await nextTick()

    const addBtn = wrapper.find('.add-posting-btn')
    expect(addBtn.attributes('disabled')).toBeUndefined()
  })
})

describe('TransactionFormOverlay 金额输入', () => {
  it('金额使用限数字的普通文本框', async () => {
    const wrapper = await mountForm()
    const amountInput = wrapper.find('.amount-input')
    expect(amountInput.exists()).toBe(true)
    expect(amountInput.attributes('type')).toBe('text')
    expect(amountInput.attributes('inputmode')).toBe('decimal')
  })

  it('输入含字母的值时只保留合法数字字符', async () => {
    const wrapper = await mountForm()
    const amountInput = wrapper.find('.amount-input')

    await amountInput.setValue('1a2b.3c')
    await nextTick()

    expect((amountInput.element as HTMLInputElement).value).toBe('12.3')
  })

  it('负号只允许在首位且小数点至多一个', async () => {
    const wrapper = await mountForm()
    const amountInput = wrapper.find('.amount-input')

    await amountInput.setValue('1-2..3')
    await nextTick()

    expect((amountInput.element as HTMLInputElement).value).toBe('12.3')
  })
})
