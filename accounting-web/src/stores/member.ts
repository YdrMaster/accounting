import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchMembers } from '../api/client'
import type { MemberDto } from '../types/api'

export const useMemberStore = defineStore('member', () => {
  const members = ref<MemberDto[]>([])
  const currentMemberId = ref<number | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(force = false) {
    if (!force && members.value.length > 0) return

    loading.value = true
    error.value = null
    try {
      members.value = await fetchMembers()
      // Set current member to first member if not set
      if (members.value.length > 0 && currentMemberId.value === null) {
        currentMemberId.value = members.value[0].id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { members, currentMemberId, loading, error, load }
})
