import type { AccountDto } from '../types/api'

export interface GridItem {
  account: AccountDto | null
  isPlaceholder: boolean
  hasChildren: boolean
}

export interface GridRow {
  items: GridItem[]
  depth: number
  expandedIndex: number | null
  expandedAccountId: number | null
  parentRowIndex: number | null
}

export interface RowNode {
  row: GridRow
  children: RowNode[]
}

export function compileRows(
  roots: AccountDto[],
  expandedPath: number[],
  columns: number,
  getChildren: (id: number) => AccountDto[]
): GridRow[] {
  const rows: GridRow[] = []
  const pathSet = new Set(expandedPath)

  type Level = { accounts: AccountDto[]; index: number }
  const levels: Level[] = [{ accounts: roots, index: 0 }]
  const parentStack: (number | null)[] = [null]
  let depth = 0

  while (true) {
    const level = levels[depth]

    if (level.index >= level.accounts.length) {
      if (depth === 0) break
      depth--
      continue
    }

    const rowItems: GridItem[] = []
    let expandedIndex: number | null = null
    let expandedAccountId: number | null = null

    for (let i = 0; i < columns; i++) {
      const account = level.accounts[level.index]
      if (!account) {
        for (let j = i; j < columns; j++) {
          rowItems.push({ account: null, isPlaceholder: true, hasChildren: false })
        }
        break
      }

      const accountHasChildren = getChildren(account.id).length > 0
      rowItems.push({ account, isPlaceholder: false, hasChildren: accountHasChildren })
      level.index++

      if (pathSet.has(account.id) && accountHasChildren) {
        expandedIndex = i
        expandedAccountId = account.id
      }
    }

    rows.push({
      items: rowItems,
      depth,
      expandedIndex,
      expandedAccountId,
      parentRowIndex: parentStack[depth],
    })

    if (expandedAccountId !== null) {
      depth++
      levels[depth] = { accounts: getChildren(expandedAccountId), index: 0 }
      parentStack[depth] = rows.length - 1
      continue
    }
  }

  return rows
}

export function buildRowTree(rows: GridRow[]): RowNode[] {
  const roots: RowNode[] = []
  const stack: RowNode[] = []

  for (const row of rows) {
    while (stack.length > 0 && stack[stack.length - 1].row.depth >= row.depth) {
      stack.pop()
    }

    const node: RowNode = { row, children: [] }
    if (stack.length === 0) {
      roots.push(node)
    } else {
      stack[stack.length - 1].children.push(node)
    }

    stack.push(node)
  }

  return roots
}
