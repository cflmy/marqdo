# 外联内核（Foreign Kernel）

| | |
|---|---|
| 状态 | **F0–F3.5 已实现** |
| 日期 | 2026-10-17 |
| 层 | Runtime（`src/foreign/` · `src/host/foreign*` · `lib/foreign.mq.md`） |
| 相关 | [foreign-notebook-decisions.md](../research/foreign-notebook-decisions.md) · [notebook-syntax.md](notebook-syntax.md) · [notebook-session.md](notebook-session.md) · [stdlib-foreign.md](stdlib-foreign.md) · [stdlib-subtask.md](stdlib-subtask.md) |
| 核心表面 Δ | 无 |

> 外联内核 = 一个长期存活的外部语言进程 + 生命周期 + 类型桥 + 记录。它既能被 `marqdo run` 里的显式调用使用，也能被 view 的 notebook 会话使用。

---

## 1. 定位与边界

### 1.1 负责

- 解释器发现、注册、配置、探测。
- 一次性执行：临时脚本、stdin/stdout/stderr、超时、输出上限。
- 持久内核：启动、求值、状态、关闭、列表。
- 状态桥：`session_set` / `session_get` / `session_names`。
- JSON 值桥：Marqdo `Value` ↔ JSON。
- 结构化记录：`ok/code/stdout/stderr/ms/timed_out/truncated/result/error`。
- 安全：`exec` 门禁、`cwd` 沙箱、进程组、并发/输出上限。
- 视图诊断：代码卡片 Run 的完整结果。

### 1.2 不负责

- 文档语法（`内核表` / `会话表`）→ [notebook-syntax.md](notebook-syntax.md)。
- 文档会话 / cell 顺序 / journal / view UI → [notebook-session.md](notebook-session.md)。
- Foreign → Marqdo 函数回调（v2 不做）。
- 依赖安装（pip/npm）。
- 远程 / 容器执行。

### 1.3 与相邻模块的区别

| 模块 | 生命周期 | 用途 |
|---|---|---|
| `foreign` | 一次性或持久 | 调外部语言；状态可选 |
| `subtask` | 一次性并发 | 后台任务、文件子任务、函数子任务 |
| `plugin` | 进程内 | 原生 C ABI 扩展 |
| `writeback` | 文档级 | 把输出写回 `.mq.md` |

`subtask` 不管理持久状态；`foreign` 不管理文档执行顺序。

---

## 1.4 实施状态（2026-10-17）

| 阶段 | 状态 | 关键文件 |
|---|---|---|
| F0 安全执行 / `invoke` 记录 | ✅ | `src/host/foreign.rs` |
| F1 预设 / probe / register / config | ✅ | `src/foreign/registry.rs` · `src/host/foreign.rs` |
| F2 JSON 桥 / run_many / call_many | ✅ | `src/host/foreign.rs` |
| F3 Python/Node 持久内核 | ✅ | `src/host/foreign_session.rs` |
| F3.5 session_set/get/names | ✅ | 同上 |
| F4 history / view 打磨 / 发布 | 部分 | — |

测试：`cargo test --lib foreign`（11+）、`cargo test --test gold lib_foreign`（12，python 条件跳过）、`tests/notebook/shared.mq.md` 手工 view 验证。

---

## 2. 语言与内核模型

### 2.1 语言预设

| lang | 别名 | 默认命令 | 扩展名 | 版本探测 | 会话 |
|---|---|---|---|---|---|
| `python` | `python3`, `py` | `python3`（Windows `python`） | `.py` | `--version` | ✅ |
| `node` | `javascript`, `js` | `node` | `.js` | `-v` | ✅ |
| `deno` | — | `deno run` | `.ts` | `--version` | ❌ |
| `bun` | — | `bun` | `.js` | `--version` | ❌ |
| `ruby` | — | `ruby` | `.rb` | `--version` | ❌ |
| `perl` | — | `perl` | `.pl` | `--version` | ❌ |
| `php` | — | `php -r {inline}` | `.php` | `--version` | ❌ |
| `lua` | — | `lua` | `.lua` | `-v` | ❌ |
| `r` | `rscript` | `Rscript` | `.R` | `--version` | ❌ |
| `julia` | — | `julia` | `.jl` | `--version` | ❌ |
| `go` | — | `go run {file}` | `.go` | `version` | ❌ |
| `bash` | `sh`, `shell` | `bash` | `.sh` | `--version` | ❌ |
| `powershell` | `pwsh`, `ps1` | `pwsh`（Windows `powershell`） | `.ps1` | `--version` | ❌ |
| `sqlite` | `sqlite3` | `sqlite3` | `.sql` | `--version` | ❌ |
| `tsx` | `typescript` | `tsx` | `.ts` | `--version` | ❌ |

说明：

- 预设只是默认值；注册 / 配置 / 环境变量整体覆盖。
- 会话能力首批只有 Python / Node；其它语言一次性执行。
- `php` 用 `{inline}`，围栏里直接写 PHP，不需要 `<?php`。
- `deno` 默认不给权限；需要时通过配置追加 `--allow-*`。

### 2.2 解析优先级

从高到低：

1. 调用级 `cmd`（整行 argv，shell-like 切分）+ `args`（追加）。
2. 进程内注册：`register` / `set_cmd`。
3. 环境变量 `MARQDO_FOREIGN_<LANG>`（整行或 JSON 数组）。
4. 项目配置 `.marqdo/foreign.toml` / `MARQDO_FOREIGN_CONFIG`。
5. 内置预设。
6. 都没有 → 带修复建议的诊断。

Python 额外发现顺序（在预设层内）：

1. `VIRTUAL_ENV/bin/python`（Windows `Scripts/python.exe`）；
2. `<cwd>/.venv` / `venv` / `env`；
3. 入口目录与 `fs_root` 向上查找；
4. PATH 的 `python3` / `python`。

`MARQDO_FOREIGN_NO_VENV=1` 关闭 venv 探测；显式 `cmd` / `set_cmd` / `MARQDO_FOREIGN_PYTHON` 优先。

相对路径：配置里的 `cmd` 相对配置目录；注册 / `set_cmd` / 环境变量里的 `cmd` 相对 `cwd`。

### 2.3 注册与模板

`register(lang, cmd?, args?, template?, ext?, env?, version_args?)`。

模板占位符：

| 占位符 | 展开 |
|---|---|
| `{file}` | 临时脚本绝对路径 |
| `{dir}` | 临时目录 |
| `{base}` | 脚本文件名 |
| `{inline}` | 源码作为一个 argv；不写临时文件 |
| `{lang}` | 语言 id |
| `{cwd}` | 实际工作目录 |

规则：

- `cmd` 与 `template` 至少一个；都给 → 报错。
- `{file}` 与 `{inline}` 不能同时出现。
- 都没有时自动追加 `{file}`。
- `{inline}` 超过 60 KiB → 报错并建议 `{file}`。
- 模板不做 shell 转义；需要 shell 时显式写 `bash -c`，路径由引擎生成（安全字符）。

配置示例：

```toml
[python]
cmd = ".venv/bin/python"
args = ["-u"]

[go]
template = ["go", "run", "{file}"]

[rust]
template = ["bash", "-c", "rustc -O {file} -o {dir}/main && {dir}/main"]

[php]
template = ["php", "-r", "{inline}"]
```

---

## 3. 一次性执行

### 3.1 API

| 英文 | 中文 | 形参 | 结果 |
|---|---|---|---|
| `run` | `运行` | `code`, 可选 `stdin` `env` `cwd` `timeout` `args` `max_output` | stdout 文本（v1 兼容） |
| `run_lang` | `按语言运行` | `lang`, `source`, 可选同上 | stdout 文本 |
| `invoke` | `执行` | 可选 `code` / `lang`+`source` / `stdin` / `input` / `env` / `cwd` / `timeout` / `args` / `output` / `max_output` / `cmd` / `keep` / `cache` | 记录 map |
| `call` | `调用` | 可选 `code` / `lang`+`source` / `input` / `stdin` / `env` / `cwd` / `timeout` / `args` | 解析后的 JSON 值 |
| `run_many` | `批量运行` | `runs`, 可选 `concurrency` | 记录列表 |
| `call_many` | `批量调用` | `inputs`, 可选 `code` / `lang`+`source` / `cmd` / `env` / `cwd` / `timeout` / `args` / `max_output` / `concurrency` | 结果值列表 |

### 3.2 记录结构

`invoke` / `run_many` / `session_eval` 共用：

| 键 | 类型 | 含义 |
|---|---|---|
| `ok` | Bool | exit=0 且未超时 |
| `code` | Int | 退出码；未启动 / kill → -1 |
| `stdout` | Text | 原始 stdout（可能截断） |
| `stderr` | Text | 原始 stderr（可能截断） |
| `ms` | Int | 墙钟毫秒 |
| `lang` | Text | 实际语言 |
| `cmd` | Text | 实际 argv |
| `timed_out` | Bool | 是否超时 |
| `truncated` | Bool | stdout/stderr 是否截断 |
| `result` | 任意 / None | 按 `output` 解析 |
| `result_error` | Text | json 解析失败原因 |
| `error` | Text | 启动 / 超时 / 策略原因 |
| `script` | Text | `keep=True` 的脚本路径 |
| `cached` | Bool | 缓存命中（P4） |

`session_eval` 额外：`id`、`alive`。

### 3.3 输出模式

| `output=` | `result` |
|---|---|
| `auto`（默认） | 尝试 JSON；失败 → None |
| `text` | None |
| `json` | 解析；失败 → `result_error` |
| `lines` | 行列表 |
| `bytes` | stdout 原始字节的 base64 |

`call` 固定 JSON：空 stdout → None；非空非法 JSON → 错误。

### 3.4 失败语义

- `run` / `call`：执行失败 → `Err`；错误包含语言、argv、退出码、stderr（截 4 KiB）、stdout 头。
- `invoke` / `run_many`：执行失败 → 记录；只有参数非法 / 策略拒绝 → `Err`。
- 输出过大：截断 + `truncated=true`，不再 OOM / 报错。

---

## 4. JSON 状态桥

### 4.1 值映射

| Marqdo | JSON |
|---|---|
| None | null |
| Bool | bool |
| Int | number |
| Num | number（NaN/Inf → null） |
| Text | string |
| List | array |
| Map | object |
| Formula | 矩阵 → 二维数组；其它 → 数字或字符串 |
| Code | 源码字符串 |
| Secret | JSON 中 `<secret>`；`env` 显式交付时取真实值 |

### 4.2 输入

- `input=` → JSON；写 stdin；同时提供 `MARQDO_INPUT`（≤8 KiB）与 `MARQDO_INPUT_FILE`（任意大小）。
- `stdin=` 文本与 `input=` 互斥。
- 都不给 → stdin EOF。
- 大输入走文件，避免环境变量大小限制。

### 4.3 输出

- `result` 按 `output` 解析。
- 原始 stdout/stderr 始终保留（截断到上限）。
- `call` 返回解析值；`invoke` 返回记录。

### 4.4 类型限制

v2 是**值拷贝**，不是对象引用：

- Python DataFrame / NumPy / Node 对象 → 需要先 `to_dict` / `to_json` / `JSON.stringify`。
- 大对象传输成本高；后续考虑 Arrow / Parquet / 文件句柄。
- Secret 不默认进入 JSON；需要 env 显式交付。

---

## 5. 持久内核与会话

### 5.1 API

| 英文 | 中文 | 形参 | 结果 |
|---|---|---|---|
| `session_open` | `会话开启` | `lang`, 可选 `cmd` `env` `cwd` `timeout` | 会话 id（Text） |
| `session_eval` | `会话求值` | `id`, 可选 `code` / `source` / `input` / `timeout` | 记录（含 id/alive） |
| `session_set` | `会话设值` | `id`, `name`, `value` | None |
| `session_get` | `会话取值` | `id`, `name` | JSON 值 / None |
| `session_names` | `会话变量` | `id` | 名称列表 |
| `session_close` | `会话关闭` | `id` | Bool |
| `session_list` | `会话表` | — | map 列表 |

### 5.2 支持语言

| 语言 | 方式 |
|---|---|
| python | `python3 -u -c <bootstrap>` |
| node | `node -e <bootstrap>` |
| 其它 | 报错：`lang=… has no session bootstrap; use run/invoke` |

### 5.3 线路协议

stdin 每行一个 JSON 请求；原始 stdout 每行一个 JSON 响应：

```json
{"id":1,"code":"x = 41","input":null}
{"id":2,"code":"x + 1","input":null}
```

```json
{"id":1,"ok":true,"stdout":"","stderr":"","result":null,"error":""}
{"id":2,"ok":true,"stdout":"","stderr":"","result":42,"error":""}
```

- 同一会话严格串行：等到匹配响应后才发下一条。
- 响应单行；JSON 转义换行。
- 崩溃 / EOF → `alive=false`，会话移除。
- 超时 → kill 会话（状态不可信）。

### 5.4 Python bootstrap

- 保留原始 `sys.stdout` 写协议；用户输出用 `redirect_stdout/stderr` 捕获。
- `ast.parse`：最后一条是表达式 → 先 exec 前面，再 eval 最后，作为 result。
- 异常 → `traceback.format_exc(limit=8)`，`ok=false`；会话继续。
- `input` → 全局 `marqdo_input`。
- 不可序列化 → `default=str`。

### 5.5 Node bootstrap

- 保存原始 `process.stdout.write` 写协议。
- eval 期间替换 `console.*` 与 `process.stdout/stderr.write`；结束后恢复。
- `vm.runInThisContext` 保留全局状态；`require` 挂到 `globalThis`。
- Promise → await；顶层 await 需用户包 `(async()=>{…})()`。
- 序列化失败 → `String(result)`。

### 5.6 状态桥

- `session_set`：值 → JSON → 请求 `{"set": name, "value": ...}` 或执行一段赋值。
- `session_get`：请求 `{"get": name}` → JSON → Marqdo Value。
- `session_names`：请求变量名列表。
- 与 `session_eval` 同一线路协议；严格串行。
- 名称必须是合法标识符；保留名（`marqdo_input` 等）另行文档。

### 5.7 生命周期

- id 为 `s1`、`s2`…。
- 每会话独立进程、cwd、env。
- `HostContext` drop → 全部 kill + wait。
- `subtask` 不继承会话。
- 不跨文档共享。

---

## 6. 环境契约

| 变量 | 值 |
|---|---|
| `MARQDO_FOREIGN` | `1` |
| `MARQDO_FOREIGN_LANG` | 语言 id |
| `MARQDO_FOREIGN_SCRIPT` | 脚本路径（文件模式） |
| `MARQDO_FOREIGN_DIR` | 脚本目录 |
| `MARQDO_FOREIGN_CWD` | 实际 cwd |
| `MARQDO_ENTRY` | 入口 `.mq.md` 路径 |
| `MARQDO_ENTRY_DIR` | 入口目录 |
| `MARQDO_INPUT` | input JSON（≤8 KiB） |
| `MARQDO_INPUT_FILE` | input JSON 文件（任意大小） |

语言默认：

- Python：`PYTHONUNBUFFERED=1`、`PYTHONIOENCODING=utf-8`、`PYTHONUTF8=1`；`PYTHONPATH` 前插 cwd。
- Node：`NODE_PATH` 前插 cwd（可选）。

`env` 是 map；值 `as_display()`；Secret 取真实值（显式交付）。不主动清空父环境。

---

## 7. 临时文件与 cwd

- 默认 `<cwd>/.marqdo/tmp/foreign-<pid>-<nanos>/`；不可写 → 系统临时目录。
- `MARQDO_FOREIGN_TMPDIR` 覆盖。
- 脚本 `main.<ext>`；Python 加 utf-8 声明。
- 默认 cwd = 入口目录；调用级 cwd 经 `resolve_path` + `fs_root` 校验。
- 成功/失败/超时都清理；`keep=True` 保留并回填 `script`。
- Node 相对 `require('./x')` 相对脚本目录；需要项目相对路径时用 `MARQDO_FOREIGN_CWD` 或 `{inline}` 模板。

---

## 8. 超时、IO、进程组

- `timeout`：秒，可小数；默认 10s；上限 3600s；`<=0` 报错。
- 超时 kill 进程组：
  - Unix：`process_group(0)` + `kill(-pid, SIGKILL)`；
  - Windows：`taskkill /PID <pid> /T /F`。
- `max_output`：每流默认 1 MiB；范围 1 KiB–64 MiB；保留前 N 字节，余下丢弃并继续排空；`truncated=true`。
- stdin 独立写线程，避免双向管道死锁；EPIPE 忽略。
- stdout/stderr 两个读线程；UTF-8 lossy。
- 记录原始字节（至上限）以支持 `bytes` 输出。

---

## 9. 安全

| 项 | 策略 |
|---|---|
| 能力门禁 | `run` / `invoke` / `probe` / `session_*` 全部检查 `allow_exec()`；沙箱 → 拒绝 |
| shell | 默认不经 shell；模板显式 `bash -c` 才走 |
| cwd | `resolve_path` + `fs_root`；越界拒绝 |
| 输出 | 有界读取 + 丢弃式排空 |
| 时间 | 上限 + 进程组 kill |
| 并发 | 默认 4，上限 32 |
| 会话 | 每文档独立；关闭/drop 清理；超时即杀 |
| Secret | JSON 不默认暴露；env 显式；面板打码 |
| wasm | `exec-host` 关闭时全部返回清晰错误；`langs` 返回 [] |

---

## 10. 兼容性

- `run` / `run_lang` / `set_cmd` / `langs` 既有调用可用。
- `run` 成功仍返回 stdout（去一个尾换行）；非 0 仍报错。
- `set_cmd` 增加可选 `args`。
- `langs` 扩展为「预设 ∪ 注册 ∪ 配置」；原集合是子集。
- `subtask` 的 `spawn code=` / `lang=` 仍返回 stdout 文本。
- HostFn id 只追加；字节码参数 < 255。
- 无核心表面 Δ。

---

## 11. 测试矩阵

| 测试 | 依赖 | 断言 |
|---|---|---|
| 既有 `foreign-python` / `run-lang` / `外联-python` | python | 兼容 |
| `foreign-invoke` | node | ok/code/stdout/stderr/ms |
| `foreign-env-cwd` | node | env/cwd/MARQDO_* |
| `foreign-timeout` | node | timed_out, 无遗留进程 |
| `foreign-call-json` | node | 嵌套 list/map 往返 |
| `foreign-call-many` / `run-many` | node | 顺序/隔离/并发 |
| `foreign-session` | node（python 可选） | 跨 eval 状态、异常不杀 |
| `foreign-session-set-get` | node | 变量推拉 |
| `foreign-session-timeout` | node | alive=false |
| `foreign-probe` | node | path/version/ok |
| `foreign-register` | node | 模板 `{inline}` |
| `foreign-config` | node | `MARQDO_FOREIGN_CONFIG` |
| `foreign-caps` | 无 | exec=false 拒绝 |
| 单元测试 | 无 | shell 切分/模板/别名/JSON |

当前环境无 Python：node + bash 为主；python 条件跳过；Windows 用 `python_available()` / `bash_available()` 守卫。

---

## 12. 实现落点

| 文件 | 内容 |
|---|---|
| `src/foreign/registry.rs` | 预设/别名/模板/venv/PATH/配置/shell 切分（纯逻辑） |
| `src/host/foreign.rs` | Job/Execution/ForeignEnv；run/invoke/call/run_many/call_many；probe/register/unset/langs/load_config |
| `src/host/foreign_session.rs` | Python/Node bootstrap；会话生命周期；set/get/names |
| `src/host/dispatch.rs` | HostFn 追加 |
| `src/aliases.rs` | 参数别名 |
| `lib/foreign.mq.md` / `lib/外联.mq.md` | L1 API + 文档 |
| `src/view/mod.rs` / `html.rs` / `debug_page.rs` | Run 诊断；后续接文档会话 |
| `tests/gold.rs` / `tests/lib/foreign-*.mq.md` | 测试 |

---

## 13. 明确不做

- Foreign line-level step。
- Foreign → Marqdo 函数 callback。
- 自动镜像变量。
- 跨语言共享对象引用。
- 全进程快照。
- 自动安装依赖。
- 浏览器 wasm 外联。
