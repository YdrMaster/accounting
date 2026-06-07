<template>
  <div class="transaction-form">
    <h2 class="form-title">{{ isEdit ? '编辑交易' : '记一笔' }}</h2>
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
        <div
          v-for="(posting, index) in postings"
          :key="index"
          class="posting-row"
        >
          <a-tree-select
            v-model:value="posting.accountId"
            :tree-data="accountTreeData"
            :field-names="{ children: 'children', label: 'title', value: 'key' }"
            placeholder="选择账户"
            style="flex: 1"
            tree-default-expand-all
          />
          <a-select v-model:value="posting.commodity" style="width: 100px">
            <a-select-option value="CNY">CNY</a-select-option>
            <a-select-option value="USD">USD</a-select-option>
          </a-select>
          <a-select v-model:value="posting.kind" style="width: 80px">
            <a-select-option value="normal">普通</a-select-option>
            <a-select-option value="refund">退款</a-select-option>
            <a-select-option value="reimbursement">报销</a-select-option>
          </a-select>
          <a-input-number
            v-if="posting.kind !== 'normal'"
            v-model:value="posting.linked_posting_id"
            placeholder="原分录 ID"
            style="width: 120px"
          />
          <a-input v-model:value="posting.amount" placeholder="金额" style="width: 120px" />
          <a-button type="link" danger @click="removePosting(index)">删除</a-button>
        </div>
        <a-button type="dashed" block @click="addPosting">
          <PlusOutlined />
          添加分录
        </a-button>
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
import { useChannelStore } from '@/stores/channel'
import api from '@/api/client'

const route = useRoute()
const router = useRouter()
const transactionStore = useTransactionStore()
const memberStore = useMemberStore()
const accountStore = useAccountStore()
const channelStore = useChannelStore()

const isEdit = computed(() => !!route.query.id)
const editId = computed(() => Number(route.query.id))

const dateTime = ref<Dayjs>(dayjs())
const description = ref('')
const channelId = ref<number | undefined>(undefined)
const postings = ref<{ accountId?: number; commodity: string; amount: string; kind: string; linked_posting_id?: number }[]>([])
const selectedTagNames = ref<string[]>([])
const tags = ref<{ id: number; name: string }[]>([])
const submitting = ref(false)

const accountTreeData = computed(() => {
  const accounts = accountStore.accounts
  const map = new Map<number, any>()
  const roots: any[] = []

  accounts.forEach((acc) => {
    const segments = acc.full_name.split(':')
    map.set(acc.id, {
      title: segments[segments.length - 1],
      key: acc.id,
      children: [],
    })
  })

  accounts.forEach((acc) => {
    const node = map.get(acc.id)
    if (!node) return
    if (acc.parent_id && map.has(acc.parent_id)) {
      map.get(acc.parent_id)!.children.push(node)
    } else {
      roots.push(node)
    }
  })

  return roots
})

function addPosting() {
  const sum = postings.value.reduce((acc, p) => {
    const val = parseFloat(p.amount)
    return acc + (isNaN(val) ? 0 : val)
  }, 0)
  const balancedAmount = sum === 0 ? '' : String(-sum)
  postings.value.push({ commodity: 'CNY', amount: balancedAmount, kind: 'normal' })
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

  // 验证分录
  const validPostings = postings.value.filter((p) => p.accountId && p.amount.trim() !== '')
  if (validPostings.length < 2) {
    message.error('至少需要两条有效分录（借方和贷方）')
    return
  }

  // 验证金额格式
  for (const p of validPostings) {
    if (isNaN(Number(p.amount))) {
      message.error(`金额格式错误: ${p.amount}`)
      return
    }
  }

  const accountMap = new Map(accountStore.accounts.map((a) => [a.id, a.full_name]))

  const data: CreateTransactionData = {
    date_time: dateTime.value.format('YYYY-MM-DD HH:mm:ss'),
    description: description.value,
    member_id: memberStore.currentMember?.id,
    channel_id: channelId.value,
    postings: validPostings.map((p) => ({
      account: accountMap.get(p.accountId!) || '',
      commodity: p.commodity,
      amount: p.amount.trim(),
      kind: p.kind || 'normal',
      linked_posting_id: p.linked_posting_id,
    })),
    tags: selectedTagNames.value,
  }

  submitting.value = true
  try {
    if (isEdit.value) {
      await transactionStore.updateTransaction(editId.value, data)
      message.success('更新成功')
    } else {
      await transactionStore.createTransaction(data)
      message.success('记账成功')
    }
    router.push('/')
    router.push('/')
  } catch (err: any) {
    const msg = err?.response?.data?.error || err?.message || (isEdit.value ? '更新失败' : '记账失败')
    message.error(msg)
  } finally {
    submitting.value = false
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
      amount: String(p.amount),
      kind: p.kind || 'normal',
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
  const dateParam = route.query.date as string | undefined
  if (dateParam) {
    dateTime.value = dayjs(dateParam).startOf('day')
  }
  memberStore.fetchMe()
  channelStore.fetchChannels()
  accountStore.fetchAccounts().then(() => {
    if (isEdit.value) {
      loadTransaction(editId.value)
    }
  })
  fetchTags()
})
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
</style>
