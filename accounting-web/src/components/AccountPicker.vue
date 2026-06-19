<template>
  <div class="account-picker">
    <button
      type="button"
      class="picker-trigger"
      :class="{ disabled: props.disabled }"
      @click="openSheet"
      :disabled="props.disabled"
    >
      <span v-if="selectedAccount" class="selected-name">{{ selectedAccount.full_name }}</span>
      <span v-else class="placeholder">{{ placeholder }}</span>
    </button>

    <BottomSheet v-model:open="showSheet" title="选择账户">
      <AccountTreeList
        :accounts="accountStore.accounts"
        :parent-id="null"
        :type="props.accountType"
        :model-value="props.modelValue"
        mode="select"
        :allow-add="true"
        :allow-drag="false"
        @update:model-value="handleSelect"
        @add="handleAdd"
      />
    </BottomSheet>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import { useAccountStore } from '@/stores/account'
import BottomSheet from './BottomSheet.vue'
import AccountTreeList from './AccountTreeList.vue'

const props = defineProps<{
  modelValue?: number
  accountType?: string
  placeholder?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [id: number]
}>()

const accountStore = useAccountStore()
const showSheet = ref(false)

const selectedAccount = computed(() => {
  if (!props.modelValue) return null
  return accountStore.accounts.find((a) => a.id === props.modelValue) || null
})

function openSheet() {
  showSheet.value = true
}

function handleSelect(id: number) {
  emit('update:modelValue', id)
}

function handleAdd(_parentId: number | null) {
  // 临时提示，任务 6 会替换为内联添加输入
  message.info('添加账户功能即将接入')
}
</script>

<style scoped>
.account-picker {
  position: relative;
  width: 100%;
}
.picker-trigger {
  width: 100%;
  height: 38px;
  padding: 0 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  text-align: left;
  font-size: 14px;
  transition: border-color 0.2s;
  display: flex;
  align-items: center;
}
.picker-trigger:hover {
  border-color: #40a9ff;
}
.picker-trigger.disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background: #f5f5f5;
}
.picker-trigger.disabled:hover {
  border-color: #d9d9d9;
}
.placeholder {
  color: #bfbfbf;
}
.selected-name {
  color: #333;
}
</style>
