# 子任务标准库（subtask）

| | |
|---|---|
| 状态 | v1 已落地 |
| 库 | `lib/subtask.mq.md` · `lib/子任务.mq.md` |

## 定位

OS 进程式并发：`spawn` 另起 `marqdo run <path>` 子进程；**父进程结束（含 panic）时 KillOnDrop 终止所有未 join 的子进程**。

v1 不共享 Marqdo 堆；子任务通过 stdout / 文件 / 环境通信。

## API

| EN | ZH | 说明 |
|----|-----|------|
| `spawn` | `启动` | `path=`（`.mq.md`） |
| `poll` | `探测` | `id=` → status map |
| `wait` | `等待` | `id=` → exit code（避免与内置 `join` 冲突） |
| `kill` | `终止` | `id=` |
| `wait_all` | `等待全部` | 所有子进程 exit codes |

Host：`host_subtask_*`（`src/host/subtask.rs`）。
