# View：执行轨迹与文档面保真

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-10-15 |
| 相关 | [view-document-annotations.md](view-document-annotations.md) · [view-frame-selection.md](view-frame-selection.md) · [view.md](view.md) · [view-markup-surface.md](view-markup-surface.md) |

---

## 1. 目标

`marqdo view` Browse Structure 必须同时做到：

1. **标记面**：变量名 / 可执行语句 / 返回值按表面渲染（无源码定界符字面量）。
2. **结构块**：GFM 表、循环（`-`）、分支（`1.`）有正确 HTML 与缩进。
3. **语句后值**：赋值行标注该语句执行后的值，不用入口终态回填。
4. **执行臂**：高亮实际走的分支臂。
5. **多调用帧**：同函数多次调用可用下拉切换经历。
6. **AI 可读**：与 HTML 同源的 `ViewExecutionReport` JSON。

Debug 仍用 AST 卡片 + 断点；不做文档 callout。

## 2. 标记面

| 源标记 | 显示（无定界符） | class |
|--------|------------------|-------|
| `` `名` `` / `` `名`=默认 `` | `名`（默认次要后缀） | `mq-anchor mq-decl` |
| `**…**`（可执行） | 内层文本 | `mq-anchor mq-code` |
| `*…*`（返回） | 内层文本 | `mq-anchor mq-return` |

`data-byte-start/end` 仍为源码字节偏移（供 CSS L-leader）。软强调不进 anchor。

## 3. 结构块

| 块 | 呈现 |
|----|------|
| GFM 表 | `<table class="doc-table">` |
| `-` 循环 | `<ul class="doc-loop">`，体按源缩进 `--doc-indent` |
| `1.` 分支 | `<ol class="doc-branch">`；命中 `arm-taken`，未走 `arm-miss` |
| 公式 / 外联代码 | 现有 formula-card / code-card |
| `>` 调用 | 表面高亮，非整行灰 code |

## 4. StmtEvent / CallFrame

解释器 capture 时静默采集（非 `trace_eval` stderr）：

```text
StmtEvent {
  line, kind,           // assign | return | call | branch | loop | bind_ref
  name?, value_after?,  // post-stmt 或求值结果
  arm_index?, iter?,
  call_id               // 当前用户函数调用帧
}

CallFrame {
  call_id, fn_name, fn_line, ordinal,
  args_brief,           // 短摘要供下拉
  events[]              // 或全局 events 用 call_id 过滤
}
```

- Assign → 写入后的值挂在**该行**。
- 循环内多次写入：UI 默认末次；可带次数。
- Branch → taken `arm_index` + 条件摘要。
- 进入用户函数分配 `call_id`；`# main` 单帧无下拉。

## 5. ViewExecutionReport（AI）

```json
{
  "version": 1,
  "path": "…",
  "ok": true,
  "calls": [{ "call_id": 1, "fn": "等级说明", "ordinal": 1, "args_brief": "等级=及格" }],
  "events": [{ "line": 24, "kind": "assign", "name": "总分", "value_after": "0", "call_id": 0 }],
  "bindings_final": { "均分": "73", "等级": "及格" }
}
```

产出：页内 `#mq-view-trace` JSON；live `GET /api/view-trace?path=`；SSE `done.trace_json`。

## 6. UI

- 函数标题旁 `<select class="call-picker">`（调用数 ≥ 2）。
- 循环头旁 `<select class="loop-picker">`（迭代数 ≥ 2）；默认最后一轮；摘要 `分 = 分数表[i] · value`。
- **源码常显**：切换帧只改 chip / 臂高亮，不 `hidden` 结构节点。详见 [view-frame-selection.md](view-frame-selection.md)。
- CSS L-leader（非 SVG）；拥挤 keyed 降级。

## 7. 验收

1. 无字面 `**` / 返回定界 `*` / 变量外层 `` ` ``（Source 除外）。
2. 表为真表格；循环/分支有缩进列表；未选中臂源码仍可见。
3. `总分 = 0` 标注 `0`，非终态累加。
4. 命中臂可视化；多调用下只显示当前帧的 taken。
5. 循环可按迭代切换；JSON 含 `loops`；单测覆盖。

## 8. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-10-15 | 初稿：标记面、结构块、StmtEvent、CallFrame、AI 报告 |
| 2026-10-15 | 帧选择保真：call_id 分支、LoopFrame、源码常显 |
