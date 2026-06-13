import { defineStore } from 'pinia'
import { ref } from 'vue'
import api from '@/api/client'

export interface Commodity {
  id: number
  symbol: string
  name: string
  precision: number
}

export const useCommodityStore = defineStore('commodity', () => {
  const commodities = ref<Commodity[]>([])

  async function fetchCommodities() {
    try {
      const res = await api.get<Commodity[]>('/commodities')
      commodities.value = res.data
    } catch (e) {
      console.error('获取货币列表失败', e)
    }
  }

  function getPrecision(symbol: string): number {
    return commodities.value.find(c => c.symbol === symbol)?.precision ?? 2
  }

  return { commodities, fetchCommodities, getPrecision }
})
