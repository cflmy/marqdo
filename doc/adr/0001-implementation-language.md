# ADR 0001：参考实现语言与 Markdown 前端

| | |
|---|---|
| 状态 | **Accepted** |
| 日期 | 2026-08-04 |
| 决策者 | chaungming + 协作 |
| 相关 | [tech-stack.md](../design/tech-stack.md) · [interpreter 路线图](../roadmap/interpreter.md) · [spike/REPORT.md](../../spike/REPORT.md) |

---

## 背景

语法宪法 v0.1 已定；Python Spike（S1–S5）已通过。需要选定**正经参考解释器**的实现语言（非玩具脚本）。

目标：实现完整流水线（词法/行分类 → 语法 → 语义 → 树遍历解释，日后字节码），使 `marqdo run index.mq.md` 得到真实结果。编译到龙芯/LLVM **不是** v1 目标（见路线图与 [USTC 编译课](https://ustc-compiler-2025.github.io/homepage/) 的对照说明）。

---

## 决策

1. **参考解释器实现语言：Python 3.11+**  
   - Markdown 辅助：`markdown-it-py`（表格/强调等）  
   - 测试：`pytest` + examples 金样例  
   - CLI：`marqdo` 入口  

2. **架构分期**（强制）：  
   - **Phase I**：递归下降 AST + **树遍历解释器**（对齐 Crafting Interpreters jlox）  
   - **Phase II**：AST → **字节码 + VM**（对齐 clox / 主流脚本语言实现）  
   - 详情：[interpreter.md](../roadmap/interpreter.md)  

3. **`spike/`** 仅作风险探测存档；正式代码在可安装包 `marqdo/`（或 `src/marqdo/`）中生长。  

4. **长期**：不排除 Rust 重写 VM/全前端；须另开 ADR，且金样例行为不变。

---

## 后果

- 立即按路线图建立正式包与 M0–M4，禁止 hello 特判冒充完成。  
- TypeScript / 纯编译后端不作为 v1 主路径。  
- USTC 课程中的词法/语法/语义/IR 思想复用；Flex/Bison/龙芯后端不搬用。

---

## 否决的替代

| 方案 | 否决原因 |
|------|----------|
| 只做正则玩具跑 hello | 达不到「一门语言」 |
| v1 直接 LLVM/龙芯 | 与「文档即运行」产品形态错配；工期错置 |
| Spike 未过就上 TS/npm | 已用 Python 验证前端风险，换栈无增益 |
