# Web 热路径：单解释器串行化（设计缺陷说明）

| | |
|---|---|
| 状态 | **Accepted**（缺陷记录 + 缓解已落地） |
| 日期 | 2026-10-26 |
| 规范决策 | [ADR 0008](../adr/0008-web-invoke-serialization.md) |
| 相关 | [ext-web-go-rewrite.md](ext-web-go-rewrite.md) · [ext-abi.md](ext-abi.md) · [ADR 0004](../adr/0004-web-plugin-go.md) · [ADR 0007](../adr/0007-web-document-endpoint.md) |

本文把「页面卡死 / 像单线程」拆成可验证的架构事实、错误语义与演进边界。  
**结论先行：HTTP 是多线程的；卡住是因为所有 Endpoint 共享一把解释器锁。**

增量实施提案见 [隔离执行与响应式 Web 运行时（2026-10-26）](isolated-web-runtime-development-2026-10-26.md)：独立进程池、控制/读取容量、请求级事件与取消、缓存和恢复。提案通过验收前，本文件及 ADR 0008 的现行单飞约束不变。

---

## 1. 问题陈述

### 1.1 用户可见症状

1. 发起一次长对话（OpenAI 兼容 `POST …/v1/chat/completions` + `stream:true` / `sse-live`）。
2. 对话尚未结束时，打开设置、拉 `/v1/models`、或其它短 JSON API。
3. **旧行为**：浏览器请求一直 pending，整页像死机；只能杀 `marqdo run serve…` 恢复。
4. **目标行为**：短请求在约 1.5s 内得到 **HTTP 503**，错误体标明 `interpreter busy`；对话 SSE 继续；结束后短请求立刻恢复 200。

### 1.2 错误归因（不要再说错）

| 错误说法 | 正确说法 |
|----------|----------|
| 「web 插件是单线程的」 | Go `net/http` 与 SSE 刷写是多 goroutine |
| 「忘记启动进程所以卡死」 | 进程在跑；是 **锁争用** 导致短请求永不返回 |
| 「把 SSE 的等待时间调大就能修好」 | 进程级拉长 `MARQDO_INVOKE_WAIT_MS` 会让短请求一起卡死 |
| 「去掉 INVOKE_LOCK 即可并行」 | `HostContext` 非 Sync，无锁并行会数据竞争 / 未定义行为 |

---

## 2. 调用栈与锁边界

```
Browser
  │  GET /v1/models          POST /v1/chat/completions (SSE)
  ▼
Go plugins/web  (多 goroutine)
  handleInvoke / handleInvokeLiveSSE
  │
  ▼  C ABI host_query("call_lib_path", {path, args})
Rust src/host/plugin.rs
  ┌─────────────────────────────┐
  │  INVOKE_LOCK  (进程级 Mutex) │  ← 同一时刻只允许一个 worker 进入
  └─────────────────────────────┘
  │
  ▼
HostContext：装载/求值作者 .mq.md、写回、事件总线、LLM 插件回调…
```

要点：

1. **锁在宿主，不在 Go 业务路由表。** 增加更多 Go handler 不会自动并行执行 Marqdo。
2. **SSE 持锁时间 ≈ 整次 Endpoint 求值时间**（含多次 LLM 往返），可达数十秒到数分钟。
3. **事件转发**（`MARQDO_EVENT_FORWARD` NDJSON）与 **取消**（`MARQDO_CANCEL_TOKEN`）在持锁期间仍可由其它 goroutine 写文件/读桥；它们不进入解释器，故不受 `INVOKE_LOCK` 保护，也不应误设长等待。

---

## 3. 为何不能简单「多线程跑 Marqdo」

`HostContext` 持有（非穷尽）：

- 当前模块图、函数表、写回缓冲与 FS 根；
- 插件注册与 `CURRENT_HOST` / 全局 host 指针；
- Agent / LLM 运行时与 stream 发布钩子；
- 与文档写回、会话态相关的可变结构。

这些状态在实现上按**单线程求值**建模。把它标成 `Sync` 或去掉 `INVOKE_LOCK` 而没有隔离运行时，属于未定义行为，不是性能优化。

**真并行的合法方向（未实现）：**

1. **解释器池**：每请求或每会话一个 `HostContext`（或可重置的 isolate），路由层选闲置实例；
2. **只读快照 + 写队列**：热路径只读克隆，写回串行合并（适合纯查询 Endpoint，不适合带写回的 agent）；
3. **把长任务移出解释器**：Marqdo 只做编排，重活在独立进程/队列（与「代码即文档」主路径权衡后再定）。

在上述任一方案落地并经金样例验收前，**产品必须以「单飞解释器」为约束设计 UX**。

---

## 4. 缓解契约（已实现）

### 4.1 宿主：限时争锁

文件：`src/host/plugin.rs`

- 仅当从 worker 线程进入（`CURRENT_HOST` 为空、走全局 host）时争 `INVOKE_LOCK`。
- `acquire_invoke_lock`：`try_lock` 循环，预算默认 **1500ms**（`MARQDO_INVOKE_WAIT_MS`，最小 1）。
- 超时：`set_err("interpreter busy: another request holds the Marqdo invoke lock (waited …ms)")`，返回非 0。
- **不要**在 SSE 入口把该环境变量设成数分钟：它是进程级的，会污染并发短请求。

### 4.2 Web 插件：503

文件：`plugins/web/internal/httpx/routes_invoke.go`

- 普通 JSON invoke：错误信息含 `interpreter busy` → **503** + `Retry-After: 2` + `{ok:false, error:…}`。
- Live SSE：若启动时即 busy，以失败事件/错误结束该流，**不得**无限挂起握手。

### 4.3 客户端：超时与文案

短 API（模型列表、设置、registry 探测等）必须：

- 设置合理 `fetch` 超时（秒级，例如 8s）；
- 将 503 / `interpreter busy` 展示为「服务正忙（对话进行中）」类可读提示，而不是空白转圈。

（qdagent 前端 `frontend/src/lib/api.ts` 已按此契约处理。）

### 4.4 验收探针

在本地 serve 上：

```bash
# 终端 A：启动长 SSE 对话（持锁）
curl -N -m 120 -H 'Authorization: Bearer …' -H 'Content-Type: application/json' \
  -d '{"stream":true,…}' http://127.0.0.1:PORT/v1/chat/completions

# 终端 B：对话进行中
curl -sS -w 'http=%{http_code} time=%{time_total}\n' --max-time 5 \
  -H 'Authorization: Bearer …' http://127.0.0.1:PORT/v1/models
# 期望：http=503，time≈1.5，body 含 interpreter busy

# 对话结束后再打同一 URL → http=200，time≪1s
```

---

## 5. 运维与调参

| 变量 | 默认 | 含义 |
|------|------|------|
| `MARQDO_INVOKE_WAIT_MS` | `1500` | worker 进入解释器前最长等待；超时 → busy |

建议：

- 开发机 / 产品默认保持 1–3s。
- **禁止**为「让第二个聊天排队」而把默认调到分钟级——那等于恢复卡死。
- 需要排队时，应在 **应用层**（会话队列、UI 禁用「再发一条」），而不是拉长宿主锁等待。

---

## 6. 与其它机制的边界

| 机制 | 与 INVOKE_LOCK 的关系 |
|------|----------------------|
| `MARQDO_EVENT_FORWARD` | SSE 期间写 NDJSON；不替代争锁；不延长等待 |
| `MARQDO_CANCEL_TOKEN` | 断开连接时可取消 LLM；释放锁仍取决于解释器何时回到 FFI |
| `web.serve` / 静态资源 | 通常不经 `call_lib_path`，不受本锁影响 |
| 插件内部 Tokio/Go 并发 | 可以；只要不再次 `call_lib_path` 进同一 HostContext |

---

## 7. 演进检查清单（根治前勿宣称「已支持并行 Endpoint」）

- [ ] 多 `HostContext` 或 isolate 的生命周期与 ABI（装载、shutdown、写回冲突）有设计稿与金样例。
- [ ] 池化后 `call_lib_path` 不再依赖进程级单锁，或锁粒度降到实例级。
- [ ] 两个并发长 SSE + 一个短 `/v1/models` 的压力测试：短请求稳定 200，长请求互不污染会话写回。
- [ ] 更新本文件与 ADR 0008 Status（例如改为 Superseded by ADR NNNN）。

在清单完成前，对外与文档中应明确：**Marqdo Web Endpoint 求值在单进程内串行；HTTP 层并发，解释器层单飞。**
