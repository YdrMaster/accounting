import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '@/views/Dashboard.vue'
import TransactionForm from '@/views/TransactionForm.vue'
import AccountTree from '@/views/AccountTree.vue'
import Members from '@/views/Members.vue'
import Tags from '@/views/Tags.vue'
import ReportView from '@/views/ReportView.vue'

const routes = [
  { path: '/', component: Dashboard },
  { path: '/transaction', component: TransactionForm },
  { path: '/accounts', component: AccountTree },
  { path: '/members', component: Members },
  { path: '/tags', component: Tags },
  { path: '/reports', component: ReportView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
