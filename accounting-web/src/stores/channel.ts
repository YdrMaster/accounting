import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Channel {
  id: number
  name: string
  description: string | null
}

export const useChannelStore = defineStore('channel', () => {
  const channels = ref<Channel[]>([])

  async function fetchChannels() {
    const res = await api.get<Channel[]>('/channels')
    channels.value = res.data
  }

  async function createChannel(name: string, description?: string) {
    await api.post('/channels', { name, description })
    await fetchChannels()
  }

  async function deleteChannel(id: number) {
    await api.delete(`/channels/${id}`)
    await fetchChannels()
  }

  return { channels, fetchChannels, createChannel, deleteChannel }
})
