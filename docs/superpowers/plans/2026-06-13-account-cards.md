> 注：本文档包含已废弃的 `Liability` / `负债` 账户类型引用，仅供参考。当前系统仅保留 `Asset`、`Equity`、`Income`、`Expense` 四种类型。
>
> 相关变更见：`docs/superpowers/specs/2026-06-22-remove-liability-design.md`

# 账户卡片重设计实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将账户页面从窄树改为 tabs + 卡片手风琴，支持重命名/关闭/重开/拖拽排序，新增 position 字段。

**架构：** accounts 表加 `position` 列，repo 按 position 排序 + reorder 方法，API 新增 rename/close/reopen/reorder 端点，前端整体替换 AccountTree.vue 为 tabs + flex-wrap 卡片 + 手风琴 + vuedraggable。

**技术栈：** Rust (axum, rusqlite), Vue 3 + AntDV + Pinia + vuedraggable

**提交前必须：** `cargo fmt && cargo clippy --workspace && cargo test --workspace`

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `accounting-sql/src/schema.rs` | accounts 表新增 position INTEGER NOT NULL DEFAULT 0 |
| `accounting/src/account.rs` | Account 结构体新增 position |
| `accounting-sql/src/repo/account.rs` | 查询按 position 排序；insert 默认 max+1；新增 reorder 方法 |
| `accounting-api/src/dto.rs` | AccountDto 新增 position；新增 ReorderRequest |
| `accounting-api/src/handlers/account.rs` | 新增 close/reopen/rename/reorder 端点 |
| `accounting-web/src/stores/account.ts` | Account 接口加 position/cloned_at；新增 reorderAccounts/renameAccount/closeAccount/reopenAccount |
| `accounting-web/src/views/AccountTree.vue` | 整体重写：tabs + 卡片 + 手风琴 + 拖拽 + 详情面板 |

---

### 任务 1：Schema 与领域模型 — position 字段

**文件：**
- 修改：`accounting-sql/src/schema.rs`
- 修改：`accounting/src/account.rs`
- 修改：`accounting-api/src/dto.rs`

---

- [ ] **步骤 1：在 SCHEMA_SQL 中为 accounts 表添加 position 列**

编辑 `accounting-sql/src/schema.rs`，在 `CREATE TABLE accounts` 的 `repayment_day` 后新增一行：

```sql
position INTEGER NOT NULL DEFAULT 0,
```

- [ ] **步骤 2：在 Account 结构体添加 position 字段**

编辑 `accounting/src/account.rs`，在结构体中新增：

```rust
/// 排序位置（同级内排序）
pub position: i64,
```

- [ ] **步骤 3：在 AccountDto 添加 position 字段**

编辑 `accounting-api/src/dto.rs` 的 `AccountDto`：

```rust
pub struct AccountDto {
    pub id: i64,
    pub full_name: String,
    pub account_type: String,
    pub parent_id: Option<i64>,
    pub closed_at: Option<String>,
    pub is_system: bool,
    pub billing_day: Option<u8>,
    pub repayment_day: Option<u8>,
    pub position: i64,
    pub owner_ids: Vec<i64>,
}
```

- [ ] **步骤 4：在 handler 中映射 position**

编辑 `accounting-api/src/handlers/account.rs` 的 `list_accounts` 函数，在 `AccountDto` 构造中添加：

```rust
position: account.position,
```

- [ ] **步骤 5：编译验证并 commit**

```bash
cd /home/mechdancer/repos/accounting && cargo fmt && cargo check 2>&1 | tail -5
```

因 repo 层还未适配 position，可能有编译错误（后续任务修复），但 `accounting` crate 自身应编译通过。

```bash
git add accounting-sql/src/schema.rs accounting/src/account.rs accounting-api/src/dto.rs accounting-api/src/handlers/account.rs
git commit -m "feat: accounts 表新增 position 字段"
```

---

### 任务 2：AccountRepo — 排序查询 + reorder

**文件：**
- 修改：`accounting-sql/src/repo/account.rs`

---

- [ ] **步骤 1：更新 insert 方法自动计算 position**

将 INSERT SQL 改为包含 position 列：

```sql
INSERT INTO accounts (full_name, account_type, parent_id, is_system, billing_day, repayment_day, position)
SELECT ?1, ?2, ?3, ?4, ?5, ?6, COALESCE(MAX(position), -1) + 1 FROM accounts WHERE parent_id IS ?3
```

`?3` 是 `parent_id`（同一父账户下 position 递增）。

- [ ] **步骤 2：更新 create 和所有 SELECT 查询按 position 排序**

在所有 `list`、`list_all` 等查询末尾追加 `ORDER BY position`。

- [ ] **步骤 3：新增 reorder 方法**

在 `AccountRepo` trait 中新增：

```rust
/// 更新账户排序位置
fn reorder(&self, conn: &Connection, ids: &[AccountId]) -> Result<(), crate::error::DbError>;
```

SQLite 实现：

```rust
fn reorder(&self, conn: &Connection, ids: &[AccountId]) -> Result<(), crate::error::DbError> {
    for (i, id) in ids.iter().enumerate() {
        conn.execute("UPDATE accounts SET position = ?1 WHERE id = ?2", params![i as i64, id.0])?;
    }
    Ok(())
}
```

- [ ] **步骤 4：更新测试 helper 添加 position**

在测试的 `sample_account` 等辅助函数中添加 `position: 0`。

- [ ] **步骤 5：运行测试并 commit**

```bash
cd /home/mechdancer/repos/accounting && cargo fmt && cargo test -p accounting-sql --lib repo::account 2>&1 | tail -10
git add accounting-sql/src/repo/account.rs
git commit -m "feat: AccountRepo 按 position 排序, 新增 reorder 方法"
```

---

### 任务 3：Account API — 重命名/关闭/重开/重排

**文件：**
- 修改：`accounting-api/src/dto.rs`
- 修改：`accounting-api/src/handlers/account.rs`

---

- [ ] **步骤 1：新增 DTO**

在 `accounting-api/src/dto.rs` 中添加：

```rust
/// 重命名账户请求
#[derive(Deserialize)]
pub struct RenameAccountRequest {
    pub full_name: String,
}

/// 账户排序请求
#[derive(Deserialize)]
pub struct ReorderRequest {
    pub ids: Vec<i64>,
}
```

- [ ] **步骤 2：新增 handler — 重命名**

```rust
async fn rename_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<RenameAccountRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let conn = db.connection();
    // 检查同名
    let accounts = db.account_repo().list(&conn).map_err(|e| e.to_string())?;
    let target = accounts.iter().find(|a| a.id.0 == id).ok_or("账户不存在")?;
    let siblings = accounts.iter().filter(|a| a.parent_id == target.parent_id);
    if siblings.any(|a| a.id.0 != id && a.full_name == req.full_name) {
        return Err("同名账户已存在".to_string());
    }
    db.account_repo().rename(&conn, AccountId(id), &req.full_name).map_err(|e| e.to_string())?;
    Ok("renamed".to_string())
}
```

- [ ] **步骤 3：新增 handler — 关闭/重开**

```rust
async fn close_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    db.account_repo().close(&db.connection(), AccountId(id)).map_err(|e| e.to_string())?;
    Ok("closed".to_string())
}
```

`close` 方法复用现有的（或新增，取决于 repo 实现）。

- [ ] **步骤 4：新增 handler — reorder**

```rust
async fn reorder_accounts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReorderRequest>,
) -> Result<String, String> {
    let db = state.db().map_err(|e| e.to_string())?;
    let ids: Vec<AccountId> = req.ids.into_iter().map(AccountId).collect();
    db.account_repo().reorder(&db.connection(), &ids).map_err(|e| e.to_string())?;
    Ok("reordered".to_string())
}
```

- [ ] **步骤 5：注册路由**

```rust
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route("/api/accounts/:id/balance", get(get_balance))
        .route("/api/accounts/:id/owner", put(set_owner))
        .route("/api/accounts/:id/rename", put(rename_account))
        .route("/api/accounts/:id/close", put(close_account))
        .route("/api/accounts/:id/open", put(reopen_account))
        .route("/api/accounts/reorder", put(reorder_accounts))
}
```

- [ ] **步骤 6：运行测试并 commit**

```bash
cd /home/mechdancer/repos/accounting && cargo fmt && cargo clippy --workspace && cargo test --workspace 2>&1 | grep "test result"
git add accounting-api/src/dto.rs accounting-api/src/handlers/account.rs
git commit -m "feat: 新增 rename/close/reopen/reorder 账户 API 端点"
```

---

### 任务 4：前端 Store 更新

**文件：**
- 修改：`accounting-web/src/stores/account.ts`
- 创建：`accounting-web/src/stores/commodity.ts`（如不存在）

---

- [ ] **步骤 1：更新 Account 接口**

```typescript
export interface Account {
  id: number
  full_name: string
  account_type: string
  parent_id?: number
  closed_at?: string
  is_system: boolean
  billing_day?: number
  repayment_day?: number
  position: number
  owner_ids?: number[]
}
```

- [ ] **步骤 2：新增方法**

```typescript
async function renameAccount(id: number, fullName: string) {
  await api.put(`/accounts/${id}/rename`, { full_name: fullName })
  await fetchAccounts()
}

async function closeAccount(id: number) {
  await api.put(`/accounts/${id}/close`)
  await fetchAccounts()
}

async function reopenAccount(id: number) {
  await api.put(`/accounts/${id}/open`)
  await fetchAccounts()
}

async function reorderAccounts(ids: number[]) {
  await api.put('/accounts/reorder', { ids })
  await fetchAccounts()
}
```

- [ ] **步骤 3：验证并 commit**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit 2>&1
git add accounting-web/src/stores/account.ts
git commit -m "feat: 前端 Account Store 新增 rename/close/reopen/reorder"
```

---

### 任务 5：安装 vuedraggable 依赖

**文件：**
- 修改：`accounting-web/package.json`

---

- [ ] **步骤 1：安装 vuedraggable**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npm install vuedraggable@next 2>&1 | tail -3
```

- [ ] **步骤 2：确认安装并 commit**

```bash
git add accounting-web/package.json accounting-web/package-lock.json
git commit -m "chore: 安装 vuedraggable 依赖"
```

---

### 任务 6：AccountTree.vue — 整体重写

**文件：**
- 修改：`accounting-web/src/views/AccountTree.vue`

---

将 `AccountTree.vue` 完全重写。关键结构：

- [ ] **步骤 1：模板 — Tab 切换**

```vue
<a-tabs v-model:active-key="activeTab" @change="handleTabChange">
  <a-tab-pane key="Asset" tab="资产" />
  <a-tab-pane key="Liability" tab="负债" />
  <a-tab-pane key="Income" tab="收入" />
  <a-tab-pane key="Expense" tab="支出" />
  <a-tab-pane key="Equity" tab="权益" />
</a-tabs>
```

- [ ] **步骤 2：模板 — 递归卡片组件**

在 `CardGrid.vue` 中（或同一文件内用递归组件）：

```vue
<draggable v-model="sortedAccounts" item-key="id" @end="onDragEnd" handle=".drag-handle">
  <template #item="{ element }">
    <div class="account-card" :class="{ selected: selectedId === element.id, expanded: expandedId === element.id }">
      <div class="card-header" @click="selectAccount(element)">
        <span class="drag-handle">⠿</span>
        <span class="card-name">{{ leafName(element.full_name) }}</span>
        <span v-if="hasChildren(element)" class="toggle-btn" @click.stop="toggleExpand(element)">
          {{ expandedId === element.id ? '▲' : '▼' }}
        </span>
        <span v-else class="add-btn" @click.stop="addChild(element)">+</span>
      </div>
      <div class="card-sub">
        <span>{{ childCount(element) }} 个子账户</span>
      </div>
      <div v-if="expandedId === element.id" class="sub-cards">
        <AccountCards :parent-id="element.id" :type="activeTab" />
        <div v-if="addingToId === element.id" class="add-row">
          <a-input v-model:value="newName" placeholder="输入名称" size="small" ref="addInput" />
          <a-button size="small" @click="confirmAdd(element)">确认</a-button>
          <a-button size="small" @click="cancelAdd">取消</a-button>
        </div>
      </div>
    </div>
  </template>
</draggable>
```

- [ ] **步骤 3：模板 — 顶层也添加 AddCard**

在 card list 末尾添加：

```vue
<div v-if="addingToRoot" class="add-card" @click="startRootAdd">
  <PlusOutlined /> 添加子账户
</div>
```

- [ ] **步骤 4：模板 — 详情面板**

```vue
<div v-if="selectedAccount" class="detail-panel">
  <h3>{{ selectedAccount.full_name }}</h3>
  <div v-if="editingName">
    <a-input v-model:value="editName" />
    <a-button @click="confirmRename">确认</a-button>
    <a-button @click="editingName = false">取消</a-button>
  </div>
  <p>类型: {{ selectedAccount.account_type }}</p>
  <div v-if="selectedAccount.account_type === 'Asset'">
    <a-checkbox-group :value="selectedAccount.owner_ids" :options="memberOptions" @change="updateOwners" />
  </div>
  <a-space>
    <a-button size="small" @click="startRename">重命名</a-button>
    <a-button v-if="!selectedAccount.closed_at" size="small" @click="handleClose">关闭</a-button>
    <a-button v-else size="small" @click="handleReopen">重新开启</a-button>
  </a-space>
</div>
```

- [ ] **步骤 5：Script — 核心状态**

```typescript
const activeTab = ref('Asset')
const selectedId = ref<number | null>(null)
const expandedId = ref<number | null>(null)
const addingToId = ref<number | null>(null)
const addingToRoot = ref(false)
const editingName = ref(false)
const editName = ref('')
const newName = ref('')
```

- [ ] **步骤 6：Script — 计算属性**

```typescript
const accounts = computed(() => accountStore.accounts)
const filteredAccounts = computed(() =>
  accounts.value.filter(a => a.account_type === activeTab.value && a.parent_id == null)
    .sort((a, b) => a.position - b.position)
)
const selectedAccount = computed(() =>
  selectedId.value ? accounts.value.find(a => a.id === selectedId.value) || null : null
)
```

- [ ] **步骤 7：Script — 拖拽结束提交**

```typescript
function onDragEnd() {
  const ids = sortedAccounts.value.map(a => a.id)
  accountStore.reorderAccounts(ids)
}
```

- [ ] **步骤 8：Script — 添加/重命名逻辑**

新建时校验同层级无同名；重命名同样校验。

- [ ] **步骤 9：样式**

卡片网格 flex-wrap，`gap: 12px`；选中态蓝色边框；子账户区域缩进 `padding-left: 24px`；拖拽 handle 等。

- [ ] **步骤 10：运行验证并 commit**

```bash
cd /home/mechdancer/repos/accounting/accounting-web && npx vue-tsc --noEmit && npm run build 2>&1 | tail -2
cd .. && cargo fmt && cargo clippy --workspace && cargo test --workspace 2>&1 | grep "test result"
git add accounting-web/src/views/AccountTree.vue
git commit -m "feat: 账户页面改为 tabs + 卡片手风琴 + 拖拽排序"
```

---

## 验证命令

```bash
cd /home/mechdancer/repos/accounting
cargo fmt
cargo clippy --workspace
cargo test --workspace
cd accounting-web && npx vue-tsc --noEmit && npm run build
```
