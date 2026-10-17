# 外联内核 v2 路线图（F0–F4）

| | |
|---|---|
| 状态 | **待实施** |
| 日期 | 2026-10-17 |
| 层 | Runtime |
| 相关 | [foreign-kernel.md](../design/foreign-kernel.md) · [notebook-session.md](../design/notebook-session.md) · [notebook-syntax.md](../design/notebook-syntax.md) · [foreign-notebook-decisions.md](../research/foreign-notebook-decisions.md) · [stdlib-foreign.md](../design/stdlib-foreign.md) · [notebook-session.md](notebook-session.md)（D 层路线） |
| 核心表面 Δ | 无 |

> 本路线图只覆盖 **F：外联内核运行时**。共享状态 / 单步 / 切换内核的文档侧在 [notebook-session.md](notebook-session.md)（D 层）。

---

## 0. 现状与目标

v1：`set_cmd` / `run` / `run_lang` / `langs`；临时文件 + 子进程 + stdout 文本；无状态、无结构化结果、无 exec 门禁、无有界 IO、无进程组清理。

v2 目标：安全、可发现、结构化、可持久、可并行，并成为 D 层文档会话的可靠底座。

---

## 1. 阶段总览

| 阶段 | 内容 | 估算 | 可独立发版 |
|---|---|---|---|
| **F0** | 执行内核与安全：Job/Execution、`exec` 门禁、有界 IO、进程组、`invoke` 记录、view 诊断 | 2–3 天 | 是 |
| **F1** | 语言注册与发现：registry、15 预设、probe、register/unset、配置、venv | 2–3 天 | 是 |
| **F2** | JSON 桥与并行：call/call_many、run_many、output/result | 2–3 天 | 是 |
| **F3** | 持久内核：Python/Node session_open/eval/close/list | 3–4 天 | 是 |
| **F3.5** | 状态桥：session_set/get/names | 1–2 天 | 是 |
| **F4** | 打磨、文档、catalog、CHANGELOG、1.5.0 | 1–2 天 | 1.5.0 |

D 层（文档会话）从 F3.5 完成后开始；不要把 D 层塞进 F 层。

---

## 2. F0 — 执行内核与安全

### 内容

- `src/host/foreign.rs` 重构：
  - `Job`：lang/source/stdin/input/env/cwd/timeout/args/output/max_output/cmd/keep。
  - `Execution`：stdout/stderr/raw bytes/exit/ms/timed_out/truncated/error。
  - `ForeignEnv`：cwd/fs_root/specs/caps 快照（供并行）。
- `exec` 门禁：`run` / `invoke` / `run_many` / `probe` / `session_*` 统一检查。
- 有界 IO：读线程保留前 N 字节、继续排空；`truncated`。
- stdin 写线程；EPIPE 忽略。
- 超时：进程组 kill（Unix `process_group(0)` + `kill(-pid)`；Windows `taskkill /T /F`）。
- shell-like argv 切分（引号/转义）。
- `invoke` + 稳定记录。
- `run` / `run_lang` 增加 `env` / `cwd` / `timeout` / `args` / `max_output`。
- view `/api/foreign-run` 返回完整记录；JS 显示 stderr / exit / ms。
- `set_cmd` L1 补 `args`。

### 验收

- 既有 `lib_foreign_*` / `lib_subtask_foreign` 绿（python 可用时）。
- 新增 node/bash 金样例：非 0 退出、stderr、超时、env/cwd、截断。
- `exec=false` 拒绝。
- `cargo test` 绿；release build；wasm check。

### 主要文件

`src/host/foreign.rs` · `src/host/mod.rs` · `src/host/subtask.rs` · `src/view/mod.rs` · `src/view/html.rs` · `src/view/debug_page.rs` · `Cargo.toml` · `tests/gold.rs` · `tests/lib/foreign-*.mq.md`。

---

## 3. F1 — 语言注册与发现

### 内容

- `src/foreign/registry.rs`（纯逻辑）：
  - 15 预设、别名、扩展名、版本参数、会话能力。
  - 模板占位符 `{file}` / `{dir}` / `{base}` / `{inline}` / `{lang}` / `{cwd}`。
  - venv 探测、PATH 查找、shell-like 切分、配置解析。
- `HostContext`：
  - `foreign_specs`、`foreign_config_loaded`。
  - `probe` 缓存（可选）。
- API：`probe` / `langs` / `register` / `unset` / `load_config`。
- 配置：`.marqdo/foreign.toml` + `MARQDO_FOREIGN_CONFIG`；优先级见设计。
- `MARQDO_FOREIGN_<LANG>` 支持 JSON 数组与引号。
- `subtask` 继承注册/配置。

### 验收

- `probe` 对 node / bash 给出 path + version；缺失语言 `ok=false` 不 panic。
- 配置可把 `python` 指向 venv；模板可跑 `go run`（若本机有 go）。
- `langs` 含预设；`unset` 回退预设。
- 单元测试：切分、模板、别名、扩展名。

---

## 4. F2 — JSON 桥与并行

### 内容

- `call` / `call_many`；`invoke` 的 `input` / `output` / `result` / `result_error`。
- `Value ↔ JSON` 复用 `host::json`；`bytes` base64。
- `run_many` 并行引擎：`ForeignEnv` 快照、线程上限 32、结果同序、单项 panic 隔离。
- `call_many` 全跑完再报错（清理所有子进程）。
- 文档：Python / Node JSON 样板。

### 验收

- `call` 单列表 → map，mean 为 Num；嵌套 list/map 往返。
- `run_many` 顺序稳定；单项失败不影响其它项。
- 大输入（>8 KiB）走 `MARQDO_INPUT_FILE`。
- 并发上限与超时生效。

---

## 5. F3 — 持久内核

### 内容

- `src/host/foreign_session.rs`：
  - Python / Node bootstrap（嵌入式常量）。
  - 启动、reader/writer、严格串行、超时、崩溃、drop 清理。
  - `session_open` / `session_eval` / `session_close` / `session_list`。
- 会话记录：与 invoke 同形 + `id` / `alive`。
- 会话环境：cwd / env / cmd；`MARQDO_*`。
- `HostContext` 字段（cfg exec-host）+ Drop。

### 验收

- Node：`let x=41` → `x+1` → 42；`console.log` 捕获；异常不杀会话。
- Python：同（有 python 时）。
- 超时 → kill + `alive=false`；close/drop 无遗留进程。
- 会话与 `subtask` 边界：不继承。

---

## 6. F3.5 — 状态桥

### 内容

- `session_set` / `session_get` / `session_names`。
- 值：Marqdo → JSON → kernel；kernel → JSON → Marqdo。
- 名称校验；Secret 规则；错误可读。
- 与 `session_eval` 同一线路，严格串行。
- L1 文档 + 金样例。

### 验收

- Node：`session_set xs=[1,2,3]`，`session_eval sum(xs)` → 6；`session_get` 取回。
- 复杂嵌套 map/list 往返。
- 不存在变量 → None + 可读诊断。
- 非 JSON 可序列化 → 错误说明。

---

## 7. F4 — 打磨与发布

- `history`（可选）/ `cache`（可选，默认关）。
- view 命令记忆 / Ctrl+Enter / 错误分块。
- `lib/foreign.mq.md` / `lib/外联.mq.md` 全量。
- `public/stdlib/*` 示例。
- catalog 重新生成；CHANGELOG；版本 1.5.0。
- 性能：空跑、100 次调用、会话 vs 一次性。

### 验收

- 新函数全部有中文/英文文档与示例。
- `marqdo catalog`、`public/` 构建通过。
- 手工 view Run 一个 node 卡片，显示 stdout/stderr/exit/ms。

---

## 8. 测试矩阵

> 当前环境：node / bash / sh / perl 可用，python3 不可用。新金样例以 node + bash 为主；python 用 `python_available()` 条件跳过。

| 测试 | 依赖 | 断言 |
|---|---|---|
| 既有 3 个 foreign 用例 | python | 兼容 |
| `foreign-invoke` | node | 记录字段 |
| `foreign-env-cwd` | node | env/cwd/MARQDO_* |
| `foreign-timeout` | node | timed_out |
| `foreign-call-json` | node | 类型往返 |
| `foreign-call-many` / `run-many` | node | 顺序/隔离 |
| `foreign-session` | node | 跨 eval |
| `foreign-session-set-get` | node | 状态桥 |
| `foreign-session-timeout` | node | alive=false |
| `foreign-probe` | node | path/version |
| `foreign-register` | node | 模板 |
| `foreign-config` | node | 配置 |
| `foreign-stdout-stderr` | bash | 成功也有 stderr |
| `foreign-caps` | 无 | 拒绝 |
| 单元测试 | 无 | 切分/模板/别名/JSON |

Windows：`python_available` / `bash_available` 守卫；进程组 `taskkill` 分支编译通过。

---

## 9. 兼容性

- `run` / `run_lang` / `set_cmd` / `langs` 兼容；只增可选参数。
- `run` 成功仍返回 stdout；非 0 仍报错。
- 大输出从“报错”改为“截断 + truncated”（更安全；CHANGELOG 写明）。
- `langs` 扩展为预设 ∪ 注册 ∪ 配置。
- HostFn id 末尾追加（196+）。
- 无核心表面 Δ。

---

## 10. 风险

| 风险 | 缓解 |
|---|---|
| 会话协议脆弱 | 原始 stdout；`-u`；超时 kill；测试；文档限制 |
| 并行进程过多 | 默认 4 / 上限 32；超时；输出上限 |
| 配置优先级难记 | 设计文档固定表；`probe` 显示实际 path |
| 模板 shell 注入 | 默认不 shell；路径引擎生成；文档警示 |
| 大输出截断改变 v1 行为 | CHANGELOG；`truncated` 可检测 |
| wasm 编译 | 新字段 cfg 隔离；registry 纯逻辑 |

---

## 11. 完成定义

- F0–F3.5 全部验收。
- `cargo test` 绿；release build；wasm check。
- 文档/CHANGELOG/catalog 更新。
- 提交并推送。
- 进入 D 层（[notebook-session.md](notebook-session.md)）。
