import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchTags } from '../api/client'
import type { TagDto } from '../types/api'

export const useTagStore = defineStore('tag', () => {
  const tags = ref<TagDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(force = false) {
    if (!force && tags.value.length > 0) return

    loading.value = true
    error.value = null
    try {
      tags.value = await fetchTags()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { tags, loading, error, load }
})
