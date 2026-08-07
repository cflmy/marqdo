# 子任务标准库（subtask）

| | |
|---|---|
| 状态 | v2 |
| 库 | `lib/subtask.mq.md` · `lib/子任务.mq.md` |

## 定位

**并发**执行单元。`spawn` 四选一：

| 参数 | 执行方式 | `join` 返回值 |
|------|----------|---------------|
| `path=` | `marqdo run <file>` 子进程 | 退出码 `Int` |
| `fn=` + 可选 `args=` | 当前入口模块中的函数（进程内线程） | 函数返回值 `Value` |
| `code=` | 外联代码块（`Value::Code`）子进程 | stdout `Text` |
| `lang=` + `source=` | 外联源码子进程 | stdout `Text` |

`args=`：文件子任务为 `List`（命令行参数）；函数子任务为 `Map`（形参绑定）或 `List`（按 `arg0`… 命名）。

父进程结束（含 panic）时 **KillOnDrop** 终止所有文件/外联子进程；函数线程标记为 killed 并 detach（无法强杀 OS 线程）。

`poll` → `{ status: running | done | failed, … }`；函数 done 时含 `value`，进程/外联 failed 时含 `code` 或 `error`。

Host：`host_subtask_*`（`src/host/subtask.rs`）。
