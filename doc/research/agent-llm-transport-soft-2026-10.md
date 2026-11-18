# 缺陷记录：LLM 传输失败硬中断 Agent Runtime

| | |
|---|---|
| 状态 | **Fixed（2026-10-16）** — 见 §3；金样 `tests/ext/agent-llm-transport-soft.mq.md` |
| 日期 | 2026-10-16 |
| 发现方 | qdagent（接通 `agent.run` live 路径时） |
| 影响面 | `host/net` · `plugins/llm` · `ext/ai/llm` · `ext/ai/llm/openai` · `ext/ai/agent` |
| 相关 | [agent-run-efi-resolve-bugs-2026-10.md](agent-run-efi-resolve-bugs-2026-10.md) · [ext-agent.md](../design/ext-agent.md) |

---

## 0. 一句话

上游 LLM **连接拒绝 / 超时 / 非 200** 时，Runtime 曾以 **硬错误** 终止整个 `marqdo run`；产品（qdagent ASK/BUILD）无法落到结构化 `status=error`。

---

## 1. 复现

```bash
OPENAI_API_KEY=offline OPENAI_BASE_URL=http://127.0.0.1:9 \
  marqdo run tests/ext/agent-llm-transport-soft.mq.md
# 修复前：exit≠0，error: http POST …
# 修复后：exit 0，ask/step/run 均带 finish|status=error + error 文本
```

---

## 2. 根因

| 层 | 行为 |
|----|------|
| `host_http_post` / SSE | 传输失败 `Err(...)` → VM abort |
| `plugins/llm` `llm_chat_completions` | 同上 `Err` |
| `ext/ai/llm/openai` | HTTP≠200 → `sys.exit 1` |
| `agent.model_turn` / `step` / `plan` | 无软失败臂 |

---

## 3. 修复清单

| # | 改动 | 文件 |
|---|------|------|
| 1 | 传输失败 → `{status:0, body}` / SSE error event | `src/host/net.rs` |
| 2 | 插件返回 `{finish:error, error, …}` 而非 `Err` | `plugins/llm/src/lib.rs` |
| 3 | openai 非 200 → 结构化 pack，禁止 `sys.exit` | `ext/ai/llm/openai.mq.md` |
| 4 | `llm.ask` 透传 `error` / `finish` | `ext/ai/llm.mq.md` |
| 5 | `model_turn` / `step` / `plan` 收成 `status=error` | `ext/ai/agent.mq.md` |
| 6 | 金样 | `tests/ext/agent-llm-transport-soft.mq.md` |

**契约**：Agent Runtime 对 LLM 传输失败必须可捕获；`status=error` + `error` 字符串；进程退出码 0（程序正常返回错误图）。

---

## 4. 对 qdagent

`ensure_llm` 有 key 但上游不可达时，`ask`/`build` 可走既有 `mode=error` 臂写 workbook，不再整 CLI 崩。
