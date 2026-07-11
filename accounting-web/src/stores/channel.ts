import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchChannels } from '../api/client'
import type { ChannelDto } from '../types/api'

export const useChannelStore = defineStore('channel', () => {
  const channels = ref<ChannelDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(force = false) {
    if (!force && channels.value.length > 0) return

    loading.value = true
    error.value = null
    try {
      channels.value = await fetchChannels()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { channels, loading, error, load }
})
