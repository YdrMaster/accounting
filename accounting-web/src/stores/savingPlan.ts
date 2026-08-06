import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createSavingPlan,
  deleteSavingPlan,
  fetchSavingPlans,
  fetchSavingPlanStatus,
  fetchSavingPlanStatuses,
  updateSavingPlan,
} from '../api/client'
import type {
  CreateSavingPlanRequest,
  SavingPlanDto,
  SavingPlanStatusDto,
  UpdateSavingPlanRequest,
} from '../types/api'

export const useSavingPlanStore = defineStore('savingPlan', () => {
  const plans = ref<SavingPlanDto[]>([])
  const statuses = ref<SavingPlanStatusDto[]>([])
  const currentStatus = ref<SavingPlanStatusDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadPlans() {
    loading.value = true
    error.value = null
    try {
      plans.value = await fetchSavingPlans()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadStatuses(date?: string) {
    loading.value = true
    error.value = null
    try {
      statuses.value = await fetchSavingPlanStatuses(date)
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
      currentStatus.value = await fetchSavingPlanStatus(id, date)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(data: CreateSavingPlanRequest): Promise<SavingPlanDto> {
    const plan = await createSavingPlan(data)
    plans.value.push(plan)
    return plan
  }

  async function update(id: number, data: UpdateSavingPlanRequest): Promise<void> {
    await updateSavingPlan(id, data)
    await loadPlans()
  }

  async function remove(id: number): Promise<void> {
    await deleteSavingPlan(id)
    plans.value = plans.value.filter(p => p.id !== id)
    statuses.value = statuses.value.filter(s => s.plan.id !== id)
    if (currentStatus.value?.plan.id === id) {
      currentStatus.value = null
    }
  }

  return {
    plans,
    statuses,
    currentStatus,
    loading,
    error,
    loadPlans,
    loadStatuses,
    loadStatus,
    create,
    update,
    remove,
  }
})
