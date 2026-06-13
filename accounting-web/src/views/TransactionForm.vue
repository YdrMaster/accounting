<template>
  <div class="transaction-form">
    <h2 class="form-title">{{ formTitle }}</h2>
    <a-form layout="vertical">
      <a-form-item required>
        <a-date-picker
          v-model:value="dateTime"
          show-time
          format="YYYY-MM-DD HH:mm:ss"
          style="width: 100%"
        />
      </a-form-item>

      <a-form-item>
        <a-textarea
          v-model:value="description"
          placeholder="备注（可选）"
          :auto-size="{ minRows: 1, maxRows: 6 }"
          @press-enter="() => {}"
        />
      </a-form-item>

      <a-form-item>
        <a-select
          v-model:value="selectedTagNames"
          mode="multiple"
          placeholder="标签"
          style="width: 100%"
          allow-clear
        >
          <a-select-option
            v-for="t in tags"
            :key="t.id"
            :value="t.name"
          >
            {{ t.name }}
          </a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item>
        <a-select
          v-model:value="channelId"
          placeholder="渠道（可选）"
          style="width: 100%"
          allow-clear
        >
          <a-select-option
            v-for="c in channelStore.channels"
            :key="c.id"
            :value="c.id"
          >
            {{ c.name }}
          </a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item required>
        <!-- 普通记账模式 -->
        <template v-if="!isRefundMode && !isReimbursementMode">
          <div
            v-for="(posting, index) in postings"
            :key="index"
            class="posting-row"
          >
            <AccountPicker
              v-model="posting.accountId"
              placeholder="选择账户"
            />
            <a-select v-model:value="posting.commodity" style="width: 100px">
              <a-select-option value="CNY">CNY</a-select-option>
              <a-select-option value="USD">USD</a-select-option>
            </a-select>
            <a-input v-model:value="posting.amount" placeholder="金额" style="width: 120px" />
            <a-button
              v-if="isExpenseAccount(posting.accountId)"
              :type="posting.is_reimbursable ? 'primary' : 'default'"
              size="small"
              @click="posting.is_reimbursable = !posting.is_reimbursable"
            >
              报销
            </a-button>
            <a-button v-else size="small" disabled style="opacity: 0.4; cursor: default;">
              报销
            </a-button>
            <a-button type="link" danger @click="removePosting(index)">删除</a-button>
          </div>
          <a-button type="dashed" block @click="addPosting">
            <PlusOutlined />
            添加分录
          </a-button>
        </template>

        <!-- 退款/报销 分组模式 -->
        <template v-else>
          <div v-for="(group, gi) in groups" :key="gi" class="posting-group">
            <div class="group-header">
              <span class="group-account">{{ group.original.account }}</span>
              <span class="group-meta">{{ group.original.date }} · {{ group.original.description }}</span>
              <span class="group-original-amount">原: ¥{{ group.original.amount }}</span>
              <span class="group-reversal-total">已退款: ¥{{ group.original.reversal_total }}</span>
            </div>
            <!-- 冲减分录（锁定不可编辑），从右到左：action | amount | ± | commodity | tree -->
            <div class="posting-row reversal-row">
              <AccountPicker
                :model-value="group.original.accountId"
                placeholder="选择账户"
                disabled
              />
              <span class="col-commodity">{{ group.original.commodity }}</span>
              <span class="col-sign">-</span>
              <span class="col-amount">
                <a-input :value="formatReversalAmountAbs(group)" style="width: 100%" disabled />
              </span>
              <span class="col-action"><span class="reversal-badge">冲减</span></span>
            </div>
            <!-- 可编辑的资产分录 -->
            <div
              v-for="(asset, ai) in group.assets"
              :key="ai"
              class="posting-row"
            >
              <AccountPicker
                v-model="asset.accountId"
                account-type="Asset"
                placeholder="选择资产账户"
              />
              <span class="col-commodity">{{ group.original.commodity }}</span>
              <span class="col-sign">+</span>
              <span class="col-amount">
                <a-input v-model:value="asset.amount" placeholder="金额" style="width: 100%" />
              </span>
              <span class="col-action"><a-button type="link" danger @click="removeAssetRow(gi, ai)">删除</a-button></span>
            </div>
            <a-button type="dashed" block size="small" @click="addAssetRow(gi)">
              <PlusOutlined />
              添加分录
            </a-button>
          </div>
          <!-- 总退款行 -->
          <div v-if="groups.length > 0" class="total-reversal">
            <span class="total-label">{{ formKind === 'refund' ? '总退款' : '总报销' }}</span>
            <span class="total-amount">¥{{ totalReversalAmount }}</span>
          </div>
        </template>
      </a-form-item>

      <a-form-item>
        <div class="btn-row">
          <a-button class="btn-cancel" @click="handleCancel">
            取消
          </a-button>
          <a-button type="primary" class="btn-submit" :loading="submitting" @click="handleSubmit">
            {{ isEdit ? '保存' : '确认' }}
          </a-button>
        </div>
      </a-form-item>
    </a-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import dayjs from 'dayjs'
import type { Dayjs } from 'dayjs'
import { PlusOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { useTransactionStore, type CreateTransactionData } from '@/stores/transaction'
import { useMemberStore } from '@/stores/member'
import { useAccountStore } from '@/stores/account'
import { useCommodityStore } from '@/stores/commodity'
import { useChannelStore } from '@/stores/channel'
import api from '@/api/client'
import AccountPicker from '@/components/AccountPicker.vue'

const route = useRoute()
const router = useRouter()
const transactionStore = useTransactionStore()
const memberStore = useMemberStore()
const accountStore = useAccountStore()
const channelStore = useChannelStore()
const commodityStore = useCommodityStore()

const isEdit = computed(() => !!route.query.id)
const editId = computed(() => Number(route.query.id))

// 专用模式检测
const isRefundMode = computed(() => route.path === '/transaction/refund')
const isReimbursementMode = computed(() => route.path === '/transaction/reimbursement')
const formKind = computed(() => {
  if (isRefundMode.value) return 'refund'
  if (isReimbursementMode.value) return 'reimbursement'
  return 'normal'
})
const formTitle = computed(() => {
  if (isRefundMode.value) return '录入退款'
  if (isReimbursementMode.value) return '录入报销'
  return isEdit.value ? '编辑交易' : '记一笔'
})

function isExpenseAccount(accountId?: number): boolean {
  if (!accountId) return false
  const account = accountStore.accounts.find(a => a.id === accountId)
  return account?.account_type === 'Expense'
}

const dateTime = ref<Dayjs>(dayjs())
const description = ref('')
const channelId = ref<number | undefined>(undefined)
const postings = ref<{ accountId?: number; commodity: string; amount: string; is_reimbursable: boolean; linked_posting_id?: number }[]>([])

// 退款/报销分组数据
interface AssetRow { accountId?: number; amount: string }
interface PostingGroup {
  original: { id: number; account: string; accountId: number; commodity: string; amount: string; reversal_total: string; date: string; description: string }
  assets: AssetRow[]
}
const groups = ref<PostingGroup[]>([])
const groupsLoading = ref(false)

function formatReversalAmountAbs(group: PostingGroup): string {
  const sum = group.assets.reduce((acc, a) => acc + (parseFloat(a.amount) || 0), 0)
  return sum === 0 ? '0' : String(Math.abs(sum))
}

function addAssetRow(gi: number) {
  const assetsAccount = accountStore.accounts.find(a => a.full_name === 'Assets' || (a.account_type === 'Asset' && a.parent_id == null))
  groups.value[gi].assets.push({ accountId: assetsAccount?.id, amount: '' })
}

function removeAssetRow(gi: number, ai: number) {
  groups.value[gi].assets.splice(ai, 1)
}

const totalReversalAmount = computed(() => {
  let sum = 0
  for (const group of groups.value) {
    sum += group.assets.reduce((acc, a) => acc + (parseFloat(a.amount) || 0), 0)
  }
  return String(sum)
})
const selectedTagNames = ref<string[]>([])
const tags = ref<{ id: number; name: string }[]>([])
const submitting = ref(false)

function addPosting() {
  const sum = postings.value.reduce((acc, p) => acc + (parseFloat(p.amount) || 0), 0)
  const balancedAmount = sum === 0 ? '' : String(-sum)
  postings.value.push({ commodity: 'CNY', amount: balancedAmount, is_reimbursable: false })
}

function removePosting(index: number) {
  postings.value.splice(index, 1)
}

async function handleSubmit() {
  // 验证日期
  if (!dateTime.value || !dateTime.value.isValid()) {
    message.error('请选择日期时间')
    return
  }

  submitting.value = true
  try {
    if (isRefundMode.value || isReimbursementMode.value) {
      await submitRefundReimbursement()
    } else {
      await submitNormal()
    }
    router.push('/')
  } catch (err: any) {
    const msg = err?.response?.data?.error || err?.message || '提交失败'
    message.error(msg)
  } finally {
    submitting.value = false
  }
}

async function submitRefundReimbursement() {
  const accountMap = new Map(accountStore.accounts.map((a) => [a.id, a.full_name]))
  const allPostings: any[] = []

  for (const group of groups.value) {
    // 验证资产分录
    const validAssets = group.assets.filter(a => a.accountId && parseFloat(a.amount) > 0)
    if (validAssets.length === 0) {
      message.error(`"${group.original.account}" 至少需要一个资产分录`)
      return
    }

    // 计算冲减金额：正值的资产之和
    const assetSum = validAssets.reduce((acc, a) => acc + parseFloat(a.amount), 0)
    const reversalAmount = String(-assetSum)

    // 冲减分录
    allPostings.push({
      account: group.original.account,
      commodity: group.original.commodity,
      amount: reversalAmount,
      is_reimbursable: false,
      linked_posting_id: group.original.id,
    })

    // 资产分录（用户输入正数）
    for (const a of validAssets) {
      allPostings.push({
        account: accountMap.get(a.accountId!) || '',
        commodity: group.original.commodity,
        amount: a.amount,
        is_reimbursable: false,
      })
    }
  }

  const data: CreateTransactionData = {
    date_time: dateTime.value.format('YYYY-MM-DD HH:mm:ss'),
    description: description.value,
    kind: formKind.value,
    member_id: memberStore.currentMember?.id,
    channel_id: channelId.value,
    postings: allPostings,
    tags: selectedTagNames.value,
  }

  await transactionStore.createTransaction(data)
  message.success(formKind.value === 'refund' ? '退款录入成功' : '报销录入成功')
}

async function submitNormal() {
  // 验证分录
  const validPostings = postings.value.filter((p) => p.accountId && p.amount.trim() !== '')
  if (validPostings.length < 2) {
    message.error('至少需要两条有效分录（借方和贷方）')
    return
  }

  const accountMap = new Map(accountStore.accounts.map((a) => [a.id, a.full_name]))

  const data: CreateTransactionData = {
    date_time: dateTime.value.format('YYYY-MM-DD HH:mm:ss'),
    description: description.value,
    member_id: memberStore.currentMember?.id,
    channel_id: channelId.value,
    kind: formKind.value,
    postings: validPostings.map((p) => ({
      account: accountMap.get(p.accountId!) || '',
      commodity: p.commodity,
      amount: p.amount.trim(),
      is_reimbursable: p.is_reimbursable,
      linked_posting_id: p.linked_posting_id,
    })),
    tags: selectedTagNames.value,
  }

  if (isEdit.value) {
    await transactionStore.updateTransaction(editId.value, data)
    message.success('更新成功')
  } else {
    await transactionStore.createTransaction(data)
    message.success('记账成功')
  }
}

function handleCancel() {
  router.push('/')
}

async function fetchTags() {
  try {
    const res = await api.get<{ id: number; name: string }[]>('/tags')
    tags.value = res.data
  } catch (e) {
    console.error('获取标签失败', e)
  }
}

async function loadTransaction(id: number) {
  try {
    const res = await api.get(`/transactions/${id}`)
    const tx = res.data
    dateTime.value = dayjs(tx.date_time)
    description.value = tx.description || ''
    if (tx.member_id) {
      // 不强制切换当前成员，仅填充表单
    }

    const accountMap = new Map(accountStore.accounts.map((a) => [a.full_name, a.id]))
    postings.value = (tx.postings || []).map((p: any) => ({
      accountId: accountMap.get(p.account),
      commodity: p.commodity,
      amount: String(p.amount ?? ''),
      is_reimbursable: p.is_reimbursable || false,
      linked_posting_id: p.linked_posting_id,
    }))

    if (tx.tags) {
      selectedTagNames.value = tx.tags
    }
    if (tx.channel_id) {
      channelId.value = tx.channel_id
    }
  } catch (e) {
    console.error('加载交易失败', e)
    message.error('加载交易失败')
  }
}

onMounted(() => {
  commodityStore.fetchCommodities()
  const dateParam = route.query.date as string | undefined
  if (dateParam) {
    dateTime.value = dayjs(dateParam).startOf('day')
  }
  memberStore.fetchMe()
  channelStore.fetchChannels()
  accountStore.fetchAccounts().then(() => {
    if (isEdit.value) {
      loadTransaction(editId.value)
    } else if (isRefundMode.value || isReimbursementMode.value) {
      loadOriginalPostings()
    }
  })
  fetchTags()
})

async function loadOriginalPostings() {
  const ids = route.query.posting_ids as string
  if (!ids) return
  const idList = ids.split(',').map(Number).filter(n => !isNaN(n))
  if (idList.length === 0) return

  groupsLoading.value = true
  try {
    const postings = await Promise.all(idList.map(id => transactionStore.fetchPosting(id)))

    const txIds = new Set(postings.map(p => p.transaction_id))
    const txMap = new Map<number, { date: string; description: string }>()
    await Promise.all(
      Array.from(txIds).map(async (txId) => {
        const res = await api.get<{ date_time: string; description: string }>(`/transactions/${txId}`)
        txMap.set(txId, { date: res.data.date_time.slice(0, 10), description: res.data.description })
      })
    )

    for (const p of postings) {
      const tx = txMap.get(p.transaction_id)
      groups.value.push({
        original: {
          id: p.id,
          account: p.account,
          accountId: accountStore.accounts.find(a => a.full_name === p.account)?.id || 0,
          commodity: p.commodity,
          amount: p.amount,
          reversal_total: p.reversal_total || '0',
          date: tx?.date || '—',
          description: tx?.description || '—',
        },
        assets: [],
      })
    }
  } catch (e) {
    console.error('加载原分录失败', e)
    message.error('加载原分录失败')
  } finally {
    groupsLoading.value = false
  }
}
</script>

<style scoped>
.transaction-form {
  max-width: 600px;
  margin: 0 auto;
  background: #fff;
  padding: 24px;
  border-radius: 8px;
}

.form-title {
  margin-bottom: 24px;
}

.posting-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  align-items: center;
}

.posting-row :deep(.account-picker) {
  flex: 1;
  min-width: 0;
}

.btn-row {
  display: flex;
  gap: 12px;
}

.btn-cancel {
  flex: 1;
}

.btn-submit {
  flex: 2;
}

/* 退款/报销 分组样式 */
.posting-group {
  border: 1px solid #e8e8e8;
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 16px;
  background: #fafafa;
}

.group-header {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
}
.group-account { font-weight: 600; color: #333; }
.group-meta { color: #666; }
.group-original-amount { color: #f5222d; font-weight: 500; }
.group-reversal-total { color: #999; }

.reversal-row {
  opacity: 0.75;
}

.col-commodity {
  flex-shrink: 0;
  width: 44px;
  font-size: 14px;
  color: #333;
  text-align: center;
}

.col-sign {
  flex-shrink: 0;
  width: 10px;
  font-weight: 600;
  font-size: 14px;
  text-align: center;
}

.col-amount {
  flex-shrink: 0;
  width: 90px;
  font-size: 14px;
  text-align: left;
  color: #52c41a;
}

.col-action {
  flex-shrink: 0;
  width: 62px;
  display: flex;
  justify-content: center;
}

.reversal-badge {
  color: #fa8c16;
  border: 1px solid #fa8c16;
  font-size: 11px;
  padding: 0 6px;
  border-radius: 2px;
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  height: 24px;
  box-sizing: border-box;
}


.total-reversal {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  border-radius: 6px;
  margin-top: 8px;
  font-size: 15px;
}
.total-label { font-weight: 500; color: #333; }
.total-amount { color: #52c41a; font-weight: 600; }
</style>
