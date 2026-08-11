# 父智能体 Plan-and-Move（`## plan` 行动面）

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-08-11 |
| 父文档 | [ext-agent-plan.md](ext-agent-plan.md) · [ext-agent.md](ext-agent.md) |
| 相关 | [stdlib-subtask.md](stdlib-subtask.md) · [okf.md](okf.md) · [ai-skill.md](ai-skill.md) |

---

## 1. 一句话

**父智能体 = Plan-and-Move 控制器**：观察工作簿与子任务结果 → 规划下一步 → 行动（读 / 调工具 / 补丁 / 再跑 / 停）；Marqdo「文档即代码」让工具、记忆与程序都落在 `.mq.md` 上。

---

## 2. 为何需要强化

单靠 `DECISION: DONE|CONTINUE|RUN` 时，父侧几乎没有「手」：

| 缺口 | 后果 |
|------|------|
| 看不到子源码 / 报错 / stdout | 空想 FIND/REPLACE，输出又臭又长 |
| 标准库未进入行动表 | 无法在规划环内用 `fs` / `json` / `subtask` 等 |
| 不能用 Marqdo 造工具 | 无法把可复用步骤沉淀为 `##` |
| 提示词过长且互相打架 | 模型「继续帮忙」而不停机 |

业界对 agent 回环的共识：成功标准、停机动作、**确定性行动面**三者必须写清；停机最终靠代码护栏，不只靠 prompt。

---

## 3. Marqdo 赋予的特性（相对通用 Plan-and-Move）

| 特性 | 含义 |
|------|------|
| 工作簿即程序 | 子执行物是可 `spawn path=` 的 `.mq.md`，不是隐式任务图 |
| 固化即记忆 | `DECISION: DONE` → `agent_workbook_solidify` + OKF 晋升 |
| `lib/*` 即工具 | 标准库是父可调度能力；经父 helper 或写入工作簿/`tools/` 后调用 |
| 补丁即改代码 | `CONTINUE` + 精确 FIND/REPLACE；禁止整文件 LLM 重写 |
| 文档即提示 | 站立提示、Skill 摘要、观察切片都是可读 Markdown，而非黑盒 system blob |

---

## 4. 循环形状（锁定）

```text
                 ┌──────────────┐
                 │  Plan (LLM)  │
                 └──────┬───────┘
                        │ 协议回复
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       READ:/CALL:   DECISION:     (无效 → error)
          │          DONE/RUN/
          │          CONTINUE
          ▼             ▼
       Act 工具      Move（固化/补丁/spawn）
          │             │
          └──────┬──────┘
                 ▼
         Observe（有界）
                 │
                 └──► 回到 Plan 或终止
```

优先级（解析父回复时）：

1. **`CALL:<name>` / `调用：`** — 跑父侧工具，结果并入 observation，再 Plan  
2. **`READ:<kind>` / `读取：`** — 加深某一观察切片，再 Plan  
3. **`DECISION:`** — Move（含运行时 solidify / apply_patch / spawn）

---

## 5. 观察面（有界、可加深）

`inspect_workbook` / `await_workbook` 在磁盘上可持有全文；**注入父 prompt 的默认紧凑观察**必须可行动且短：

| 字段 | 默认 | `READ:` 加深 |
|------|------|----------------|
| `source_excerpt` | 去 `<!-- … -->` 后的结构摘录，硬顶 ~4KiB / ~80 行 | `READ:source` → 更大但仍有硬顶的摘录 |
| `stderr_tail` / `stdout_tail` | 子任务捕获尾部 ~2KiB | `READ:stderr` / `READ:stdout` |
| `value` | ≤200 字；否则 `value_preview` + `value_len` | （一般不必加深） |
| `slots` | `{key,line,body_len}`；`error` 槽可带短预览 | `READ:slots` |
| `has_worker_step` / `solidify_on_done` | 布尔提示 | — |

`quiet=True` **只表示不污染父 TTY**；file `wait` 仍返回截断后的 `stdout` / `stderr` 供父观察。

---

## 6. 工具面

### 6.1 父侧内建 helper（`CALL:` 首批）

写在 `ext/ai/agent.mq.md`，由 plan 循环直接调度（**不用** OpenAI function calling）：

| 名称 | 作用 |
|------|------|
| `workbook_read` | 按 depth 读工作簿摘录 |
| `workbook_excerpt` | 结构摘录（默认观察同源） |
| `lib_catalog` | 枚举 `lib/*.mq.md` 能力摘要 |
| `scratch_tool_write` | 写入 `.marqdo/agent-runs/tools/<name>.mq.md` |

### 6.2 标准库与造工具

- **标准库**：父通过 `lib_catalog` 知晓能力；把 `> lib/….mq.md` 与 `##` 写进工作簿（PATCH）或 scratch 工具文件后即可被子任务 / 后续 `CALL` 使用。  
- **造工具**：优先 `CONTINUE` + 短 PATCH 在工作簿内新增 `##`；或 `CALL:scratch_tool_write` 沉淀独立工具文件。  
- **非目标（本阶段）**：把全部 `lib/*` 自动反射成 JSON tools schema；原生 function calling。

---

## 7. 输出纪律（停机规则）

父回复**只允许**协议行 + 一行 `SUMMARY` + 短 PATCH（`<20` 行）。禁止长独白 / 行程散文 / 把 observation 粘进 REPLACE。

修订阶段硬规则（自上而下）：

1. `exit_code==0` 且 `has_value` → **立刻** `DECISION: DONE`（运行时 solidify）；`has_worker_step` **不是** CONTINUE 的理由。  
2. 仅失败 / 无值 / 明确未达目标时才 `CONTINUE`。  
3. 代码护栏：非 `improve` 时，子已返回值则 **跳过父 LLM**，直接 solidify + DONE。

---

## 8. 与 `## step` 的关系

| | `step`（子） | `plan`（父） |
|---|-------------|-------------|
| 协议 | `CALL:` 或最终答案 | `CALL:` / `READ:` / `DECISION:` |
| 工具来源 | 工作簿 / 调用方 allowlist | 父 helper + 写入的 Marqdo `##` |
| 成功出口 | 返回答案文本 | `DONE` + solidify / promote |

---

## 9. 实现落点

| 组件 | 路径 |
|------|------|
| 设计（本文） | `doc/design/ext-agent-parent.md` |
| 多步总设计增量 | `doc/design/ext-agent-plan.md` §4.1 / 协议 |
| 子任务 I/O | `src/host/subtask.rs`、`lib/subtask.mq.md` |
| 父循环 / 观察 / 提示 | `ext/ai/agent.mq.md` |
