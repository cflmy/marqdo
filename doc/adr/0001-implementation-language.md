# ADR 0001：参考实现语言与 Markdown 前端

| | |
|---|---|
| 状态 | **Spike 通过（Python）** — 参考实现语言待勾选 |
| 日期 | 2026-08-04 |
| 决策者 | chaungming + 协作 |
| 相关 | [tech-stack.md](../design/tech-stack.md) · [markdown-mapping.md](../design/markdown-mapping.md) · [spike/REPORT.md](../../spike/REPORT.md) |

---

## 背景

语法宪法 v0.1 已定。需要选定第一版参考实现的实现语言与 Markdown 前端，以便跑通 `examples/*.mq.md`，并用测试锁定行首判别、`*`/`**`、`+`/`-`、表格、frontmatter `>`。

目标是验证宪法，不是一次定终身的生产编译器。

---

## 约束（验收必须满足）

1. 能区分无序列表的 `-` vs `+` 项目符（分支/循环）。
2. GFM 表格、emphasis / strong（`*` / `**`）可解析。
3. 能做行预分类（无标记=注释）再 lower 到 Marqdo AST。
4. frontmatter 中非纯 YAML 的 `> file.mq.md` 行可解析。
5. CLI + 自动化测试可对照 `examples/` 期望输出。

---

## 选项

| ID | 方案 | 验证期适配 | 长期编译器适配 |
|----|------|------------|----------------|
| A | TypeScript + remark/mdast + Vitest | 高 | 中 |
| B | Rust + comrak / pulldown-cmark | 中 | 高 |
| C | Python + markdown-it-py + pytest | 高 | 中（可作参考实现） |
| D | Go + goldmark | 中 | 中 |

详见 [tech-stack.md](../design/tech-stack.md)。

---

## 已执行：Python Spike

因 npm 过慢，Spike **改用选项 C（Python）**，目录 [`spike/`](../../spike/)。

结果：**S1–S5 全部通过**（`pytest` 6 passed）。见 [REPORT.md](../../spike/REPORT.md)。

关键发现：

- `-` / `+` 必须用**行扫描**保留项目符。
- frontmatter 导入行不能整块丢给严格 YAML。
- `markdown-it-py` 足够解析强调与表格；勿开缺依赖的 linkify。

---

## 待确认：参考实现语言

Spike 已证明 Python 路径可行。请勾选正式参考实现：

- [ ] **继续 Python**（推荐：与 Spike 同栈，立刻可写 `marqdo run`）
- [ ] 改选 TypeScript（选项 A）
- [ ] 改选 Rust（选项 B）
- [ ] 其它：________

确认后将本 ADR 标为 **Accepted**，再建立正式包（非 `spike/` 玩具目录）。

### 长期

不排除日后用 Rust 做生产编译器；与验证期/参考实现可以分离。
