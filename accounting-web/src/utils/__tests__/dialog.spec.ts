import { describe, expect, it } from 'vitest'
import { alertDialog, confirmDialog, dialogState, resolveDialog } from '../dialog'

describe('dialog', () => {
  it('confirmDialog 展示消息并在确认后 resolve true', async () => {
    const p = confirmDialog('确定删除？')
    expect(dialogState.visible).toBe(true)
    expect(dialogState.kind).toBe('confirm')
    expect(dialogState.message).toBe('确定删除？')

    resolveDialog(true)

    await expect(p).resolves.toBe(true)
    expect(dialogState.visible).toBe(false)
  })

  it('confirmDialog 取消后 resolve false', async () => {
    const p = confirmDialog('确定删除？')
    resolveDialog(false)
    await expect(p).resolves.toBe(false)
    expect(dialogState.visible).toBe(false)
  })

  it('alertDialog 展示消息并在关闭后 resolve', async () => {
    const p = alertDialog('保存失败')
    expect(dialogState.visible).toBe(true)
    expect(dialogState.kind).toBe('alert')
    expect(dialogState.message).toBe('保存失败')

    resolveDialog(true)

    await expect(p).resolves.toBeUndefined()
    expect(dialogState.visible).toBe(false)
  })
})
