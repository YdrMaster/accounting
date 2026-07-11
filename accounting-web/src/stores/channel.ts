import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createChannel,
  deleteChannel,
  fetchChannels,
  updateChannel,
} from '../api/client'
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

  async function create(data: {
    name: string
    description?: string
    account_id?: number
  }) {
    error.value = null
    try {
      const id = await createChannel(data)
      await load(true)
      return id
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function update(
    id: number,
    data: { name?: string; description?: string; account_id?: number },
  ) {
    error.value = null
    try {
      await updateChannel(id, data)
      const idx = channels.value.findIndex((c) => c.id === id)
      if (idx !== -1) {
        const current = channels.value[idx]
        channels.value[idx] = {
          ...current,
          ...(data.name !== undefined && { name: data.name }),
          ...(data.description !== undefined && { description: data.description || null }),
          ...(data.account_id !== undefined && { account_id: data.account_id }),
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function remove(id: number) {
    error.value = null
    try {
      await deleteChannel(id)
      channels.value = channels.value.filter((c) => c.id !== id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return { channels, loading, error, load, create, update, remove }
})
