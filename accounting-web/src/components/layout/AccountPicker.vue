<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '../../stores/account'
import type { AccountDto } from '../../types/api'
import AccountPickerOverlay from './AccountPickerOverlay.vue'

const props = withDefaults(
  defineProps<{
    modelValue: number | null
    placeholder?: string
    portal?: string
    /** 限定可选择的账户类型；不传则显示全部类型分组 */
    accountType?: 'asset' | 'expense'
  }>(),
  { portal: '.picker-portal' }
)

const emit = defineEmits<{
  'update:modelValue': [accountId: number]
}>()

const { t } = useI18n()

const accountStore = useAccountStore()

onMounted(() => {
  if (accountStore.accounts.length === 0) {
    accountStore.loadAccounts()
  }
})

const selectedName = computed(() => {
  if (props.modelValue === null) return null
  const account = accountStore.accounts.find(a => a.id === props.modelValue)
  return account ? account.name : `#${props.modelValue}`
})

const showOverlay = ref(false)

function onClick() {
  showOverlay.value = true
}

function onClose() {
  showOverlay.value = false
}

function onSelect(account: AccountDto) {
  emit('update:modelValue', account.id)
  showOverlay.value = false
}
</script>

<template>
  <div class="account-picker">
    <button class="picker-trigger" @click="onClick">
      <span v-if="modelValue" class="selected-name">{{ selectedName }}</span>
      <span v-else class="placeholder">{{ placeholder || t('picker.selectPlaceholder') }}</span>
    </button>

    <Teleport :to="props.portal">
      <AccountPickerOverlay
        v-if="showOverlay"
        :current-id="modelValue"
        :account-type="accountType"
        @close="onClose"
        @select="onSelect"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.account-picker {
  position: relative;
  width: 100%;
}

.picker-trigger {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.15s;
}

.picker-trigger:hover {
  border-color: var(--accent, #646cff);
}

.placeholder {
  color: var(--text-muted);
}

.selected-name {
  color: var(--text-heading);
}
</style>
