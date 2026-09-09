# 设计：`lib/stats` / `lib/统计` 与 `log.fields`（Mid2 M10）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §5 |
| 宿主 | `src/host/stats.rs` · `host_stats_*`；`log::line` 可选 `fields` |

---

## 1. `lib/stats`

| EN | ZH | 行为 |
|----|-----|------|
| `mean` | `均值` | 算术平均 → Num |
| `median` | `中位数` | 排序后中位；偶数取两中点平均 |
| `stdev` | `标准差` | **样本**标准差（n≥2） |

**不做** dataframe、分位数全家桶、加权统计。

## 2. `log` `fields=`

各级别函数可选 `fields`（map）。宿主拼到行尾：`INFO msg key=val …`（map 插入序）。

**不做** 远程 sink、结构化 JSON 日志协议。

## 3. zip

按路线图 **默认跳过**。
