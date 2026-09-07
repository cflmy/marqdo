# `lib/fs` 加厚 — 复制 / 移动 / 临时文件（Mid M2）

| | |
|---|---|
| 状态 | **已落地（M2）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-path.md](stdlib-path.md) · [stdlib-mid.md](stdlib-mid.md) |
| 边界 | **文本与普通文件**；不引入 `bytes` 类型（`read_bytes` 延后） |

---

## 1. 新增 API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `copy_file` | `复制文件` | `src`, `dest` | `None`（覆盖已存在的目标文件） |
| `move` | `移动` | `src`, `dest` | `None`（rename；跨设备失败则报错） |
| `make_temp` | `建临时` | 可选 `prefix` | 相对 `cwd`/`fs_root` 的新空文件路径文本 |

均走现有 `resolve_path` 沙箱；写操作受 `fs_write` 策略约束。

## 2. 刻意不做

目录树 `copy_tree`、原子替换、`read_bytes`、权限 chmod。
