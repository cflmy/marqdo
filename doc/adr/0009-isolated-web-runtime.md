# ADR 0009: Isolated Web Runtime (worker process pool)

| 项目 | 约定 |
|---|---|
| 状态 | **Accepted**；HTTP **execute** 路径正式替代 [ADR 0008](0008-web-invoke-serialization.md) 的并行性结论。`legacy_serial`（未声明且非流式）仍引用 0008 单飞 / 父进程 `CallLibPath`。 |
| 日期 | 2026-10-26 |
| 设计 | [isolated-web-runtime-development-2026-10-26.md](../design/isolated-web-runtime-development-2026-10-26.md) |
| 金样例 | `tests/isolated_execute_mr01.rs`（MR-01 双 60s SSE + short reads；同文件 MR-02 槽满拒绝） |

## 背景

ADR 0008 确认：Go HTTP 多线程，但所有 Endpoint 共享一把解释器锁与进程级 SSE 桥环境变量。1500ms busy → 503 只是缓解，不是并行。

## 决策

1. **首选架构**：预热的一次性 Marqdo worker 进程池；每个租约独占一个进程与 HostContext。
2. **HTTP 默认**：`class=execute` 或 `return=sse-live` 或 body `stream:true` → `admit` → `dispatch`（`--call-lib-path` worker）；禁止父进程 `CallLibPath` / `Setenv` / `legacySSEBridgeMu`。`short` / `read` / `control` 仍父进程短路径；未声明非流式 → `legacy_serial`。
3. **控制/读取分离**：取消与事件订阅不进入任务解释器锁；短业务与长执行有独立容量（execute 槽默认 2）。
4. **协议**：`InvokeRequestV2` / worker env + NDJSON `MARQDO_EVENT_FORWARD` + `--emit-result`（首期）；`/_runtime/*` 控制面。
5. **能力探测**：`runtime.isolated_execution=1`（默认）、`runtime.task_control_v2`、`runtime.event_replay`、`view.analysis_cache_v2`；`runtime.read_lane` 仍 0。
6. **发布**：`expected_generation` CAS + publish journal；逻辑文档 ID 与物理 generation 路径解耦。

## 后果

- Execute / live SSE 不再抢进程级 SSE 桥；双流可并行（MR-01）。
- Legacy 未声明非流式路径仍可走父进程锁（ADR 0008）。
- 产品（qdagent）cancel / Last-Event-ID 接线仍为后续；不阻碍本框架发版。

## 验收（摘要）

MR-01 绿：双假上游 60s 流不混字、短读全 200、两 worker `--call-lib-path`、第三路槽满 429/503、`runtime.isolated_execution=1`。其余见设计文档 §12。
