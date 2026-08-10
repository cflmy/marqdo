# 智能体流式输出（未来规划）

| | |
|---|---|
| 状态 | **S0–S2 已落地**（S3 view 仍待做） |
| 日期 | 2026-08-10 |
| 相关 | [ext-llm.md](../design/ext-llm.md) · [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [stdlib-subtask.md](../design/stdlib-subtask.md) · [view.md](../design/view.md) |
| 痛点 | `complete` / `step` / `plan` 均为整段返回；用户看不到思考与中间输出 |

---

## 1. 问题

当前体验：

| 路径 | 行为 | 体验问题 |
|------|------|----------|
| `` `model`.complete `` | 等 HTTP 整包结束再返回字符串 | 长推理「卡住」无反馈 |
| `` `助手`.step `` | 拼提示 → complete → 解析 CALL/答案 | 工具轮次与模型吐字均不可见 |
| `` `助手`.plan `` | 多轮 spawn + 父 LLM；子任务默认 `quiet` | 父侧只有最终 `result` map；过程静默 |

这与「文档驱动、过程可审计」并不矛盾——**落盘真相**仍可以是工作簿与写回；缺的是**运行时的渐进呈现**（TTY / view / 调用方回调）。

约束回顾（不得破坏）：

1. 文件子任务默认静默：答案走 `# main` **返回值**（[stdlib-subtask.md](../design/stdlib-subtask.md)）。  
2. `ext/**` 不直调 `host_*`；流式原语进 `ext/ai/llm` + 必要时薄 host/net。  
3. 多轮哲学仍是模式 C（子文件接力 + OKF），不是把 transcript 堆进单文件。

---

## 2. 目标

1. **可选**流式看到：模型增量文本、（可选）工具调用开始/结束、plan 轮次边界。  
2. 默认保持今日语义：未开流式 = 整段 `complete` / 整包 `result`。  
3. 流式是**呈现与旁路**，不替代 `**返回**` / `plan.result` / 写回。  
4. CLI、`marqdo view`、嵌入调用方可共用同一事件模型。

非目标（本规划）：

- 把 Marqdo 做成聊天产品 UI 框架。  
- 强制所有 `print` 改流式 API。  
- 在 quiet 子进程里把子 `print` 默认真传父 TTY（与返回优先冲突）。

---

## 3. 分层设计

```text
LLM 供应商 SSE / chunked
        ↓
ext/ai/llm  stream=True  →  事件迭代 / 回调 / 写 channel
        ↓
ext/ai/agent  step/plan 订阅事件 → 可选透传或写回「过程槽」
        ↓
呈现面：TTY | view EventSource | 调用方 foreach 事件表
```

### 3.1 事件模型（锁定方向）

统一为可 JSON 化的小 map（便于写回与 view）：

| `type` | 含义 | 主要字段 |
|--------|------|----------|
| `delta` | 模型增量文本 | `text` |
| `tool_start` / `tool_end` | 工具轮 | `name`, `result?` |
| `round` | plan 轮次边界 | `round`, `workbook`, `exit_code?` |
| `decision` | DONE / CONTINUE | `decision`, `summary?` |
| `done` | 本调用结束 | `result`（与今日返回值对齐） |
| `error` | 失败 | `message` |

中英键名可在 L1 包装层镜像；插件/wire 先用英文 `type`。

### 3.2 `llm`：流式完成

```markdown
*`model` = > llm *
*`stream` = > `model`.complete prompt=… stream=True *
- [`ev`](`stream`)
  *`t` = > json.get value=`ev` key=type *
  1. `t` == delta
    *`chunk` = > json.get value=`ev` key=text *
    > print text=`chunk`
  2. `t` == done
    *`answer` = > json.get value=`ev` key=result *
```

或回调形（若日后支持函数值作实参）：`on_delta=` —— v1 优先 **事件列表 / 可迭代句柄**，少发明回调语法。

实现要点：

- `lib/net` 或 llm 插件支持 **读 SSE / chunked body**（今日 `http_post` 偏整包）。  
- `stream=False`（默认）路径零行为变化。  
- 金样：mock SSE fixture，不强制打真网。

### 3.3 `agent.step` / `plan`：过程透传

| 开关（草案） | 默认 | 行为 |
|--------------|------|------|
| `stream=False` | ✓ | 与今日相同 |
| `stream=True` | | 向**当前进程**呈现面推送事件；返回值仍是最终 map |
| `trace=True` | 可选 | 把事件追加写入调用行写回槽 `trace`（审计）；与 TTY 流式可分立 |

plan 特有：

- 每轮 `await_workbook` 前后发 `round`。  
- 父 LLM 的 `complete` 若 `stream=True`，`delta` 冒泡到 plan 的 stream。  
- **子文件 quiet 不变**；若需看子内部模型吐字，由子工作簿自己 `stream=True` 并把摘要 **返回** 或写回，而不是继承子 stdout。

### 3.4 呈现面

| 面 | 做法 |
|----|------|
| CLI | `delta` 直接写 stdout（无额外 JSON 包装）；结构化事件可 `--trace-events` 走 stderr / 旁路文件 |
| `marqdo view` | 调试/运行面板订阅同一事件（WebSocket 或 SSE） |
| 库调用方 | foreach 事件集合，自行决定 UI |

---

## 4. 与「返回优先」的关系

| 通道 | 职责 |
|------|------|
| stream / trace | **过程**（体验、调试、审计） |
| `# main` `**…**` / `plan.result` | **结果**（编排、OKF、调用方消费） |
| writeback `ok`/`error` | **落盘摘要**（默认可关） |

禁止：仅靠 stream 文本当作子任务成功条件；DONE / 固化仍看退出码、返回值与源码形态。

---

## 5. 推荐落地顺序

```text
S0  net/llm：SSE 读 + complete stream=True + 事件 foreach     ✅
S1  step stream=True（echo 增量；返回 map 不变）               ✅
S2  plan stream=True：round + 父 delta；trace 写回可选         ✅
S3  view 订阅                                                 （待做）
```

### 开放点决议（S0）

1. **Eager 事件列表**（非惰性句柄）：`complete stream=True` 返回 `List` of maps，可用今日 foreach。  
2. **默认不自动 print**：`echo=False`；需要边读边打时传 `echo=True` / `打印增量=真`。  
3. 中文：`## 运行` 用 `流式=` / `打印增量=`。

### 开放点决议（S2）

1. `plan` / `多步` 增加 `stream` / `echo` / `trace`（中文 `流式` / `打印增量` / `轨迹`）。  
2. 事件挂在返回 map 的 `events`；父 `complete` 只冒泡 `delta`/`error`（嵌套 `done` 不进列表）。  
3. `trace=True` → 写回槽 `trace`（与 TTY `echo` 可分立）。

验收：

1. 未传 `stream` 的金样零 diff。  
2. `stream=True` 时用户在首包 token 到达前不应长时间无输出（允许短连接建立延迟）。  
3. 最终 `result` 与非流式路径一致（同 prompt fixture）。  
4. quiet 子任务仍不污染父 TTY。

---

## 6. 开放点

1. ~~Eager vs 惰性~~ → S0 选 eager；惰性句柄可后补。  
2. ~~delta 默认 print~~ → 默认关；`echo=True` 可选。  
3. ~~中文面~~ → `流式=` / `打印增量=`。  
4. ~~S2~~ → `plan stream=True`：`round` / 父 delta / `decision` / `done`；可选 `trace`。  
5. **S3**：`marqdo view` 订阅同一事件模型。

S0 可与 [okf-near-match.md](okf-near-match.md) 并行，互不阻塞。
