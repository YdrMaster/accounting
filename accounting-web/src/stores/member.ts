import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Member {
  id: number
  name: string
}

export const useMemberStore = defineStore('member', () => {
  const members = ref<Member[]>([])
  const currentMember = ref<Member | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchMembers() {
    loading.value = true
    error.value = null
    try {
      const res = await api.get<Member[]>('/members')
      members.value = res.data
    } catch (e) {
      error.value = '获取成员失败'
      console.error(e)
    } finally {
      loading.value = false
    }
  }

  async function setCurrent(id: number) {
    const found = members.value.find((m) => m.id === id)
    if (found) {
      currentMember.value = found
      try {
        await api.put('/me', { member_id: id })
      } catch (e) {
        console.error('设置当前成员失败', e)
      }
    }
  }

  async function fetchMe() {
    try {
      const res = await api.get<{ member_id: number; member_name: string }>('/me')
      currentMember.value = {
        id: res.data.member_id,
        name: res.data.member_name,
      }
    } catch (e) {
      console.error('获取当前成员失败', e)
    }
  }

  return { members, currentMember, loading, error, fetchMembers, setCurrent, fetchMe }
})
