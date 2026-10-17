# 文档会话（Document Session）

| | |
|---|---|
| 状态 | **D1–D2 首版已实现；D3/D4 部分** |
| 日期 | 2026-10-17 |
| 层 | Runtime + Document（view 服务端 / notebook 交互） |
| 相关 | [notebook-syntax.md](notebook-syntax.md) · [foreign-kernel.md](foreign-kernel.md) · [foreign-notebook-decisions.md](../research/foreign-notebook-decisions.md) · [view.md](view.md) · [view-debug.md](view-debug.md) · [stdlib-subtask.md](stdlib-subtask.md) |
| 核心表面 Δ | 无（读取既有 AST 与表格） |

> 文档会话是 view 背后的“notebook 运行时”：它记住哪些 cell、在哪个内核、按什么顺序执行、状态是什么、结果是否过期。

---

## 1. 目标

1. **像 Jupyter 一样即时**：点一个 cell 就执行，状态保留。
2. **不像 Jupyter 一样隐藏**：执行顺序、内核归属、状态、过期情况都可查。
3. **不破坏 Marqdo 程序语义**：`marqdo run` 仍是规范执行；view 会话是交互镜头。
4. **Git 友好**：状态/输出不默认写进 `.mq.md`。
5. **多语言**：命名内核 + 显式状态桥。
6. **可复现**：Run All / journal / dirty / stale / replay。

---

## 1.1 实施状态（2026-10-17）

| 能力 | 状态 | 位置 |
|---|---|---|
| `内核表` / `会话表` 解析（中英列名） | ✅ | `src/notebook/mod.rs` |
| cell index（绑定名 / 行号 / source hash） | ✅ | 同上 |
| 命名内核 + 默认内核 + 语言校验 | ✅ | 同上 |
| 同内核共享 / 跨内核隔离 | ✅ | Node 金样例 + 单测 |
| `run_cell` / `step` / `run_all(restart)` | ✅ | `DocumentSession` |
| `restart_kernel` / `assign_kernel` | ✅ | 同上 |
| `kernel/interrupt` | 待做 | — |
| 状态桥 `push` / `pull` / `state_list` | ✅ | `DocumentSession` + `foreign_session` |
| journal（记录 + API + Journal 按钮） | ✅ | `DocumentSession::journal_json` · view |
| view 工具栏 / 状态徽标 / 内核下拉 / 状态表单 | ✅ | `src/view/html.rs` |
| `POST /api/session/*` | ✅（含 `reload`） | `src/view/notebook_api.rs` |
| cell step | ✅ | 同上 |
| Marqdo statement step | ✅（notebook 工具栏 Debug → 同一 view 服务的 `/debug`；`/api/debug/*`） | `src/debug` · `src/view/debug_api.rs` |
| dirty/stale 自动检测 | 部分（reload → dirty；assign → dirty；stale 传播待做） | `src/notebook/mod.rs` |
| writeback 集成 | 待做 | `writeback` |
| replay / 持久化 | 待做 | — |
| Python 会话 | ✅（本机无 python，CI/用户机条件测试） | `src/host/foreign_session.rs` |
| Node 会话 | ✅ | 同上 |

首版刻意不做：把状态写进 `.mq.md`、自动镜像变量、foreign 行级 debug、跨文档共享内核。

---

## 2. 两种执行模式

### 2.1 Program mode（规范）

- 入口：`marqdo run doc.mq.md` / CI / `view` 的 `Run document`。
- 执行：`# main` 程序语义；外联通过显式 `foreign.session_open/eval/call` 编排。
- 状态：一次运行内有效；运行结束进程退出（除非显式持久内核）。
- 结果：可复现；是文档的“真理”。

### 2.2 Notebook mode（交互）

- 入口：`marqdo view doc.mq.md` → Session 面板。
- 执行：按 `会话表`（或自动顺序）逐 cell 执行。
- 状态：内核持久；可切换、重启、单步。
- 结果：默认临时显示；可显式 writeback。
- 不是“真理”：是探索界面；需要复现时用 Run All 或把编排写进 `# main`。

### 2.3 两者关系

```text
                    .mq.md
                   /      \
          Program mode     Notebook mode
          (canonical)      (interactive)
                \             /
                 Document Session
                 ├── cell index
                 ├── kernel registry
                 ├── journal
                 └── state panel
```

- 共享 cell index 与 kernel registry。
- Program mode 可以不使用 notebook 表；Notebook mode 可以使用。
- 冲突时以 Program mode 为准；Notebook mode 不改变文档语义。

---

## 3. 文档会话模型

```text
DocumentSession
├── doc_path / doc_hash
├── cells: Vec<Cell>
│   ├── id (name / anon@line)
│   ├── name (binding)
│   ├── lang
│   ├── line_start / line_end
│   ├── source_hash
│   ├── kernel (resolved)
│   └── status: never | clean | dirty | stale | error | running
├── kernels: KernelRegistry
│   ├── name → ForeignSession
│   └── default per language
├── value_store: HashMap<String, Value>   // Marqdo 侧会话值
├── journal: Vec<ExecutionRecord>
├── created_ms / last_active_ms
└── mode: program | notebook
```

### 3.1 Cell

- 只有**绑定名**的代码围栏才是 cell：
  ```markdown
  `加载` =
  ```python
  ...
  ```
  ```
- 未绑定围栏仍是叙述，不是 cell。
- `id`：优先绑定名；匿名围栏用 `anon@<line>`（不进入会话表）。
- 同名 cell 重复 → 诊断；会话表按名字引用。
- `source_hash` 用于 dirty 判断。
- `line_start/line_end` 用于定位；编辑后重建索引。

### 3.2 Cell index 构建

1. 读取/解析文档（现有 `parse_source` / `load_module`）。
2. 遍历 AST，找 `Stmt::Assign { name, value: Expr::Code(c) }`。
3. 记录 name、lang、source、span、source hash。
4. 读取 `内核表` / `会话表` 绑定（见 [notebook-syntax.md](notebook-syntax.md)）。
5. 解析 kernel 声明与 cell 归属；校验；生成诊断。
6. 生成执行顺序：
   - 有会话表 → 行顺序；
   - 无 → 文档顺序。
7. 未入会话的 cell → 状态 `unlisted`，UI 显示徽标。

### 3.3 Kernel registry

- key：`<document_id>:<kernel_name>`。
- 默认：`default:python`、`default:node`，按需启动。
- 命名：来自 `内核表`，首次执行时启动。
- 状态：`idle | starting | ready | busy | dead`。
- 同一 kernel 的 cell 严格串行；不同 kernel 可并行。
- 重启：kill + 清空状态 + 重新启动；cell 状态变 `never`。
- 关闭：kill + 移除。

### 3.4 Value store（Marqdo 侧会话值）

- 保存可从 Marqdo 推入内核的值（来自最近一次 Program run 的 bindings，或用户手动输入）。
- 也保存从内核取回的值（`session_get` / 状态面板 Pull）。
- 不是解释器 Env 的实时镜像；它是会话层的**值缓存**。
- 注入解释器：未来通过 `SessionEnv` 或显式变量赋值；v2 不做隐式注入。
- 用途：
  - 状态面板显示；
  - Push 到内核；
  - 作为 `session_set` 的值来源；
  - journal 记录。

### 3.5 Journal

每次执行追加一条：

```json
{
  "seq": 3,
  "cell": "汇总",
  "kernel": "分析",
  "lang": "python",
  "ok": true,
  "code": 0,
  "ms": 42,
  "source_hash": "…",
  "stdout_hash": "…",
  "result_hash": "…",
  "started_ms": 1760000000000,
  "error": ""
}
```

用途：

- 执行序号 / 顺序；
- dirty/stale 判断；
- replay；
- 视图显示；
- 调试（“为什么这个 cell 有值”）。

存储：内存 + 可选 `.marqdo/sessions/<doc-hash>.json`（默认 git 忽略）。

---

## 4. 执行语义

### 4.1 操作

| 操作 | 语义 |
|---|---|
| **Run cell** | 执行选中的 cell；必要时启动/复用 kernel |
| **Run above** | 按会话顺序执行到选中 cell（含） |
| **Run all** | 按会话顺序执行全部；可选 `restart` |
| **Step next cell** | 执行会话顺序中的下一个未执行/选中 cell |
| **Step next statement** | Marqdo 语句级单步（DebugController） |
| **Restart kernel** | kill + 重建；cell 状态 `never` |
| **Interrupt** | kill 当前 cell 进程组；kernel 标记 dead 或重启 |
| **Reset session** | 关闭所有 kernel，清 journal / value store |

### 4.2 执行一个 cell

```text
resolve cell -> resolve kernel
  -> ensure kernel ready
  -> [送入?] 显式 bridge 由文档代码/UI 负责
  -> foreign.session_eval(kernel, source, input?)
  -> record -> update cell status
  -> [取回?] 由文档代码/UI 负责
  -> journal append
  -> view update
```

- 默认 cwd = 文档目录；env = kernel env。
- 输出：stdout / stderr / result / ms / truncated。
- 失败：cell 状态 `error`；journal 记录；不自动重试。
- 同一 kernel 串行；UI 禁用并发 Run。

### 4.3 Step 语义

| 类型 | 行为 |
|---|---|
| **Cell step** | 执行下一个会话 cell；状态保留 |
| **Marqdo statement step** | 执行下一条 Marqdo 语句；遇到 `foreign.session_eval` 时整个外联调用原子完成 |
| **Foreign line step** | 不做；未来 DAP |

- Cell step 是 Notebook mode 的主单步。
- Statement step 复用 `DebugController`；需要把 debug session 与 DocumentSession 关联。
- 两者可以互相切换：cell step 到某个 Marqdo cell 后可 statement step；语句执行完回到 cell 边界。

### 4.4 中断

- `Interrupt`：向当前 kernel 进程组发 SIGINT/SIGKILL；记录 `interrupted`。
- 若 kernel 状态不确定 → 标记 dead；建议 restart。
- 不允许“中断后继续用同一 kernel”除非确认状态。

### 4.5 并发

- 同一 kernel：一次一个 cell。
- 不同 kernel：可并行（例如 Python 分析与 Node 前端）。
- `Run all` 默认按会话顺序串行；可配置“按 kernel 并行、按依赖串行”（后续）。
- 并发上限沿用 foreign（32）。

---

## 5. 状态桥（会话层）

### 5.1 显式 API

文档代码：

```markdown
把 `数值表` 送进 `分析` 内核：
**> foreign.session_set id="分析" name="xs" value=`数值表`**

从 `分析` 内核取回 `mean`：
**`均值` = > foreign.session_get id="分析" name="mean"**
```

### 5.2 状态面板

- 左侧：Marqdo 侧值（最近 Program run bindings + value store）。
- 右侧：每个 kernel 的变量（`session_names` + 按需 `session_get`）。
- 操作：
  - Push：选 Marqdo 值 → kernel 变量；
  - Pull：选 kernel 变量 → value store / 显示；
  - Refresh；
  - 复制名称/值。
- Secret 打码；大对象显示摘要（类型/长度/repr 截断）。

### 5.3 自动镜像

- v2 不做。
- 后续 `mirror`：`mq_` 前缀、白名单、按值拷贝、默认关。
- 原因见 [decisions](../research/foreign-notebook-decisions.md) §1-F。

### 5.4 类型

- JSON 拷贝；复杂对象需 `to_dict` / JSON。
- 大对象：v2 显示摘要；传输用文件/DB。
- 未来：Arrow / Parquet / 共享文件句柄。

---

## 6. Dirty / Stale / Run All

### 6.1 定义

| 状态 | 条件 |
|---|---|
| `never` | 未执行 |
| `clean` | 上次执行成功后源码未变 |
| `dirty` | 源码 hash 变了 |
| `stale` | 上游 cell 变了（依赖图；v2 先按会话顺序近似） |
| `error` | 上次执行失败 |
| `running` | 正在执行 |
| `unlisted` | 不在会话表 |

### 6.2 依赖

- v2：会话表行顺序 + 简单规则：
  - 若上游 cell 的 source_hash 变了，下游标 `stale`；
  - 若 kernel 重启，全部 `never`。
- v2.1：变量级依赖图（静态分析 + bridge 表），精确 stale。
- 不假装 Jupyter 的“隐藏顺序”：顺序就是表/文档顺序。

### 6.3 Run All

- 默认：按会话顺序执行；可选 `restart=true`。
- 规范执行：**restart + run all**；UI 明确按钮 `Restart & Run All`。
- 未重启时：显示“当前 kernel 有历史状态”警告。
- 执行后：生成 session summary（ok/fail/耗时/跳过的 cell）。

### 6.4 Replay

- 用 journal 的顺序和输入 hash，在新 kernel 上重放；
- 比较 stdout/result hash；
- 输出 divergence 报告。
- v2 可选；至少保留 journal 以便未来实现。

---

## 7. 输出与文档

### 7.1 默认

- Notebook mode 输出只在 view 中显示；不写 `.mq.md`。
- 关闭/刷新后可通过 journal 恢复最近结果（若 session 持久化）。
- 不写 execution_count 到文档。

### 7.2 显式 writeback

- 复用 `lib/writeback` 的 `<!-- marqdo-out … -->`。
- cell 输出锚点：cell 名 + 行号；写入位置在围栏之后，或文档末尾的“输出”区。
- 可选：
  - 单 cell 写回；
  - 全部写回；
  - 只写回文本/数值，不写复杂对象。
- Git 友好：输出是普通 Markdown 注释块；可 diff、可忽略。

### 7.3 不写回

- 避免 Jupyter 的 `.ipynb` 输出噪声。
- 需要分享结果时，导出 HTML / 静态 view，而不是改源文件。

---

## 8. View 交互

### 8.1 工具栏

```text
[Run All] [Restart & Run All] [Step Cell] [Step Statement]
Kernel: [分析 ▾]  Status: ready  [Restart] [Interrupt]
[State] [Journal] [Writeback ▾]
```

### 8.2 每个 code card

```text
`加载`  python  ·  kernel 分析  ·  run #3  ·  42ms  ·  clean
[Run] [Step] [Kernel ▾] [State] [Copy output]
```

- 状态徽标：`never` / `clean` / `dirty` / `stale` / `error` / `running` / `unlisted`。
- 输出区：stdout / stderr / result / 截断提示。
- 内核下拉：切换归属；不迁移状态；提示需要重跑。
- `Step` = 执行此 cell（cell step）。

### 8.3 状态面板

- Marqdo / kernel 分栏。
- Push/Pull。
- Secret 打码。
- 大对象摘要 + 复制 JSON。

### 8.4 Journal 面板

- 表格：seq / cell / kernel / ok / ms / 时间 / 错误。
- 点击跳转 cell。
- 导出 JSON。

### 8.5 键盘

- `Ctrl/Cmd + Enter`：Run cell。
- `Shift + Enter`：Run cell + 下一个。
- `Alt + Enter`：Run cell + 插入？不做。
- `F10`：Step statement（与 debug 一致）。
- `F5`：Run All。

---

## 9. 服务端 API

所有 API 以文档相对路径 + session id 定位；session 内存驻留，进程退出即失。

| 方法 | 路径 | 作用 |
|---|---|---|
| POST | `/api/session/open` | 打开文档会话，返回 cells / kernels / journal |
| POST | `/api/session/close` | 关闭会话，kill 所有 kernel |
| POST | `/api/session/reload` | 重新解析文档，保留 kernel（标记 dirty） |
| POST | `/api/session/cell/run` | 执行指定 cell |
| POST | `/api/session/cell/step` | 执行下一个 cell |
| POST | `/api/session/run_all` | 按会话顺序执行；`restart` 可选 |
| POST | `/api/session/kernel/restart` | 重启指定 kernel |
| POST | `/api/session/kernel/interrupt` | 中断当前 cell |
| POST | `/api/session/kernel/assign` | 改 cell 的 kernel 归属（会话内） |
| POST | `/api/session/state/list` | Marqdo / kernel 变量列表 |
| POST | `/api/session/state/push` | Marqdo 值 → kernel |
| POST | `/api/session/state/pull` | kernel 变量 → value store |
| POST | `/api/debug/*` + `/debug` | Marqdo 语句单步（Debug 按钮，复用 debugger） |
| POST | `/api/session/writeback` | 显式写回 cell 输出 |
| POST | `/api/session/journal` | 执行日志 |
| GET | `/api/session` | 当前会话摘要 |

响应统一：`{ok, session, ...}`；错误 `{ok:false, error, code?}`。

---

## 10. 生命周期与持久化

- Session key：`<root>/<doc_rel>`；每个文档一个。
- 打开：解析、建 cell index、按需启动 kernel。
- 空闲超时：默认 30 分钟关闭（可配置）。
- 文档关闭 / 服务退出：kill 所有 kernel。
- 文件变更：view 刷新时 `reload`；cell hash 变化 → dirty；kernel 保留。
- 持久化（可选）：
  - `.marqdo/sessions/<doc-hash>.json`：kernel 名称/语言、cell 归属、journal、value store 摘要；
  - 默认 git 忽略；
  - 不持久化进程状态 / 大对象。
- 不把 session 状态写进 `.mq.md`。

---

## 11. 安全

| 项 | 策略 |
|---|---|
| exec | 所有 kernel 启动/执行检查 `allow_exec`；沙箱拒绝 |
| 生命周期 | 每文档；关闭/drop kill；不跨文档 |
| cwd | 文档目录 / `fs_root`；越界拒绝 |
| 输出 | 有界；截断标记 |
| 超时 | cell 级；进程组 kill |
| 中断 | 杀进程组；kernel 标记 dead |
| Secret | 状态面板打码；JSON 不默认暴露 |
| 未信任文档 | 不自动打开会话；必须显式 Run |
| 并发 | 同 kernel 串行；不同 kernel 上限 32 |
| writeback | 显式；路径沙箱；不自动改文档 |

---

## 12. 测试

### 12.1 服务端

- cell index：绑定名/行号/lang/source hash；未绑定围栏排除。
- 表解析：内核表/会话表/自动模式/重复/未知/语言不匹配。
- 会话：open/close/reload；kernel 启动/重启/中断。
- cell run：状态机 never→running→clean/error；dirty/stale。
- step：cell step 顺序；statement step 与 debug 联动。
- state：push/pull/list；Secret 打码；大对象摘要。
- journal：顺序、hash、失败。
- 并发：同 kernel 串行；不同 kernel 并行。
- 安全：exec=false 拒绝；cwd 越界；超时。

### 12.2 金样例

- `tests/notebook/kernels.mq.md`：内核表/会话表。
- `tests/notebook/shared-state.mq.md`：两个 Python cell 共享 `x`。
- `tests/notebook/step.mq.md`：cell step 顺序。
- `tests/notebook/state-bridge.mq.md`：Marqdo ↔ Python 推拉。
- `tests/notebook/invalid-*.mq.md`：诊断。
- 依赖：node 为主，python 条件跳过。

### 12.3 UI

- 手工冒烟：Run cell / Step / Run All / Restart / State Push/Pull / Journal。
- 不引入前端测试框架；用服务端 API + 静态检查。

---

## 13. 与 Jupyter 对照

| 维度 | Jupyter | Document Session |
|---|---|---|
| 源 | `.ipynb` | `.mq.md` |
| 执行顺序 | 隐藏的 `In[n]` | 会话表 / 文档顺序 / journal |
| 状态 | 隐藏 kernel | kernel + value store + 状态面板 |
| 多语言 | 一 notebook 一 kernel | 命名内核 + 每 cell 归属 |
| 单步 | cell | cell + statement |
| 复现 | 手动 | Restart & Run All + journal + replay |
| Git | 输出/计数噪声 | 源码干净；输出显式 |
| 知识 | 无 | OKF / catalog |
| 调试 | 语言 pdb | Marqdo 断点 + cell 级；行级后续 |

---

## 14. 实现顺序（D 层）

| 阶段 | 内容 |
|---|---|
| D0 | 设计/语法/决策文档（本文件） |
| D1 | `src/notebook/`：cell index、表解析、session 结构、诊断 |
| D2 | 服务端 API：open/close/cell run/step/run_all/restart/interrupt |
| D3 | view UI：工具栏、cell 徽标、kernel 下拉、状态面板、journal |
| D4 | statement step 接入；dirty/stale；writeback |
| D5 | replay / 持久化 / 并发优化 / 测试补全 |

依赖 F 层：F3 持久内核、F3.5 状态桥先完成，D2 才能工作。

---

## 15. 明确不做（v2）

- Foreign line-level step。
- 自动镜像变量。
- 跨文档共享 kernel。
- 全进程快照。
- 自动执行 cell。
- 把 session 状态写进 `.mq.md`。
- 前端 SPA 框架重写；沿用现有 HTML/JS。
