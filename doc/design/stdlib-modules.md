# 标准库 L1 模块

| | |
|---|---|
| 状态 | **已落地**：text/table/fs/sys/time/json/net/math/foreign/**plugin** |
| 日期 | 2026-08-07 |
| 原则 | 全部经 frontmatter **导入**；除 JSON 外中英分文件；内核保持少而精 |
| 相关 | [stdlib.md](stdlib.md) · [stdlib-i18n.md](stdlib-i18n.md) · [ext-abi.md](ext-abi.md) · [ext-llm.md](ext-llm.md) · [ext-agent.md](ext-agent.md) · [**module-namespace.md**](module-namespace.md)（导入命名空间；**M1/M2 已落地**：`库.成员` / `库.对象.成员` / `use`） |

---

## 1. 目标

在 `lib/text`·`lib/文本`、`lib/table`·`lib/表` 之上，把 Marqdo 推进到可写日常脚本与可选原生扩展。标准库**必须导入**才可用。

| 模块族 | 英文库 | 中文库 | 状态 |
|--------|--------|--------|------|
| 文本 | `lib/text.mq.md` | `lib/文本.mq.md` | 已有 |
| 表 | `lib/table.mq.md` | `lib/表.mq.md` | 已有 |
| 文件 | `lib/fs.mq.md` | `lib/文件.mq.md` | 已有 |
| 系统 | `lib/sys.mq.md` | `lib/系统.mq.md` | 已有（含 `load_dotenv`） |
| 时间 | `lib/time.mq.md` | `lib/时间.mq.md` | 已有 |
| JSON | `lib/json.mq.md` | **同文件** | 已有（含 `quote`） |
| 网络 | `lib/net.mq.md` | `lib/网络.mq.md` | 已有（HTTPS + headers） |
| 数学 | `lib/math.mq.md` | `lib/数学.mq.md` | 已有 |
| 外联 | `lib/foreign.mq.md` | `lib/外联.mq.md` | 已有 |
| **插件** | `lib/plugin.mq.md` | `lib/插件.mq.md` | **已有**（加载 C ABI 共享库） |
| **自写回** | `lib/writeback.mq.md` | `lib/自写回.mq.md` | **已有**（Jupyter 式输出写回） |
| **子任务** | `lib/subtask.mq.md` | `lib/子任务.mq.md` | **已有**（OS 子进程 + KillOnDrop） |

官方可选扩展（**非** stdlib）：`ext/llm` · `ext/agent` — 见 [ext-llm.md](ext-llm.md) / [ext-agent.md](ext-agent.md)。

设计细节：[stdlib-writeback.md](stdlib-writeback.md) · [stdlib-subtask.md](stdlib-subtask.md)。

---

## 2. 分层与实现分工

```text
用户 .mq.md
    │  > lib/fs.mq.md  /  > ext/llm.mq.md
    ▼
L1 官方库 / 官方 ext（.mq.md）
    │
    ▼
L0.5 宿主原语（Rust）或 原生插件（ABI v1）
    │
    ▼
OS / HTTP / 时钟 / 共享库 …
```

| 层 | 谁实现 | 用户是否直接调用 |
|----|--------|------------------|
| **L0** | 解释器内置 | 是（`print`/`len`/…） |
| **L0.5 宿主原语** | Rust（`src/host/`） | **原则上否**；供 L1 包装；ABI v2 `host_query` 仅插件可调 |
| **L1 官方库** | `lib/*.mq.md` | **是** |
| **官方 ext** | `ext/*.mq.md`（+ 可选 ABI 插件） | **是**（需显式导入 `ext/`）；**禁止** `host_*`，经 `lib/*` 或插件注册名 |

> **已变更（M1+M2）**：[module-namespace.md](module-namespace.md) — 取消跨文件扁平合并；L1/ext 经**裸名点号路径**调用（`time.parse`、`agent.agent`；M2：`库.对象.成员` + `use`），库名不加反引号；实例方法须 `` `var`.method ``；不再靠 import 顺序覆盖同名。

### 2.1 L1 薄包装 → 宿主（命名空间后必读）

点号路径 `net.http_post` / `json.stringify` 等解析到 **`lib/*.mq.md` 的 `##` 函数**，再转发 `host_*`。未在包装形参表声明的可选实参会被**静默丢弃**（曾导致 live agent 401：`Authorization` 等 `headers` 从未到达宿主）。

| 规则 | 说明 |
|------|------|
| 每个 `host_*` 包装 | 须在 `+` 形参表声明**全部**宿主可选参数，默认 `None`，并在 `**> host_…**` 中**原样转发** |
| `http_post` / `提交` | `content_type=None` 时宿主按默认 JSON charset 处理 |
| 已对齐（2026-08-07） | `net`/`网络`（headers、content_type、body）、`sys`/`系统`（load_dotenv、exec）、`ext/ai/llm`（load_env / 加载环境 path）、`json.stringify` indent、`foreign`/`外联` stdin、`math`/`数学` plot*/solve 可选项 |

**无**扁平合并回退；**无**裸名双轨兼容。

---

## 3. 核心模块 API（摘要）

文件 / 系统 / 时间 / JSON / 网络 API 见下文历史草图（已实现）。补充：

### 3.2 系统 — 已含 dotenv

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `load_dotenv` | `加载环境` | 可选 `path` | 新写入的键数量；缺文件 → `0`；不覆盖已有环境变量 |

### 3.4 JSON — 已含 `quote`

| 函数 | 形参 | 结果 |
|------|------|------|
| `quote` | `text` | JSON 字符串字面量（含引号） |

### 3.5 网络 — HTTPS

仅 **HTTP / HTTPS**（ureq）；可选 `headers=`（map）、`content_type=`。

### 3.6 插件 — `plugin` / `插件`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `load` | `加载` | `path` / `路径` | 注册函数个数 |
| `unload` | `卸载` | （无） | `None` |
| `list` | `列出` | （无） | 已注册名列表 |

契约：[ext-abi.md](ext-abi.md)。路径受 cwd/fs_root 沙箱约束。

### 3.1–3.5 历史 API 草图

#### 文件 — `fs` / `文件`

| 英文 | 中文 | 形参（英 / 中） | 结果 |
|------|------|-----------------|------|
| `read_text` | `读文本` | `path` / `路径` | 文本 |
| `write_text` | `写文本` | `path`, `text` / `路径`, `内容` | `None` |
| `append_text` | `追加文本` | 同上 | `None` |
| `exists` | `存在` | `path` / `路径` | 布尔 |
| `list_dir` | `列目录` | `path` / `路径` | 文本列表 |
| `make_dir` | `建目录` | `path` / `路径` | `None` |
| `remove` | `删除` | `path` / `路径` | `None` |

#### 系统 — `sys` / `系统`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `env_get` | `取环境` | `name` / `名` | 文本或 `None` |
| `env_set` | `设环境` | `name`, `value` / `名`, `值` | `None` |
| `load_dotenv` | `加载环境` | 可选 `path` | 见上 |
| `args` | `参数表` | （无） | 文本列表 |
| `cwd` | `工作目录` | （无） | 文本 |
| `exit` | `退出` | `code` / `码` | 不返回 |
| `exec` | `执行` | `cmd`, 可选 `args` | 退出码 |

#### 时间 — `time` / `时间`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `now_unix` | `此刻秒` | （无） | 整数（Unix 秒） |
| `now_ms` | `此刻毫秒` | （无） | 整数 |
| `format` | `格式化` | `unix`, `pattern` / `秒`, `格式` | 文本 |
| `sleep_ms` | `睡眠毫秒` | `ms` / `毫秒` | `None` |
| `parse` | `解析时间` | `text`, `pattern` / `内容`, `格式` | 整数秒或诊断失败 |

#### JSON — `json`

| 函数 | 形参 | 结果 |
|------|------|------|
| `parse` | `text` | 值 |
| `stringify` | `value`, 可选 `indent` | JSON 文本 |
| `get` | `value`, `key` | 值或 `None` |
| `keys` | `value` | 文本列表 |
| `quote` | `text` | JSON 字符串字面量 |

#### 网络 — `net` / `网络`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `http_get` | `获取` | `url` / `地址` | 响应文本 |
| `http_post` | `提交` | `url`, `body`, 可选 `content_type` / `headers` | 同上 |
| `url_encode` | `编码地址` | `text` / `内容` | 文本 |

**路径字面量：** 具名实参里 `path=a/b` 会被解析成除法。请用无斜杠文件名，或先赋给变量再传入。

---

## 4. 中英路径总表

```text
lib/
  text.mq.md / 文本.mq.md
  table.mq.md / 表.mq.md
  fs.mq.md / 文件.mq.md
  sys.mq.md / 系统.mq.md
  time.mq.md / 时间.mq.md
  json.mq.md                     # 中英共用
  net.mq.md / 网络.mq.md
  math.mq.md / 数学.mq.md
  foreign.mq.md / 外联.mq.md
  plugin.mq.md / 插件.mq.md

ext/   # 非 stdlib
  llm.mq.md / 大模型.mq.md
  agent.mq.md / 智能体.mq.md
```

别名：`std/…` ≡ `lib/…`。扩展解析：`MARQDO_EXT` / `./ext` / 二进制旁 `ext/`。

---

## 5. 安全与 view / 静态导出

`marqdo run`、live `view`、`view output` 对标准库 **默认同一套能力**。差异仅在软退出 / sleep 钳制。相对路径落在源文件目录（或 view 根）下，防止 `..` 逃逸。插件路径同沙箱。

---

## 6. 一句话

**L1 含文件 / 系统 / 时间 / JSON / 网络 / 数学 / 外联 / 插件；`ext/` 为可选官方扩展（llm、agent）。**
