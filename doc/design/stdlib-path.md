# `lib/path` — 路径代数（Mid M2）

| | |
|---|---|
| 状态 | **已落地（M2）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 导入 | `lib/path.mq.md` · `lib/路径.mq.md` |
| Host | `host_path_*`（`std::path`；无 I/O、不经 `fs_root`） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `join` | `拼接` | `a`, `b` | 文本路径 |
| `split` | `拆分` | `path` | 组件文本列表（去空段） |
| `file_name` | `文件名` | `path` | 最后一段；根路径可为空文本 |
| `parent` | `父目录` | `path` | 父路径文本；无父则 `"."` |
| `extension` | `扩展名` | `path` | 不含点；无扩展名为 `""` |
| `normalize` | `规范化` | `path` | 清理 `.` / `..`（纯字符串，不访问磁盘） |
| `is_absolute` | `是绝对` | `path` | 布尔 |

## 2. 语义

- 分隔符与绝对路径判定跟随 **宿主 OS**（Linux CI 为 `/`）。  
- **不做** I/O；逃逸检查仍由 `lib/fs` 在读写时执行。  
- `join`：若 `b` 为绝对路径，结果为 `b`（同 `Path::join`）。

## 3. 刻意不做（v1）

`glob`、符号链接解析、`relative_to`、Windows UNC 专项文档。
