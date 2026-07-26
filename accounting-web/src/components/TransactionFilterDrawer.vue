<script setup lang="ts">
import { computed, onMounted, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AccountPicker from './layout/AccountPicker.vue'
import DateRangePicker from './DateRangePicker.vue'
import { useAccountStore } from '../stores/account'
import { useChannelStore } from '../stores/channel'
import { useMemberStore } from '../stores/member'
import { useTagStore } from '../stores/tag'
import { useTransactionStore } from '../stores/transaction'
import type { TxFilters } from '../types/api'
import { formatDate } from '../utils/date'
import { isFilterActive } from '../utils/txFilter'

const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()
const txStore = useTransactionStore()
const accountStore = useAccountStore()
const tagStore = useTagStore()
const channelStore = useChannelStore()
const memberStore = useMemberStore()

onMounted(() => {
  accountStore.loadAccounts()
  tagStore.load()
  channelStore.load()
  memberStore.load()
})

const filter = reactive<TxFilters>({
  from: txStore.activeFilter?.from,
  to: txStore.activeFilter?.to,
  accounts: [...(txStore.activeFilter?.accounts ?? [])],
  members: [...(txStore.activeFilter?.members ?? [])],
  tags: [...(txStore.activeFilter?.tags ?? [])],
  channels: [...(txStore.activeFilter?.channels ?? [])],
  keyword: txStore.activeFilter?.keyword,
  reimbursable: txStore.activeFilter?.reimbursable,
})

const active = computed(() => isFilterActive(filter))

let debounceTimer: ReturnType<typeof setTimeout> | null = null
watch(
  filter,
  () => {
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      txStore.setFilter(active.value ? { ...filter } : null)
    }, 300)
  },
  { deep: true }
)

// ─── Time presets ───

type PresetKey = 'thisMonth' | 'lastMonth' | 'last3Months' | 'thisYear' | 'all'

function today(): Date {
  return new Date()
}

function presetRange(key: PresetKey): { from?: string; to?: string } {
  const now = today()
  switch (key) {
    case 'thisMonth':
      return { from: formatDate(new Date(now.getFullYear(), now.getMonth(), 1)), to: formatDate(now) }
    case 'lastMonth': {
      const start = new Date(now.getFullYear(), now.getMonth() - 1, 1)
      const end = new Date(now.getFullYear(), now.getMonth(), 0)
      return { from: formatDate(start), to: formatDate(end) }
    }
    case 'last3Months': {
      const start = new Date(now.getFullYear(), now.getMonth() - 2, 1)
      return { from: formatDate(start), to: formatDate(now) }
    }
    case 'thisYear':
      return { from: formatDate(new Date(now.getFullYear(), 0, 1)), to: formatDate(now) }
    case 'all':
      return { from: undefined, to: undefined }
  }
}

const activePreset = computed<PresetKey | null>(() => {
  const keys: PresetKey[] = ['thisMonth', 'lastMonth', 'last3Months', 'thisYear', 'all']
  for (const key of keys) {
    const range = presetRange(key)
    if ((range.from ?? undefined) === (filter.from ?? undefined) &&
        (range.to ?? undefined) === (filter.to ?? undefined)) {
      return key
    }
  }
  return null
})

function applyPreset(key: PresetKey) {
  const range = presetRange(key)
  filter.from = range.from
  filter.to = range.to
}

// ─── Multi-select toggles ───

function addAccount(id: number) {
  if (!filter.accounts.includes(id)) filter.accounts.push(id)
}

function removeAccount(id: number) {
  const idx = filter.accounts.indexOf(id)
  if (idx !== -1) filter.accounts.splice(idx, 1)
}

function accountName(id: number): string {
  const acc = accountStore.accounts.find(a => a.id === id)
  return acc ? acc.name : `#${id}`
}

function toggleTag(name: string) {
  const idx = filter.tags.indexOf(name)
  if (idx === -1) filter.tags.push(name)
  else filter.tags.splice(idx, 1)
}

function toggleChannel(id: number) {
  const idx = filter.channels.indexOf(id)
  if (idx === -1) filter.channels.push(id)
  else filter.channels.splice(idx, 1)
}

function toggleMember(id: number) {
  const idx = filter.members.indexOf(id)
  if (idx === -1) filter.members.push(id)
  else filter.members.splice(idx, 1)
}

// ─── Actions ───

function reset() {
  filter.from = undefined
  filter.to = undefined
  filter.accounts = []
  filter.members = []
  filter.tags = []
  filter.channels = []
  filter.keyword = undefined
  filter.reimbursable = undefined
}

function done() {
  emit('close')
}
</script>

<template>
  <div class="filter-drawer">
    <div class="drawer-content">
      <!-- Time range -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.timeRange') }}</h3>
        <div class="preset-chips">
          <button
            v-for="key in (['thisMonth', 'lastMonth', 'last3Months', 'thisYear', 'all'] as PresetKey[])"
            :key="key"
            class="chip"
            :class="{ selected: activePreset === key }"
            @click="applyPreset(key)"
          >
            {{ t(`txFilter.preset.${key}`) }}
          </button>
        </div>
        <DateRangePicker
          :from="filter.from"
          :to="filter.to"
          @update:from="filter.from = $event"
          @update:to="filter.to = $event"
        />
      </section>

      <!-- Accounts -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.accounts') }}</h3>
        <AccountPicker
          :model-value="null"
          :placeholder="t('txFilter.addAccount')"
          portal=".tx-filter-portal"
          @update:model-value="addAccount"
        />
        <ul v-if="filter.accounts.length" class="selected-list">
          <li v-for="id in filter.accounts" :key="id" class="selected-item">
            <span class="selected-name">{{ accountName(id) }}</span>
            <button class="remove-btn" @click="removeAccount(id)">×</button>
          </li>
        </ul>
      </section>

      <!-- Tags -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.tags') }}</h3>
        <div class="chip-grid">
          <button
            v-for="tag in tagStore.tags"
            :key="tag.id"
            class="chip"
            :class="{ selected: filter.tags.includes(tag.name) }"
            @click="toggleTag(tag.name)"
          >
            {{ tag.name }}
          </button>
        </div>
      </section>

      <!-- Channels -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.channels') }}</h3>
        <div class="chip-grid">
          <button
            v-for="ch in channelStore.channels"
            :key="ch.id"
            class="chip"
            :class="{ selected: filter.channels.includes(ch.id) }"
            @click="toggleChannel(ch.id)"
          >
            {{ ch.name }}
          </button>
        </div>
      </section>

      <!-- Members -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.members') }}</h3>
        <div class="chip-grid">
          <button
            v-for="m in memberStore.members"
            :key="m.id"
            class="chip"
            :class="{ selected: filter.members.includes(m.id) }"
            @click="toggleMember(m.id)"
          >
            {{ m.name }}
          </button>
        </div>
      </section>

      <!-- Keyword + Reimbursable -->
      <section class="filter-section">
        <h3 class="section-title">{{ t('txFilter.keyword') }}</h3>
        <input
          type="text"
          v-model="filter.keyword"
          class="keyword-input"
          :placeholder="t('txFilter.keywordPlaceholder')"
        />
        <label class="toggle-row">
          <span>{{ t('txFilter.reimbursable') }}</span>
          <input type="checkbox" v-model="filter.reimbursable" class="toggle-checkbox" />
        </label>
      </section>
    </div>

    <!-- Footer -->
    <div class="drawer-footer">
      <button class="btn-reset" :disabled="!active" @click="reset">
        {{ t('txFilter.reset') }}
      </button>
      <button class="btn-done" @click="done">
        {{ t('txFilter.done') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.filter-drawer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 60%;
  display: flex;
  flex-direction: column;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border);
  border-radius: 0 0 1rem 1rem;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.25);
  z-index: 90;
  animation: slide-down 0.25s ease-out;
}

@keyframes slide-down {
  from {
    transform: translateY(-100%);
  }
  to {
    transform: translateY(0);
  }
}

.drawer-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  scrollbar-width: none;
}

.drawer-content::-webkit-scrollbar {
  display: none;
}

.filter-section {
  margin-bottom: 1.25rem;
}

.section-title {
  margin: 0 0 0.5rem;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-muted);
}

.preset-chips,
.chip-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.chip {
  padding: 0.3rem 0.7rem;
  border-radius: 1rem;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-body);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
}

.chip:hover {
  border-color: var(--accent, #646cff);
}

.chip.selected {
  background: var(--accent, #646cff);
  border-color: var(--accent, #646cff);
  color: #fff;
}

.keyword-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: transparent;
  color: var(--text-body);
  font-size: 0.875rem;
  box-sizing: border-box;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 0.75rem;
  font-size: 0.875rem;
  color: var(--text-body);
  cursor: pointer;
}

.toggle-checkbox {
  width: 1.125rem;
  height: 1.125rem;
  accent-color: var(--accent, #646cff);
}

.drawer-footer {
  display: flex;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--border);
}

.btn-reset,
.btn-done {
  flex: 1;
  padding: 0.5rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.btn-reset {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-body);
}

.btn-reset:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-done {
  background: var(--accent, #646cff);
  color: #fff;
}

.selected-list {
  list-style: none;
  margin: 0.5rem 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.selected-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 0.625rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  font-size: 0.8125rem;
}

.selected-name {
  color: var(--text-body);
}

.remove-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
}

.remove-btn:hover {
  color: #e55;
  background: rgba(229, 85, 85, 0.1);
}
</style>
