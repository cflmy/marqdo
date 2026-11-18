# ADR 0009: Isolated Web Runtime (worker process pool)

| 项目 | 约定 |
|---|---|
| 状态 | **Accepted**；HTTP **execute** 路径正式替代 [ADR 0008](0008-web-invoke-serialization.md) 的并行性结论。`legacy_serial`（未声明且非流式）仍引用 0008 单飞 / 父进程 `CallLibPath`。 |
| 日期 | 2026-10-26（修订 2026-10-27：disconnect≠cancel；control/read/short 无父锁） |
| 设计 | [isolated-web-runtime-development-2026-10-26.md](../design/isolated-web-runtime-development-2026-10-26.md) |
| 金样例 | `tests/isolated_execute_mr01.rs`（MR-01 双 60s SSE + short reads；同文件 MR-02 槽满拒绝；disconnect-keeps-running；control-under-dual-execute） |

## 背景

ADR 0008 确认：Go HTTP 多线程，但所有 Endpoint 共享一把解释器锁与进程级 SSE 桥环境变量。1500ms busy → 503 只是缓解，不是并行。

## 决策

1. **首选架构**：预热的一次性 Marqdo worker 进程池；每个租约独占一个进程与 HostContext。
2. **HTTP 默认**：`class=execute` 或 `return=sse-live` / body `stream:true` → `admit` → `dispatch`（`--call-lib-path` worker）；**禁止**父进程 `CallLibPath` / `Setenv` / `legacySSEBridgeMu`。声明的 `short`/`read`/`control` 走父进程短路径（execute 已离进程，父锁通常空闲）；未声明非流式 → `legacy_serial`。
3. **控制/读取分离**：`/_runtime/*`（cancel、events、task get）不进入解释器锁。短读勿挤占 execute worker 槽（池默认 2）；产品 cancel 优先 `/_runtime`。
4. **订阅 ≠ 取消**：`sse-live` 观察者 `ctx.Done()` 默认**只结束订阅**，不 `runtime.cancel`；后台仍 reap worker 以释放槽。仅显式 header `X-Marqdo-Cancel-On-Disconnect: 1` 才 disconnect=cancel。同步 `waitIsolatedJSON` 可保留 disconnect 取消。重连：`GET /_runtime/tasks/{id}/events` + `Last-Event-ID` / `after_seq`。
5. **协议**：`InvokeRequestV2` / worker env + NDJSON `MARQDO_EVENT_FORWARD` + `--emit-result`（首期）；`/_runtime/*` 控制面。
6. **能力探测**：`runtime.isolated_execution=1`、`runtime.task_control_v2`、`runtime.event_replay`、`view.analysis_cache_v2`、`runtime.read_lane=1`（短读不与 execute 抢父锁）。
7. **发布**：`expected_generation` CAS + publish journal；逻辑文档 ID 与物理 generation 路径解耦。

## 后果

- Execute / live SSE 不再抢进程级 SSE 桥；双流可并行（MR-01）。
- 短读走父进程：在 dual execute 下通常仍 200（父锁空闲）；勿把 short/read/control 塞进 2 槽 execute 池。
- Legacy 未声明非流式路径仍可走父进程锁（ADR 0008）。
- 产品（qdagent）`turn_events` / Last-Event-ID / cancel CAS 接线由 qdagent 验收；框架侧 disconnect≠cancel 与 `/_runtime` 无锁控制面已落地。

## 验收（摘要）

MR-01 绿：双假上游 60s 流不混字、短读全 200、两 worker `--call-lib-path`、第三路槽满 429/503、`runtime.isolated_execution=1`、`runtime.read_lane=1`。观察者断连后任务仍跑完并可经 `/_runtime` 重放。其余见设计文档 §12。
