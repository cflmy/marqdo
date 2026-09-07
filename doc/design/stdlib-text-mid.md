# `lib/text` 加厚（Mid M4）

| | |
|---|---|
| 状态 | **已落地（M4）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-csv.md](stdlib-csv.md) · [stdlib-mid.md](stdlib-mid.md) |
| 导入 | `lib/text.mq.md` · `lib/文本.mq.md` |
| Host | `host_text_*`（Unicode 大小写 / 子串；非正则替换） |

---

## 1. 新增 API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `contains` | `包含` | `text`, `sub` | 布尔 |
| `starts_with` | `开头是` | `text`, `prefix` | 布尔 |
| `ends_with` | `结尾是` | `text`, `suffix` | 布尔 |
| `replace` | `替换` | `text`, `old`, `new`, 可选 `count` | 文本（字面替换，非正则） |
| `to_upper` | `大写` | `text` | 文本（Unicode） |
| `to_lower` | `小写` | `text` | 文本 |
| `repeat` | `重复` | `text`, `n` | 文本 |
| `pad` | `填充` | `text`, `width`, 可选 `fill`=`" "`，可选 `align`=`left`/`right`/`center` | 文本 |

保留原有 `str_trim` / `str_split` / `str_join`（及中文去空白/拆分/拼接）。

## 2. 刻意不做

正则替换（用 `lib/re`）、完整 `format` 模板引擎。
