import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { importBill } from '../../../api/client'
import { i18n, setLocale } from '../../../i18n'
import { useChannelStore } from '../../../stores/channel'
import { useMemberStore } from '../../../stores/member'
import type { ChannelDto, ImportResultDto } from '../../../types/api'
import ChannelMappingSection from '../ChannelMappingSection.vue'
import ConfigPanel from '../ConfigPanel.vue'

vi.mock('../../../api/client', () => ({
  fetchChannels: vi.fn().mockResolvedValue([]),
  fetchMembers: vi.fn().mockResolvedValue([{ id: 1, name: 'Alice' }]),
  fetchTags: vi.fn().mockResolvedValue([]),
  fetchAccounts: vi.fn().mockResolvedValue([]),
  fetchMappings: vi.fn().mockResolvedValue([]),
  fetchMe: vi.fn(),
  login: vi.fn(),
  loginTotp: vi.fn(),
  logout: vi.fn(),
  totpSetup: vi.fn(),
  totpEnable: vi.fn(),
  apiErrorMessage: vi.fn((e: unknown) => (e instanceof Error ? e.message : String(e))),
  importBill: vi
    .fn()
    .mockResolvedValue({ imported: 0, skipped: 0, pending_tag_name: null, errors: [] }),
}))

const importBillMock = vi.mocked(importBill)

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

  const card = wrapper.findAll('.channel-card').find(c =>
    c
      .find('.channel-header')
      .text()
      .includes(channels.find(ch => ch.id === channelId)!.name)
  )!
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

const userAdapterChannel: ChannelDto = {
  id: 3,
  name: '我的招行',
  description: null,
  account_id: null,
  is_system: false,
  has_import_adapter: true,
}

const systemPlainChannel: ChannelDto = {
  id: 4,
  name: '现金',
  description: null,
  account_id: null,
  is_system: true,
  has_import_adapter: false,
}

async function mountChannels(
  channels: ChannelDto[],
  members: { id: number; name: string }[] = [{ id: 1, name: 'Alice' }]
) {
  const channelStore = useChannelStore()
  channelStore.channels = channels
  const memberStore = useMemberStore()
  memberStore.members = members

  const wrapper = mount(ConfigPanel, { global: { plugins: [i18n] } })
  await flushPromises()
  await wrapper.findAll('.tab-btn')[1].trigger('click')
  await flushPromises()
  return { wrapper, channelStore }
}

function findCard(wrapper: Awaited<ReturnType<typeof mountChannels>>['wrapper'], name: string) {
  return wrapper
    .findAll('.channel-card')
    .find(c => c.find('.channel-header').text().includes(name))!
}

function setInputFiles(input: HTMLInputElement, files: File[]) {
  Object.defineProperty(input, 'files', { value: files, configurable: true })
}

async function confirmDialog(wrapper: Awaited<ReturnType<typeof mountChannels>>['wrapper']) {
  await wrapper.find('.import-dialog .dialog-confirm-btn').trigger('click')
  await flushPromises()
}

describe('ConfigPanel 渠道卡片导入账单', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    importBillMock.mockResolvedValue({
      imported: 0,
      skipped: 0,
      pending_tag_name: null,
      errors: [],
    })
    setLocale('zh-CN')
    document.body.insertAdjacentHTML('beforeend', '<div class="picker-portal"></div>')
  })

  it('按渠道类型渲染头部按钮：适配器渠道显示导入按钮', async () => {
    const { wrapper } = await mountChannels([
      adapterChannel,
      plainChannel,
      userAdapterChannel,
      systemPlainChannel,
    ])

    const adapterCard = findCard(wrapper, '支付宝')
    expect(adapterCard.find('.import-btn').exists()).toBe(true)
    expect(adapterCard.find('.delete-btn').exists()).toBe(false)

    const plainCard = findCard(wrapper, '云闪付')
    expect(plainCard.find('.delete-btn').exists()).toBe(true)
    expect(plainCard.find('.import-btn').exists()).toBe(false)

    // 用户渠道同时有适配器时，导入按钮优先
    const userAdapterCard = findCard(wrapper, '我的招行')
    expect(userAdapterCard.find('.import-btn').exists()).toBe(true)
    expect(userAdapterCard.find('.delete-btn').exists()).toBe(false)

    const systemPlainCard = findCard(wrapper, '现金')
    expect(systemPlainCard.find('.import-btn').exists()).toBe(false)
    expect(systemPlainCard.find('.delete-btn').exists()).toBe(false)
  })

  it('点击导入按钮触发文件选择，选中文件后弹出成员确认对话框且不展开卡片', async () => {
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    const clickSpy = vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(() => {})
    await card.find('.import-btn').trigger('click')
    expect(clickSpy).toHaveBeenCalledTimes(1)
    // @click.stop：不触发展开/折叠
    expect(card.find('.channel-body').exists()).toBe(false)

    const file = new File(['csv-content'], 'bill.csv', { type: 'text/csv' })
    const input = wrapper.find('input.import-file-input').element as HTMLInputElement
    setInputFiles(input, [file])
    await wrapper.find('input.import-file-input').trigger('change')
    await flushPromises()

    // 弹出确认对话框，尚未发起导入
    expect(wrapper.find('.import-dialog').exists()).toBe(true)
    expect(importBillMock).not.toHaveBeenCalled()

    // 确认后以默认成员（第一个成员）导入
    await confirmDialog(wrapper)
    expect(importBillMock).toHaveBeenCalledTimes(1)
    expect(importBillMock).toHaveBeenCalledWith(adapterChannel.id, 1, file)
    expect(wrapper.find('.import-dialog').exists()).toBe(false)
    clickSpy.mockRestore()
  })

  it('拖拽文件到适配器卡片：悬停高亮，松手后弹出确认对话框，确认后导入', async () => {
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('dragover')
    expect(card.classes()).toContain('drag-over')
    await card.trigger('dragleave')
    expect(card.classes()).not.toContain('drag-over')

    await card.trigger('dragover')
    const file = new File(['csv-content'], 'bill.csv')
    await card.trigger('drop', { dataTransfer: { files: [file] } })
    await flushPromises()

    expect(card.classes()).not.toContain('drag-over')
    expect(wrapper.find('.import-dialog').exists()).toBe(true)
    expect(importBillMock).not.toHaveBeenCalled()

    await confirmDialog(wrapper)
    expect(importBillMock).toHaveBeenCalledTimes(1)
    expect(importBillMock).toHaveBeenCalledWith(adapterChannel.id, 1, file)
  })

  it('成员确认对话框默认选中第一个成员，可改选其他成员导入', async () => {
    const { wrapper } = await mountChannels(
      [adapterChannel],
      [
        { id: 1, name: 'Alice' },
        { id: 2, name: 'Bob' },
      ]
    )
    const card = findCard(wrapper, '支付宝')

    const file = new File(['csv-content'], 'bill.csv')
    await card.trigger('drop', { dataTransfer: { files: [file] } })
    await flushPromises()

    const select = wrapper.find('.import-dialog .import-member-select')
    expect((select.element as HTMLSelectElement).value).toBe('1')

    await select.setValue('2')
    await confirmDialog(wrapper)
    expect(importBillMock).toHaveBeenCalledTimes(1)
    expect(importBillMock).toHaveBeenCalledWith(adapterChannel.id, 2, file)
  })

  it('取消成员确认对话框则不发起导入', async () => {
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
    await flushPromises()
    expect(wrapper.find('.import-dialog').exists()).toBe(true)

    await wrapper.find('.import-dialog .dialog-cancel-btn').trigger('click')
    await flushPromises()

    expect(wrapper.find('.import-dialog').exists()).toBe(false)
    expect(importBillMock).not.toHaveBeenCalled()
  })

  it('无适配器卡片不响应拖放', async () => {
    const { wrapper } = await mountChannels([plainChannel, systemPlainChannel])

    for (const name of ['云闪付', '现金']) {
      const card = findCard(wrapper, name)
      await card.trigger('dragover')
      expect(card.classes()).not.toContain('drag-over')
      await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
      await flushPromises()
    }
    expect(wrapper.find('.import-dialog').exists()).toBe(false)
    expect(importBillMock).not.toHaveBeenCalled()
  })

  it('导入中按钮禁用并显示 loading 文案，忽略重复拖放与点击', async () => {
    let resolveImport!: (v: ImportResultDto) => void
    importBillMock.mockReturnValueOnce(new Promise<ImportResultDto>(r => (resolveImport = r)))
    const { wrapper, channelStore } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
    await flushPromises()
    await confirmDialog(wrapper)
    expect(channelStore.importingChannelId).toBe(adapterChannel.id)

    const btn = card.find('.import-btn')
    expect(btn.attributes('disabled')).toBeDefined()
    expect(btn.text()).toContain('导入中')

    // 重复拖放与点击被忽略
    await card.trigger('drop', { dataTransfer: { files: [new File(['y'], 'b.csv')] } })
    const clickSpy = vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(() => {})
    await card.find('.import-btn').trigger('click')
    await flushPromises()
    expect(importBillMock).toHaveBeenCalledTimes(1)
    expect(clickSpy).not.toHaveBeenCalled()
    clickSpy.mockRestore()

    resolveImport({ imported: 1, skipped: 0, pending_tag_name: null, errors: [] })
    await flushPromises()
    expect(channelStore.importingChannelId).toBeNull()
    expect(card.find('.import-btn').attributes('disabled')).toBeUndefined()
  })

  it('全部成功时 toast 显示摘要且不提供展开入口', async () => {
    importBillMock.mockResolvedValueOnce({
      imported: 320,
      skipped: 0,
      pending_tag_name: null,
      errors: [],
    })
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
    await flushPromises()
    await confirmDialog(wrapper)

    const toast = wrapper.find('.import-toast')
    expect(toast.exists()).toBe(true)
    expect(toast.text()).toContain('导入 320 条，跳过 0 条')
    expect(toast.find('.import-toast-toggle').exists()).toBe(false)
  })

  it('含跳过记录时 toast 可展开查看逐行原因', async () => {
    importBillMock.mockResolvedValueOnce({
      imported: 318,
      skipped: 2,
      pending_tag_name: 'pending',
      errors: [
        { row: 15, detail: '金额无效' },
        { row: 22, detail: '日期无法解析' },
      ],
    })
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
    await flushPromises()
    await confirmDialog(wrapper)

    const toast = wrapper.find('.import-toast')
    expect(toast.text()).toContain('导入 318 条，跳过 2 条')
    const toggle = toast.find('.import-toast-toggle')
    expect(toggle.exists()).toBe(true)
    expect(toast.find('.import-toast-errors').exists()).toBe(false)

    await toggle.trigger('click')
    const errors = toast.find('.import-toast-errors')
    expect(errors.exists()).toBe(true)
    expect(errors.text()).toContain('15')
    expect(errors.text()).toContain('金额无效')
    expect(errors.text()).toContain('22')
    expect(errors.text()).toContain('日期无法解析')
  })

  it('导入失败时显示错误 toast', async () => {
    importBillMock.mockRejectedValueOnce(new Error('无法解析文件'))
    const { wrapper } = await mountChannels([adapterChannel])
    const card = findCard(wrapper, '支付宝')

    await card.trigger('drop', { dataTransfer: { files: [new File(['x'], 'a.csv')] } })
    await flushPromises()
    await confirmDialog(wrapper)

    const toast = wrapper.find('.import-toast')
    expect(toast.exists()).toBe(true)
    expect(toast.classes()).toContain('error')
    expect(toast.text()).toContain('无法解析文件')
  })
})
