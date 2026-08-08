import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n } from '../../../i18n'
import { useChannelStore } from '../../../stores/channel'
import type { ChannelDto } from '../../../types/api'
import ChannelPathInput from '../ChannelPathInput.vue'

vi.mock('../../../api/client', () => ({
  fetchChannels: vi.fn().mockResolvedValue([]),
  createChannel: vi.fn(),
  deleteChannel: vi.fn(),
  updateChannel: vi.fn(),
  importBill: vi.fn(),
}))

const channels: ChannelDto[] = [
  { id: 1, name: '支付宝', description: null, account_id: null, is_system: false, has_import_adapter: false },
  { id: 2, name: '招行', description: null, account_id: null, is_system: false, has_import_adapter: false },
  { id: 3, name: '现金', description: null, account_id: null, is_system: false, has_import_adapter: false },
]

function chainNodes(...ids: number[]) {
  return ids.map((channel_id, position) => ({ position, channel_id, status: 'default' }))
}

beforeEach(() => {
  setActivePinia(createPinia())
  const channelStore = useChannelStore()
  channelStore.channels = channels
})

function mountChain(modelValue: ReturnType<typeof chainNodes>) {
  return mount(ChannelPathInput, {
    props: { modelValue },
    global: { plugins: [i18n] },
  })
}

describe('ChannelPathInput 线性链', () => {
  it('层级在同一行内用 ▸ 分隔', () => {
    const wrapper = mountChain(chainNodes(1, 2))

    const chain = wrapper.find('.path-chain')
    expect(chain.exists()).toBe(true)
    const html = chain.html()
    expect(html.indexOf('支付宝')).toBeLessThan(html.indexOf('▸'))
    expect(html.indexOf('▸')).toBeLessThan(html.indexOf('招行'))
  })

  it('只有最右侧芯片显示 ×,点击弹出链尾节点', async () => {
    const wrapper = mountChain(chainNodes(1, 2))
    const chips = wrapper.findAll('.channel-chip')
    expect(chips.length).toBe(2)
    expect(chips[0].find('button').exists()).toBe(false)
    expect(chips[1].find('button').exists()).toBe(true)

    await chips[1].find('button').trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[chainNodes(1)]])
  })

  it('已选渠道在候选中禁用,选中后按链长追加节点', async () => {
    const wrapper = mountChain(chainNodes(1))
    const select = wrapper.find('.chain-select')
    expect(select.exists()).toBe(true)

    const options = select.findAll('option')
    const alipay = options.find(o => o.text() === '支付宝')
    const cmb = options.find(o => o.text() === '招行')
    expect(alipay?.attributes('disabled')).toBeDefined()
    expect(cmb?.attributes('disabled')).toBeUndefined()

    await select.setValue('2')
    expect(wrapper.emitted('update:modelValue')).toEqual([[chainNodes(1, 2)]])
  })

  it('channel_id 为 0 的占位节点不渲染芯片', () => {
    const wrapper = mountChain(chainNodes(1, 0))
    expect(wrapper.findAll('.channel-chip').length).toBe(1)
    expect(wrapper.text()).not.toContain('渠道 #0')
  })

  it('不再有添加链路级别按钮', () => {
    const wrapper = mountChain(chainNodes(1))
    expect(wrapper.find('.add-level-btn').exists()).toBe(false)
  })
})
