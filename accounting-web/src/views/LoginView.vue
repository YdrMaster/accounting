<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ApiError, apiErrorMessage } from '../api/client'
import { useAuthStore } from '../stores/auth'

const { t } = useI18n()
const auth = useAuthStore()

/** 后端 MSG_BAD_TOTP 文案：login/totp 返回该 401 表示动态码错误，其余 401 视为 pending 过期 */
const SERVER_BAD_CODE = '验证码错误'

const username = ref('')
const password = ref('')
const code = ref('')
const pendingToken = ref<string | null>(null)
const error = ref<string | null>(null)
const submitting = ref(false)

function toErrorMessage(e: unknown, fallbackKey: string): string {
  if (e instanceof ApiError && e.status === 429) return t('auth.rateLimited')
  return apiErrorMessage(e) || t(fallbackKey)
}

async function submitPassword() {
  if (submitting.value || !username.value.trim() || !password.value) return
  error.value = null
  submitting.value = true
  try {
    const result = await auth.login(username.value.trim(), password.value)
    if (result.require_totp && result.pending_token) {
      pendingToken.value = result.pending_token
      code.value = ''
    }
  } catch (e) {
    error.value = toErrorMessage(e, 'auth.loginFailed')
  } finally {
    // 无论成败都清空密码，保留用户名
    password.value = ''
    submitting.value = false
  }
}

async function submitCode() {
  if (submitting.value || !pendingToken.value || !code.value.trim()) return
  error.value = null
  submitting.value = true
  try {
    await auth.loginTotp(username.value.trim(), pendingToken.value, code.value.trim())
  } catch (e) {
    if (e instanceof ApiError && e.status === 401 && apiErrorMessage(e) !== SERVER_BAD_CODE) {
      // pending 已过期/作废：退回密码输入界面重新登录
      pendingToken.value = null
      code.value = ''
      error.value = t('auth.pendingExpired')
    } else {
      code.value = ''
      error.value = toErrorMessage(e, 'auth.verifyFailed')
    }
  } finally {
    submitting.value = false
  }
}

function backToPassword() {
  pendingToken.value = null
  code.value = ''
  error.value = null
}
</script>

<template>
  <div class="login-page">
    <div class="login-card">
      <h1 class="login-title">{{ t('header.title') }}</h1>

      <div v-if="error" class="login-error">{{ error }}</div>

      <!-- 密码表单 -->
      <form v-if="!pendingToken" class="login-form" @submit.prevent="submitPassword">
        <div class="field">
          <label class="field-label" for="login-username">{{ t('auth.username') }}</label>
          <input
            id="login-username"
            v-model="username"
            class="field-input"
            autocomplete="username"
          />
        </div>
        <div class="field">
          <label class="field-label" for="login-password">{{ t('auth.password') }}</label>
          <input
            id="login-password"
            v-model="password"
            type="password"
            class="field-input"
            autocomplete="current-password"
          />
        </div>
        <button type="submit" class="submit-btn" :disabled="submitting">
          {{ submitting ? t('auth.submitting') : t('auth.submit') }}
        </button>
      </form>

      <!-- TOTP 两步输入 -->
      <form v-else class="login-form" @submit.prevent="submitCode">
        <p class="totp-hint">{{ t('auth.totpHint') }}</p>
        <div class="field">
          <label class="field-label" for="login-code">{{ t('auth.codeLabel') }}</label>
          <input
            id="login-code"
            v-model="code"
            class="field-input"
            inputmode="numeric"
            autocomplete="one-time-code"
            :placeholder="t('auth.codePlaceholder')"
          />
        </div>
        <button type="submit" class="submit-btn" :disabled="submitting">
          {{ submitting ? t('auth.submitting') : t('auth.verify') }}
        </button>
        <button type="button" class="back-btn" @click="backToPassword">
          {{ t('auth.backToPassword') }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  padding: 1rem;
}

.login-card {
  width: 100%;
  max-width: 22rem;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  padding: 1.5rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.login-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-heading);
  text-align: center;
}

.login-error {
  color: var(--color-expense);
  font-size: 0.8125rem;
  padding: 0.375rem 0.625rem;
  background: rgba(231, 76, 60, 0.1);
  border-radius: 0.375rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.totp-hint {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--text-muted);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.field-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-weight: 500;
}

.field-input {
  width: 100%;
  padding: 0.5rem 0.625rem;
  border-radius: 0.375rem;
  border: 1px solid var(--border);
  background: var(--card-bg-alt);
  color: var(--text-heading);
  font-size: 0.875rem;
  outline: none;
  box-sizing: border-box;
}

.field-input:focus {
  border-color: var(--accent);
}

.submit-btn {
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid var(--accent);
  background: var(--accent);
  color: #fff;
  font-size: 0.875rem;
  cursor: pointer;
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.back-btn {
  background: none;
  border: none;
  padding: 0.25rem;
  color: var(--accent);
  font-size: 0.8125rem;
  cursor: pointer;
}
</style>
