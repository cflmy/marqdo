# 设计：`lib/datetime` / `lib/日期时间`（Mid2 M7）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §2.1 |
| 宿主 | `src/host/datetime.rs` · `host_datetime_*` |

---

## 1. 时刻形状

每次返回 **map**：

| 键 | 含义 |
|----|------|
| `unix` | 秒级 Unix 戳（int） |
| `zone` | 固定偏移文本，如 `+08:00` / `+00:00` |
| `iso` | RFC3339（秒精度） |

`to_unix` 也接受裸 int 或 ISO 文本。

## 2. API

| EN | ZH | 说明 |
|----|-----|------|
| `now` | `现在` | UTC 此刻（wasm 无时钟桥则失败） |
| `from_unix` | `自戳` | `unix` + 可选 `zone` |
| `to_unix` | `成戳` | |
| `parse` | `解析` | 默认 RFC3339 / `YYYY-MM-DD[ HH:MM:SS]`；`pattern=` |
| `format` | `格式化` | `rfc3339`/`date`/`time` 或 strftime |
| `add` | `加` | `days`/`hours`/`minutes`/`seconds` |
| `in_zone` | `时区` | 同瞬间换区 |

**区名**：`+HH:MM` / `UTC` / `Z`，以及少量固定偏移别名（如 `Asia/Shanghai`）。**不做**完整 tzdb / DST。

## 3. 与 `lib/time`

`lib/time` 保留极简 unix/format/parse/sleep。日历语义用本库。
