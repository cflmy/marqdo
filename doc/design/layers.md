# 三层归属与准入戒律（layers）

| | |
|---|---|
| 状态 | **Accepted · 准入制度（问题一落地）** |
| 日期 | 2026-09-22 |
| 相关 | [core-surface.md](core-surface.md)（核心表面守卫）· [three-problems.md](../roadmap/three-problems.md) 问题一 · [three-problems-plan.md](../roadmap/three-problems-plan.md) T1.2 |

> **目的**：语言核心保持小——**Language / Runtime / Document 三层分明，生态功能不淹没语言身份**。本制度是机制不是宣言：违反即 CI 红 / PR 不评审。

## 1. 三层归属表

| 层 | 内容 | 仓库位置 |
|----|------|----------|
| **Language** | 标记映射、词法/行分类、AST、语义、升参、契约检查器、诊断 | `src/lex/` `src/parse/` `src/ast.rs` `src/interp.rs` `src/diagnostics.rs`（及 `contract`/`check`）· 宪法 [markdown-mapping-v0.3.md](markdown-mapping-v0.3.md) · [core-surface.md](core-surface.md) |
| **Runtime** | stdlib、插件 ABI、WASM、debugger、view、catalog / schema / mcp CLI、字节码后端 | `lib/` `plugins/` `crates/` `src/catalog.rs` `src/view/` `src/debug/` `src/builtin.rs` `src/bytecode/` |
| **Document** | 知识单元、AI skills、示例、官方扩展、用户站 | `*.mq.md` `skills/` `ext/` `public/` `examples/` `doc/` |

## 2. 准入戒律

1. **先填层归属**：任何新特性提案（issue / PR / 设计文档）必须先声明进哪层；未填 = 不评审。
2. **Language 层变更 = ADR**：新增核心标记必须有 `doc/adr/` 编号 + 「为什么现有标记不够」+ [core-surface.md](core-surface.md) 增行 + 样例 + `CORE_CONSTRUCTS` 同步（缺任一 → CI 守卫红）。
3. **核心计数只许持平或下降**：每个 release 的核心标记计数不得增长；确需增长即触发 Language 层复盘（ADR 里写明不可替代理由）。
4. **默认进 ext**：能用扩展、stdlib、插件解决的，一律不进核心。
5. **CHANGELOG 单列「核心表面 Δ」**：每个 release 写明增删（无 Δ 写「无」）；发版 checklist（`.cursor/skills/marqdo-release/SKILL.md`）含此步。

## 3. PR 模板

`.github/PULL_REQUEST_TEMPLATE.md` 含「层归属」必填项与「核心表面」勾选——与本制度一一对应。
