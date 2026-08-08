<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { setUnauthorizedHandler } from './api/client'
import AppDialog from './components/AppDialog.vue'
import ResponsiveShell from './components/layout/ResponsiveShell.vue'
import { useAuthStore } from './stores/auth'
import LoginView from './views/LoginView.vue'

const { t } = useI18n()
const auth = useAuthStore()

// 业务请求 401（会话过期）→ 切回登录页；当前视图由 shell 的模块级滚动位置自动保留
setUnauthorizedHandler(() => auth.markUnauthed())

onMounted(() => auth.init())
</script>

<template>
  <div v-if="auth.status === 'unknown'" class="boot-loading">{{ t('common.loading') }}</div>
  <LoginView v-else-if="auth.status === 'unauthed'" />
  <ResponsiveShell v-else />
  <AppDialog />
</template>

<style scoped>
.boot-loading {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  color: var(--text-muted);
  font-size: 0.875rem;
}
</style>
