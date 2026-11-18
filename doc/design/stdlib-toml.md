# 设计：`lib/toml` / `lib/配置表`（Mid2 M8）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §3.1 |
| 宿主 | `src/host/toml_ops.rs` · `host_toml_parse`（手写子集，无 `toml` crate） |

---

## 1. API

| EN | ZH | 行为 |
|----|-----|------|
| `parse` | `解析` | TOML 文本 → map/list |

**不做** `stringify`；写出继续用 JSON。

## 2. 子集

支持：`#` 注释、基本/字面字符串、int/float/bool、标量数组、`[table]` 与 dotted key。

**不支持：** `[[array of tables]]`、内联表、TOML datetime、多行字符串。
