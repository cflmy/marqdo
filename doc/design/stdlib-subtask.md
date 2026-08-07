# 子任务标准库（subtask）

| | |
|---|---|
| 状态 | v2 |
| 库 | `lib/subtask.mq.md` · `lib/子任务.mq.md` |

## 定位

**并发**执行单元。`spawn` 四选一：

| 参数 | 执行方式 | `join` / `wait` 返回值 |
|------|----------|------------------------|
| `path=` | `marqdo run <file> --emit-result <tmp>` 子进程 | map `{ code: Int, value: Value\|null }`（`# main` 返回值；与 `quiet` 无关） |
| `fn=` + 可选 `args=` | 当前入口模块中的函数（进程内线程） | 函数返回值 `Value` |
| `code=` | 外联代码块（`Value::Code`）子进程 | stdout `Text` |
| `lang=` + `source=` | 外联源码子进程 | stdout `Text` |

`args=`：文件子任务为 `List`（命令行参数）；函数子任务为 `Map`（形参绑定）或 `List`（按 `arg0`… 命名）。

`quiet=` / `静默=`（默认 `True`）：仅影响 **文件** 子任务的 **TTY**。默认接到 null（父终端不被子 `print` 污染）；`False` 时继承父进程 stdout/stderr。**答案载荷走 `# main` 的返回值**（经 `--emit-result` 旁路），不依赖 print。外联子任务仍经管道收集 stdout 供 `join` 返回，不受此参数影响。

父进程结束（含 panic）时 **KillOnDrop** 终止所有文件/外联子进程；函数线程标记为 killed 并 detach（无法强杀 OS 线程）。

`poll` → `{ status: running | done | failed, … }`；函数 done 时含 `value`，进程/外联 failed 时含 `code` 或 `error`。

Host：`host_subtask_*`（`src/host/subtask.rs`）。
