# 设计：`lib/html` / `lib/超文本`（Mid2 M8）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §3.2 |
| 宿主 | `src/host/html_ops.rs` · `host_html_escape` / `host_html_unescape` |

---

## API

| EN | ZH | 行为 |
|----|-----|------|
| `escape` | `转义` | `& < > " '` → 实体（`'` → `&#x27;`） |
| `unescape` | `还原` | 上述实体（含 `&#39;` / `&apos;`） |

**不做** DOM / 模板引擎。
