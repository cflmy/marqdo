# `lib/encoding` — 编解码（Mid M1）

| | |
|---|---|
| 状态 | **已落地（M1）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [stdlib-re.md](stdlib-re.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 导入 | `lib/encoding.mq.md` · `lib/编码.mq.md` |
| Host | `host_encoding_*`（无第三方 crate：标准 Base64 / hex） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `base64_encode` | `基64编码` | `text` | 文本（UTF-8 字节 → 标准 Base64，含 `=` 填充） |
| `base64_decode` | `基64解码` | `text` | 文本（解码后须为合法 UTF-8） |
| `hex_encode` | `十六进制编码` | `text` | 小写 hex |
| `hex_decode` | `十六进制解码` | `text` | 文本（偶数字节；合法 UTF-8） |

## 2. 与 `net.url_encode` 的边界

| 能力 | 归属 |
|------|------|
| `application/x-www-form-urlencoded` 风格 | `net.url_encode` |
| Base64 / hex | **本库** |
| 百分号编解码通用版 | 可日后加 `encoding.percent_*`；本波不做 |

## 3. 刻意不做（v1）

URL-safe Base64 变体开关、忽略空白的宽松解码、二进制 `bytes` 类型（仍经 UTF-8 文本往返）。
