import { reactive } from 'vue'

export type DialogKind = 'confirm' | 'alert'

interface DialogState {
  visible: boolean
  kind: DialogKind
  message: string
}

/** 全局应用内对话框状态，由 AppDialog 组件渲染 */
export const dialogState = reactive<DialogState>({
  visible: false,
  kind: 'confirm',
  message: '',
})

let resolver: ((confirmed: boolean) => void) | null = null

/** 应用内确认框，替代浏览器原生 confirm() */
export function confirmDialog(message: string): Promise<boolean> {
  dialogState.kind = 'confirm'
  dialogState.message = message
  dialogState.visible = true
  return new Promise(resolve => {
    resolver = resolve
  })
}

/** 应用内提示框，替代浏览器原生 alert() */
export function alertDialog(message: string): Promise<void> {
  dialogState.kind = 'alert'
  dialogState.message = message
  dialogState.visible = true
  return new Promise(resolve => {
    resolver = () => resolve()
  })
}

/** 由 AppDialog 在用户点击后调用 */
export function resolveDialog(confirmed: boolean): void {
  dialogState.visible = false
  resolver?.(confirmed)
  resolver = null
}
