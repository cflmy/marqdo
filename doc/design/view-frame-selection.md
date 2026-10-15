# View：调用帧 / 循环帧选择与结构保真

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-10-15 |
| 相关 | [view-execution-trace.md](view-execution-trace.md) · [view-document-annotations.md](view-document-annotations.md) |

---

## 1. 问题（对照实测）

1. **结构缺失**：切换函数调用下拉时，JS 按 `data-call-id` **隐藏** `.doc-exec`。未在本帧执行的分支臂体（赋值/返回）只有其它帧的 call_id，或仅有执行帧的 id → **未选中臂的源码被藏掉**，看起来像「臂消失 / 代码没渲染」。
2. **调用记录混叠**：`Branch` 标注未带 `call_id`；`taken_arms` 按行号只保留最后一次命中。多帧下同一分支同时出现 `arm 1 taken` 与 `arm 2 taken`。
3. **循环无帧**：循环只有总次数摘要；无法按迭代查看；`` - [分](分数表) `` 不能展示「当前是集合第几个元素」。

## 2. 定稿原则

| 原则 | 含义 |
|------|------|
| **源码常显** | 表 / 列表 / 分支臂 / 语句源码**永不**因帧选择 `hidden` |
| **轨迹随帧** | 标注 chip、臂高亮、变量值随「当前调用帧 / 当前迭代」变化 |
| **默认末帧** | 函数默认最后一次调用；循环默认最后一次迭代 |
| **一帧一真相** | 同时只展示一个 call_id（函数）或一个 (loop_id, iter)（循环）的轨迹 |

## 3. 函数调用帧

### 数据

`BranchHit` / 相关 `StmtEvent` 必须带 `call_id`。`ViewAnnot` 的 branch/bind/return 同样带 `call_id`。

### UI

- 函数标题旁 `<select class="call-picker" data-fn="…" data-scope="fn">`
- 选项：`1: 等级=及格` / `2: 等级=优秀`
- 默认选中最后一次

### 客户端（`applyCallFrame`）

1. 读 `#mq-view-trace` JSON。
2. 过滤 `events` where `call_id == selected`。
3. **不**设置结构节点 `hidden`。
4. 更新该函数作用域内：
   - `.annot-chip[data-call-id]`：仅显示匹配帧；无 call_id 的结构说明类可保留
   - `.doc-branch[data-line]`：按该帧的 branch 事件设置 `arm-taken` / `arm-miss`（全部臂仍可见）
   - 绑定 chip 的 `.annot-value`：用该帧 assign/return 事件刷新（同名同行取末次）

## 4. 循环迭代帧

### 数据

```text
LoopFrame {
  loop_id, line,
  item_name?,      // "分"
  collection?,     // "分数表"
  iterations: [{ index, item_value, call_id }]  // index 0-based
}
```

`StmtEvent` 增加可选 `loop_id`；循环体内事件带 `iter`（1-based 或 0-based，JSON 内统一 **0-based index**）。

### UI

- 循环头（`- [分](分数表)`）旁：`<select class="loop-picker" data-loop-id="…">`
- 选项：`迭代 1: 分=92` … `迭代 4: 分=88`
- 默认最后一次
- 头旁摘要 chip：`分 = 分数表[3] · 88`（下标 0-based 显示为 `[i]`）

### 客户端（`applyLoopFrame`）

1. **只显示当前迭代**：带 `data-iter` 的 chip 仅保留选中项；其余 `hidden` + `is-frame-hidden`（SSR 默认已按末轮预隐藏）。
2. 更新头摘要 chip（无 `data-iter`）为当前选项文案。
3. 不隐藏循环体源码。

## 5. 分支臂渲染

- 所有臂（含 else）**始终**渲染完整头 + 体（含 `*返回*` / 赋值）。
- `arm-miss` 仅降低不透明度，**不** `display:none` / `hidden`。
- 高亮仅反映**当前帧**的 taken `arm_index`。

## 6. AI JSON 增量

`ViewExecutionReport` 增加：

```json
"loops": [{
  "loop_id": 1,
  "line": 27,
  "item": "分",
  "collection": "分数表",
  "iters": [{ "index": 0, "value": "92" }, { "index": 3, "value": "88" }]
}]
```

`events[].call_id` 必填；循环体内事件带 `loop_id` + `iter`（index）。

## 7. 验收

1. 未选中分支臂源码始终可见（含返回句）。
2. 切换「等级说明」调用：只显示该帧的 arm taken；变量/返回值对应该帧。
3. 循环下拉默认最后一轮；`分 = 分数表[3] · 88`；切换迭代后体内累加值变化。
4. `#mq-view-trace` 含 `loops` 与带 `call_id` 的 branch 事件。
5. `cargo test --lib view::` 全绿。

## 8. 开发计划（已执行）

| 阶段 | 内容 | 状态 |
|------|------|------|
| D0 | 本文档 + `view-execution-trace.md` 交叉引用 | ✅ |
| D1 | `BranchHit.call_id`；`LoopFrame` / `LoopIter`；`StmtEvent.loop_id` | ✅ |
| D2 | 解释器：foreach/while 推栈；BindRef `分=分数表[i]`；assign 带 iter | ✅ |
| D3 | `build_view_annots`：分支带 call_id；LoopFrame 选项；报告 JSON `loops` | ✅ |
| D4 | 文档面：源码不贴 `data-call-ids`；call picker `value=call_id`；loop picker；SSR taken=末帧 | ✅ |
| D5 | JS：`applyCallFrame` / `applyLoopFrame` — 只滤 chip + 重算 arm，不 hidden 结构 | ✅ |
| D6 | 单测：多调用不同臂、loop 末项、HTML 三臂返回 + picker | ✅ |

## 9. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-10-15 | 初稿：源码常显、帧过滤、循环迭代 picker |
| 2026-10-15 | 按计划落地 D1–D6 |
