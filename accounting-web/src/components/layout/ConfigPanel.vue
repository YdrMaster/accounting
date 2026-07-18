<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, type AppLocale } from '../../i18n'
import { useAccountStore } from '../../stores/account'
import { useChannelStore } from '../../stores/channel'
import { useMemberStore } from '../../stores/member'
import { useTagStore } from '../../stores/tag'
import type { ChannelDto, MemberDto, TagDto } from '../../types/api'
import AccountPicker from './AccountPicker.vue'

const emit = defineEmits<{
  close: []
}>()

const { t, locale } = useI18n()

const memberStore = useMemberStore()
const channelStore = useChannelStore()
const tagStore = useTagStore()
const accountStore = useAccountStore()

type Tab = 'member' | 'channel' | 'tag' | 'language'
const activeTab = ref<Tab>('member')

const languageOptions: { value: AppLocale; labelKey: string }[] = [
  { value: 'zh-CN', labelKey: 'language.zh-CN' },
  { value: 'en', labelKey: 'language.en' },
]

async function switchLanguage(lang: AppLocale) {
  if (locale.value === lang) return
  setLocale(lang)
  await Promise.all([
    accountStore.loadAccounts(),
    memberStore.load(),
    channelStore.load(),
    tagStore.load(),
  ])
}

onMounted(async () => {
  await Promise.all([memberStore.load(), channelStore.load(), tagStore.load()])
})

// ─── 成员 ───
const newMemberName = ref('')
const renamingMemberId = ref<number | null>(null)
const renamingMemberName = ref('')
const memberInputRef = ref<HTMLInputElement | null>(null)

function startRenameMember(member: MemberDto) {
  renamingMemberId.value = member.id
  renamingMemberName.value = member.name
  nextTick(() => memberInputRef.value?.focus())
}

async function commitRenameMember() {
  if (renamingMemberId.value === null) return
  const name = renamingMemberName.value.trim()
  if (!name) {
    renamingMemberId.value = null
    return
  }
  await memberStore.rename(renamingMemberId.value, name)
  renamingMemberId.value = null
}

async function addMember() {
  const name = newMemberName.value.trim()
  if (!name) return
  await memberStore.create(name)
  newMemberName.value = ''
}

async function removeMember(id: number) {
  await memberStore.remove(id)
}

// ─── 标签 ───
const newTagName = ref('')
const renamingTagId = ref<number | null>(null)
const renamingTagName = ref('')
const tagInputRef = ref<HTMLInputElement | null>(null)

function startRenameTag(tag: TagDto) {
  renamingTagId.value = tag.id
  renamingTagName.value = tag.name
  nextTick(() => tagInputRef.value?.focus())
}

async function commitRenameTag() {
  if (renamingTagId.value === null) return
  const name = renamingTagName.value.trim()
  if (!name) {
    renamingTagId.value = null
    return
  }
  await tagStore.update(renamingTagId.value, { name })
  renamingTagId.value = null
}

async function addTag() {
  const name = newTagName.value.trim()
  if (!name) return
  await tagStore.create(name)
  newTagName.value = ''
}

async function removeTag(id: number) {
  await tagStore.remove(id)
}

// ─── 渠道 ───
const newChannelName = ref('')
const expandedChannelId = ref<number | null>(null)

watch(
  () => channelStore.channels,
  channels => {
    if (expandedChannelId.value !== null && !channels.some(c => c.id === expandedChannelId.value)) {
      expandedChannelId.value = null
    }
  }
)

function toggleChannel(id: number) {
  expandedChannelId.value = expandedChannelId.value === id ? null : id
}

async function addChannel() {
  const name = newChannelName.value.trim()
  if (!name) return
  await channelStore.create({ name })
  newChannelName.value = ''
}

async function removeChannel(id: number) {
  await channelStore.remove(id)
}

async function onChannelNameChange(channel: ChannelDto, newName: string) {
  const trimmed = newName.trim()
  if (!trimmed || trimmed === channel.name) return
  await channelStore.update(channel.id, { name: trimmed })
}

async function onChannelDescChange(channel: ChannelDto, newDesc: string) {
  const desc = newDesc.trim() || undefined
  const currentDesc = channel.description || undefined
  if (desc === currentDesc) return
  await channelStore.update(channel.id, { description: desc })
}

function onChannelAccountChange(channel: ChannelDto, accountId: number) {
  if (accountId === channel.account_id) return
  channelStore.update(channel.id, { account_id: accountId })
}
</script>

<template>
  <div class="drawer-container" @click.self="emit('close')">
    <div class="drawer">
      <div class="drawer-handle" />

      <div class="drawer-header">
        <h2 class="drawer-title">{{ t('config.title') }}</h2>
        <button type="button" class="close-btn" @click="emit('close')">&times;</button>
      </div>

      <div class="tab-bar">
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'member' }"
          @click="activeTab = 'member'"
        >
          {{ t('config.tabs.member') }}
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'channel' }"
          @click="activeTab = 'channel'"
        >
          {{ t('config.tabs.channel') }}
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'tag' }"
          @click="activeTab = 'tag'"
        >
          {{ t('config.tabs.tag') }}
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'language' }"
          @click="activeTab = 'language'"
        >
          {{ t('config.tabs.language') }}
        </button>
      </div>

      <div class="drawer-body">
        <!-- 成员 tab -->
        <div v-if="activeTab === 'member'" class="list-section">
          <div v-if="memberStore.error" class="store-error">{{ memberStore.error }}</div>
          <div v-for="member in memberStore.members" :key="member.id" class="list-item">
            <input
              v-if="renamingMemberId === member.id"
              ref="memberInputRef"
              v-model="renamingMemberName"
              class="inline-input"
              @keyup.enter="commitRenameMember"
              @blur="commitRenameMember"
              @keydown.escape="renamingMemberId = null"
            />
            <span v-else class="item-name clickable" @click="startRenameMember(member)">
              {{ member.name }}
            </span>
            <button type="button" class="delete-btn" @click="removeMember(member.id)">
              &times;
            </button>
          </div>
          <div class="add-row">
            <input
              v-model="newMemberName"
              class="inline-input"
              :placeholder="t('config.newNamePlaceholder')"
              @keyup.enter="addMember"
            />
            <button type="button" class="add-btn" @click="addMember">{{ t('common.add') }}</button>
          </div>
        </div>

        <!-- 渠道 tab -->
        <div v-if="activeTab === 'channel'" class="list-section">
          <div v-if="channelStore.error" class="store-error">{{ channelStore.error }}</div>
          <div v-for="channel in channelStore.channels" :key="channel.id" class="channel-card">
            <div class="channel-header" @click="toggleChannel(channel.id)">
              <span class="expand-icon">
                {{ expandedChannelId === channel.id ? '▾' : '▸' }}
              </span>
              <span class="item-name">{{ channel.name }}</span>
              <button type="button" class="delete-btn" @click.stop="removeChannel(channel.id)">
                &times;
              </button>
            </div>
            <div v-if="expandedChannelId === channel.id" class="channel-body">
              <div class="field">
                <label class="field-label">{{ t('common.name') }}</label>
                <input
                  :value="channel.name"
                  class="field-input"
                  @change="e => onChannelNameChange(channel, (e.target as HTMLInputElement).value)"
                />
              </div>
              <div class="field">
                <label class="field-label">{{ t('common.description') }}</label>
                <input
                  :value="channel.description || ''"
                  class="field-input"
                  :placeholder="t('config.channelDescPlaceholder')"
                  @change="e => onChannelDescChange(channel, (e.target as HTMLInputElement).value)"
                />
              </div>
              <div class="field">
                <label class="field-label">{{ t('config.linkedAccount') }}</label>
                <AccountPicker
                  :model-value="channel.account_id"
                  :placeholder="t('config.notLinked')"
                  @update:model-value="id => onChannelAccountChange(channel, id)"
                />
              </div>
            </div>
          </div>
          <div class="add-row">
            <input
              v-model="newChannelName"
              class="inline-input"
              :placeholder="t('config.newNamePlaceholder')"
              @keyup.enter="addChannel"
            />
            <button type="button" class="add-btn" @click="addChannel">{{ t('common.add') }}</button>
          </div>
          <div class="picker-portal" />
        </div>

        <!-- 标签 tab -->
        <div v-if="activeTab === 'tag'" class="list-section">
          <div v-if="tagStore.error" class="store-error">{{ tagStore.error }}</div>
          <div v-for="tag in tagStore.tags" :key="tag.id" class="list-item">
            <div class="item-content">
              <input
                v-if="renamingTagId === tag.id"
                ref="tagInputRef"
                v-model="renamingTagName"
                class="inline-input"
                @keyup.enter="commitRenameTag"
                @blur="commitRenameTag"
                @keydown.escape="renamingTagId = null"
              />
              <span
                v-else
                class="item-name"
                :class="{ clickable: !tag.is_system }"
                @click="!tag.is_system && startRenameTag(tag)"
              >
                {{ tag.name }}
              </span>
              <span v-if="tag.description" class="item-desc">{{ tag.description }}</span>
            </div>
            <button
              v-if="!tag.is_system"
              type="button"
              class="delete-btn"
              @click="removeTag(tag.id)"
            >
              &times;
            </button>
          </div>
          <div class="add-row">
            <input
              v-model="newTagName"
              class="inline-input"
              :placeholder="t('config.newNamePlaceholder')"
              @keyup.enter="addTag"
            />
            <button type="button" class="add-btn" @click="addTag">{{ t('common.add') }}</button>
          </div>
        </div>
        <!-- 语言 tab -->
        <div v-if="activeTab === 'language'" class="list-section">
          <div class="field">
            <label class="field-label">{{ t('language.label') }}</label>
            <div class="lang-options">
              <button
                v-for="option in languageOptions"
                :key="option.value"
                type="button"
                class="lang-btn"
                :class="{ active: locale === option.value }"
                @click="switchLanguage(option.value)"
              >
                {{ t(option.labelKey) }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drawer-container {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  pointer-events: auto;
}

.drawer {
  position: relative;
  width: 100%;
  max-width: 600px;
  max-height: 66vh;
  background: var(--bg);
  border-radius: 1rem 1rem 0 0;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.25s ease;
  overflow: hidden;
  margin: 0 auto;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
}

@keyframes slideUp {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}

.drawer-handle {
  width: 2.5rem;
  height: 0.25rem;
  background: var(--border);
  border-radius: 0.125rem;
  margin: 0.75rem auto 0;
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.25rem;
}

.drawer-title {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-heading);
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
}

.tab-bar {
  display: flex;
  gap: 0;
  padding: 0 1.25rem;
  border-bottom: 1px solid var(--border);
}

.tab-btn {
  padding: 0.5rem 1rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.tab-btn.active {
  color: var(--text-heading);
  border-bottom-color: var(--accent);
  font-weight: 500;
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem 1.25rem 1.25rem;
}

.store-error {
  color: #e74c3c;
  font-size: 0.8125rem;
  padding: 0.375rem 0.625rem;
  background: rgba(231, 76, 60, 0.1);
  border-radius: 0.375rem;
  margin-bottom: 0.5rem;
}

.list-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
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

.item-name.clickable {
  cursor: pointer;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  border: 1px solid transparent;
  transition: background 0.15s;
}

.item-name.clickable:hover {
  background: var(--card-bg-alt);
}

.item-desc {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.inline-input {
  flex: 1;
  padding: 0.375rem 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid var(--border);
  background: var(--card-bg-alt);
  color: var(--text-heading);
  font-size: 0.9375rem;
  outline: none;
  min-width: 0;
}

.inline-input:focus {
  border-color: var(--accent);
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
  color: #e74c3c;
}

.add-row {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.375rem;
  align-items: center;
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

/* 渠道卡片 */
.channel-card {
  border-radius: 0.5rem;
  background: var(--card-bg);
  overflow: hidden;
}

.channel-header {
  display: flex;
  align-items: center;
  padding: 0.5rem 0.625rem;
  cursor: pointer;
  gap: 0.5rem;
}

.expand-icon {
  font-size: 0.75rem;
  color: var(--text-muted);
  width: 1rem;
  flex-shrink: 0;
}

.channel-body {
  padding: 0.5rem 0.625rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  border-top: 1px solid var(--border);
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

.picker-portal {
  position: fixed;
  inset: 0;
  z-index: 200;
  pointer-events: none;
}

.picker-portal > * {
  pointer-events: auto;
}

.lang-options {
  display: flex;
  gap: 0.5rem;
}

.lang-btn {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border);
  background: var(--card-bg);
  color: var(--text-heading);
  font-size: 0.875rem;
  cursor: pointer;
}

.lang-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 500;
}
</style>
