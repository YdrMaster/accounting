import { mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import AppDialog from '../../components/AppDialog.vue'
import { i18n } from '../../i18n'
import { alertDialog, confirmDialog, dialogState, resolveDialog } from '../../utils/dialog'

let wrapper: VueWrapper | null = null

function mountDialog() {
  wrapper = mount(AppDialog, { global: { plugins: [i18n] }, attachTo: document.body })
}

function bodyDialog(): HTMLElement | null {
  return document.body.querySelector('.app-dialog')
}

function bodyButtons(): NodeListOf<HTMLButtonElement> {
  return document.body.querySelectorAll('.app-dialog button')
}

afterEach(() => {
  resolveDialog(false)
  wrapper?.unmount()
  wrapper = null
})

describe('AppDialog', () => {
  it('隐藏时不渲染对话框', () => {
    mountDialog()
    expect(bodyDialog()).toBeNull()
  })

  it('confirm 模式显示消息与两个按钮，点确认 resolve true', async () => {
    mountDialog()
    const p = confirmDialog('确定要删除这条交易吗？')
    await nextTick()

    expect(bodyDialog()?.textContent).toContain('确定要删除这条交易吗？')
    const buttons = bodyButtons()
    expect(buttons.length).toBe(2)

    buttons[1].click()
    await expect(p).resolves.toBe(true)
  })

  it('alert 模式只显示一个确认按钮', async () => {
    mountDialog()
    const p = alertDialog('名称不能为空')
    await nextTick()

    expect(bodyDialog()?.textContent).toContain('名称不能为空')
    const buttons = bodyButtons()
    expect(buttons.length).toBe(1)

    buttons[0].click()
    await expect(p).resolves.toBeUndefined()
  })

  it('点击遮罩等同于取消', async () => {
    mountDialog()
    const p = confirmDialog('确定？')
    await nextTick()

    const overlay = document.body.querySelector<HTMLElement>('.app-dialog-overlay')
    overlay?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await expect(p).resolves.toBe(false)
    expect(dialogState.visible).toBe(false)
  })
})
