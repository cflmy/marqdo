# `lib/csv` — 逗号表（Mid M4）

| | |
|---|---|
| 状态 | **已落地（M4）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-text-mid.md](stdlib-text-mid.md) · [stdlib-mid.md](stdlib-mid.md) |
| 导入 | `lib/csv.mq.md` · `lib/逗号表.mq.md` |
| Host | `host_csv_*`（RFC4180 最小集，无第三方 crate） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `parse` | `解析` | `text` | **list of maps**：首行为表头 |
| `stringify` | `序列化` | `rows` | CSV 文本（表头 = 首行 map 的键顺序） |

## 2. 方言（v1）

- 分隔符 `,`；引号 `"`；`""` 转义。  
- 换行 `\n` / `\r\n`；忽略末尾空行。  
- 字段内可含逗号与换行（须加引号）。  
- **不**支持自定义分隔符 / 无表头模式（可后续加）。

## 3. 刻意不做

流式大文件、类型推断、Excel 方言。
