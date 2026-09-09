# 路线图：中层标准库第二波（M7–M10）

| | |
|---|---|
| 状态 | **Active · M7–M8 已落地 · M9–M10 规划** |
| 日期 | 2026-09-09 |
| 设计 | [stdlib-mid2.md](../design/stdlib-mid2.md) |
| 缺口 | [stdlib-gaps-after-mid.md](../research/stdlib-gaps-after-mid.md) |
| 前序 | [stdlib-mid.md](stdlib-mid.md)（**M1–M6 done**） |
| 基线 | v0.3.7+ |

> **不预标 SemVer。** 波次名 **M7…M10**；每波可单独演示、可回滚。

---

## 0. 总序

```text
M7  时间与地址        datetime + url
M8  配置与安全字面    toml(只读) + html.escape + encoding/base32
M9  文件再厚         fs make_dirs / remove_tree / stat / walk
M10 收口糖（可选）    stats 薄 · log 字段 · （zip 默认跳过）
```

原则：

1. 继承 Mid1 宪法：通用、难写对、小宿主、可冻 API。  
2. **TOML 优先于 YAML**（对齐 PEP 680 教训）。  
3. 不阻塞宿主 / ext 产品线；但新「通用」ABI 插件默认拒绝。

---

## 1. 波次明细

### M7 — datetime + url — **done**

| 交付 | 说明 |
|------|------|
| `lib/datetime` · `日期时间` | now/parse/format/add/unix/zone 子集 |
| `lib/url` · `地址` | parse · query_parse/stringify（手写；无 `url` crate） |
| Host | `host_datetime_*` · `host_url_*` |
| 金样 | `tests/lib/datetime-smoke` · `url-smoke`（中英） |
| public | `14-datetime-url` · `14-日期时间地址` |
| 设计 | [stdlib-datetime.md](../design/stdlib-datetime.md) · [stdlib-url.md](../design/stdlib-url.md) |

### M8 — toml + html + encoding — **done**

| 交付 | 说明 |
|------|------|
| `lib/toml` · `配置表` | 只读子集 `parse` |
| `lib/html` · `超文本` | `escape` / `unescape` |
| `encoding` base32 | RFC 4648 |
| 金样 | `toml-html-encoding-smoke`（中英） |
| public | `15-toml-html-encoding` · `15-配置表超文本编码` |
| 设计 | [stdlib-toml.md](../design/stdlib-toml.md) · [stdlib-html.md](../design/stdlib-html.md) |

### M9 — fs 加厚 — **todo**

| 交付 | 说明 |
|------|------|
| `make_dirs` / `remove_tree` / `stat` / `walk` | 见设计文 |
| 金样 | `fs-mid2-smoke` |

### M10 — 可选收口 — **todo**

| 交付 | 说明 |
|------|------|
| `stats` 或 `math` 均值/中位数 | 薄 |
| `log` 可选 `fields=` | 非远程 sink |
| zip | **默认跳过** |

---

## 2. 进度勾选

- [x] M7 datetime + url  
- [x] M8 toml + html + base32  
- [ ] M9 fs 加厚  
- [ ] M10 可选  
- [ ] 研究文高优先级缺口 → closed  
- [ ] `stdlib-modules.md` / `stdlib.md` 状态同步  

---

## 3. 成功标准

对外可宣称：在 Mid1（文本/路径/哈希/CSV/CLI/日志）之上，**日历、URL、TOML 配置、HTML 转义、实用 fs** 无需外联即可完成常见脚本与 catalog 工具——领域能力仍在 `ext/`。
