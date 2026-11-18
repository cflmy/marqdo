# 缺陷记录：`agent.run` EFI / resolve 在消费方崩溃（qdagent 复现）

| | |
|---|---|
| 状态 | **Fixed（2026-10-16）** — 见 §3；金样 `tests/ext/agent-run-cold-efi.mq.md` |
| 日期 | 2026-10-16 |
| 发现方 | qdagent（`~/work/qdagent`）接通 Marqdo ≥1.4.2 Agent EFI 全路径时 |
| 影响面 | `ext/ai/agent` · `## run` · `plugins/agent` `agent_resolve` |
| 相关 | [ext-agent.md](../design/ext-agent.md) · [engineering-knowledge.md](../design/engineering-knowledge.md) · [ai-agent-llm-optimize-2026-10.md](ai-agent-llm-optimize-2026-10.md) |

---

## 0. 一句话

`agent.run` 在 **空 OKF / 无 skill 命中** 或 **`efi=True`** 时，会以硬错误退出，而不是降级到 `plan` 探索。  
这是 **Agent Runtime 契约不完整**，不是 qdagent 业务写错。

---

## 1. 复现（qdagent）

环境：工作区 Marqdo `1.4.2` + `MARQDO_EXT=~/work/marqdo/ext`；qdagent 调用：

```text
助手 = > agent.agent …
`助手`.run task=… efi=True   # 或 efi=False 后仍进 resolve
```

本仓几乎没有 `.marqdo/agent-kb` 编译 skill。

### Bug A — `efi=True` → `unknown function preflight`

**位置**：[`ext/ai/agent.mq.md`](../../ext/ai/agent.mq.md) `## run`

```markdown
1. `efi`
  **efi_pf = preflight task=`task`**
```

同文件其它同对象方法一律写 `` `self`.route `` / `` `self`.resolve ``。  
裸标识 `preflight` 在导入消费方（如 `cli/求道-构建.mq.md`）里被解析成**全局函数**，找不到 → 硬错。

**期望**：`` **efi_pf = > `self`.preflight task=`task`** ``（或等价 method call）。

### Bug B — miss resolve → `missing map key llm_free`

**位置**：

1. Plugin：[`plugins/agent/src/memory.rs`](../../plugins/agent/src/memory.rs) `resolve_task`  
   当 `kb_lookup` 返回 null：

```rust
Ok(json!({
    "mode": "unknown",
    "skill": null,
    "resource": null,
    "confidence": 0.0,
    // 缺 llm_free
}))
```

2. Facade：`## run` 在 resolve 之后：

```markdown
**llm_free = [llm_free](resolved)**
```

脚注式取键在缺键时 **硬失败**，无法落到后面的 `plan` 探索臂。

**期望**：

- miss 结果始终带 `"llm_free": false`（与命中路径字段集对齐）
- facade 侧对可选键用 `json.get`（缺省 `False`），防御其它调用方

---

## 2. 为何金样没拦住

| 金样 | 为何漏网 |
|------|----------|
| `tests/ext/agent-zero-llm.mq.md` | 先编译 skill / policy，再 `run`；resolve **命中** |
| `tests/ext/eng-preflight.mq.md` | 只测 `agent_eng_preflight`，**不**走 `agent.run efi=` |
| 多数 `agent-*-live` | 自带 kb 或 `efi=False` / 不经过 cold miss |

qdagent 是第一个「冷库 + 产品 EFI + `run`」消费方，因此打穿。

---

## 3. 修复清单（本轮）

| # | 改动 | 文件 |
|---|------|------|
| 1 | miss resolve 补 `llm_free: false` | `plugins/agent/src/memory.rs` |
| 2 | `` `self`.preflight ``；`llm_free`/`mode` 用 `json.get` | `ext/ai/agent.mq.md` |
| 3 | 回归金样：冷 kb + `run efi=True` 不崩溃，落到 explore/plan | `tests/ext/agent-run-cold-efi.mq.md` |
| 4 | 重建 `marqdo_plugin_agent` 并跑金样 | `cargo build -p marqdo_plugin_agent --release` |

---

## 4. 对 qdagent 的含义

修复后 qdagent 可恢复：

```text
product preflight → latch → agent.run (efi=True|False) / plan
```

而不必长期绕开 `agent.run`。  
在插件未装入新 so 前，qdagent 侧 EFI latch + `step`/`plan` 仍是合法降级。

---

## 5. 非目标

- 不改 EKC / `agent_eng_preflight` 语义  
- 不引入第二套 Agent loop  
- 不把向量检索升为权威真相  
