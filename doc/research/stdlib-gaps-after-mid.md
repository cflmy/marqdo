# 调研：Mid M1–M6 之后标准库还缺什么

| | |
|---|---|
| 状态 | **调研笔记 · 缺口盘点（M1–M6 闭环之后）** |
| 日期 | 2026-09-08 |
| 基线 | **v0.3.7+** · Mid M1–M6 **done** |
| 已落地设计 | [stdlib-mid.md](../design/stdlib-mid.md) · [stdlib-modules.md](../design/stdlib-modules.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 补齐设计 | [stdlib-mid2.md](../design/stdlib-mid2.md) |
| 实现路线 | [stdlib-mid2.md](../roadmap/stdlib-mid2.md) |
| 外部对照 | Python stdlib（[use-case 索引](https://www.socratopia.app/library/python-programming-ai-era-en/chapter-15) · [PEP 680 tomllib](https://peps.python.org/pep-0680/)）· Go `strings`/`regexp`/`encoding/*`/`crypto`/`path`/`time`/`net/url` · Node 内置 vs userland |

---

## 0. 一句话结论

**M1–M6 已把「写日常脚本 / 多数扩展库不再事事插件」的硬缺口补齐。** 相对 Python / Go 的「通用原语层」，Marqdo **不再缺** 正则、编解码、路径、哈希、CSV、集合糖、CLI、日志、UUID。

仍缺的是 **第二档通用能力**：日历与时区、URL/查询串、配置格式（优先 TOML）、HTML 安全转义、文件系统再厚一点（`mkdir -p` / 元数据 / 递归删除）、轻量统计。这些**值得开 Mid2（M7+）**，但**不应**再开一轮「事事插件」；也**不要**把 YAML 全家桶、HTTP 服务端、SQLite、图像/ML 塞进 `lib/`。

---

## 1. 已有基线（不是缺口）

| 族 | Marqdo | 对标 |
|----|--------|------|
| 文本 / 正则 | `text`++ · `re` | Python `str`/`re` · Go `strings`/`regexp` |
| 编解码 | `encoding`（base64/hex）· `net.url_encode` | Go `encoding/base64`/`hex` · Python `base64`/`binascii` |
| 路径 / 文件 | `path` · `fs`（含 copy/move/temp） | Go `path/filepath` · Python `pathlib`（子集） |
| 完整性 / 机密 | `hash` · `secrets` | Go `crypto` 子集 · Python `hashlib`/`secrets` |
| 表数据 | `csv` · `table`++ · `json` | Go `encoding/csv`/`json` · Python `csv`/`json` |
| CLI / 可观测 | `cli` · `log` · `uuid` | Python `argparse`/`logging`/`uuid`（窄） |
| 时间（极简） | `time`：unix/ms · format · parse · sleep | **远薄于** Python `datetime`/`zoneinfo` · Go `time` |
| 网络客户端 | `net`：HTTP/SSE/cookie/multipart/markdown | Go `net/http` **客户端**侧；服务端在 `ext/web` |
| 数学 | `math`（含公式/作图） | Python `math` + 教学向扩展 |

准入宪法仍见 [stdlib-mid.md](../design/stdlib-mid.md) §3：跨领域、难写对、解锁无插件写库、宿主面积极小。

---

## 2. 对照总表（按「脚本作者是否还要 foreign / 自造轮子」）

| 能力 | Python / Go 常态 | Marqdo 现状 | 缺口级别 | Mid2 建议 |
|------|------------------|-------------|----------|-----------|
| 日历日期 / 时区 / timedelta | `datetime`+`zoneinfo` · `time` | 仅 unix 戳 + 简单 format/parse | **高** | **M7** `datetime` |
| URL 解析 / 查询串 | `urllib.parse` · `net/url` | 仅 `url_encode` | **高** | **M7** `url`（或并入 `net`） |
| 配置 TOML 只读 | `tomllib`（PEP 680） | 无 | **中高** | **M8** `toml`（只读优先） |
| 配置 YAML | 第三方；**故意不进** CPython | 无 | **低 / 慎入** | 默认 **不做**；与 Python 同理（复杂度/安全） |
| HTML / XML 转义 | `html` · `html/template` 转义 | 无（`markdown_parse` ≠ 转义） | **中** | **M8** `html` 或 `text.html_escape` |
| `mkdir -p` / 递归删 / stat | `pathlib`/`shutil` | `make_dir`/`remove` 偏浅 | **中** | **M9** `fs` 加厚 |
| 文件元数据（size/mtime） | `os.stat` · `Path.stat` | 无一等 API | **中** | **M9** |
| base32 / percent 完整 | `base64.b32*` · `url` | 无 base32 | **低–中** | **M8** `encoding` 加厚 |
| 轻量统计 | `statistics` | `math` 无 mean/median | **低–中** | **M10** `stats` 或 `math` 薄加 |
| 计数 / 默认字典 / deque | `collections` | `table` 可凑 | **低** | 优先用 `table`；必要时 M10 `collections` 极薄 |
| 组合迭代 | `itertools` | 无 | **低** | 多数可用 `table`；**延后** |
| 模板引擎 | Jinja / Go `text/template` | 无 | **不准入 Mid** | 应用层 / 未来 ext |
| SQLite 客户端 | `sqlite3` · `database/sql` | **`ext/web` `# db`** | **不准入** | 保持扩展 |
| HTTP 服务端 | — | **`ext/web`** | **不准入** | 已关闭（宿主波次另文） |
| 密码哈希 Argon2 | 第三方 / 框架 | `ext/web` 已有 | **不准入 Mid** | 安全产品面 |
| 完整 TLS/证书 API | `ssl` · `crypto/tls` | 客户端 HTTPS 由 ureq | **不准入** | 宿主实现细节即可 |
| 压缩 zip/tar | `zipfile` · `archive/zip` | 无 | **低** | 可选 M10；体积敏感 |
| 环境 `.env` | 第三方；Go 常自写 | **`sys.load_dotenv` 已有** | 无 | — |

---

## 3. 分项说明（高优先级）

### 3.1 日历 / 时区（M7 候选之首）

**问题**  
脚本与站点大量需要「今天」「本地日界」「RFC3339」「时区换算」。现有 `time.parse`/`format` 够演示，不够「难写对」的日历语义（闰秒除外，但 DST / offset 已够劝退字符串拼接）。

**对照**  
Python 强调 **aware datetime + zoneinfo**；Go `time.Location` 是日常依赖。Mid 原文已把「完整 datetime 时区库」标为刻意延后——M1–M6 完成后应升格。

**建议窄 API（设计文锁定）**  
`now` / `parse` / `format` / `add`（天/时）/ `to_unix` / `from_unix` / 可选 `in_zone`（IANA 或固定 offset）。**不做**完整历法 UI、农历、业务假期库。

### 3.2 URL 与查询串

**问题**  
`url_encode` 只解决「编码一段」；解析 `https://a/b?x=1&y=2#frag`、合并 query、取 path 仍靠 `re` 自造。浏览器代理 / `invoke` / webhook 场景高频。

**建议**  
`url.parse` → `{scheme,host,path,query,fragment}`；`url.query_parse` / `query_stringify`；与 `url_encode` 边界写进 `net` vs 新 `lib/url`。

### 3.3 TOML vs YAML

**外部事实**  
Python 因 **打包引导**（`pyproject.toml`）接受只读 `tomllib`（[PEP 680](https://peps.python.org/pep-0680/)）；YAML 多次被拒：规范复杂、实现难维护、安全面大。

**对 Marqdo**  
catalog / 工具配置若走向 TOML，只读 `toml.parse` 性价比高。YAML：**默认不准入 Mid**；若 public 强需求，单独 ADR，且优先「JSON 超集子集」而非完整 1.2。

### 3.4 HTML 转义

**问题**  
`ext/web` 与浏览器 WASM 拼 HTML 时，缺 `escape` 易 XSS。这是「难写对 / 安全相关」的经典 stdlib 项（Python `html.escape`）。

**建议**  
极薄：`html.escape` / `unescape`（或挂 `text`）。**不做** DOM / 模板语言。

### 3.5 文件系统再厚

**问题**  
`make_dir` 是否递归、`remove` 是否递归、无 `stat`（size/mtime）、无 `walk`——脚本仍会 `foreign` 或脆弱字符串。

**建议**  
`fs.make_dirs` · `fs.remove_tree` · `fs.stat` · 可选 `fs.walk`（返回路径列表，非惰性迭代器协议）。

---

## 4. 明确「不是缺口」

| 项 | 说明 |
|----|------|
| Mid M1–M6 清单本身 | 已收口；不要用本表重开 base64/re |
| HTTP 服务 / DB / Agent | `ext/*`；见 [ext-hosting](ext-hosting-gaps.md) |
| Dense embedding / 向量库默认化 | OKF 路线不变 |
| 完整 `collections` / `itertools` 类型系统 | 语言没有泛型迭代协议；用 `table` |
| 「对齐 Python 模块个数」 | 反模式；对齐**用例与难写对** |

---

## 5. 优先级建议（转入 Mid2）

| 优先级 | 波次草案 | 内容 |
|--------|----------|------|
| **高** | **M7** | `datetime`（日历/时区子集）+ `url`（parse/query） |
| **高** | **M8** | `toml` 只读 + `html.escape` + `encoding` base32 |
| **中** | **M9** | `fs` 加厚（dirs/stat/walk） |
| **低** | **M10** | `stats` 薄 · 可选 zip · 结构化 log 字段 |
| **won't** | — | 完整 YAML · 模板引擎 · sqlite 进 `lib/` |

---

## 6. 与其它文档

| 文档 | 关系 |
|------|------|
| [stdlib-mid2.md](../design/stdlib-mid2.md) | **补齐设计（作者 API + 边界）** |
| [stdlib-mid2.md](../roadmap/stdlib-mid2.md) | 实现波次 M7–M10 |
| [stdlib-mid.md](../design/stdlib-mid.md) | Mid1 宪法；本调研不改准入原则 |
| [web-net-capabilities.md](../design/web-net-capabilities.md) | 网络服务端能力在 ext，不进 Mid2 |
