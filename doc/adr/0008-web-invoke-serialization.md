# ADR 0008：Web 热路径单解释器串行化（设计缺陷与缓解）

| | |
|---|---|
| Status | **Accepted**（缺陷已记录；缓解已落地；真并行未决） |
| Date | 2026-10-26 |
| Related | [web-invoke-serialization.md](../design/web-invoke-serialization.md) · ADR [0004](0004-web-plugin-go.md) · ADR [0007](0007-web-document-endpoint.md) · [ext-abi.md](../design/ext-abi.md) · [ext-web-go-rewrite.md](../design/ext-web-go-rewrite.md) |

## Context

- Go `plugins/web` 的 HTTP 服务是**多 goroutine** 的：每个请求可并发进入 `handleInvoke` / `handleInvokeLiveSSE`。
- 每个 Endpoint 最终经 C ABI `host_query("call_lib_path", …)` 回到 Rust 宿主，在**同一个** `HostContext`（Marqdo 解释器状态）上执行作者 `.mq.md`。
- `HostContext` **不是** `Sync`：共享可变会话、库装载、写回、事件总线等状态不能安全地被多线程同时求值。
- 因此宿主用进程级 `INVOKE_LOCK`（`src/host/plugin.rs`）串行化所有来自 worker 线程的 `call_lib_path`。

### 常见误解

> 「web 都是单线程的吗？不能多线程吗？」

| 层 | 并发模型 |
|----|----------|
| Go HTTP / SSE flush / 取消 | **多线程（多 goroutine）** |
| Marqdo 解释器 `call_lib_path` | **单飞（一把全局锁）** |
| 浏览器 UI | 与后端锁无关；但若短 API 同步阻塞等待锁，会表现为「整页卡死」 |

**缺陷不在「HTTP 单线程」，而在「多 HTTP 工作线程共享一个非 Sync 解释器」。**  
真并行需要多解释器实例（或可共享的隔离运行时），不是去掉 Go 侧并发。

### 事故形态（已观测）

长对话（`return=sse-live` / `stream:true`）在 `CallLibPath` 期间一直持有 `INVOKE_LOCK`。并发的短路由（如 `/v1/models`、设置页探测）在旧实现里对锁 **无限期** `lock()`，HTTP 挂起 → 前端 `fetch` 无超时 → 页面无响应。重启进程可恢复，但属于运维掩盖，不是正确错误处理。

## Decision

1. **承认并文档化**本限制为宿主架构缺陷（单 `HostContext` + 全局 `INVOKE_LOCK`），不得再被表述为「web 插件只能单线程」。
2. **缓解（已实现，非根治）**：
   - `call_lib_path` 从 worker 进入时用 **限时 try-lock**（默认 `MARQDO_INVOKE_WAIT_MS=1500`），超时返回错误串 `interpreter busy: …`。
   - Go `handleInvoke` 将含 `interpreter busy` 的错误映射为 **HTTP 503**，并设 `Retry-After: 2`。
   - **禁止**在 SSE 期间把进程级 `MARQDO_INVOKE_WAIT_MS` 拉长：那会让并发短请求重新卡死。
   - 长 SSE 在启动时用同一短预算争锁；一旦拿到则持有至调用结束（对话本来就要跑完）。
3. **客户端契约**：短 API 必须有超时与 503 文案；不得假设「任意时刻都能同步进入解释器」。
4. **根治（未决，另案）**：解释器池 / per-request isolate / 可 `Send` 的会话切片。未落地前，产品行为以本 ADR 的 503 语义为准。

## Consequences

### Positive

- 对话进行中，设置/模型列表等短请求约 1.5s 内失败并可读，UI 不再无限挂起。
- 缺陷有名有姓，后续多解释器工作不会被「修好了」的假象挡住。

### Negative / Risks

- 同一进程仍无法真正并行跑两个 Marqdo Endpoint 体；第二个长 SSE 也会在短预算后失败。
- 极短窗口内，短请求若正好撞上锁交接，可能偶发 503（可重试）。
- `MARQDO_INVOKE_WAIT_MS` 是进程环境变量；误设过大仍会复现卡顿。

### Neutral

- Go 多 goroutine、SSE 桥（`MARQDO_EVENT_FORWARD`）、取消令牌（`MARQDO_CANCEL_TOKEN`）保持不变。
- 不改变 Document / Endpoint 作者面（ADR 0007）；只约束宿主与 web 插件错误语义。

## Implementation pointers

| 位置 | 职责 |
|------|------|
| `src/host/plugin.rs` → `acquire_invoke_lock` | 限时争锁；失败 `set_err("interpreter busy…")` |
| `plugins/web/.../routes_invoke.go` → `handleInvoke` | `interpreter busy` → 503 + `Retry-After` |
| 设计长文 | [web-invoke-serialization.md](../design/web-invoke-serialization.md) |
