# `lib/log` + `lib/uuid`（Mid M6）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |

---

## 1. 日志 `lib/log` / `lib/日志`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `set_level` | `设级别` | `level`（`debug`/`info`/`warn`/`error`） | `None` |
| `debug` / `info` / `warn` / `error` | `调试` / `信息` / `警告` / `错误` | `text` | 达级别则 `print` 一行 `LEVEL message`；否则无输出 |

默认级别：`info`（隐藏 `debug`）。中文级别别名：`调试`/`信息`/`警告`/`错误`。

实现：宿主记最小级别；`host_log_line` 返回待打印文本或 `None`；L1 再 `print`（接入现有 stdout / capture）。

## 2. UUID `lib/uuid` / `lib/标识`

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `v4` | `版本4` | （无） | 标准 8-4-4-4-12 小写 hex（`getrandom`） |

## 3. 刻意不做

结构化 JSON 日志、远程 sink、UUID v1/v7、yaml。
