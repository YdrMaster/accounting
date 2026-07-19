import type { InjectionKey, Ref } from 'vue'

export interface PanelAction {
  label: string
  disabled: boolean
  onClick: () => void
}

export const panelActionKey: InjectionKey<Ref<PanelAction | null>> = Symbol('panelAction')
