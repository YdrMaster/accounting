<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '../../stores/account'
import { useMappingStore } from '../../stores/mapping'
import { useMemberStore } from '../../stores/member'
import type { ChannelDto } from '../../types/api'
import AccountPicker from './AccountPicker.vue'

const props = defineProps<{
  channel: ChannelDto
}>()

const { t } = useI18n()

const memberStore = useMemberStore()
const accountStore = useAccountStore()
const mappingStore = useMappingStore()

const selectedMemberId = ref<number | null>(null)

const mappings = computed(() =>
  selectedMemberId.value === null
    ? []
    : mappingStore.forKey(selectedMemberId.value, props.channel.id)
)

function accountName(accountId: number): string {
  const account = accountStore.accounts.find(a => a.id === accountId)
  return account ? account.name : `#${accountId}`
}

onMounted(async () => {
  await memberStore.load()
  if (selectedMemberId.value === null && memberStore.members.length > 0) {
    selectedMemberId.value = memberStore.members[0].id
  }
})

watch(selectedMemberId, id => {
  if (id !== null) {
    mappingStore.load(id, props.channel.id)
  }
})

// ─── 添加映射 ───
const newCategory = ref('')
const newAccountId = ref<number | null>(null)

async function addMapping() {
  const category = newCategory.value.trim()
  if (!category || newAccountId.value === null || selectedMemberId.value === null) return
  await mappingStore.set({
    member_id: selectedMemberId.value,
    channel_id: props.channel.id,
    category,
    account_id: newAccountId.value,
  })
  newCategory.value = ''
  newAccountId.value = null
}

async function removeMapping(category: string) {
  if (selectedMemberId.value === null) return
  await mappingStore.remove(selectedMemberId.value, props.channel.id, category)
}
</script>

<template>
  <div class="mapping-section">
    <div class="mapping-header">
      <span class="field-label">{{ t('config.importRules') }}</span>
      <select v-model="selectedMemberId" class="field-input member-select">
        <option v-for="member in memberStore.members" :key="member.id" :value="member.id">
          {{ member.name }}
        </option>
      </select>
    </div>

    <div v-if="mappingStore.error" class="store-error">{{ mappingStore.error }}</div>

    <div v-for="mapping in mappings" :key="mapping.category" class="list-item mapping-item">
      <div class="item-content">
        <span class="item-name">{{ mapping.category }}</span>
        <span class="item-desc">{{ accountName(mapping.account_id) }}</span>
      </div>
      <button type="button" class="delete-btn" @click="removeMapping(mapping.category)">
        &times;
      </button>
    </div>

    <div class="add-row">
      <input
        v-model="newCategory"
        class="field-input"
        :placeholder="t('config.mappingCategoryPlaceholder')"
        @keyup.enter="addMapping"
      />
      <div class="mapping-picker">
        <AccountPicker
          :model-value="newAccountId"
          :placeholder="t('config.mappingAccountPlaceholder')"
          @update:model-value="id => (newAccountId = id)"
        />
      </div>
      <button type="button" class="add-btn" @click="addMapping">{{ t('common.add') }}</button>
    </div>
  </div>
</template>

<style scoped>
.mapping-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.mapping-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.field-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-weight: 500;
}

/* 共享 .field-input 基础上的紧凑变体 */
.member-select {
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  font-size: 0.8125rem;
}

.store-error {
  color: var(--color-expense);
  font-size: 0.8125rem;
  padding: 0.375rem 0.625rem;
  background: rgba(231, 76, 60, 0.1);
  border-radius: 0.375rem;
}

.list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.625rem;
  border-radius: 0.5rem;
  background: var(--card-bg-alt);
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
  font-size: 0.875rem;
  color: var(--text-heading);
}

.item-desc {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.delete-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.125rem;
  cursor: pointer;
  line-height: 1;
  padding: 0.125rem 0.375rem;
  flex-shrink: 0;
}

.delete-btn:hover {
  color: var(--color-expense);
}

.add-row {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.375rem;
  align-items: center;
}

/* 共享 .field-input 基础上的布局补充：行内输入框占满剩余宽度 */
.add-row .field-input {
  flex: 1;
}

.mapping-picker {
  flex: 1;
  min-width: 0;
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
</style>
