import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createMember,
  deleteMember,
  fetchMembers,
  renameMember,
} from '../api/client'
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
      if (members.value.length > 0 && currentMemberId.value === null) {
        currentMemberId.value = members.value[0].id
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(name: string) {
    error.value = null
    try {
      const member = await createMember(name)
      members.value.push(member)
      return member
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function rename(id: number, name: string) {
    error.value = null
    try {
      const updated = await renameMember(id, name)
      const idx = members.value.findIndex((m) => m.id === id)
      if (idx !== -1) {
        members.value[idx] = updated
      }
      return updated
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(id: number) {
    error.value = null
    try {
      await deleteMember(id)
      members.value = members.value.filter((m) => m.id !== id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return { members, currentMemberId, loading, error, load, create, rename, remove }
})
