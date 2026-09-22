# 三大问题落地计划（core 收口 · 渐进式契约 · MLSP）

| | |
|---|---|
| 状态 | **本期开发依据 · 执行中** |
| 日期 | 2026-09-22 |
| 设计 | [three-problems.md](three-problems.md)（解决方向与验证设计；判据矩阵 §5.1） |
| 范围 | **工程任务 = Phase 1–3**；验证实验（three-problems.md §5.2）**不属于开发任务**，待「性能验证设计」（`doc/next/`，用户维护）导入后对接 |
| 约定 | 每任务带机器可验证验收；**任务完成 = 验收全绿 + 全量回归绿**；只改宣言不改行为 = 未完成 |

## 0. 本期范围与依赖

| 阶段 | 定位 | 本期 |
|------|------|------|
| **Phase 1** 核心收口 + 诊断地基 | 一切的地基 | ✅ 启动 |
| **Phase 2** 渐进式契约层 | 校验层（schema 源头 = 表格/段落） | ✅ 本期主目标 |
| **Phase 3** MLSP（AI 语言服务） | **下一步的超级核心**（AI 不必学语法 + 可靠反馈闭环） | ⏭ 下期启动，本期只留接口 |
| **Phase 4** 验证实验 | 性能验证（非开发任务） | ❌ 不认领 |

依赖链（**跳步会倒逼「另造一套错误 / 另写一套 schema」的遮盖式捷径**）：

```text
T1.4 Diagnostic ──是──> T2.x 契约诊断的唯一出口
T2.x 契约 ───────是──> T3.1 schema 注册表的主数据源
T1.1 core-surface ─是─> T3.1 语言面 + T1.3 守卫
```

---

## Phase 1 · 核心收口 + 诊断地基（本期启动）

### T1.1 核心表面清单

- **产出**：`doc/design/core-surface.md`
- **实现要点**：
  - 两张表：**行构造**（heading-unit `#`/`##`、call-line `>`、branch-list `1.`、loop-list `-`、param `+ 名`、table、fence、hr、writeback、narrative、blank）与**内联标记**（ident `` `名` ``、bold-code `**…**`、italic-return `*…*`、empty-return、link-index `[键](集合)`、bracket-call `[函数]`、footnote-index `[^键]` 过渡）；
  - 每行含：**id**（英文 kebab，守卫用）、Markdown 外形、语义、**戒律**（边界 + 为什么不能是普通 Markdown）、**金样**（`tests/markup-v03/…` 路径）；
  - 与 [markdown-mapping-v0.3.md](../design/markdown-mapping-v0.3.md) §5 / §10 对齐；词法大类（`src/lex/mod.rs` `LineKind`）另列一节说明与构造表的关系。
- **验收**：每行金样文件真实存在（T1.3 断言）；id 全集 = T1.3 `CORE_CONSTRUCTS` 全集。

### T1.2 三层归属与准入戒律

- **产出**：`doc/design/layers.md` + `.github/PULL_REQUEST_TEMPLATE.md`（无则新建）
- **实现要点**：
  - 归属表（含仓库路径）：Language = `src/lex` `src/parse` `src/ast` `src/interp` `src/diagnostics` + 契约检查器；Runtime = `lib/` `plugins/` `crates/` `src/catalog.rs` `src/view` `src/debug` + 未来的 check/schema/mcp；Document = `*.mq.md` `skills/` `ext/` `public/` `examples/`；
  - 准入戒律：新特性先填层归属；进 Language 必答「为什么现有标记不够」；默认进 ext；核心标记计数每 release 只许持平或下降；
  - 发布 checklist：CHANGELOG 增「核心表面 Δ」栏（写入 `skills/marqdo-release/SKILL.md` 一步）。
- **验收**：PR 模板含「层归属」必填项；layers.md 与 core-surface.md 互相引用闭合。

### T1.3 核心表面守卫（CI）

- **产出**：`tests/core_surface.rs` + `src/parse/mod.rs` 增 `pub const CORE_CONSTRUCTS: &[(&str, &str)]`（id、外形；**紧邻分类代码维护**）
- **实现要点**：`include_str!` 解析 core-surface.md 表格 id 列 ↔ `CORE_CONSTRUCTS` **双向集合相等**；每行「金样」路径 `Path::exists()` 断言。
- **验收（反遮盖核心）**：
  - 新增 `CORE_CONSTRUCTS` 条目不改文档 → **红**；
  - 文档加行不改代码 → **红**；
  - 删除/改名金样文件 → **红**。

### T1.4 诊断对象扩展（Phase 2/3 的唯一出口）

- **产出**：`src/diagnostics.rs` 扩展 + `src/main.rs` `run --json`
- **实现要点**：
  - `Diagnostic` 增 `code`（如 `parse.*` / `contract.*` / `runtime.*`）、`severity`、`suggestion`、`doc_anchor`、`doc_quote`、`contract_ref`（Phase 2 起填值）；`::new/at` 签名不变（全仓调用零改动），新增 builder 方法；
  - `serde::Serialize` + `to_json()`；**Display 保持 `path:line:col: message`**（人类输出零回归）；
  - `run --json`：顶层错误路径把 Diagnostic 序列化输出（stderr JSON 行）。
- **验收**：现有错误金样文本输出**零变化**；新金样断言 JSON `code`/`span` 非空；`cargo test` 全绿。

---

## Phase 2 · 渐进式契约层（本期主目标）

### T2.1 契约 AST + 提取器

- **产出**：`src/contract.rs`（`Contract` / `ParamContract` / `FieldContract` + 类型词汇 `text/number/bool/list/map/any/对象名`）；`src/parse/`（提取）；`src/ast.rs`（挂载，带 span）
- **实现要点**：按 [three-problems.md](three-problems.md) §2.3/§2.4——**未绑定文档表** + 列头关键字（`参数/返回/字段/类型/说明`）+ 允许位置（函数体首个可执行行前 / 绑定前 / 对象体首成员前）；**被绑定的表永不视为契约表**。
- **验收**：三附着面金样 + 「绑定表不吃」反例金样（web_db_migration 式 `字段|类型|可空` 绑定数据表必须原样是数据）；未知类型名（`nubmer`）必产诊断。

### T2.2 四边界运行时校验

- **产出**：`src/interp.rs` 接入四边界——调用实参 / 字段访问（`[键](集合)`、`.field`）/ 表绑定 / `*…*` 返回
- **实现要点**：只在**该边界有契约**时校验（无契约 = 全动态，P1）；诊断一律走 T1.4 出口（`code=contract.*`）。
- **验收**：有契约 + 错值 → 必产诊断；**无契约路径全量回归零变化**；three-problems.md §5.2(c) 消融材料可产。

### T2.3 `marqdo check` + `--json`

- **产出**：`src/check.rs` + `src/main.rs` `Commands::Check`
- **实现要点**：静态**按需**校验（只查有契约处）；不做全局推导。
- **验收**：契约样例工程 `check` 退出码非 0 且 JSON 合协议；干净工程 0 退出零输出。

### T2.4 锚点诊断

- **产出**：契约诊断填 `doc_anchor`（契约表行段 `file#Lx-Ly`）+ `doc_quote`（表行原文）
- **验收**：金样断言锚点非空且行号回读命中契约表；three-problems.md §5.2(a) 材料就绪。

### T2.5 契约 vs 实现互查（防漂移）

- **产出**：check + 运行时提示——契约参数名 ≠ 升参名 / 未知类型 / 返回契约明显不符 → 诊断
- **验收**：故意写错契约的金样**必红**（three-problems.md §5.2(d) 材料就绪）。

### T2.6 运行时内省 `get_schema`（可延后，P3）

- **产出**：`src/builtin.rs` 内省 builtin（契约是数据，P3）
- **验收**：demo 运行中取回自身函数契约并打印。

---

## Phase 3 · MLSP（下一步的超级核心；本期只留接口）

### T3.1 `marqdo schema`（schema 注册表，三来源零手写）

- **产出**：`src/schema.rs` + `Commands::Schema`——语言面（core-surface 机械转 JSON）+ 函数/类型面（**契约** + stdlib/ext 签名）+ 项目面（`catalog.yaml`）
- **验收**：两次生成 byte-identical；stdlib 主函数覆盖率 ≥ 90%。

### T3.2 `marqdo mcp`（AI 工具面）

- **产出**：MCP server（参考 `plugins/agent/src/mcp_server.rs`）——`syntax_list` / `syntax_get` / `fn_schema` / `type_schema` / `validate` / `repair` / `run`
- **验收**：MCP 客户端冒烟：`fn_schema` → `validate` → `repair` 全链路。

### T3.3 查询式 AI Skill

- **产出**：`skills/marqdo/` 改写——十条硬规则 + 查询工作流（construct → validate → 修 → run）；细节一律走 `syntax_get` / `fn_schema`
- **验收**：skill 体积减半以上；金样全绿。

### T3.4 修复回路

- **产出**：repair 规则库 + 语句级 patch（AST span 对齐；预算 N=3；防循环）
- **验收**：注入 bug 金样一轮修复率可测（three-problems.md §5.2(a) 执行就绪）。

---

## Phase 4 · 验证实验（**非开发任务**）

- 对接「性能验证设计」（`doc/next/`，用户维护，待导入）；three-problems.md §5.2 实验 (a)–(e) 的**材料由 Phase 1–3 的验收金样直接供给**；开发侧仅保留「实验基建适配」支持义务（复用 MarqdoThesis `formal/exp1–3` runner/oracle）。

---

## 完成定义

| 阶段 | 完成定义 |
|------|----------|
| Phase 1 | T1.1–T1.4 验收全绿 |
| Phase 2 | T2.1–T2.5 验收全绿（T2.6 可延后） |
| Phase 3 | T3.1–T3.4 验收全绿 |
| 三大问题「已解决」 | [three-problems.md](three-problems.md) §5.1 判据矩阵全绿 + 实验数字可复现（Phase 4） |
