import type { InjectionKey, Ref } from 'vue'

export interface PanelAction {
  label: string
  icon?: string
  disabled: boolean
  onClick: () => void
}

export const panelActionKey: InjectionKey<Ref<PanelAction[]>> = Symbol('panelAction')
