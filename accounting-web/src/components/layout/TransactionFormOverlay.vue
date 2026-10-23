<script setup lang="ts">
import Decimal from 'decimal.js'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { fetchTransaction } from '../../api/client'
import { useAccountStore } from '../../stores/account'
import { useChannelStore } from '../../stores/channel'
import { useCommodityStore } from '../../stores/commodity'
import { useMemberStore } from '../../stores/member'
import { useTagStore } from '../../stores/tag'
import { useTransactionStore } from '../../stores/transaction'
import type { ChannelPathNodeInput, CreateTransactionData } from '../../types/api'
import { alertDialog, confirmDialog } from '../../utils/dialog'
import AccountPicker from './AccountPicker.vue'
import ChannelPathInput from './ChannelPathInput.vue'

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
const accountStore = useAccountStore()

const { t } = useI18n()

onMounted(async () => {
  await Promise.all([
    memberStore.load(),
    commodityStore.load(),
    channelStore.load(),
    tagStore.load(),
    accountStore.loadAccounts(),
  ])

  if (props.editId) {
    await loadTransaction(props.editId)
  } else {
    // Default values for new transaction（成员默认未选中，用户手工选择；只自动创建一个分录）
    dateTime.value = formatDateTime(new Date())
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

/** 候选标签：只支持从已有标签中选择，不允许手工输入 */
const availableTags = computed(() =>
  tagStore.tags.filter(tag => !selectedTags.value.includes(tag.name))
)

function addTag(e: Event) {
  const select = e.target as HTMLSelectElement
  if (select.value) {
    selectedTags.value.push(select.value)
    select.value = ''
  }
}

/** 备注框高度自适应：自动放大到无需滚动条即可显示全部文本 */
const descRef = ref<HTMLTextAreaElement | null>(null)

function autoGrow() {
  const el = descRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${el.scrollHeight}px`
}

watch(description, () => nextTick(autoGrow))

const isEdit = computed(() => !!props.editId)
const formTitle = computed(() => (isEdit.value ? t('txForm.editTitle') : t('txForm.createTitle')))

function formatDateTime(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** 金额文本合法性：可选前导负号、至多一个小数点、至少一位数字（如 -5、10.、.5、0） */
const AMOUNT_PATTERN = /^-?(\d+(\.\d*)?|\.\d+)$/

/** 金额文本是否合法；空串视为「未填」，不算合法 */
function isAmountValid(text: string): boolean {
  return AMOUNT_PATTERN.test(text)
}

/** 容错解析金额文本：空串或非法文本返回 null，避免 decimal.js 抛异常 */
function parseAmount(text: string): Decimal | null {
  return isAmountValid(text) ? new Decimal(text) : null
}

function addPosting() {
  // 非法/未填金额不入和（容错，避免解析异常导致表单崩溃）
  const sum = postings.value.reduce((acc, p) => {
    const amt = parseAmount(p.amount)
    return amt === null ? acc : acc.plus(amt)
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

/** 金额输入过滤：只允许 -?\d*\.?\d*(负号仅首位、至多一个小数点) */
function onAmountInput(index: number, e: Event) {
  const input = e.target as HTMLInputElement
  let v = input.value.replace(/[^\d.-]/g, '')
  v = v.replace(/(?!^)-/g, '')
  const firstDot = v.indexOf('.')
  if (firstDot !== -1) {
    v = v.slice(0, firstDot + 1) + v.slice(firstDot + 1).replace(/\./g, '')
  }
  input.value = v
  postings.value[index].amount = v
}

function onAccountSelect(index: number, accountId: number) {
  postings.value[index].accountId = accountId
  postings.value[index].accountName = accountStore.accountPath(accountId)
}

const isBalanced = computed(() => {
  if (postings.value.length < 2) return false
  // 非法或未填金额直接判为不平，避免 new Decimal('-') 抛出 DecimalError
  let sum = new Decimal(0)
  for (const p of postings.value) {
    const amt = parseAmount(p.amount)
    if (amt === null) return false
    sum = sum.plus(amt)
  }
  return sum.isZero()
})

/** 每条分录都必须有账户且金额非空且合法（非法金额不算已填写） */
const allAmountsValid = computed(() =>
  postings.value.every(p => p.accountId && isAmountValid(p.amount))
)

/** 存在未填完或非法的分录（分类、金额为空或金额非法）时不允许再添加新分录 */
const canAddPosting = computed(() => allAmountsValid.value)

const canSubmit = computed(() => {
  if (!isBalanced.value) return false
  if (!dateTime.value) return false
  if (!memberId.value) return false
  return allAmountsValid.value
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
      accountId: p.account_id,
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
      // 后端 amount 要求字符串
      amount: String(p.amount),
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
    alertDialog(t('txForm.saveFailed', { message: e instanceof Error ? e.message : String(e) }))
  }
}

async function handleDelete() {
  if (!isEdit.value || props.editId == null) return
  if (!(await confirmDialog(t('transactions.confirmDelete')))) return
  try {
    await txStore.remove(props.editId)
    emit('close')
  } catch (e) {
    alertDialog(t('txForm.deleteFailed', { message: e instanceof Error ? e.message : String(e) }))
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
        <div class="field-row">
          <div class="field field-member">
            <label>{{ t('txForm.member') }}</label>
            <select v-model="memberId">
              <option :value="null" disabled>{{ t('txForm.memberPlaceholder') }}</option>
              <option v-for="m in memberStore.members" :key="m.id" :value="m.id">
                {{ m.name }}
              </option>
            </select>
          </div>

          <div class="field field-date">
            <label>{{ t('txForm.dateTime') }}</label>
            <input v-model="dateTime" type="datetime-local" step="1" />
          </div>
        </div>

        <div class="field">
          <label>{{ t('txForm.description') }}</label>
          <textarea
            ref="descRef"
            v-model="description"
            class="desc-textarea"
            rows="1"
            :placeholder="t('txForm.optional')"
            @input="autoGrow"
          ></textarea>
        </div>

        <div class="field field-tags">
          <label>{{ t('txForm.tags') }}</label>
          <div class="tag-input">
            <span v-for="tag in selectedTags" :key="tag" class="tag-chip">
              {{ tag }}
              <button @click="selectedTags = selectedTags.filter(item => item !== tag)">×</button>
            </span>
            <select class="tag-select" @change="addTag">
              <option value="">{{ t('txForm.tagPlaceholder') }}</option>
              <option v-for="tag in availableTags" :key="tag.id" :value="tag.name">
                {{ tag.name }}
              </option>
            </select>
          </div>
        </div>

        <div class="field">
          <label>{{ t('txForm.channelPath') }}</label>
          <ChannelPathInput v-model="channelPaths" />
        </div>

        <div class="section-title">{{ t('txForm.postings') }}</div>

        <div v-for="(posting, index) in postings" :key="index" class="posting-row">
          <div class="posting-field">
            <label>{{ t('txForm.account') }}</label>
            <AccountPicker
              :model-value="posting.accountId"
              @update:model-value="id => onAccountSelect(index, id)"
            />
          </div>

          <div class="posting-field">
            <label>{{ t('txForm.commodity') }}</label>
            <select v-model="posting.commodity">
              <option v-for="c in commodityStore.commodities" :key="c.id" :value="c.symbol">
                {{ c.symbol }}
              </option>
            </select>
          </div>

          <div class="posting-field">
            <label>{{ t('txForm.amount') }}</label>
            <input
              :value="posting.amount"
              class="amount-input"
              :class="{ invalid: posting.amount !== '' && !isAmountValid(posting.amount) }"
              type="text"
              inputmode="decimal"
              placeholder="0.00"
              @input="e => onAmountInput(index, e)"
            />
          </div>

          <div class="posting-field checkbox-field">
            <label>
              <input v-model="posting.isReimbursable" type="checkbox" />
              {{ t('txForm.reimbursable') }}
            </label>
          </div>

          <button class="remove-btn" :disabled="postings.length <= 2" @click="removePosting(index)">
            ×
          </button>
        </div>

        <button class="add-posting-btn" :disabled="!canAddPosting" @click="addPosting">
          {{ t('txForm.addPosting') }}
        </button>
      </div>

      <div class="form-footer">
        <button v-if="isEdit" class="delete-btn" @click="handleDelete">
          {{ t('common.delete') }}
        </button>
        <button class="cancel-btn" @click="emit('close')">{{ t('common.cancel') }}</button>
        <button class="submit-btn" :disabled="!canSubmit" @click="handleSubmit">
          {{ isEdit ? t('common.save') : t('common.confirm') }}
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
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
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

/* 成员 + 日期同一横排，成员宽度由内部文本自适应，日期占满剩余 */
.field-row {
  display: flex;
  gap: 0.75rem;
}

.field-member {
  flex: 0 0 auto;
}

.field-member select {
  align-self: flex-start;
  width: auto;
}

.field-date {
  flex: 1;
  min-width: 0;
}

/* 备注框：禁止拖动调整大小，宽度固定，高度随内容自适应，两端对齐 */
.desc-textarea {
  resize: none;
  width: 100%;
  overflow: hidden;
  text-align: justify;
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

/* 下拉不显示箭头 */
.field select,
.posting-field select {
  appearance: none;
}

.field input:focus,
.field textarea:focus,
.field select:focus {
  border-color: var(--accent, #646cff);
}

/* 标签横排：与渠道链路同一视觉规格，只是没有 ▸ 分隔符 */
.tag-input {
  display: flex;
  align-items: center;
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
  gap: 0.5ch;
  background: var(--accent, #646cff);
  color: #fff;
  padding: 0.25rem 0.5ch 0.25rem 0.5rem;
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

/* 幽灵芯片风格的内联下拉（嵌套一层 .tag-input 以压过 .field select 的优先级） */
.tag-input .tag-select {
  appearance: none;
  background: transparent;
  border: 1px dashed var(--border);
  border-radius: 0.25rem;
  padding: 0.25rem 0.5rem;
  color: var(--text-muted);
  font-size: 0.75rem;
  outline: none;
  cursor: pointer;
}

.tag-input .tag-select:hover {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
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

.amount-input.invalid,
.amount-input.invalid:focus {
  border-color: var(--color-expense);
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
  color: var(--color-expense);
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

.add-posting-btn:hover:not(:disabled) {
  border-color: var(--accent, #646cff);
  color: var(--accent, #646cff);
}

.add-posting-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.form-footer {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  border-top: 1px solid var(--border);
}

.delete-btn {
  flex: 1;
  background: none;
  border: 1px solid var(--color-expense);
  border-radius: 0.5rem;
  padding: 0.625rem;
  color: var(--color-expense);
  font-size: 0.875rem;
  cursor: pointer;
}

.delete-btn:hover {
  background: var(--color-expense);
  color: #fff;
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
