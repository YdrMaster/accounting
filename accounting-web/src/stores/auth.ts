import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  login as apiLogin,
  loginTotp as apiLoginTotp,
  logout as apiLogout,
  fetchMe,
} from '../api/client'
import type { LoginResultDto, MeDto } from '../types/api'

export type AuthStatus = 'unknown' | 'authed' | 'unauthed'

export const useAuthStore = defineStore('auth', () => {
  const status = ref<AuthStatus>('unknown')
  const user = ref<MeDto | null>(null)

  const displayName = computed(() => user.value?.display_name ?? '')

  /** 启动时以 /auth/me 为准确定登录状态（cookie 前端不可读） */
  async function init(): Promise<void> {
    try {
      user.value = await fetchMe()
      status.value = 'authed'
    } catch {
      user.value = null
      status.value = 'unauthed'
    }
  }

  /** 密码登录；require_totp 时保持 unauthed，由调用方继续两步流程 */
  async function login(username: string, password: string): Promise<LoginResultDto> {
    const result = await apiLogin(username, password)
    if (!result.require_totp) {
      user.value = { username, display_name: result.display_name, totp_enabled: false }
      status.value = 'authed'
    }
    return result
  }

  /** TOTP 第二步（动态码或恢复码），成功后建立会话 */
  async function loginTotp(username: string, pendingToken: string, code: string): Promise<void> {
    const result = await apiLoginTotp(pendingToken, code)
    user.value = { username, display_name: result.display_name, totp_enabled: true }
    status.value = 'authed'
  }

  /** 登出：即使接口失败也清除本地状态 */
  async function logout(): Promise<void> {
    try {
      await apiLogout()
    } finally {
      markUnauthed()
    }
  }

  /** 会话过期（业务请求 401）时由 api client 回调触发 */
  function markUnauthed(): void {
    user.value = null
    status.value = 'unauthed'
  }

  /** TOTP 绑定成功后同步本地状态 */
  function markTotpEnabled(): void {
    if (user.value) user.value = { ...user.value, totp_enabled: true }
  }

  return {
    status,
    user,
    displayName,
    init,
    login,
    loginTotp,
    logout,
    markUnauthed,
    markTotpEnabled,
  }
})
