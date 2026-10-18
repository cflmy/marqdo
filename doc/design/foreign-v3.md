# 外联运行时 v3 — 能力缺口与深化设计（Proposed）

| | |
|---|---|
| 状态 | **Proposed；尚未实现** |
| 日期 | 2026-10-18 |
| 层 | Runtime（foreign）· Document（`foreign.session`）· View（投影） |
| 相关 | [foreign-kernel.md](foreign-kernel.md) · [notebook-session.md](notebook-session.md) · [foreign-v2.md](../roadmap/foreign-v2.md) · [stdlib-writeback.md](stdlib-writeback.md) · [view-static-interactive.md](view-static-interactive.md) |
| 核心表面 Δ | **无增删**：不新增 Markdown 标记、不新增 notebook CLI |

> 本文只定义设计，不承诺已实现。当前基线是 v1.5.0 之后的本地工作：`foreign.session` 句柄、多内核、values 初始化、view 控制、静态执行与显式 writeback 已可用；本文处理仍然缺失的能力。

---

## 1. 设计约束

v3 必须继续满足以下约束，否则不进入实现：

1. **外联库拥有内核与状态**：`foreign` 管理内核、会话、cell 计划、输入、输出、journal 和生命周期。`view` 只消费句柄，不拥有第二套运行时。
2. **表与围栏是值**：`内核表`、`单元表`、`输入表`、`值表` 都是普通 Marqdo 值；没有保留表名，没有新增标记。
3. **单一执行语义**：`run` 是规范执行；`view` 是同一文档的交互投影；静态导出是同一会话的快照，不另写解释器。
4. **没有 Notebook 模式**：没有 `marqdo notebook`，没有 `In/Out`，没有“普通 view / notebook view”两套布局。
5. **显式 I/O**：Marqdo 值通过 `values` / `session_set` 传递；程序 stdin 是单独概念，不能伪装成 `marqdo_input`。
6. **显式持久化**：默认不改源码；writeback 必须由文档显式调用，或由用户显式选择策略。
7. **安全默认值**：执行受 capability 控制；输入/输出有界；不反序列化任意对象；跨文档共享必须显式开启。

---

## 2. 基线：现在已经有什么

| 能力 | 状态 |
|---|---|
| `foreign.run` / `run_lang` / `invoke` / `probe` | 已实现 |
| `foreign.call` / `call_many` / JSON 进出 / 并行 | 已实现 |
| `session_open` / `eval` / `set` / `get` / `names` / `close` / `list` | 已实现 |
| `foreign.session kernels= cells= values=` | 已实现 |
| 多内核 view 会话、cell 顺序、内核归属 | 已实现 |
| Marqdo 表通过 `values` 传入多个内核 | 已实现 |
| 无 `会话` 时无状态 cells | 已实现 |
| view 每 cell 内核选择 / Clean / Run | 已实现 |
| 静态导出执行 cells 并嵌入真实输出 | 已实现 |
| 静态页 session summary、内核归属、多内核展示 | 已实现 |
| 显式 `writeback.record` | 已实现（库能力） |
| 静态页按钮的“不可执行”提示 | 已实现 |

---

## 3. 缺口清单

| ID | 缺口 | 优先级 | 当前行为 | 目标 |
|---|---|---|---|---|
| G1 | 持久内核 raw stdin | P0 | 持久内核只有 `marqdo_input` | 程序 stdin 与控制协议分离 |
| G2 | 每 cell 独立 stdin | P0 | 页面级 Cell stdin，Run All 共用一个输入 | cell 级输入表 + UI |
| G3 | 动态 plan | P2 | 只读顶层 `会话` | 明确边界，或受控动态 |
| G4 | 运行时更新 plan | P1 | values 只在打开时初始化 | 显式 session 变更 API |
| G5 | kernel interrupt | P0 | 只能等超时/杀进程组 | 可中断当前 eval |
| G6 | 状态持久化 / replay | P1 | journal/dirty 只在内存 | 导出/恢复/replay |
| G7 | 跨文档内核 | P2 | 每文档独立 | 显式 scope/空间 |
| G8 | 非 Python/Node 持久内核 | P3 | Go 等一次性执行 | LangSpec 会话适配器 |
| G9 | live → static 输入捕获 | P0 | 静态构建不捕获临时 stdin | 显式 snapshot 捕获 |
| G10 | 自动 writeback | P2 | 必须显式调用 | 显式策略下的“运行后写回” |

---

## 4. 统一模型

### 4.1 会话句柄

`foreign.session` 返回的句柄是一个普通 Marqdo 值，建议稳定形状：

```text
{
  kind: "foreign-session",
  id: "<session-id>",
  document: "<entry-source-hash>",
  status: "created | ready | running | interrupted | dirty | closed",
  kernels: [...],
  cells: [...],
  inputs: [...],
  values: [...],
  generation: 1
}
```

`view` 只读这些字段；任何执行操作都通过 `foreign` 的 host API。句柄不是第二套运行时。

### 4.2 三种输入

必须严格区分：

| 输入 | 目标 | 形态 | 当前 |
|---|---|---|---|
| `values` | 内核变量 | Marqdo 值 → JSON copy | 已实现 |
| `input` | `marqdo_input` | JSON 值 | 已实现（低层） |
| `stdin` | 程序 stdin | 原始文本/字节 | 仅无状态 cell 有 |

v3 不允许把 raw stdin 偷偷塞进 `marqdo_input`；两者语义不同。

### 4.3 每 cell 输入表

不新增标记，仍然使用普通表：

```mq.md
`单元输入` =
| 单元 | stdin | input |
|------|-------|-------|
| 计算 | hello | {rate: 0.08} |
| 复核 |       | {check: true} |
```

`foreign.session` 增加可选参数：

```text
foreign.session kernels=... cells=... values=... inputs=`单元输入`
```

规则：

- `stdin` 只给该 cell 的程序 stdin；
- `input` 只给该 cell 的 `marqdo_input`；
- 空值表示不提供；
- 输入大小有上限，超限显式报错；
- 静态导出使用同一张表，因此 view 与静态快照可以一致。

### 4.4 执行流水线

```text
evaluate 会话 handle
  → open kernels
  → apply values
  → apply cell inputs
  → run cell
  → record {stdout, stderr, input_echo, result}
  → next cell
```

失败不终止整个 Run All；错误写入 cell record，journal 保留。

---

## 5. G1/G2：持久内核 stdin 与每 cell 输入

### 5.1 问题

当前持久内核的 worker 用 stdin 传输 JSON 控制协议，因此程序自己的 stdin 不可用；`marqdo_input` 只是 JSON 变量，不是 stdin。

### 5.2 方案 A（推荐）：控制通道与程序 stdin 分离

- Unix：父进程创建 **socketpair**（或两个 pipe），worker 通过 fd 3/4 读写控制帧；程序 stdin 使用单独的 pipe。
- Windows：named pipe 或可继承的匿名 pipe 作为控制通道；程序 stdin 单独 pipe。
- worker 协议升级为 `protocol: 2`：
  - 控制帧：`eval` / `set` / `get` / `interrupt` / `close`
  - eval 帧可带 `stdin`（UTF-8）或 `stdin_b64`（二进制，后续）
- Python worker：每次 eval 前把 `sys.stdin` 替换为 `io.TextIOWrapper`/`StringIO`，eval 后恢复控制通道；用户代码读到的是真正的 stdin。
- Node worker：控制 readline 移到控制 pipe；每次 eval 把 `process.stdin` 替换为 `PassThrough`/`Readable`，eval 后恢复。
- 超时/中断关闭该次 stdin，不污染下一次 eval。

### 5.3 方案 B（降级）：只支持 `input`

如果控制通道暂不实现，持久内核只支持 `input`，并显式拒绝 raw stdin：

```text
persistent kernel stdin is unavailable; use input= or stateless cells
```

不能静默把 stdin 变成 `marqdo_input`。

### 5.4 方案 C：stdin cell 降级为一次性

持久会话中遇到 raw stdin 时，用一次性进程执行该 cell；明确标记 `state: none`，不假装内核态连续。

### 5.5 UI 与静态

- `view`：每个代码卡提供 **Cell stdin** 小输入框，或者页面级输入表；推荐每 cell，因为它们经常不同。
- 静态：读取 `inputs` 表；没有输入时显示“requires stdin”说明，而不是 traceback。
- Run All 使用每 cell 的输入；不再把一个页面级 stdin 复制给所有 cell。

### 5.6 验收

- Python 与 Node 持久内核都能读取真实 stdin；
- 同一个 cell 连续 eval 不串输入；
- 超时/中断后下一次 eval 仍可读 stdin；
- 无状态 cell 行为不变；
- 静态快照使用相同输入表，结果与 live 一致（在相同输入下）。

---

## 6. G3：动态 plan 的边界

### 6.1 问题

`foreign.session` 目前只识别 entry 顶层的 `会话` 绑定。分支、函数、运行时循环中定义的 plan 不会被 view 发现。

### 6.2 设计决定（建议）

**v3 不支持任意控制流中的 plan。** 这是有意的语义边界：

- plan 是文档的声明，不是程序运行时的副作用；
- view 不应为了找 plan 而执行 `# main`；
- 避免 view 再次长出一套“运行程序再猜计划”的隐式逻辑。

允许的动态形式：

- 顶层赋值右侧的纯表达式，包括引入的纯函数；
- `foreign.session` 的 tables/values 可以由顶层表函数计算；
- 运行时变化通过 G4 的显式 session API 完成。

**不支持**：

- `##` 函数内部的 plan；
- `if`/`while` 分支里的 plan；
- 依赖 `# main` 副作用才能得到的 plan。

### 6.3 诊断

如果源码中出现非顶层 `foreign.session`，给出：

```text
foreign.session: plan must be a top-level binding (会话 = > foreign.session ...)
```

而不是静默降级为无状态。

---

## 7. G4：运行时更新 plan

### 7.1 目标

已打开的 view session 允许用户显式修改 cell 归属、增加/移除 cell、更新内核变量；所有修改都要可追踪。

### 7.2 API

```text
foreign.session_assign session=<handle> cell="计算" kernel="复核"
foreign.session_add_cell session=<handle> cell="新单元" kernel="分析"
foreign.session_remove_cell session=<handle> cell="旧单元"
foreign.session_set_value session=<handle> kernel="分析" name="rate" value=0.08
```

所有操作：

- 校验 cell/kernel 语言；
- 递增 `generation`；
- 写入 journal（kind=plan_change / value_change）；
- 标记受影响 cell 为 `dirty`；
- 失败不改变 handle。

### 7.3 View

- 内核下拉复用 `session_assign`；
- 增加/删除 cell 放在显式菜单，不做隐式拖拽写入源码；
- UI 显示 generation / dirty 状态。

### 7.4 静态

静态快照不执行变更；它记录变更后的 plan，或在构建时执行一次并固化。

---

## 8. G5：kernel interrupt

### 8.1 API

```text
foreign.session_interrupt id=<session-id>
```

### 8.2 行为

- 控制通道发送 `interrupt`；
- worker 取消当前 eval，返回 `{ok:false, interrupted:true}`；
- 保留已完成的值和 journal；
- 状态从 `running` → `interrupted`；
- 下一次 eval 可以继续使用内核（除非进程已损坏）；
- 超时继续作为最后一道防线。

### 8.3 View

- 工具栏增加 Interrupt；执行中的 cell 显示 interrupted；
- 不自动重跑；
- 静态导出没有 interrupt（快照不执行）。

### 8.4 跨平台

Unix：进程组信号 + 控制帧；Windows：控制帧 + `taskkill /T` fallback。两者都必须有测试。

---

## 9. G6：状态持久化与 replay

### 9.1 不做的事

- 不做 Python pickle / Node 堆快照；
- 不做任意对象反序列化；
- 不把内核内存默认写进 `.mq.md`。

### 9.2 显式快照

```text
foreign.session_export session=<handle> path=".marqdo/sessions/doc-1.json"
foreign.session_import path=".marqdo/sessions/doc-1.json"
```

快照只包含：

```text
{
  version: 1,
  document_hash: "...",
  plan_hash: "...",
  kernels: [{name, lang, cmd, values: {...}}],
  cells: [{name, kernel, status, last_ok, last_error}],
  journal: [...],
  inputs: [...]
}
```

值只允许 JSON 可表达的 Marqdo 值；不可序列化的值显式失败。

### 9.3 replay

```text
foreign.session_replay path="..." verify=True
```

按 journal 顺序重放 cell + inputs，比较 stdout/status；差异写入 report。replay 不覆盖原 journal，除非显式 `append`。

### 9.4 存储

- 默认目录 `.marqdo/sessions/`（gitignore 可选）；
- 用户可通过 path 放到受控目录；
- 不默认提交，不默认写源码。

---

## 10. G7：跨文档内核

### 10.1 原则

- 默认每文档独立；
- 跨文档必须显式 `scope=` / `space=`；
- 不自动按变量名或路径共享。

### 10.2 API

```text
foreign.session_open lang="python" scope="shared:analytics"
foreign.session_list scope="shared:analytics"
foreign.session_close id=... scope=...
```

- scope 注册表由 foreign runtime 管理；
- 同名冲突报错，不覆盖；
- view 只能打开当前文档声明允许的 scope；
- 生命周期由 owner/refcount 管理，最后一个引用关闭时清理。

### 10.3 安全

- 跨文档共享可能违反文档隔离，必须 capability + 显式配置；
- journal 标记 scope；
- 静态导出默认不打开跨文档 scope。

---

## 11. G8：非 Python/Node 持久内核

### 11.1 模型

不是所有语言都需要持久内核。`LangSpec` 增加能力标志：

```text
session: none | protocol_v2
```

- `none`：只能 `run` / `invoke` 一次性；
- `protocol_v2`：实现控制通道与 eval 协议，可以 `session_open`。

### 11.2 适配器

Go 等语言通过一个薄 worker 实现：

```text
control frame: eval {code, stdin, input}
result frame: {ok, stdout, stderr, result, error}
```

不允许“看起来像持久内核、实际每次重启”的假实现。

### 11.3 失败语义

- `session_open` 对 `session: none` 的语言明确报错；
- `foreign.session` 的 plan 只能使用支持 session 的语言；
- 静态/无状态仍可使用一次性执行。

---

## 12. G9：live → static 输入捕获

### 12.1 问题

静态构建在构建环境执行，无法知道用户在 live view 中临时输入的 stdin；同一文档的结果可能不同。

### 12.2 设计

增加显式的“捕获快照”：

```text
view → Capture snapshot → .marqdo/snapshots/<doc>.view.json
```

快照内容：

- document source hash；
- session plan（kernels/cells/values）；
- 每 cell 的 stdin/input；
- 每 cell 的 stdout/stderr/status；
- journal 摘要、语言/解释器版本；
- 捕获时间。

静态导出：

```text
marqdo view output --snapshot .marqdo/snapshots/doc.view.json
```

优先使用快照中的记录，校验 source hash，不重新执行；若 hash 不匹配，显式失败或 `--execute` 重跑。

### 12.3 安全/隐私

- 快照可能包含 secrets；默认不提交、不发布；
- 提供 `redact` 字段/正则；
- 静态页面只发布脱敏后的快照。

### 12.4 验收

- live 中成功的结果可以在静态页面原样显示；
- 不捕获内核内存，只捕获输入/输出/状态；
- 没有快照时回退到构建时执行，行为明确。

---

## 13. G10：自动 writeback

### 13.1 原则

**“自动”必须是用户显式选择的策略，不是默认副作用。**

### 13.2 API

```text
foreign.session persist=`写回策略`
```

策略表：

```mq.md
`写回策略` =
| 单元 | key |
|------|-----|
| 计算 | total |
| 复核 | check |
```

或 view 的 **Persist results** 按钮：将当前 cell 结果写到 `writeback.record key=...`。

### 13.3 语义

- 默认不写源码；
- 显式策略只在 **用户点击 Persist** 或 `marqdo run --writeback` 时执行；
- 静态构建默认不写回；`--writeback` 必须显式；
- writeback 只支持 JSON 可表达值；
- 写入位置/格式仍由 `writeback` 库决定。

### 13.4 与静态快照的关系

- 静态快照是只读发布；
- writeback 是源码内的活结果；
- 两者可以共存，但不能互相隐式触发。

---

## 14. API / 端点汇总

### 14.1 新增 foreign 函数

```text
foreign.session inputs=
foreign.session_assign
foreign.session_add_cell
foreign.session_remove_cell
foreign.session_set_value
foreign.session_interrupt
foreign.session_export
foreign.session_import
foreign.session_replay
foreign.session_open scope=
foreign.session_list scope=
```

### 14.2 View API

```text
POST /api/session/cell/run      { cell, stdin }
POST /api/session/run_all       { restart, inputs? }
POST /api/session/cell/clear
POST /api/session/plan/assign
POST /api/session/plan/add
POST /api/session/plan/remove
POST /api/session/plan/value
POST /api/session/interrupt
POST /api/session/snapshot/capture
```

所有端点只操作 foreign 句柄；view 不直接读写内核。

### 14.3 CLI

```text
marqdo view output --snapshot FILE
marqdo view output --execute
marqdo view output --no-exec
marqdo run --writeback
```

不新增 `marqdo notebook`。

---

## 15. 语义矩阵

| 能力 | run | view | static |
|---|---|---|---|
| `foreign.session` plan | 作为普通值执行 | 消费句柄 | 可执行或快照 |
| raw stdin | 支持（一次性/session） | 每 cell | inputs 表/快照 |
| interrupt | 无 UI | 可调用 | 不可用 |
| state export | 可调用 | 可显式导出 | 不导出或快照 |
| cross-doc | 显式 scope | 仅允许的 scope | 默认关闭 |
| writeback | `--writeback` | 显式按钮 | 默认不写 |

---

## 16. 实施顺序

### P0（先做，直接影响真实使用）

1. **G1/G2 stdin**：控制通道 + 每 cell 输入表 + UI。
2. **G5 interrupt**：避免只能等超时。
3. **G9 snapshot capture**：解决 live/static 不一致。

### P1

4. **G4 runtime plan updates**。
5. **G6 export/replay**。

### P2

6. **G7 cross-document scope**。
7. **G3 dynamic plan**：按第 6 节的边界，只做顶层纯表达式；任意控制流不承诺。
8. **G10 explicit writeback policy**。

### P3

9. **G8 non-Python/Node persistent adapters**。

---

## 17. 测试设计

### 单元

- `stdin` 协议：空输入、大输入、超时、中断、连续 eval；
- 每 cell 输入隔离；
- 返回值锚定在调用点（已修复的 CallFrame.call_line）；
- 表格不产生 call chip（已修复）；
- plan top-level 诊断；
- session generation / dirty；
- export/import/replay 校验。

### 集成

- Python 持久内核读 `sys.stdin`；
- Node 持久内核读 `process.stdin`；
- view Run All 使用每 cell 输入；
- interrupt 后下一次 eval 正常；
- static snapshot 与 live 结果一致；
- snapshot hash 不匹配显式失败；
- cross-doc scope 冲突/清理；
- Go 一次性 vs 持久能力诊断；
- writeback 只显式写入。

### 静态

- 输出无 traceback；
- 没有输入时显示说明而非伪造；
- 多内核 controls 显示；
- writeback slot 单块、暗色主题；
- 静态按钮提示。

---

## 18. 兼容与迁移

- 现有 `foreign.session kernels= cells= values=` 保持兼容；
- 新 `inputs=` 可选；
- `session_open` 旧协议继续可用；v2 控制通道通过 handshake 协商；
- 旧页面没有 `inputs` 时仍按空 stdin 执行/显示说明；
- 不修改已发布的 v1.5.0 tag；v3 以新版本发布。

---

## 19. 需要拍板的决策

1. **G1**：是否接受控制通道与程序 stdin 分离的协议改造？否则只能保留 `input`，不能宣称 raw stdin。
2. **G3**：是否接受“plan 必须是顶层声明”的边界？若不接受，需要定义 view 可安全调用的 plan 入口，风险更高。
3. **G6**：状态快照只允许 JSON 值，是否可接受？不做任意对象序列化。
4. **G9**：快照文件是否默认放在 `.marqdo/snapshots/` 且不提交？涉及 secrets。
5. **G10**：自动 writeback 是否只允许显式按钮/flag，而不是每次运行自动写源码？

---

## 20. 非目标

- 不新增 Markdown 标记；
- 不新增 notebook CLI/模式；
- 不把任意内核内存序列化；
- 不默认修改 `.mq.md`；
- 不自动镜像变量；
- 不把 view 变成第二套解释器；
- 不把“未实现”伪装成支持。

