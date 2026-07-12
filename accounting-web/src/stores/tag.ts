import { defineStore } from 'pinia'
import { ref } from 'vue'
import { createTag, deleteTag, fetchTags, updateTag } from '../api/client'
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

  async function create(name: string, description?: string) {
    error.value = null
    try {
      const tag = await createTag({ name, description })
      tags.value.push(tag)
      return tag
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function update(id: number, data: { name?: string; description?: string }) {
    error.value = null
    try {
      const updated = await updateTag(id, data)
      const idx = tags.value.findIndex(t => t.id === id)
      if (idx !== -1) {
        tags.value[idx] = updated
      }
      return updated
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(id: number) {
    error.value = null
    try {
      await deleteTag(id)
      tags.value = tags.value.filter(t => t.id !== id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return { tags, loading, error, load, create, update, remove }
})
