# 账户视图扁平网格化设计

## 背景

当前 `accounting-web` 的账户视图（`AccountsView.vue`）使用递归组件 `AccountNode.vue` 渲染账户树：点击有子账户的卡片后，子卡片以内嵌缩进的形式出现在父卡片下方。这种布局在层级较深时横向空间浪费明显，且嵌套结构难以快速扫视。

用户希望参考 `.ignore/show-layout/src/main.rs` 中的 `compile` 函数，将展开的账户树结构渲染成**扁平的卡片网格**，同时保留按账户类型分组。

## 目标

- 删除递归嵌套卡片树，改为单一扁平网格布局。
- 严格遵循 `compile` 函数的计算逻辑：按固定列数生成行，空位用占位卡片补齐，展开某个卡片后其子账户在下方新行中继续按同样规则渲染。
- 列数根据容器宽度响应式变化（CSS 容器查询）。
- 保留现有交互：点击账户打开抽屉、根账户不可编辑、系统账户不可删除等。

## 设计决策

### 1. 状态模型

用 `expandedPath: number[]` 替代原来的 `expandedAccountIds: Set<number>`。

- 数组元素从该类型的根子账户到当前最深被展开节点依次排列。
- 单路径展开：同一时刻只保留一条展开路径。
- 点击路径中的某个祖先节点会截断路径到该节点；点击无子节点的账户只打开抽屉，不修改路径。

### 2. 行计算算法

为每个账户类型区块独立计算一个 `GridRow[]`，算法严格对应 `compile`：

- 维护一个“迭代器栈”，每一层深度对应一个账户迭代器。
- 每次从当前深度的迭代器取出最多 `columns` 个账户生成一行。
- 若当前行存在被展开的账户，则将其子账户迭代器压入栈并进入下一深度。
- 若当前深度迭代器耗尽：
  - 位于深度 0 且无输出项时结束该类型的计算；
  - 位于更深深度时弹出栈回到上一层；
  - 位于更深深度但在本行已有输出项时，用占位卡片补齐本行剩余列，然后回到上一层。
- 空位生成占位 `GridItem`，确保网格列对齐。

### 3. 响应式列数

使用 CSS 容器查询在 `.type-section` 上设置 `--grid-columns`：

- 默认 `2` 列；
- 容器宽度 ≥ 600px 时为 `3` 列；
- 容器宽度 ≥ 900px 时为 `4` 列。

Vue 组件通过 `ResizeObserver` 读取计算后的 `--grid-columns` 值，作为行生成算法的 `columns` 参数。容器查询保证响应基于实际容器宽度，而非视口宽度。

### 4. 组件结构

- **`AccountsView.vue`**：加载账户、管理 `expandedPath`、计算每个类型的 `GridRow[]`、渲染 `AccountGrid`、处理抽屉事件。
- **`AccountGrid.vue`**（新增）：接收一个类型的 `GridRow[]`，渲染类型标题、网格容器、占位卡片和子账户分组容器。
- **`AccountCard.vue`**（新增/由 `AccountNode` 改造）：纯展示组件，渲染普通卡片或占位卡片；普通卡片显示名称、展开指示器、选中/关闭态。
- `AccountNode.vue` 的递归嵌套逻辑由 `AccountGrid.vue` 与 `AccountCard.vue` 取代；实现完成后删除该文件，或仅保留无递归的卡片展示部分。

### 5. 视觉样式

- 网格使用 `display: grid`，列数由 `--grid-columns` 决定。
- 被展开的卡片使用 `.selected` 主题色背景。
- 同一父账户的所有后代行被包裹在一个带左边框和浅背景的容器内；容器宽度为全宽，内部仍使用与外层完全一致的 `--grid-columns` 网格，因此不同级别的卡片在行、列上完全对齐。
- 已关闭账户保持 `opacity: 0.5` 与删除线。
- 占位卡片透明无边框，仅占据列位置。

### 6. 交互与边界

- 点击根账户：不打开抽屉，仅切换展开路径。
- 点击非根、无子账户的账户：打开抽屉，路径不变。
- 点击非根、有子账户的账户：打开抽屉并展开路径。
- 删除账户后，从 `expandedPath` 中移除对应 ID 并关闭抽屉。
- 列数变化时重新计算行，保持当前 `expandedPath` 不变。

## 数据类型

```ts
interface GridItem {
  account: AccountDto | null
  isPlaceholder: boolean
}

interface GridRow {
  items: GridItem[]
  depth: number
  expandedIndex: number | null
  expandedAccountId: number | null
}
```

## 测试要点

- 无展开、单路径展开、深层展开时的行生成结果。
- 列数变化（2/3/4 列）时占位卡片数量是否正确。
- 同一行中存在多个可展开账户时，仅最后一个被展开的子树先渲染（与 `compile` 一致）。
- 删除/重命名账户后 `expandedPath` 与 UI 状态同步。

## 参考实现

`.ignore/show-layout/src/main.rs` 中的 `compile` 函数如下：

```rust
fn compile(input: &str, width: usize) -> String {
    let mut levels = input.split(';').map(|s| s.split(',')).collect::<Vec<_>>();
    let mut depth = 0;
    let mut ans = String::new();

    'outer: loop {
        let mut pointer = None;
        let source = &mut levels[depth];

        for i in 0..width {
            let Some(mut item) = source.next() else {
                if i == 0 {
                    if depth == 0 {
                        return ans;
                    }
                    depth -= 1;
                    continue 'outer;
                }
                for _ in i..width {
                    write!(ans, ",").unwrap()
                }
                break;
            };

            if let Some(item_) = item.strip_prefix('+') {
                pointer = Some(i);
                item = item_;
                depth += 1
            }

            write!(ans, "{item},").unwrap()
        }

        writeln!(ans).unwrap();
        if let Some(pointer) = pointer {
            for _ in 0..pointer {
                write!(ans, "_").unwrap()
            }
            write!(ans, "^").unwrap();
            for _ in pointer + 1..width {
                write!(ans, "_").unwrap()
            }
            writeln!(ans).unwrap()
        }
    }
}
```

该函数的核心行为：

1. 输入按 `;` 分成若干层，每层按 `,` 分成若干节点；节点前的 `+` 表示该节点有下一层子节点。
2. 每次从当前层取最多 `width` 个节点输出一行。
3. 若当前行有节点被标记为 `+`，则记录其列位置并进入下一层。
4. 当前层耗尽时：若已在最顶层且无输出则结束；否则回到上一层；若本行已有输出则用逗号补齐剩余列。
5. 行与行之间若存在 `+` 节点，输出一行 `__^__` 形式的指针行，标识哪一列被展开。

## 效果示例

以下示例假设网格宽度为 `3`，`+` 表示该账户有子账户，`.` 表示占位空位。

### 示例 1：单层展开

输入：`"A,+B,C;BA,BB"`

`compile` 输出：

```
A,B,C,
_^_
BA,BB,,
```

对应账户树：

```
A   B*  C
    |
   BA  BB
```

UI 效果（字符画）：

```
+-----+-----+-----+
|  A  |  B* |  C  |    ← 根层，B 被展开
+-----+-----+-----+
| [BA]| [BB]|     |    ← B 的子账户行，与根层同列对齐
+-----+-----+-----+
```

### 示例 2：多层深度展开

输入：`"A,B,+C;CA,+CB;CBA,CBB"`

`compile` 输出：

```
A,B,C,
__^
CA,CB,
_^
CBA,CBB,
```

对应账户树：

```
A   B   C*
        |
       CA  CB*
            |
           CBA  CBB
```

UI 效果（字符画）：

```
+-----+-----+-----+
|  A  |  B  |  C* |    ← 根层
+-----+-----+-----+
| [CA]| [CB*]|    |    ← C 的子账户行
+-----+-----+-----+
| [CBA]| [CBB]|   |    ← CB 的子账户行
+-----+-----+-----+
```

### 示例 3：深度优先与兄弟节点

输入：`"A,+B,C,D;BA,BB"`

`compile` 输出：

```
A,B,C,
_^_
BA,BB,,
D,,,
```

对应账户树：

```
A   B*   C   D
    |
   BA  BB
```

UI 效果（字符画）：

```
+-----+-----+-----+
|  A  |  B* |  C  |    ← 根层
+-----+-----+-----+
| [BA]| [BB]|     |    ← B 的子账户行（右侧空位占位）
+-----+-----+-----+
|  D  |     |     |    ← 根层剩余兄弟节点
+-----+-----+-----+
```

该示例体现 `compile` 的深度优先特性：B 的整个子树渲染完毕后，才继续渲染根层的剩余兄弟 D。

## 风险与注意事项

- 容器查询在现代浏览器中已广泛支持，项目目标 `es2023` 与 Vite 8 无兼容问题。
- 行计算依赖 DOM 宽度，需要在组件挂载及容器尺寸变化时重新计算；注意避免在 SSR 或测试环境中读取 `getComputedStyle` 报错。
- 严格遵循 `compile` 意味着子账户行同样从第 0 列开始填充，右侧不满留空，不强行把子账户放在父账户正下方；但所有级别的行共享同一套列网格，因此在行、列上是完全对齐的。视觉关联通过被展开卡片的高亮及其后代行的分组容器实现。
