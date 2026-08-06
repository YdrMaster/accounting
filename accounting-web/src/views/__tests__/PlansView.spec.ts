import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { inject, nextTick, ref, watchEffect } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../i18n'
import { panelActionKey, type PanelAction } from '../../components/layout/panelAction'
import PlansView from '../PlansView.vue'

function makeViewStub(name: string, cls: string, label: string) {
  return {
    name,
    template: `<div class="${cls}">${label}</div>`,
    setup() {
      const action = inject(panelActionKey)
      watchEffect(() => {
        if (action) action.value = [{ label, disabled: false, onClick: () => {} }]
      })
      return {}
    },
  }
}

// 子视图较重，stub 成能识别身份并模拟 panelAction 注入行为的轻组件
vi.mock('../BudgetView.vue', () => ({
  default: makeViewStub('BudgetView', 'budget-view-stub', '新建预算'),
}))

vi.mock('../SavingPlanView.vue', () => ({
  default: makeViewStub('SavingPlanView', 'saving-plan-view-stub', '新建攒钱计划'),
}))

function mountPlans() {
  setActivePinia(createPinia())
  setLocale('zh-CN')
  const panelAction = ref<PanelAction[]>([])
  const wrapper = mount(PlansView, {
    global: {
      plugins: [i18n],
      provide: { [panelActionKey as symbol]: panelAction },
    },
  })
  return { wrapper, panelAction }
}

describe('PlansView', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('shows budget view by default with both tabs', async () => {
    const { wrapper, panelAction } = mountPlans()
    await nextTick()

    const tabs = wrapper.findAll('.tab-btn')
    expect(tabs).toHaveLength(2)
    expect(tabs[0].text()).toBe('预算')
    expect(tabs[1].text()).toBe('攒钱计划')
    expect(tabs[0].classes()).toContain('active')

    expect(wrapper.find('.budget-view-stub').exists()).toBe(true)
    expect(wrapper.find('.saving-plan-view-stub').exists()).toBe(false)
    expect(panelAction.value[0].label).toBe('新建预算')
  })

  it('switches to saving plan view on tab click and back', async () => {
    const { wrapper, panelAction } = mountPlans()
    await nextTick()

    const tabs = wrapper.findAll('.tab-btn')
    await tabs[1].trigger('click')
    expect(wrapper.find('.saving-plan-view-stub').exists()).toBe(true)
    expect(wrapper.find('.budget-view-stub').exists()).toBe(false)
    expect(tabs[1].classes()).toContain('active')
    expect(panelAction.value[0].label).toBe('新建攒钱计划')

    await tabs[0].trigger('click')
    expect(wrapper.find('.budget-view-stub').exists()).toBe(true)
    expect(wrapper.find('.saving-plan-view-stub').exists()).toBe(false)
    expect(panelAction.value[0].label).toBe('新建预算')
  })
})
