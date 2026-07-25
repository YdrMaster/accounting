import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { i18n, setLocale } from '../../../i18n'
import { useChannelStore } from '../../../stores/channel'
import { useMemberStore } from '../../../stores/member'
import type { ChannelDto } from '../../../types/api'
import ChannelMappingSection from '../ChannelMappingSection.vue'
import ConfigPanel from '../ConfigPanel.vue'

vi.mock('../../../api/client', () => ({
  fetchChannels: vi.fn().mockResolvedValue([]),
  fetchMembers: vi.fn().mockResolvedValue([{ id: 1, name: 'Alice' }]),
  fetchTags: vi.fn().mockResolvedValue([]),
  fetchAccounts: vi.fn().mockResolvedValue([]),
  fetchMappings: vi.fn().mockResolvedValue([]),
}))

const adapterChannel: ChannelDto = {
  id: 1,
  name: '支付宝',
  description: null,
  account_id: null,
  is_system: true,
  has_import_adapter: true,
}

const plainChannel: ChannelDto = {
  id: 2,
  name: '云闪付',
  description: null,
  account_id: null,
  is_system: false,
  has_import_adapter: false,
}

async function mountAndExpand(channels: ChannelDto[], channelId: number) {
  const channelStore = useChannelStore()
  channelStore.channels = channels
  const memberStore = useMemberStore()
  memberStore.members = [{ id: 1, name: 'Alice' }]

  const wrapper = mount(ConfigPanel, { global: { plugins: [i18n] } })
  await flushPromises()

  const tabBtns = wrapper.findAll('.tab-btn')
  await tabBtns[1].trigger('click')
  await flushPromises()

  const card = wrapper
    .findAll('.channel-card')
    .find(c => c.find('.channel-header').text().includes(
      channels.find(ch => ch.id === channelId)!.name
    ))!
  await card.find('.channel-header').trigger('click')
  await flushPromises()

  return { wrapper, card }
}

describe('ConfigPanel 渠道卡片导入规则区块', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    setLocale('zh-CN')
    document.body.insertAdjacentHTML('beforeend', '<div class="picker-portal"></div>')
  })

  it('展开的适配器渠道卡片显示导入规则区块', async () => {
    const { card } = await mountAndExpand([adapterChannel, plainChannel], adapterChannel.id)

    expect(card.findComponent(ChannelMappingSection).exists()).toBe(true)
  })

  it('展开的普通渠道卡片不显示导入规则区块', async () => {
    const { card } = await mountAndExpand([adapterChannel, plainChannel], plainChannel.id)

    expect(card.findComponent(ChannelMappingSection).exists()).toBe(false)
  })
})
