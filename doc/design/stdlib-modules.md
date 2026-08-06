# 标准库下一波（L1 扩展模块）

| | |
|---|---|
| 状态 | **五库 + 数学已落地**；下一波：**外联**（[stdlib-foreign.md](stdlib-foreign.md)） |
| 日期 | 2026-08-05 |
| 本波范围 | 文件 / 系统 / 时间 / JSON / 网络 / 数学（已完成） |
| 下一波 | **外联**（本机解释器 / 具名围栏） |
| 暂缓 | — |
| 原则 | 全部经 frontmatter **导入**；除 JSON 外中英分文件；内核保持少而精 |
| 相关 | [stdlib.md](stdlib.md) · [stdlib-i18n.md](stdlib-i18n.md) · [keywords-i18n.md](keywords-i18n.md) · [view.md](view.md) |

---

## 1. 目标

在现有 `lib/text`·`lib/文本`、`lib/table`·`lib/表` 之上，把 Marqdo 推进到可写日常脚本。标准库**必须导入**才可用。

| 模块族 | 英文库 | 中文库 | 本波 |
|--------|--------|--------|------|
| 文本（已有） | `lib/text.mq.md` | `lib/文本.mq.md` | 已有 |
| 表（已有） | `lib/table.mq.md` | `lib/表.mq.md` | 已有 |
| **文件** | `lib/fs.mq.md` | `lib/文件.mq.md` | **做** |
| **系统** | `lib/sys.mq.md` | `lib/系统.mq.md` | **做** |
| **时间** | `lib/time.mq.md` | `lib/时间.mq.md` | **做** |
| **JSON** | `lib/json.mq.md` | **同文件（特例，§3.4）** | **做** |
| **网络（简）** | `lib/net.mq.md` | `lib/网络.mq.md` | **做** |
| 数学 | `lib/math.mq.md` | `lib/数学.mq.md` | **下一波** |
| 外联 | `lib/foreign.mq.md` | `lib/外联.mq.md` | **做** |

---

## 2. 分层与实现分工

```text
用户 .mq.md
    │  > lib/fs.mq.md
    ▼
L1 官方库（.mq.md）
    │
    ▼
L0.5 宿主原语（Rust，英文蛇形）
    │
    ▼
OS / HTTP / 时钟 / …
```

| 层 | 谁实现 | 用户是否直接调用 |
|----|--------|------------------|
| **L0** | 解释器内置 | 是（`print`/`len`/…） |
| **L0.5 宿主原语** | Rust（`src/builtin` 或 `src/host/`） | **原则上否**；供 L1 包装；金样例可直测 |
| **L1 官方库** | `lib/*.mq.md` | **是**（文档与用户站只教这一层） |

规则：

1. **导入即选择语言**（JSON 除外）：`lib/fs.mq.md` → 英文 API；`lib/文件.mq.md` → 中文 API（见 [stdlib-i18n.md](stdlib-i18n.md)）。  
2. **不设 `lang:`**。  
3. 宿主原语保持英文；中英差异只存在于 L1 包装层（JSON 无第二套包装）。  
4. 失败一律 `path:line:col: message`（[stdlib.md](stdlib.md) §4）。

---

## 3. 核心模块 API 草图（L1）

### 3.1 文件 — `fs` / `文件`

| 英文 | 中文 | 形参（英 / 中） | 结果 |
|------|------|-----------------|------|
| `read_text` | `读文本` | `path` / `路径` | 文本 |
| `write_text` | `写文本` | `path`, `text` / `路径`, `内容` | `None` |
| `append_text` | `追加文本` | 同上 | `None` |
| `exists` | `存在` | `path` / `路径` | 布尔 |
| `list_dir` | `列目录` | `path` / `路径` | 文本列表 |
| `make_dir` | `建目录` | `path` / `路径` | `None` |
| `remove` | `删除` | `path` / `路径` | `None` |

约定：路径相对工作目录或绝对路径；二进制 I/O 本波不做；写/删默认需 `--allow-fs-write`（§6）。

### 3.2 系统 — `sys` / `系统`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `env_get` | `取环境` | `name` / `名` | 文本或 `None` |
| `env_set` | `设环境` | `name`, `value` / `名`, `值` | `None` |
| `load_dotenv` | `加载环境` | 可选 `path` | 新写入的键数量；缺文件 → `0`；不覆盖已有环境变量 |
| `args` | `参数表` | （无） | 文本列表 |
| `cwd` | `工作目录` | （无） | 文本 |
| `exit` | `退出` | `code` / `码` | 不返回 |
| `exec` | `执行` | `cmd`, 可选 `args` | 退出码（实现时写死语义） |

`exec` 默认关（`--allow-exec`）；view / 静态站永不开放。

### 3.3 时间 — `time` / `时间`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `now_unix` | `此刻秒` | （无） | 整数（Unix 秒） |
| `now_ms` | `此刻毫秒` | （无） | 整数 |
| `format` | `格式化` | `unix`, `pattern` / `秒`, `格式` | 文本 |
| `sleep_ms` | `睡眠毫秒` | `ms` / `毫秒` | `None` |
| `parse` | `解析时间` | `text`, `pattern` / `内容`, `格式` | 整数秒或诊断失败 |

view 中 `sleep_ms` 设上限（如 5s）。

### 3.4 JSON — `json`（**中英同库特例**）

JSON 是专有缩写，**很难也不必要**做合规中文库名/API 翻译。本波定稿：

| 项 | 约定 |
|----|------|
| 唯一路径 | **`lib/json.mq.md`**（中英文档均导入此路径） |
| 函数名 | 国际通行英文：`parse` / `stringify` / `get` / `keys` |
| 形参 | 英文：`text` / `value` / `key` / `indent` |
| 不设 | `lib/数据.mq.md`、音译名、中文函数双份 |

这是对 [stdlib-i18n.md](stdlib-i18n.md)「一概念两文件」的**显式例外**；其它模块仍中英分文件。

| 函数 | 形参 | 结果 |
|------|------|------|
| `parse` | `text` | 值（见下） |
| `stringify` | `value`, 可选 `indent` | JSON 文本 |
| `get` | `value`, `key` | 值或 `None` |
| `keys` | `value` | 文本列表 |
| `quote` | `text` | JSON 字符串字面量（含引号，便于拼请求体） |

**类型映射（草案）**：

| JSON | Marqdo Value |
|------|----------------|
| `null` | `None` |
| bool | bool |
| number（整数可表） | int |
| number（其余） | 文本（本波不引入 float）或诊断拒绝 |
| string | text |
| array | list |
| object | 倾向新宿主类型 **`map`**（`type` → `map`）；否则体验差 |

`map` 为 L0.5 类型扩展，可与 json 同期或紧随；不阻塞 `fs`/`time` 先行。

### 3.5 网络（简）— `net` / `网络`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `http_get` | `获取` | `url` / `地址` | 响应文本（或 map：status/body） |
| `http_post` | `提交` | `url`, `body`, 可选 `content_type` | 同上 |
| `url_encode` | `编码地址` | `text` / `内容` | 文本 |

仅 **HTTP / HTTPS**（ureq）；短超时、响应体上限；默认 CLI 开网络。可选具名实参 `headers=`（map）、`content_type=`；`http_post` 默认 JSON Content-Type。

**路径字面量：** 具名实参里 `path=a/b` 会被解析成除法。请用无斜杠文件名，或先赋给变量再传入。

官方可选扩展（非 stdlib）：[`ext-llm.md`](ext-llm.md)（`ext/llm` OpenAI 兼容聊天）。

---

## 4. 中英路径总表（本波）

```text
lib/
  text.mq.md / 文本.mq.md       # 已有
  table.mq.md / 表.mq.md         # 已有
  fs.mq.md / 文件.mq.md          # 本波
  sys.mq.md / 系统.mq.md
  time.mq.md / 时间.mq.md
  json.mq.md                     # 本波；中英共用，无第二文件
  net.mq.md / 网络.mq.md
  # 暂缓：math / 数学 · foreign / 外联
```

别名：`std/…` ≡ `lib/…`。

---

## 5. 落地分期（本波 = M1–M5）

| 期 | 内容 | 完成定义 |
|----|------|----------|
| **M1** | 宿主能力框架：权限旗标、`host` 分发、错误文案 | `--allow-*` 可测 |
| **M2** | `fs` + `time`（只读 fs + 时钟） | 中英库 + 金样例 |
| **M3** | `sys`（env/cwd/args；`exec` 默认关） | 同上 |
| **M4** | `json`（+ 可选 `map`） | 往返金样例；唯 `lib/json.mq.md` |
| **M5** | `net`（默认关） | 本地 mock 或录制夹具 |

**不做 M6/M7**（数学 / 外联）：详稿仅存档，待五库稳定后再议。

文档站：`fs`/`sys`/`time`/`net` 英中各一页；`json` 可一页双语叙述、同一导入路径。

---

## 6. 安全与 view / 静态导出

### 6.1 行为一致

`marqdo run`、live `view`、`view output` 对标准库 **默认同一套能力**（读/写盘、HTTP、exec 均可用）。导入库即表示要用；不再要求 `--allow-*`。

差异仅在 **软退出**：

| | CLI `run` | view / capture / 静态导出 |
|--|-----------|---------------------------|
| `exit` | 结束进程 | 报诊断，不杀 view/构建进程 |
| `sleep_ms` | 默认可睡（有上限） | live 有上限；导出通常为 0 |

沙箱：相对路径落在源文件目录（或 view 根）下，防止 `..` 逃逸。

### 6.2 静态部署

访客浏览器 **不再执行** Marqdo；HTML 里是构建时冻结的输出。源码中的路径/URL 访客改不了、也触发不了新的 host 调用。审 PR 时审查写死的目标即可。

勿在静态页嵌入「用户可改 URL 再请求」的浏览器脚本。

---

## 7. 与现有库的关系

- `text` / `table` 继续纯包装、无 IO。  
- JSON `stringify` ≠ `str`（显示）。

---

## 8. 刻意不做（本波）

- 数学库、外联/胶水库（及其依赖、权限旗标实现）。  
- 数据库、异步事件循环、WebSocket、原始 TCP。  
- float/复数进 L0。  
- `lang:` 或按 locale 自动选库。  
- 为 JSON 强造中文库名或中文 API 双份。

---

## 9. 一句话

**本波只落地文件 / 系统 / 时间 / JSON / 网络；JSON 中英共用 `lib/json.mq.md`；数学与外联暂缓。**
