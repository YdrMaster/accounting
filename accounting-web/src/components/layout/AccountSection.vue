<script setup lang="ts">
import QRCode from 'qrcode'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { apiErrorMessage, totpEnable, totpSetup } from '../../api/client'
import { useAuthStore } from '../../stores/auth'

const { t } = useI18n()
const auth = useAuthStore()

const error = ref<string | null>(null)
const logoutPending = ref(false)

async function onLogout() {
  if (logoutPending.value) return
  logoutPending.value = true
  try {
    await auth.logout()
  } catch {
    // 接口失败时本地状态已在 store 中清除，静默即可
  } finally {
    logoutPending.value = false
  }
}

// ─── TOTP 绑定 ───
type TotpStep = 'idle' | 'scan' | 'recovery'
const totpStep = ref<TotpStep>('idle')
const qrDataUrl = ref('')
const code = ref('')
const recoveryCodes = ref<string[]>([])
const totpPending = ref(false)

async function startSetup() {
  if (totpPending.value) return
  error.value = null
  totpPending.value = true
  try {
    const { otpauth_uri } = await totpSetup()
    qrDataUrl.value = await QRCode.toDataURL(otpauth_uri, { width: 200, margin: 1 })
    code.value = ''
    totpStep.value = 'scan'
  } catch (e) {
    error.value = apiErrorMessage(e) || t('auth.totp.setupFailed')
  } finally {
    totpPending.value = false
  }
}

async function confirmEnable() {
  if (totpPending.value || !code.value.trim()) return
  error.value = null
  totpPending.value = true
  try {
    const { recovery_codes } = await totpEnable(code.value.trim())
    recoveryCodes.value = recovery_codes
    auth.markTotpEnabled()
    totpStep.value = 'recovery'
  } catch (e) {
    code.value = ''
    error.value = apiErrorMessage(e) || t('auth.totp.enableFailed')
  } finally {
    totpPending.value = false
  }
}

function finishRecovery() {
  totpStep.value = 'idle'
  qrDataUrl.value = ''
  recoveryCodes.value = []
}
</script>

<template>
  <div class="list-section">
    <div v-if="error" class="store-error">{{ error }}</div>

    <!-- 当前用户与登出 -->
    <div class="list-item">
      <div class="item-content">
        <span class="item-name">{{ auth.displayName }}</span>
        <span class="item-desc">{{ auth.user?.username }}</span>
      </div>
      <button type="button" class="logout-btn" :disabled="logoutPending" @click="onLogout">
        {{ t('auth.logout') }}
      </button>
    </div>

    <!-- TOTP 两步验证 -->
    <div class="totp-card">
      <div class="totp-header">
        <span class="item-name">{{ t('auth.totp.title') }}</span>
        <span v-if="auth.user?.totp_enabled" class="totp-badge">{{ t('auth.totp.enabled') }}</span>
      </div>

      <button
        v-if="totpStep === 'idle'"
        type="button"
        class="add-btn totp-start"
        :disabled="totpPending"
        @click="startSetup"
      >
        {{ auth.user?.totp_enabled ? t('auth.totp.rebind') : t('auth.totp.start') }}
      </button>

      <template v-if="totpStep === 'scan'">
        <p class="totp-hint">{{ t('auth.totp.scanHint') }}</p>
        <img v-if="qrDataUrl" :src="qrDataUrl" class="totp-qr" alt="TOTP QR" />
        <div class="add-row">
          <input
            v-model="code"
            class="field-input"
            inputmode="numeric"
            autocomplete="one-time-code"
            :placeholder="t('auth.codePlaceholder')"
            @keyup.enter="confirmEnable"
          />
          <button type="button" class="add-btn" :disabled="totpPending" @click="confirmEnable">
            {{ t('auth.totp.enable') }}
          </button>
        </div>
      </template>

      <template v-if="totpStep === 'recovery'">
        <p class="recovery-warning">{{ t('auth.totp.recoveryHint') }}</p>
        <ul class="recovery-list">
          <li v-for="rc in recoveryCodes" :key="rc" class="recovery-code">{{ rc }}</li>
        </ul>
        <button type="button" class="add-btn totp-start" @click="finishRecovery">
          {{ t('auth.totp.done') }}
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.list-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.store-error {
  color: var(--color-expense);
  font-size: 0.8125rem;
  padding: 0.375rem 0.625rem;
  background: rgba(231, 76, 60, 0.1);
  border-radius: 0.375rem;
  margin-bottom: 0.5rem;
}

.list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.625rem;
  border-radius: 0.5rem;
  background: var(--card-bg);
  gap: 0.5rem;
}

.item-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.item-name {
  font-size: 0.9375rem;
  color: var(--text-heading);
}

.item-desc {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.logout-btn {
  padding: 0.375rem 0.75rem;
  border-radius: 0.375rem;
  border: 1px solid var(--color-expense);
  background: transparent;
  color: var(--color-expense);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}

.logout-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.add-btn {
  padding: 0.375rem 0.75rem;
  border-radius: 0.375rem;
  border: 1px solid var(--accent);
  background: transparent;
  color: var(--accent);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}

.add-btn:hover {
  background: var(--accent);
  color: #fff;
}

.add-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.add-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.add-row .field-input {
  flex: 1;
}

.field-input {
  width: 100%;
  padding: 0.375rem 0.5rem;
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

.totp-card {
  border-radius: 0.5rem;
  background: var(--card-bg);
  padding: 0.5rem 0.625rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.totp-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.totp-badge {
  font-size: 0.6875rem;
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  background: var(--accent);
  color: #fff;
}

.totp-start {
  align-self: flex-start;
}

.totp-hint {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--text-muted);
}

.totp-qr {
  align-self: center;
  width: 200px;
  height: 200px;
  border-radius: 0.5rem;
  background: #fff;
}

.recovery-warning {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--color-expense);
}

.recovery-list {
  margin: 0;
  padding: 0.5rem 0.625rem;
  list-style: none;
  border-radius: 0.375rem;
  background: var(--card-bg-alt);
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.25rem;
}

.recovery-code {
  font-family: monospace;
  font-size: 0.8125rem;
  color: var(--text-heading);
  user-select: all;
}
</style>
