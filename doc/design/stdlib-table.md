# `lib/table` / `lib/表` — 列表与字典

| | |
|---|---|
| 状态 | **已落地**（可用级集合 API） |
| 相关 | [stdlib.md](stdlib.md) · [stdlib-modules.md](stdlib-modules.md) · [table-cell-expressions.md](table-cell-expressions.md) · [roadmap/tables-maps-footnotes.md](../roadmap/tables-maps-footnotes.md) |

---

## 1. 读写对

| 读 | 写 |
|----|----|
| 脚注 `` `xs`[^1] `` / `` `m`[^key] `` / 嵌套 `` `m`[^a][^1] `` | **`table.put` / `表.改`** |

`put` / `改` 的 `at=`（`于=`）与脚注同构：文本键、**1-based** 整数下标，或键/下标组成的路径列表。

```markdown
---
> lib/table.mq.md
---

# main

*`h` = > table.put in=None at=Authorization value=Bearer-x*

`xs` =

| v |
|---|
| a |
| b |

*`xs` = > table.put in=`xs` at=1 value=A*
```

`in=None` 且 `at=` 为单个文本键 → 新建单键字典。值一律不可变更新（返回新集合）。

---

## 2. 索引约定

| API | 基数 |
|-----|------|
| `put` / `改` 路径中的整数；脚注 `[^n]` | **1-based** |
| `at` / `row_at` / `set_at` / `insert` / `remove_at` / `slice` / `index_of` | **0-based** |

---

## 3. API 摘要（EN ↔ ZH）

| EN | ZH | 说明 |
|----|----|------|
| `put` | `改` | 按路径改一元（首选写入） |
| `len` / `rows` | `长度` / `行数` | 长度 |
| `at` / `row_at` | `取` / `取行` | 0-based 取元 |
| `append` / `prepend` / `concat` | `追加` / `前插` / `拼接` | 列表 |
| `insert` / `set_at` / `remove_at` / `pop` | `插入` / `改位` / `删位` / `弹出` | 列表位操作 |
| `first` / `last` / `slice` | `首` / `末` / `切片` | |
| `contains` / `index_of` / `reverse` | `含有` / `下标` / `反转` | |
| `clear` | `清空` | 列表或字典 |
| `get` / `set` / `delete` / `has` | `取键` / `设` / `删键` / `有键` | 浅层字典 |
| `keys` / `values` / `items` / `merge` / `size` | `键表` / `值表` / `项表` / `合并` / `键数` | |
| `empty_list` / `empty_map` | `空表` / `空字典` | |

嵌套改写优先 `put`/`改`；`set`/`set_at` 仅浅层。

---

## 4. 与 `lib/json`

| 用途 | 用 |
|------|----|
| 解析 / 序列化 / `quote` | `json.parse` · `json.stringify` · `json.quote` |
| 集合读写 | **`table.*` / `表.*`** |
| 兼容 | `json.set` / `json.append` / `json.get` / `json.keys` 仍为别名，新代码勿依赖 |

---

## 5. 宿主

`host_collection_put` + list/map 原语（见 `src/host/collection.rs`）。作者面经 `lib/table` / `lib/表` 调用，不直接写 `host_*`。
