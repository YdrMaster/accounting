<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import Decimal from 'decimal.js'
import AccountPicker from './AccountPicker.vue'
import ChannelPathInput from './ChannelPathInput.vue'
import { useTransactionStore } from '../../stores/transaction'
import { useMemberStore } from '../../stores/member'
import { useCommodityStore } from '../../stores/commodity'
import { useChannelStore } from '../../stores/channel'
import { useTagStore } from '../../stores/tag'
import { fetchTransaction } from '../../api/client'
import type { CreateTransactionData, ChannelPathNodeInput } from '../../types/api'

const props = defineProps<{
  editId?: number
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const txStore = useTransactionStore()
const memberStore = useMemberStore()
const commodityStore = useCommodityStore()
const channelStore = useChannelStore()
const tagStore = useTagStore()

onMounted(async () => {
  await Promise.all([
    memberStore.load(),
    commodityStore.load(),
    channelStore.load(),
    tagStore.load(),
  ])

  if (props.editId) {
    await loadTransaction(props.editId)
  } else {
    // Default values for new transaction
    dateTime.value = formatDateTime(new Date())
    memberId.value = memberStore.currentMemberId
    addPosting()
    addPosting()
  }
})

const dateTime = ref('')
const description = ref('')
const memberId = ref<number | null>(null)
const selectedTags = ref<string[]>([])
const channelPaths = ref<ChannelPathNodeInput[]>([])

interface PostingDraft {
  accountId: number | null
  accountName: string
  commodity: string
  amount: string
  isReimbursable: boolean
}

const postings = ref<PostingDraft[]>([])

const isEdit = computed(() => !!props.editId)
const formTitle = computed(() => isEdit.value ? '编辑交易' : '新建交易')

function formatDateTime(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function addPosting() {
  const sum = postings.value.reduce((acc, p) => {
    const amt = new Decimal(p.amount || 0)
    return acc.plus(amt)
  }, new Decimal(0))

  const balancedAmount = sum.isZero() ? '' : sum.negated().toFixed(2)

  postings.value.push({
    accountId: null,
    accountName: '',
    commodity: 'CNY',
    amount: balancedAmount,
    isReimbursable: false,
  })
}

function removePosting(index: number) {
  postings.value.splice(index, 1)
}

function onAccountSelect(index: number, accountId: number, accountName: string) {
  postings.value[index].accountId = accountId
  postings.value[index].accountName = accountName
}

const isBalanced = computed(() => {
  if (postings.value.length < 2) return false
  const sum = postings.value.reduce((acc, p) => {
    const amt = new Decimal(p.amount || 0)
    return acc.plus(amt)
  }, new Decimal(0))
  return sum.isZero()
})

const canSubmit = computed(() => {
  if (!isBalanced.value) return false
  if (!dateTime.value) return false
  if (!memberId.value) return false
  // Check all postings have accounts and amounts
  return postings.value.every(p => p.accountId && p.amount)
})

async function loadTransaction(id: number) {
  try {
    const tx = await fetchTransaction(id)
    dateTime.value = tx.date_time
    description.value = tx.description
    memberId.value = tx.member_id
    selectedTags.value = [...tx.tags]
    channelPaths.value = tx.channel_paths.map(cp => ({
      position: cp.position,
      channel_id: cp.channel_id,
      status: cp.status,
    }))

    postings.value = tx.postings.map(p => ({
      accountId: null, // We don't have account ID in PostingDto, only account name
      accountName: p.account,
      commodity: p.commodity,
      amount: p.amount,
      isReimbursable: p.is_reimbursable,
    }))
  } catch (e) {
    console.error('Failed to load transaction:', e)
  }
}

async function handleSubmit() {
  if (!canSubmit.value) return

  const data: CreateTransactionData = {
    date_time: dateTime.value,
    description: description.value,
    kind: 'normal',
    member_id: memberId.value!,
    channel_paths: channelPaths.value,
    postings: postings.value.map(p => ({
      account: p.accountName,
      commodity: p.commodity,
      amount: p.amount,
      is_reimbursable: p.isReimbursable,
    })),
    tags: selectedTags.value,
  }

  try {
    if (isEdit.value) {
      await txStore.update(props.editId!, data)
    } else {
      await txStore.create(data)
    }
    emit('saved')
    emit('close')
  } catch (e) {
    console.error('Failed to save transaction:', e)
    alert('保存失败: ' + (e instanceof Error ? e.message : String(e)))
  }
}
</script>

<template>
  <div class="form-overlay">
    <div class="form-content-wrapper">
      <div class="form-header">
        <span class="form-title">{{ formTitle }}</span>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>

      <div class="form-body">
        <div class="field">
          <label>日期时间</label>
          <input v-model="dateTime" type="datetime-local" step="1" />
        </div>

        <div class="field">
          <label>备注</label>
          <textarea v-model="description" rows="2" placeholder="可选"></textarea>
        </div>

        <div class="field">
          <label>成员</label>
          <select v-model="memberId">
            <option :value="null" disabled>选择成员</option>
            <option v-for="m in memberStore.members" :key="m.id" :value="m.id">
              {{ m.name }}
            </option>
          </select>
        </div>

        <div class="field">
          <label>标签</label>
          <div class="tag-input">
            <span v-for="tag in selectedTags" :key="tag" class="tag-chip">
              {{ tag }}
              <button @click="selectedTags = selectedTags.filter(t => t !== tag)">×</button>
            </span>
            <input
              type="text"
              placeholder="添加标签"
              @keydown.enter="e => {
                const input = e.target as HTMLInputElement
                if (input.value.trim()) {
                  selectedTags.push(input.value.trim())
                  input.value = ''
                }
              }"
            />
          </div>
        </div>

        <div class="field">
          <label>渠道链路</label>
          <ChannelPathInput v-model="channelPaths" />
        </div>

        <div class="section-title">分录</div>

        <div v-for="(posting, index) in postings" :key="index" class="posting-row">
          <div class="posting-field">
            <label>账户</label>
            <AccountPicker
              :model-value="posting.accountId"
              @update:model-value="(id) => onAccountSelect(index, id, `账户 #${id}`)"
            />
          </div>

          <div class="posting-field">
            <label>币种</label>
            <select v-model="posting.commodity">
              <option v-for="c in commodityStore.commodities" :key="c.id" :value="c.symbol">
                {{ c.symbol }}
              </option>
            </select>
          </div>

          <div class="posting-field">
            <label>金额</label>
            <input v-model="posting.amount" type="number" step="0.01" placeholder="0.00" />
          </div>

          <div class="posting-field checkbox-field">
            <label>
              <input type="checkbox" v-model="posting.isReimbursable" />
              报销
            </label>
          </div>

          <button class="remove-btn" @click="removePosting(index)" :disabled="postings.length <= 2">
            ×
          </button>
        </div>

        <button class="add-posting-btn" @click="addPosting">+ 添加分录</button>
      </div>

      <div class="form-footer">
        <button class="cancel-btn" @click="emit('close')">取消</button>
        <button class="submit-btn" :disabled="!canSubmit" @click="handleSubmit">
          {{ isEdit ? '保存' : '确认' }}
        </button>
      </div>
    </div>

    <!-- Portal container for AccountPicker Teleport -->
    <div class="picker-portal"></div>
  </div>
</template>

<style scoped>
.form-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  background: var(--card-bg, #1e1e1e);
  z-index: 100;
  display: flex;
  flex-direction: column;
  animation: slideIn 0.2s ease-out;
}

.form-content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Portal container for AccountPicker - must fill entire form overlay */
.picker-portal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.picker-portal > * {
  pointer-events: auto;
}

@keyframes slideIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.form-title {
  font-weight: 600;
  color: var(--text-heading);
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  color: var(--text-muted);
  cursor: pointer;
  line-height: 1;
}

.form-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-height: 0;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.field label {
  color: var(--text-muted);
  font-size: 0.8125rem;
}

.field input,
.field textarea,
.field select {
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  outline: none;
}

.field input:focus,
.field textarea:focus,
.field select:focus {
  border-color: var(--accent, #646cff);
}

.tag-input {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
}

.tag-chip {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  background: var(--accent, #646cff);
  color: #fff;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
}

.tag-chip button {
  background: none;
  border: none;
  color: #fff;
  cursor: pointer;
  font-size: 0.875rem;
  line-height: 1;
  opacity: 0.7;
}

.tag-chip button:hover {
  opacity: 1;
}

.tag-input input {
  flex: 1;
  min-width: 100px;
  background: transparent;
  border: none;
  color: var(--text-heading);
  font-size: 0.875rem;
  outline: none;
}

.section-title {
  font-weight: 500;
  color: var(--text-heading);
  margin-top: 0.5rem;
}

.posting-row {
  display: grid;
  grid-template-columns: 1fr 80px 100px auto 32px;
  gap: 0.5rem;
  align-items: end;
}

.posting-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.posting-field label {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.posting-field input,
.posting-field select {
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.375rem 0.5rem;
  color: var(--text-heading);
  font-size: 0.8125rem;
  outline: none;
}

.checkbox-field label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--text-heading);
  font-size: 0.8125rem;
  cursor: pointer;
  padding: 0.375rem 0;
}

.remove-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.375rem;
  line-height: 1;
}

.remove-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.remove-btn:hover:not(:disabled) {
  color: #e74c3c;
}

.add-posting-btn {
  background: var(--card-bg-alt, #252525);
  border: 1px dashed var(--border);
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: var(--text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  transition: border-color 0.15s;
}

.add-posting-btn:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.form-footer {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  border-top: 1px solid var(--border);
}

.cancel-btn {
  flex: 1;
  background: var(--card-bg-alt, #252525);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  padding: 0.625rem;
  color: var(--text-heading);
  font-size: 0.875rem;
  cursor: pointer;
}

.submit-btn {
  flex: 2;
  background: var(--accent, #646cff);
  border: none;
  border-radius: 0.5rem;
  padding: 0.625rem;
  color: #fff;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.submit-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>

<style>
/* Non-scoped styles for Teleport target */
.form-overlay .picker-portal {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.form-overlay .picker-portal > * {
  pointer-events: auto;
}
</style>
