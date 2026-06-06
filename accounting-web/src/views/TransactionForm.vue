<template>
  <div class="transaction-form">
    <h2>记一笔</h2>
    <a-form layout="vertical" @finish="handleSubmit">
      <a-form-item label="日期时间" required>
        <a-date-picker
          v-model:value="dateTime"
          show-time
          format="YYYY-MM-DD HH:mm:ss"
          style="width: 100%"
        />
      </a-form-item>

      <a-form-item label="备注">
        <a-input v-model:value="description" placeholder="请输入备注" />
      </a-form-item>

      <a-form-item label="分录" required>
        <div
          v-for="(posting, index) in postings"
          :key="index"
          class="posting-row"
        >
          <a-input v-model:value="posting.account" placeholder="账户" />
          <a-select v-model:value="posting.commodity" style="width: 100px">
            <a-select-option value="CNY">CNY</a-select-option>
            <a-select-option value="USD">USD</a-select-option>
          </a-select>
          <a-input v-model:value="posting.amount" placeholder="金额" />
          <a-button type="link" danger @click="removePosting(index)">删除</a-button>
        </div>
        <a-button type="dashed" block @click="addPosting">
          <PlusOutlined />
          添加分录
        </a-button>
      </a-form-item>

      <a-form-item label="标签">
        <a-select
          v-model:value="tags"
          mode="tags"
          placeholder="输入标签"
          style="width: 100%"
        />
      </a-form-item>

      <a-form-item>
        <a-button type="primary" html-type="submit" block :loading="submitting">
          确认
        </a-button>
      </a-form-item>
    </a-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import dayjs from 'dayjs'
import type { Dayjs } from 'dayjs'
import { PlusOutlined } from '@ant-design/icons-vue'
import { useTransactionStore } from '@/stores/transaction'
import { useMemberStore } from '@/stores/member'

const route = useRoute()
const router = useRouter()
const transactionStore = useTransactionStore()
const memberStore = useMemberStore()

const dateTime = ref<Dayjs>(dayjs())
const description = ref('')
const postings = ref<{ account: string; commodity: string; amount: string }[]>([
  { account: '', commodity: 'CNY', amount: '' },
])
const tags = ref<string[]>([])
const submitting = ref(false)

function addPosting() {
  postings.value.push({ account: '', commodity: 'CNY', amount: '' })
}

function removePosting(index: number) {
  postings.value.splice(index, 1)
}

async function handleSubmit() {
  submitting.value = true
  try {
    await transactionStore.createTransaction({
      date_time: dateTime.value.format('YYYY-MM-DD HH:mm:ss'),
      description: description.value,
      member_id: memberStore.currentMember?.id,
      postings: postings.value.filter((p) => p.account && p.amount),
      tags: tags.value,
    })
    router.push('/')
  } finally {
    submitting.value = false
  }
}

onMounted(() => {
  const dateParam = route.query.date as string | undefined
  if (dateParam) {
    dateTime.value = dayjs(dateParam).startOf('day')
  }
  memberStore.fetchMe()
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

.posting-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  align-items: center;
}
</style>
