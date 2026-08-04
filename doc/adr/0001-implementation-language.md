# ADR 0001：参考实现语言与前端工具

| | |
|---|---|
| 状态 | **Accepted**（修订：实现语言 = Rust） |
| 日期 | 2026-08-04 |
| 决策者 | chaungming + 协作 |
| 相关 | [dependencies.md](../design/dependencies.md) · [interpreter 路线图](../roadmap/interpreter.md) · [spike/REPORT.md](../../spike/REPORT.md) |

---

## 背景

语法宪法 v0.1 已定；Python Spike 已验证 GFM/行扫描风险。产品目标是正经**解释器**（`marqdo run index.mq.md`），实现语言需兼顾安全与可维护性。

---

## 决策

1. **参考解释器实现语言：Rust（stable）**  
   - 内存安全、单静态二进制、适合长期演进到字节码 VM。  
   - Python Spike **保留为算法原型/对照**，不是运行时依赖。

2. **词法 / 语法：不使用 Flex、Bison**  
   - **主因是 Marqdo 的 Markdown 语法形态**，不是「因为选了 Rust」。换成 C++ 同样不建议用 Flex/Bison 做整语言前端。  
   - 解释器需要词法·语法**阶段**；采用自研行分类 + GFM 库 + **手写递归下降** → AST。  
   - 详见 [dependencies.md](../design/dependencies.md)。

3. **架构分期不变**  
   - Phase I：树遍历解释器  
   - Phase II：字节码 + VM  
   - 见 [interpreter.md](../roadmap/interpreter.md)

4. **Markdown 库**  
   - `pulldown-cmark` 或 `comrak` 二选一（建包前短 Spike 钉死）。

---

## 后果

- 正式代码以 Cargo 工程生长；不把 Flex/Bison/LLVM 列入 v1 依赖。  
- 金样例仍为 `tests/`；行为与宪法一致。  
- 此前「Python 作参考实现」的倾向由本修订取代；Spike 结论（行扫描等）仍然有效并迁入 Rust。

---

## 否决

| 方案 | 原因 |
|------|------|
| Flex + Bison 作主前端 | 与 Markup-as-Syntax / Rust 栈错配；解释器非必需 |
| v1 上 LLVM | 产品是解释执行文档，不是机器后端 |
| 仅 Python 正则玩具 | 达不到安全与长期目标 |
