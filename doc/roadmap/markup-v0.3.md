# 路线图：Markup 映射 v0.3（叙述统一）

| | |
|---|---|
| 状态 | **Active（v0.3 sole · 存量已迁移 · gold 绿）** |
| 日期 | 2026-09-13 |
| 设计 | [markdown-mapping-v0.3.md](../design/markdown-mapping-v0.3.md) |
| ADR | [0005-prose-unified-markup.md](../adr/0005-prose-unified-markup.md) |
| 依赖 | 现有管线 [interpreter.md](interpreter.md) · [pipeline-debug.md](../design/pipeline-debug.md) |

---

## 目标

把 v0.3 宪法落到词法 → AST → 语义 → 双后端，使动机示例（加一函数）成为金样例，并安全迁移存量代码。

---

## 里程碑

### M0 — 设计冻结与双模式骨架（文档 ✅ / 代码 ⏳）

- [x] 设计文档 + ADR Proposed + 本路线图  
- [x] **钉死**：空返回词法（设计 §7.2）；提及/未使用变量（§4.4）；升参 vs 绑定图（§8.2）  
- [x] **钉死**：一行多 `**…**` 按序执行（设计 §12 #6）  
- [x] **不做双模式** — 直接 v0.3 默认  

**出门：** 设计评审通过；旗标可切换；无行为默认变化。

### M1 — 内联扫描与 AST

- [x] 金样例目录 `tests/markup-v03/`（夹具 + `gold.rs` `#[ignore]`；见目录 README）  
- [x] `Prose` 行内联扫描：`` `ident` `` / `**…**` / `*…*`  
- [x] `Stmt::Return` ← italic；`Stmt` ← bold Assign/Call  
- [x] `Param.inferred` + 升参  
- [x] `tests/markup-v03/` 金样通过（默认运行时）  

**出门：** `cargo test markup_v03`（无 ignore）绿。

### M2 — 形参推断

- [x] `InferParams`：首次 `` `名` `` = 必选；`` `名`=默认 `` = 可选  
- [x] 体赋值 `n=n+1` 不误判为默认  
- [x] 与显式 `+` 合并规则 + 警告  
- [x] 方法 `self`/`自` 排除  
- [x] 金样例：默认实参、位置/具名调用、粗体内调用

**出门：** 推断签名与 `+` 兼容套件绿。

### M3 — 调用面与消歧

- [x] `**callee args**` ≡ `> callee args`  
- [x] else 臂 `N. *` 与返回斜体消歧  
- [x] 一行多 `**…**` 按序执行  
- [x] 金样例：`---` / `*None*` / 整行 `**` / 兼容 `****`；叙述死绑定不升参  
- [x] 诊断：必选缺失、未知 callee（已有 errors 金样）；未闭合代码形粗体；死变量不报错  
- [ ] （可选）死变量 hint lint；空返回其它歧义边角  

**出门：** `tests/markup-v03/*` 覆盖设计 §9 示例。

### M4 — 双后端与工具

- [x] 字节码发射跟随新 Stmt 源  
- [x] `view` / Structure：推断形参 `chip inferred` + Functions 大纲标注  
- [ ] （可选）`marqdo migrate markup-v03` 机械改写提示/工具  

**出门：** tree 与 bytecode 后端对 `markup-v03` 金样一致。

### M5 — 默认切换与存量迁移

- [x] 默认 `MarkupEdition::V03`  
- [x] 迁移 `tests/**`、`lib/**`、`ext/**`、`public/**`、教程  
- [x] 更新 `.cursor/skills/marqdo/SKILL.md` 硬规则  
- [x] 修订 [markdown-mapping.md](../design/markdown-mapping.md) 为 v0.3 正式正文（或声明由 v0.3 文件接任）  
- [x] ADR 0005 → **Accepted**  
- [ ] CHANGELOG / 发版说明（破坏性：标记对调）

**出门：** 全量 gold 默认绿（live LLM 无钥跳过）；无 v0.2 双模式。

---

## 非目标（本路线）

- 不改插件 ABI / Go libweb  
- 不引入 `def`/`return` 关键字  
- M0–M4 不强制改生产 `.mq.md`  

---

## 风险

| 风险 | 缓解 |
|------|------|
| 标记对调导致海量假失败 | 双模式 + 分目录金样；默认晚切换 |
| 形参推断误伤局部变量 | 金样例钉启发式；允许过渡期写 `+` |
| 空返回 vs `**` 词法 | §7.2 已钉：整行 `**` = 空返回；推荐优先 `---` |
| 叙述反引号误升参 | §8.2 升参需可执行面读取；死绑定不进 params |
| Skill/AI 仍按 v0.2 生成 | M5 同步 Skill；过渡期文档标明版本 |

---

## 修订历史

| 日期 | 说明 |
|------|------|
| 2026-09-13 | 初稿与设计/ADR 同步 |
| 2026-09-13 | 空返回与提及性反引号决议合入；M0 文档项勾选 |
