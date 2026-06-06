import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '@/views/Dashboard.vue'
import TransactionForm from '@/views/TransactionForm.vue'
import AccountTree from '@/views/AccountTree.vue'
import Settings from '@/views/Settings.vue'
import ReportView from '@/views/ReportView.vue'

const routes = [
  { path: '/', component: Dashboard },
  { path: '/transaction', component: TransactionForm },
  { path: '/accounts', component: AccountTree },
  { path: '/settings', component: Settings },
  { path: '/reports', component: ReportView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
