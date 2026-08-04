# ADR 0001：参考实现语言与 Markdown 前端

| | |
|---|---|
| 状态 | **提议中（Proposed）** — 待确认后 Accepted |
| 日期 | 2026-08-04 |
| 决策者 | chaungming + 协作 |
| 相关 | [tech-stack.md](../design/tech-stack.md) · [markdown-mapping.md](../design/markdown-mapping.md) |

---

## 背景

语法宪法 v0.1 已定。需要选定**第一版参考实现**的实现语言与 GFM 解析前端，以便：

- 跑通 `examples/*.mq.md`
- 用测试锁定行首判别、`*`/`**`、`+`/`-`、表格、frontmatter `>`
- **不**在未论证前把某语言脚手架当成既成事实

目标是**验证宪法**，不是一次定终身的生产编译器。

---

## 约束（验收必须满足）

1. 能区分无序列表的 **`-` vs `+`** 项目符（分支/循环）。  
2. GFM **表格**、**emphasis / strong**（`*` / `**`）边界可映射回源位置。  
3. 能做**行预分类**（无标记=注释）再 lower 到 Marqdo AST。  
4. frontmatter 中非纯 YAML 的 **`> file.mq.md` 行**可解析。  
5. CLI + 自动化测试可对照 `examples/` 期望输出。

---

## 选项

| ID | 方案 | 验证期适配 | 长期编译器适配 |
|----|------|------------|----------------|
| A | **TypeScript + remark/rehype（mdast）+ Vitest** | 高 | 中（可留作工具链） |
| B | **Rust + comrak 或 pulldown-cmark + 集成测试** | 中 | 高 |
| C | Python + mistune/markdown-it-py | 高（原型） | 低 |
| D | Go + goldmark | 中 | 中 |

详见对比：[tech-stack.md](../design/tech-stack.md)。

---

## 提议决策（请确认）

### 验证期（现在 → 跑通 examples）

**采用选项 A：TypeScript + remark-gfm + mdast + Vitest + Node ≥ 20。**

理由：

1. 与 GitHub 预览最接近，减少「能渲染不能跑」。  
2. mdast 保留 list 的 `spread` / 子节点；需 Spike 确认 **bullet 字符 `-`/`+` 可取**（取不到则自研行扫描补一层）。  
3. 迭代宪法快，符合「设计刚定、语义还在磨」的阶段。

### 长期（宪法稳定后）

**不绑定 A。** 另开 ADR 评估 Rust（选项 B）作正式编译器/单二进制；TS 实现可降级为兼容测试预言机或弃用。

### 明确不选（验证期）

- **C Python** 作唯一实现：易成抛弃原型，双维护差。  
- **未 Spike 就写死 B**：怕拖慢验证。若你更熟 Rust且愿意先 Spike B，可对调。

---

## Spike 清单（定案前 ≥ 完成 A；可选 B）

| # | 用例 | 通过标准 |
|---|------|----------|
| S1 | `*a*` vs `**a**` | 得到 emphasis / strong，且能还原源码切片 |
| S2 | `- item` vs `+ item` | AST **能区分** bullet（或源行补救成功） |
| S3 | 单列表格 | 表头+单元格可遍历 |
| S4 | 文件头 `---` + 正文内 `---` | 能分开 frontmatter 与可选框线 |
| S5 | frontmatter 行 `> x.mq.md` | 不被 YAML 解析掐死，可抽出导入 |

**提议流程：** 先做 **Spike A（≤1 日）** → 若 S2 失败再评估自研行扫描或转 Spike B → 将本 ADR 标为 **Accepted** → 再允许添加 `package.json` 与 `src/`。

---

## 后果

- Accepted 后：仓库可建 `src/`、`tests/`，CLI `marqdo run`。  
- 生成 OKF 清单、LSP 等不在本 ADR 范围。  
- 实现须遵守：先做行分类，再求值（[code-vs-comment.md](../design/code-vs-comment.md)）。

---

## 确认栏

- [ ] 同意验证期 = TypeScript + remark  
- [ ] 改选：验证期 = Rust（先 Spike B）  
- [ ] 改选：其它：________  

确认后把状态改为 **Accepted**，并开始 Spike（不是直接铺全量解释器）。
