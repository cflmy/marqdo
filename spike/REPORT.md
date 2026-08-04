# Spike 报告：Python + markdown-it-py（S1–S5）

| | |
|---|---|
| 日期 | 2026-08-04 |
| 状态 | **通过** |
| 命令 | `cd spike && pytest -q` → `6 passed` |

---

## 结果

| ID | 项 | 结果 | 备注 |
|----|----|------|------|
| S1 | `*` vs `**` | 通过 | `em_*` / `strong_*` 在 inline children 中 |
| S2 | `-` vs `+` | 通过 | **行扫描** `scan_bullets` 保留项目符；库只保证有 `bullet_list` |
| S3 | 单列表格 | 通过 | `table` 规则开启后可解析 |
| S4 | FM `---` vs 正文 `---` | 通过 | 自研 `split_front_matter` |
| S5 | FM 内 `> *.mq.md` | 通过 | 行正则抽取，不整块丢给严格 YAML |

---

## 对实现的含义

1. **行扫描是必需层**：`-`/`+` 消歧、frontmatter 导入、注释行分类，不宜只靠 MD 库 AST。  
2. **markdown-it-py** 足够做块/inline 结构（标题、表、强调、列表）；避免 `gfm-like` 默认 `linkify`（缺可选依赖会炸）。  
3. Spike **不**等于选定长期实现语言；仅证明 Python 路径可验证宪法。参考实现可继续 Python，或再评 TS/Rust。

---

## 建议下一步

1. 将 ADR 0001 中 Spike 标为通过。  
2. 拍板：参考实现是否 **继续 Python**（推荐，与 Spike 同栈）或换栈。  
3. 再搭建正式包（`src/marqdo`）与 `marqdo run`，按 examples 金样例推进。
