import { defineStore } from 'pinia'
import { ref } from 'vue'
import { deleteMapping, fetchMappings, upsertMapping } from '../api/client'
import type { MappingDto } from '../types/api'

function cacheKey(memberId: number, channelId: number): string {
  return `${memberId}:${channelId}`
}

export const useMappingStore = defineStore('mapping', () => {
  const mappings = ref<Record<string, MappingDto[]>>({})
  const loading = ref(false)
  const error = ref<string | null>(null)

  function forKey(memberId: number, channelId: number): MappingDto[] {
    return mappings.value[cacheKey(memberId, channelId)] ?? []
  }

  async function load(memberId: number, channelId: number) {
    loading.value = true
    error.value = null
    try {
      mappings.value[cacheKey(memberId, channelId)] = await fetchMappings(memberId, channelId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function set(dto: MappingDto) {
    error.value = null
    try {
      await upsertMapping(dto)
      const key = cacheKey(dto.member_id, dto.channel_id)
      const list = mappings.value[key] ?? []
      const idx = list.findIndex(m => m.category === dto.category)
      mappings.value[key] = idx !== -1 ? list.map((m, i) => (i === idx ? dto : m)) : [...list, dto]
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(memberId: number, channelId: number, category: string) {
    error.value = null
    try {
      await deleteMapping(memberId, channelId, category)
      const key = cacheKey(memberId, channelId)
      const list = mappings.value[key]
      if (list) {
        mappings.value[key] = list.filter(m => m.category !== category)
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return { mappings, loading, error, forKey, load, set, remove }
})
