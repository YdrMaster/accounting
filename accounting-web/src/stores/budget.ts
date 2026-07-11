import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  fetchBudgets,
  fetchBudgetDetail,
  fetchBudgetStatus,
  createBudget,
  updateBudget,
  deleteBudget,
} from '../api/client'
import type {
  BudgetDto,
  BudgetDetailDto,
  BudgetStatusDto,
  CreateBudgetRequest,
} from '../types/api'

export const useBudgetStore = defineStore('budget', () => {
  const budgets = ref<BudgetDto[]>([])
  const currentDetail = ref<BudgetDetailDto | null>(null)
  const currentStatus = ref<BudgetStatusDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadBudgets() {
    loading.value = true
    error.value = null
    try {
      budgets.value = await fetchBudgets()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadDetail(id: number) {
    loading.value = true
    error.value = null
    try {
      currentDetail.value = await fetchBudgetDetail(id)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadStatus(id: number, date?: string) {
    loading.value = true
    error.value = null
    try {
      currentStatus.value = await fetchBudgetStatus(id, date)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(data: CreateBudgetRequest): Promise<BudgetDto> {
    const budget = await createBudget(data)
    budgets.value.push(budget)
    return budget
  }

  async function update(id: number, data: CreateBudgetRequest): Promise<void> {
    await updateBudget(id, data)
    await loadBudgets()
  }

  async function remove(id: number): Promise<void> {
    await deleteBudget(id)
    budgets.value = budgets.value.filter((b) => b.id !== id)
    if (currentDetail.value?.budget.id === id) {
      currentDetail.value = null
    }
    if (currentStatus.value?.budget.id === id) {
      currentStatus.value = null
    }
  }

  return {
    budgets,
    currentDetail,
    currentStatus,
    loading,
    error,
    loadBudgets,
    loadDetail,
    loadStatus,
    create,
    update,
    remove,
  }
})
