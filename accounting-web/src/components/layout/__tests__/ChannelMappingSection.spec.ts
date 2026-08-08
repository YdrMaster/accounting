import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../../i18n'
import { useAccountStore } from '../../../stores/account'
import { useMemberStore } from '../../../stores/member'
import type { AccountDto, ChannelDto, MemberDto } from '../../../types/api'
import AccountPicker from '../AccountPicker.vue'
import ChannelMappingSection from '../ChannelMappingSection.vue'

vi.mock('../../../api/client', () => ({
  fetchMappings: vi.fn(),
  upsertMapping: vi.fn(),
  deleteMapping: vi.fn(),
  fetchAccounts: vi.fn().mockResolvedValue([]),
}))

import { deleteMapping, fetchMappings, upsertMapping } from '../../../api/client'

const mockedFetch = vi.mocked(fetchMappings)
const mockedUpsert = vi.mocked(upsertMapping)
const mockedDelete = vi.mocked(deleteMapping)

const channel: ChannelDto = {
  id: 2,
  name: '支付宝',
  description: null,
  account_id: null,
  is_system: true,
  has_import_adapter: true,
}

const members: MemberDto[] = [
  { id: 1, name: 'Alice' },
  { id: 2, name: 'Bob' },
]

function makeAccount(id: number, name: string): AccountDto {
  return {
    id,
    name,
    account_type: 'Expense',
    parent_id: null,
    closed_at: null,
    is_system: false,
    billing_day: null,
    repayment_day: null,
    owner_ids: [],
  }
}

function mountSection() {
  return mount(ChannelMappingSection, {
    props: { channel },
    global: { plugins: [i18n] },
  })
}

describe('ChannelMappingSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
    // AccountPicker 内部 Teleport 到 .picker-portal，测试中补上该挂载点
    document.body.insertAdjacentHTML('beforeend', '<div class="picker-portal"></div>')
    const memberStore = useMemberStore()
    memberStore.members = members
    const accountStore = useAccountStore()
    accountStore.accounts = [makeAccount(10, '餐饮'), makeAccount(11, '余额宝')]
    mockedFetch.mockResolvedValue([])
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('loads mappings for the first member on mount', async () => {
    mountSection()
    await flushPromises()

    expect(mockedFetch).toHaveBeenCalledWith(1, channel.id)
  })

  it('reloads mappings when the member selection changes', async () => {
    const wrapper = mountSection()
    await flushPromises()
    mockedFetch.mockClear()

    const option = wrapper.findAll('select.member-select option')[1]
    ;(option.element as HTMLOptionElement).selected = true
    await wrapper.find('select.member-select').trigger('change')
    await flushPromises()

    expect(mockedFetch).toHaveBeenCalledWith(2, channel.id)
  })

  it('renders category and resolved account name for each mapping', async () => {
    mockedFetch.mockResolvedValue([
      { member_id: 1, channel_id: 2, category: 'Expenses:餐饮美食', account_id: 10 },
    ])
    const wrapper = mountSection()
    await flushPromises()

    const text = wrapper.text()
    expect(text).toContain('Expenses:餐饮美食')
    expect(text).toContain('餐饮')
  })

  it('falls back to #<id> when the account does not exist', async () => {
    mockedFetch.mockResolvedValue([
      { member_id: 1, channel_id: 2, category: 'Assets:余额宝', account_id: 99 },
    ])
    const wrapper = mountSection()
    await flushPromises()

    expect(wrapper.text()).toContain('#99')
  })

  it('adds a mapping with the entered category and picked account', async () => {
    mockedUpsert.mockResolvedValue('ok')
    const wrapper = mountSection()
    await flushPromises()

    await wrapper.find('.add-row input.field-input').setValue('Expenses:交通出行')
    wrapper.findComponent(AccountPicker).vm.$emit('update:modelValue', 10)
    await flushPromises()
    await wrapper.find('.add-row .add-btn').trigger('click')
    await flushPromises()

    expect(mockedUpsert).toHaveBeenCalledWith({
      member_id: 1,
      channel_id: 2,
      category: 'Expenses:交通出行',
      account_id: 10,
    })
    expect(wrapper.text()).toContain('Expenses:交通出行')
  })

  it('does not submit when the category is empty', async () => {
    const wrapper = mountSection()
    await flushPromises()

    wrapper.findComponent(AccountPicker).vm.$emit('update:modelValue', 10)
    await flushPromises()
    await wrapper.find('.add-row .add-btn').trigger('click')
    await flushPromises()

    expect(mockedUpsert).not.toHaveBeenCalled()
  })

  it('does not submit when no account is picked', async () => {
    const wrapper = mountSection()
    await flushPromises()

    await wrapper.find('.add-row input.field-input').setValue('Expenses:交通出行')
    await wrapper.find('.add-row .add-btn').trigger('click')
    await flushPromises()

    expect(mockedUpsert).not.toHaveBeenCalled()
  })

  it('removes a mapping via its delete button', async () => {
    mockedFetch.mockResolvedValue([
      { member_id: 1, channel_id: 2, category: 'Expenses:餐饮美食', account_id: 10 },
    ])
    mockedDelete.mockResolvedValue(undefined)
    const wrapper = mountSection()
    await flushPromises()

    await wrapper.find('.mapping-item .delete-btn').trigger('click')
    await flushPromises()

    expect(mockedDelete).toHaveBeenCalledWith(1, 2, 'Expenses:餐饮美食')
    expect(wrapper.text()).not.toContain('Expenses:餐饮美食')
  })
})
