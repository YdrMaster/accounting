import { ref } from 'vue'
import { useAccountStore } from './account'
import { useBudgetStore } from './budget'
import { useReportStore } from './report'
import { useSavingPlanStore } from './savingPlan'

/**
 * 账目数据版本号：任何账目变更后 +1，供视图层（如日历的本地统计）监听刷新。
 * 失效编排集中在本模块，业务 store 之间不互相 import。
 */
export const dataVersion = ref(0)

/**
 * 交易数据变更（增删改、导入）后的派生数据刷新：
 * 只重拉"此前已加载过"的数据域，避免多余请求；失败不抛错（降级为下次加载）。
 */
export async function notifyTransactionsChanged(): Promise<void> {
  dataVersion.value++
  const budget = useBudgetStore()
  const savingPlan = useSavingPlanStore()
  const account = useAccountStore()
  const report = useReportStore()
  const jobs: Promise<unknown>[] = [report.refresh()]
  if (budget.statusesLoaded) jobs.push(budget.loadStatuses())
  if (savingPlan.statusesLoaded) jobs.push(savingPlan.loadStatuses())
  if (account.loaded) jobs.push(account.loadAccounts())
  await Promise.allSettled(jobs)
}

/** 账户数据变更（增删改、移动）后的刷新：账户列表 + 报表。 */
export async function notifyAccountsChanged(): Promise<void> {
  dataVersion.value++
  const account = useAccountStore()
  const report = useReportStore()
  const jobs: Promise<unknown>[] = [report.refresh()]
  if (account.loaded) jobs.push(account.loadAccounts())
  await Promise.allSettled(jobs)
}
