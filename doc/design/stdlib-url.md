# 设计：`lib/url` / `lib/地址`（Mid2 M7）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §2.2 |
| 宿主 | `src/host/url_ops.rs` · `host_url_*`（手写子集，无 `url` crate） |

---

## 1. 与 `net.url_encode`

| API | 用途 |
|-----|------|
| `net.url_encode` | **单段** form 编码（空格→`+`） |
| `url.parse` | 整 URL → `{scheme,userinfo,host,port,path,query,fragment}` |
| `url.query_parse` / `query_stringify` | 查询串 ↔ map（重复键→列表；空格→`%20`） |

## 2. 解析子集

`scheme://[userinfo@]host[:port]/path[?query][#fragment]`，含 IPv6 `[addr]`。缺 path 时规范为 `/`。

**不做**：相对 URL join、完整 WHATWG 边角、IDNA 规范化（host 原样保留）。
