# 标准库（P2）

| | |
|---|---|
| 状态 | **S0–S3 落地** |
| 日期 | 2026-08-04 |
| 相关 | [keywords.md](keywords.md) · [call-arguments.md](call-arguments.md) · [stdlib-i18n.md](stdlib-i18n.md) · [stdlib-modules.md](stdlib-modules.md) · [roadmap/next-phase.md](../roadmap/next-phase.md) |

> **多语言库命名**：见 [stdlib-i18n.md](stdlib-i18n.md)（不设 `lang:`；中英分文件）。  
> **下一波模块（本波：文件 / 系统 / 时间 / JSON / 网络；数学·外联暂缓）**：见 [stdlib-modules.md](stdlib-modules.md)。

---

## 1. 分层

| 层 | 内容 |
|----|------|
| **L0 内置** | 解释器内：`print` `input` `len` `str` `int` `type` `trim` `split` `join` `at` |
| **L1 官方库** | 仓库根目录 [`lib/`](../../lib/)（`.mq.md`）；可用 `lib/…` 或别名 `std/…` 导入 |
| **L2** | catalog / 用户站收录（随发版） |

---

## 2. 内置一览（S0 + S1）

| 名 | 形参 | 结果 |
|----|------|------|
| `len` | `value` | 文本标量数或列表长度 |
| `str` | `value` | 显示文本 |
| `int` | `value` | 整数（失败 → 诊断） |
| `type` | `value` | `none`/`bool`/`int`/`text`/`list` |
| `trim` | `value` | 去首尾空白 |
| `split` | `value`, `sep` | 文本列表 |
| `join` | `value`, `sep` | 拼接文本 |
| `at` | `value`, `index` | 列表下标（越界 → `None`） |

表（单列表格）在运行时是 `list`：行数用 `len`，取行用 `at`。

---

## 3. 官方库导入（S2）

Frontmatter：

```markdown
---
> lib/text.mq.md
> lib/table.mq.md
---
```

`std/text.mq.md` 等价于 `lib/text.mq.md`。解析顺序：

1. 相对当前文件  
2. `MARQDO_LIB`（可指向 `lib` 目录或仓库根）  
3. 当前工作目录的 `lib/`  
4. 可执行文件附近的 `lib/`（含 `cargo run` 的相对路径）

当前模块：

| 路径 | 导出函数 |
|------|----------|
| `lib/fs.mq.md` / `lib/文件.mq.md` | `read_text`… / `读文本`… |
| `lib/time.mq.md` / `lib/时间.mq.md` | `now_unix`… / `此刻秒`… |
| `lib/sys.mq.md` / `lib/系统.mq.md` | `env_get`… / `取环境`… |
| `lib/json.mq.md` | `parse` `stringify` `get` `keys`（中英同文件） |
| `lib/net.mq.md` / `lib/网络.mq.md` | `http_get`… / `获取`… |
| `lib/text.mq.md` | `str_trim` `str_split` `str_join` |
| `lib/文本.mq.md` | `去空白` `拆分` `拼接` |
| `lib/table.mq.md` | `rows` `row_at` |
| `lib/表.mq.md` | `行数` `取行` |

规划中的 L1 扩展见 [stdlib-modules.md](stdlib-modules.md)。

---

## 4. 转换错误约定（S3）

用户可见失败统一为：

```text
path:line:col: message
```

内置转换相关文案（稳定子集，金样例断言）：

| 情况 | message |
|------|---------|
| 非法整数文本 | `cannot convert to int: "…"` |
| `int` 类型不对 | `int needs int, bool, or text` |
| `len` 类型不对 | `len needs text or list` |
| `trim` 非文本 | `trim needs text` |
| `split`/`join` 参数类型 | `split needs text value` 等 |
| 空分隔符 | `split sep must not be empty` |

不引入独立异常类型对象；一律走 `Diagnostic`。
