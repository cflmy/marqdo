# `lib/re` — 正则（Mid M1）

| | |
|---|---|
| 状态 | **已落地（M1）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 导入 | `lib/re.mq.md` · `lib/正则.mq.md` |
| Host | `host_re_*`（`regex` crate，默认 Unicode） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `is_match` | `匹配` | `text`, `pattern` | 布尔 |
| `find` | `查找` | `text`, `pattern` | 首个匹配文本，或 `None` |
| `find_all` | `查找全部` | `text`, `pattern` | 文本列表 |
| `replace` | `替换` | `text`, `pattern`, `with`, 可选 `count` | 文本（`count` 缺省 = 全部） |
| `split` | `拆分` | `text`, `pattern` | 文本列表 |

非法 pattern → 诊断失败（含 crate 错误信息）。

## 2. 语义

- 引擎：Rust [`regex`](https://docs.rs/regex)（无回溯引用；与 RE2 同族限制）。  
- 按 **UTF-8 文本**匹配，不做字节模式。  
- `find` / `find_all` 返回整段匹配（不含捕获组拆分；捕获组 v1 不做）。

## 3. 刻意不做（v1）

命名捕获、替换回调、`Regex` 对象缓存 API、字节模式。
