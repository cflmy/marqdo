# 设计：中层标准库第二波（Mid2 · M7–M10）

| | |
|---|---|
| 状态 | **Draft · M7 已落地；M8–M10 锁定方向** |
| 日期 | 2026-09-09 |
| 缺口盘点 | [stdlib-gaps-after-mid.md](../research/stdlib-gaps-after-mid.md) |
| 路线图 | [stdlib-mid2.md](../roadmap/stdlib-mid2.md) |
| 宪法 | [stdlib-mid.md](stdlib-mid.md) §3（准入 / 不准入 **不变**） |
| 目标 | 在 M1–M6 之上补「日历 · URL · 配置 · HTML 安全 · fs 再厚」，仍保持宿主面积极小 |

---

## 0. 一句话

**Mid2 = 第二档通用原语，不是第二套 stdlib。** 仍落在 `lib/` + 薄 `host_*`；禁止借机引入 YAML 全家桶、HTTP 服务端、数据库。

---

## 1. 分层与禁止

| 允许 | 禁止 |
|------|------|
| 新 `lib/*.mq.md` 中英对偶 | 新开 `lib-mid2/` 目录树 |
| 少数 `host_datetime_*` / `host_url_*` / `host_toml_parse` | 为 Mid 新增 ABI 插件 |
| 加厚既有 `fs` / `encoding` / `net` / `text` | 把 `ext/web` 能力下沉进 `lib/net` 服务端 |
| TOML **只读** | 完整 YAML 1.2、任意对象反序列化 |
| HTML **转义** | 模板引擎 / CSS 选择器引擎 |

---

## 2. M7 — `datetime` + `url`

### 2.1 `lib/datetime.mq.md` / `lib/日期时间.mq.md`

| 函数（英） | 中文建议 | 行为 |
|------------|----------|------|
| `now` | `现在` | 返回结构化时刻（见下）或 ISO 文本（二选一，实现时锁定） |
| `from_unix` / `to_unix` | `自戳` / `成戳` | 秒级；可选 `ms` |
| `parse` | `解析` | 支持 RFC3339 / 常见 `YYYY-MM-DD[ HH:MM:SS]` |
| `format` | `格式化` | strftime 子集或命名样式 `rfc3339`/`date`/`time` |
| `add` | `加` | `days`/`hours`/`minutes`/`seconds` 具名 |
| `in_zone` | `时区` | IANA 名或 `+08:00`；失败 → 诊断 |

**结构化时刻（建议 map）**：`{unix, zone, iso}` 或宿主不透明柄 + `_type=datetime`。优先 **map + 稳定字段**，便于 GFM/table 协作。

**刻意不做**：农历、营业日、完整 tzdb 热更新 UI、闰秒政治。

### 2.2 `lib/url.mq.md` / `lib/地址.mq.md`（或并入 `net`）

| 函数 | 行为 |
|------|------|
| `parse` | → `{scheme, userinfo, host, port, path, query, fragment}` |
| `query_parse` | `a=1&b=2` → map（多值用列表） |
| `query_stringify` | map → query |
| `join` | base + relative（可选，可二期） |

既有 `net.url_encode` **保留**；文档写清：编码单段用 `net`/`encoding`，解析整 URL 用 `url`。

**验收**：金样解析带 query 的 HTTPS URL；往返 query。

---

## 3. M8 — `toml` + `html` + `encoding` 加厚

### 3.1 `lib/toml.mq.md` / `lib/Toml.mq.md`（或 `配置表`）

| 函数 | 行为 |
|------|------|
| `parse` | TOML 文本 → Marqdo map/list（只读） |

**不做** `stringify`（首版）；需要写出时继续用 JSON。动机对齐 PEP 680：配置引导，而非通用文档格式战争。

### 3.2 HTML 转义

二选一（实现时定一）：

- A：`lib/html.mq.md` · `escape` / `unescape`
- B：`text.html_escape` / `text.html_unescape`

推荐 **A**（安全语义独立可见）。

### 3.3 `encoding` 加厚

| 函数 | 行为 |
|------|------|
| `base32_encode` / `base32_decode` | RFC 4648 |
| （可选）`percent_decode` | 与 `url` 协作 |

---

## 4. M9 — `fs` 加厚

| 函数 | 行为 |
|------|------|
| `make_dirs` | 递归创建 |
| `remove_tree` | 递归删除（目录） |
| `stat` | `{size, mtime_unix, is_file, is_dir}` |
| `walk` | 返回路径列表（深度优先或广度；文档锁定） |

**不做**：权限 ACL、watch/inotify、硬链接全家桶。

---

## 5. M10 — 收口糖（可选）

| 项 | 说明 |
|----|------|
| `lib/stats` 或 `math.mean`/`median`/`stdev` | 极薄；非 pandas |
| `log` 结构化字段 | 可选 `fields=` map；非完整 slog 生态 |
| `zip` 读写 | **默认跳过**；有明确 Release 需求再开 |

---

## 6. 宿主与 WASM

| Host | WASM |
|------|------|
| `datetime` / `url` / `toml` / `html` / `fs.stat` | 尽量可进；IANA tz 数据体积需评估（可 host-only + 文档标注） |
| `fs.walk` / `remove_tree` | host-only 可接受 |

新 crate 写进 [dependencies.md](dependencies.md)；禁止顺手拉大型框架。

---

## 7. 作者面风格

- 中英分文件；形参表声明全部可选并转发 `host_*`。  
- 数据优先 GFM / `table`；`json` 仍只 parse/stringify/quote。  
- 每波：设计短文 → host → lib → `tests/lib/*` → `public/features` 短页。

---

## 8. 完成定义（Mid2 整包）

1. 作者可用 `datetime` + `url` + `toml.parse` + `html.escape` 写配置驱动脚本，无需 foreign。  
2. `fs.stat` / `make_dirs` 金样绿。  
3. 研究文高优先级缺口标 closed；YAML/模板/sqlite 仍标明 won't。  
4. [stdlib-modules.md](stdlib-modules.md) 总表更新；`stdlib.md` 状态改为 Mid2 进行中/完成。
