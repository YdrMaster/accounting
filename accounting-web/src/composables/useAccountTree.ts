import { computed } from 'vue'
import { useAccountStore } from '@/stores/account'

export function useAccountTree() {
  const accountStore = useAccountStore()

  const accountTreeData = computed(() => {
    const accounts = accountStore.accounts
    const map = new Map<number, any>()
    const roots: any[] = []

    accounts.forEach((acc) => {
      const segments = acc.full_name.split(':')
      map.set(acc.id, {
        title: segments[segments.length - 1],
        key: acc.id,
        children: [],
      })
    })

    accounts.forEach((acc) => {
      const node = map.get(acc.id)
      if (!node) return
      if (acc.parent_id && map.has(acc.parent_id)) {
        map.get(acc.parent_id)!.children.push(node)
      } else {
        roots.push(node)
      }
    })

    return roots
  })

  const assetAccountTreeData = computed(() => {
    const assetIds = new Set(
      accountStore.accounts.filter(a => a.account_type === 'Asset').map(a => a.id)
    )
    function filterTree(nodes: any[]): any[] {
      return nodes
        .map(n => ({ ...n, children: filterTree(n.children || []) }))
        .filter(n => assetIds.has(n.key) || n.children.length > 0)
    }
    return filterTree(accountTreeData.value)
  })

  return { accountTreeData, assetAccountTreeData }
}
