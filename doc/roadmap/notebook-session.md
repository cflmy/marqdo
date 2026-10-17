# 文档会话路线图（D0–D5）

| | |
|---|---|
| 状态 | **D1–D2 已落地；D3–D5 待做** |
| 日期 | 2026-10-17 |
| 层 | Runtime + Document（view 服务端 / 前端） |
| 相关 | [notebook-session.md](../design/notebook-session.md) · [notebook-syntax.md](../design/notebook-syntax.md) · [foreign-kernel.md](../design/foreign-kernel.md) · [foreign-v2.md](foreign-v2.md) · [foreign-notebook-decisions.md](../research/foreign-notebook-decisions.md) |
| 核心表面 Δ | 无 |

> F 层让“外部语言能持久运行”；D 层让“文档里的多个 cell 像 notebook 一样共享状态、单步、切换内核”。两层分开验收。

---

## 0. 依赖

- **F3**：Python / Node 持久内核。
- **F3.5**：`session_set/get/names`。
- 现有 `DebugController`（Marqdo 语句单步）。
- 现有 `writeback` / `annot` / view 结构。

没有 F3/F3.5，D2 只能做一次性 Run，不是共享状态。

---

## 1. 阶段总览

| 阶段 | 内容 | 估算 |
|---|---|---|
| **D0** | 设计/语法/决策文档 | 已完成 |
| **D1** | `src/notebook/`：cell index、表解析、session 结构、诊断 | 2–3 天 |
| **D2** | 服务端 API：open/close/cell run/step/run_all/restart/interrupt | 3–4 天 |
| **D3** | view UI：工具栏、cell 徽标、kernel 下拉、状态面板、journal | 3–4 天 |
| **D4** | statement step、dirty/stale、writeback | 2–3 天 |
| **D5** | replay、持久化、并发优化、测试补全、发布 | 2–3 天 |

---

## 2. D1 — Cell index 与表解析

### 内容

- 新模块 `src/notebook/`：
  - `cell.rs`：从 AST 提取绑定名代码围栏；id/name/lang/line/source hash。
  - `syntax.rs`：读取 `内核表` / `会话表`；中英列名。
  - `session.rs`：`DocumentSession` 结构、kernel registry、value store、journal。
  - `diag.rs`：重复/未知/语言不匹配/未入会话诊断。
- 自动模式：无表 → 文档顺序 + 默认内核。
- 解析器不改；只遍历 `Module.functions` + `Stmt::Assign { value: Expr::Code }`。
- 测试：单元 + `tests/notebook/*.mq.md`。

### 验收

- 两个 Python cell + 一个会话表 → cells 顺序、kernel 归属正确。
- 无表 → 自动顺序。
- 未知内核/未知单元/重复/语言不匹配 → 可读诊断。
- 未绑定围栏不进入 cell index。

---

## 3. D2 — 服务端 API

### 内容

- `src/view/notebook_api.rs`（或 `view/session_api.rs`）：
  - open / close / reload
  - cell run / step / run_all
  - kernel restart / interrupt / assign
  - state list / push / pull
  - journal
- Session 存储：`OnceLock<Mutex<HashMap<String, Arc<DocumentSession>>>>`（类似 debug sessions）。
- 每个文档一个 session；key = 相对路径。
- 执行线程：cell 执行不阻塞 HTTP；响应返回记录；SSE/EventBus 可选。
- 错误统一 JSON。

### 验收

- open 返回 cells/kernels/journal。
- cell run：never→running→clean/error；状态更新。
- step：按会话顺序执行下一个。
- run_all：按顺序；restart 选项。
- 同 kernel 串行；不同 kernel 可并行。
- close/drop kill 所有 kernel。
- API 单测/集成测试。

---

## 4. D3 — View UI

### 内容

- 工具栏：Run All / Restart & Run All / Step Cell / Step Statement / Kernel 状态 / Restart / Interrupt / State / Journal。
- Code card：
  - 状态徽标（never/clean/dirty/stale/error/running/unlisted）
  - kernel 下拉
  - Run / Step
  - 输出区（stdout/stderr/result/truncated）
  - 执行序号 / 耗时
- 状态面板：
  - Marqdo 值 / kernel 变量分栏
  - Push / Pull / Refresh
  - Secret 打码、大对象摘要
- Journal 面板：表格 + 跳转 + 导出。
- 键盘：Ctrl/Cmd+Enter、Shift+Enter、F5、F10。
- 静态导出的 Run 仍隐藏。

### 验收

- 手工冒烟：两个 Python cell 共享 `x`；切换 kernel 后隔离；Restart 清状态。
- 状态面板 Push/Pull。
- 错误 cell 红色徽标 + stderr。
- 页面刷新后 session 仍在（服务端内存）。

---

## 5. D4 — Statement step / dirty / writeback

### 内容

- Statement step：复用 `DebugController`，与 DocumentSession 关联；暂停点显示变量。
- dirty：reload 后 source hash 变化。
- stale：按会话顺序，上游变更 → 下游 stale（v2 近似）。
- writeback：单 cell / 全部；`<!-- marqdo-out -->`；不自动。
- 运行历史 / 执行序号。

### 验收

- Statement step 在 view 中可用；外联调用原子完成。
- 编辑 cell → dirty；Run All 后 clean。
- 上游变更 → 下游 stale。
- 显式 writeback 写入文档；不自动改文件。

---

## 6. D5 — Replay / 持久化 / 发布

### 内容

- Journal replay：新 kernel，按 journal 顺序/输入 hash 重放；比较 stdout/result hash；divergence 报告。
- 可选持久化 `.marqdo/sessions/<doc-hash>.json`（kernel/cell 归属/journal/value store 摘要；默认 git 忽略）。
- 并发优化：不同 kernel 并行；同 kernel 串行。
- 测试补全；文档；catalog；CHANGELOG。
- 版本：D 层随 1.5.x。

### 验收

- Replay 能复现 clean 会话；dirty 输入给出 divergence。
- 关闭/重启服务无进程泄漏。
- 文档完整；金样例绿。

---

## 7. 测试矩阵

| 测试 | 断言 |
|---|---|
| `notebook/kernels.mq.md` | 内核表/会话表解析 |
| `notebook/auto-session.mq.md` | 无表自动顺序 |
| `notebook/shared-state.mq.md` | 同 kernel 共享变量 |
| `notebook/isolated-kernels.mq.md` | 不同 kernel 隔离 |
| `notebook/step.mq.md` | cell step 顺序 |
| `notebook/state-bridge.mq.md` | Marqdo ↔ kernel 推拉 |
| `notebook/dirty.mq.md` | 编辑 → dirty/stale |
| `notebook/invalid-*.mq.md` | 诊断 |
| API 集成 | open/run/step/run_all/restart/interrupt/close |
| 安全 | exec=false、cwd 越界、超时、Secret 打码 |

依赖 node 为主；python 条件跳过。

---

## 8. 完成定义

- D1–D4 验收；D5 可选（replay/持久化）。
- 文档会话与 F 层联调通过。
- 手工 view 冒烟：共享状态 / 单步 / 切换 / 状态面板 / journal。
- 测试绿；文档更新；提交推送。
