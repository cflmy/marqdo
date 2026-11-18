# 设计：`lib/fs` Mid2 加厚（M9）

| | |
|---|---|
| 状态 | **已落地** |
| 日期 | 2026-09-09 |
| 父设计 | [stdlib-mid2.md](stdlib-mid2.md) §4 · [stdlib-fs-mid.md](stdlib-fs-mid.md)（M2） |
| 宿主 | `src/host/fs.rs` · `host_make_dirs` / `host_remove_tree` / `host_stat` / `host_walk` |

---

## API

| EN | ZH | 行为 |
|----|-----|------|
| `make_dirs` | `建目录树` | 递归创建（与既有 `make_dir` 同为 `create_dir_all`；显式命名） |
| `remove_tree` | `删目录树` | 仅删目录树；路径为文件则报错 |
| `stat` | `状态` | `{size, mtime_unix, is_file, is_dir}` |
| `walk` | `遍历` | **深度优先**；相对路径列表（正斜杠）；含目录项；同级按名排序 |

既有 `remove` 对目录仍递归删除；`remove_tree` 拒绝文件，语义更窄。

## 刻意不做

ACL、watch、硬链接、`copy_tree`、惰性迭代器协议。
