# 标准库（P2）

| | |
|---|---|
| 状态 | **S0–S3 + L1 全模块落地** |
| 日期 | 2026-08-06 |
| 相关 | [keywords.md](keywords.md) · [objects.md](objects.md) · [call-arguments.md](call-arguments.md) · [stdlib-i18n.md](stdlib-i18n.md) · [stdlib-modules.md](stdlib-modules.md) · [ext-abi.md](ext-abi.md) |

> **多语言库命名**：见 [stdlib-i18n.md](stdlib-i18n.md)（不设 `lang:`；中英分文件）。  
> **L1 模块清单**（含 math/foreign/plugin）：见 [stdlib-modules.md](stdlib-modules.md)。  
> **`ext/` 不是 stdlib**：官方可选扩展见 [ext-llm.md](ext-llm.md) / [ext-agent.md](ext-agent.md)。

---

## 1. 分层

| 层 | 内容 |
|----|------|
| **L0 内置** | 解释器内：`print` `input` `len` `str` `int` `type` `trim` `split` `join` `at`（`type` 可含 `map`） |
| **L1 官方库** | 仓库根目录 [`lib/`](../../lib/)（`.mq.md`）；可用 `lib/…` 或别名 `std/…` 导入 |
| **官方 ext** | [`ext/`](../../ext/)（`.mq.md` + 可选原生插件）；`MARQDO_EXT`；**不属于** L1 |
| **L2** | catalog / 用户站收录（随发版） |

---

## 2. 内置一览（S0 + S1）

| 名 | 形参 | 结果 |
|----|------|------|
| `len` | `value` | 文本标量数或列表长度 |
| `str` | `value` | 显示文本 |
| `int` | `value` | 整数（失败 → 诊断） |
| `type` | `value` | `none`/`bool`/`int`/`text`/`list`/`map` |
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
5. **内嵌 stdlib**（v0.1.2+，编译进二进制；无磁盘 `lib/` 时自动使用）

当前模块（API 多为 `##` 自由函数；见 [objects.md](objects.md)）：

| 路径 | 导出 |
|------|------|
| `lib/fs.mq.md` / `lib/文件.mq.md` | `read_text`… / `读文本`… |
| `lib/time.mq.md` / `lib/时间.mq.md` | `now_unix`… / `此刻秒`… |
| `lib/sys.mq.md` / `lib/系统.mq.md` | `env_get`…、`load_dotenv`… / `取环境`…、`加载环境`… |
| `lib/json.mq.md` | `parse` `stringify` `get` `keys` `quote`（中英同文件） |
| `lib/net.mq.md` / `lib/网络.mq.md` | `http_get`…（HTTPS）/ `获取`… |
| `lib/math.mq.md` / `lib/数学.mq.md` | 数值 / 随机 / 公式 / 绘图 |
| `lib/foreign.mq.md` / `lib/外联.mq.md` | 具名围栏外联 |
| `lib/plugin.mq.md` / `lib/插件.mq.md` | `load` / `unload` / `list`（原生 ABI） |
| `lib/writeback.mq.md` / `lib/自写回.mq.md` | `record` `get` `clear` `list` |
| `lib/subtask.mq.md` / `lib/子任务.mq.md` | `spawn` `poll` `wait` `kill` `wait_all` |
| `lib/text.mq.md` | `str_trim` `str_split` `str_join` |
| `lib/文本.mq.md` | `去空白` `拆分` `拼接` |
| `lib/table.mq.md` | `rows` `row_at` |
| `lib/表.mq.md` | `行数` `取行` |

完整清单见 [stdlib-modules.md](stdlib-modules.md)。

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
