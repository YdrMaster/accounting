import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchCommodities } from '../api/client'
import type { CommodityDto } from '../types/api'

export const useCommodityStore = defineStore('commodity', () => {
  const commodities = ref<CommodityDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(force = false) {
    if (!force && commodities.value.length > 0) return

    loading.value = true
    error.value = null
    try {
      commodities.value = await fetchCommodities()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { commodities, loading, error, load }
})
