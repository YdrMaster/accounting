import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import AccountCard from '../AccountCard.vue'

const account = {
  id: 1,
  name: 'Test',
  account_type: 'Asset',
  parent_id: null,
  closed_at: null,
  is_system: false,
  billing_day: null,
  repayment_day: null,
  owner_ids: [],
}

describe('AccountCard', () => {
  it('renders account name', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false } },
    })
    expect(wrapper.text()).toContain('Test')
  })

  it('renders placeholder as empty and non-clickable', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account: null, isPlaceholder: true, hasChildren: false } },
    })
    expect(wrapper.text()).toBe('')
    expect(wrapper.find('.account-card').classes()).toContain('placeholder')
  })

  it('emits click with account id', async () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false } },
    })
    await wrapper.find('.account-card').trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.emitted('click')![0]).toEqual([account])
  })

  it('shows expand indicator when expanded and item has children', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: true }, isExpanded: true },
    })
    expect(wrapper.text()).toContain('▾')
  })

  it('applies selected class', () => {
    const wrapper = mount(AccountCard, {
      props: { item: { account, isPlaceholder: false, hasChildren: false }, isSelected: true },
    })
    expect(wrapper.find('.account-card').classes()).toContain('selected')
  })

  it('applies closed class', () => {
    const closed = { ...account, closed_at: '2024-01-01' }
    const wrapper = mount(AccountCard, {
      props: { item: { account: closed, isPlaceholder: false, hasChildren: false } },
    })
    expect(wrapper.find('.account-card').classes()).toContain('closed')
  })
})
