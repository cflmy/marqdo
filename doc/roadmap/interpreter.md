# Marqdo 参考实现：正经解释器路线图

| | |
|---|---|
| 状态 | 已采纳方向 |
| 日期 | 2026-08-04 |
| 目标 | `marqdo run path/to/index.mq.md` 得到真实执行结果；非演示玩具 |
| 相关 | [markdown-mapping.md](../design/markdown-mapping.md) · [ADR 0001](../adr/0001-implementation-language.md) · [USTC 编译课 2025](https://ustc-compiler-2025.github.io/homepage/) |

---

## 1. 为何是解释器，而不是 USTC 式「编译到机器码」

[USTC《编译原理与技术》2025](https://ustc-compiler-2025.github.io/homepage/) 的实验链是经典**编译器**路径：

```
源码 → 词法(Flex) → 语法(Bison) → AST
     → 语义 / Light IR → 优化 → 后端（龙芯汇编等）
```

这条路适合 Cminusf 一类「最终变成机器指令」的语言。Marqdo 的产品形态是：

- 源即 Markdown 文档，**打开即读、运行即算**；
- 首要交付是 **从 `index.mq.md` 得到求值结果**（stdout、返回值、模块图），不是龙芯/ x86 二进制；
- 语法表面是 Markup-as-Syntax，前端与传统正则+Bison 不同，但**中后端的编译学知识仍然全部用得上**。

因此：

| USTC / 龙书阶段 | Marqdo 解释器中的对应 |
|-----------------|------------------------|
| 词法分析 | **行分类 + 标记词法**（无标记=注释；`#`/`*`/`**`/`>`/`+`/`-`/表…） |
| 语法分析 | **结构解析 → Marqdo AST**（可递归下降；不必上 Bison，除非日后自研纯文本方言） |
| 语义分析 | 作用域、提升、导入、形参绑定、类型（可后置） |
| 中间代码 | **可选**：先树遍历；第二期再降到**字节码** |
| 优化 / 目标代码 | **非 v1 目标**；日后若要可另开「编译后端」ADR |

**结论：做正经解释器 = 把编译前端做扎实 + 把运行时做完整；不是跳过词法语法，也不是一上来 LLVM。**

权威地图见 [Crafting Interpreters — A Map of the Territory](https://craftinginterpreters.com/a-map-of-the-territory.html)：用户眼里的「解释器」内部往往仍含编译到字节码（如 CPython）；树遍历是第一期，字节码 VM 是第二期。

---

## 2. 推荐总体架构

```
index.mq.md (+ 导入的 .mq.md)
        │
        ▼
┌───────────────────┐
│ 0. 源加载 / 模块图 │  frontmatter `> a.mq.md` → 依赖图
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ 1. 词法·行分类     │  注释行丢弃语义；代码行 → Token 流
│    (+ MD 块辅助)   │  pulldown-cmark/comrak；+/- 靠行扫描
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ 2. 语法分析        │  Token/块 → Marqdo AST
│    (递归下降)      │  Module / Fun / Stmt / Expr / Branch / Loop …
│                    │  **不用 Flex/Bison**（见 dependencies.md）
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ 3. 语义·绑定       │  Pass1: 收集函数（提升）+ 导入合并
│                    │  作用域链、形参、未定义名诊断
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ 4a. 树遍历解释器   │  ← Phase I（必须先完成，跑通全部 examples）
│  或                 │
│ 4b. 字节码编译+VM  │  ← Phase II（性能与「更像真语言实现」）
└─────────┬─────────┘
          ▼
     stdout / 返回值 / 退出码
```

Phase I 对齐 Crafting Interpreters 的 **jlox（树遍历）**；Phase II 对齐 **clox（字节码 VM）**（见 [Chunks of Bytecode](https://craftinginterpreters.com/chunks-of-bytecode.html)）。这是 Python/Ruby/Lua 主实现走过的正经路径，不是玩具捷径。

---

## 3. 各阶段「做满」的标准（反玩具清单）

### Phase 0 — 工程骨架（正式包，不是 `spike/`）

- 包名：`marqdo`（`src/marqdo/` 或 `marqdo/`）
- CLI：`marqdo run <file.mq.md>`，默认找目录下 `index.mq.md`
- 诊断：带文件名、行号、列号
- 测试：每个 `examples/*.mq.md` 有期望 stdout（金样例）；CI 可跑

### Phase I — 可运行的语言核心（树遍历）

必须全部为真实现，禁止「正则特判 hello」：

1. **词法 / 行分类**：注释与代码严格分离。  
2. **语法 → AST**：函数、形参、`*` 语句、`**` 返回、`>` 调用、`+` 分支、`-` 循环、表格集合、绑定与 `==`。  
3. **语义**：提升、嵌套作用域、`##` 私有、frontmatter 导入公有顶层。  
4. **运行时**：环境、调用栈、值（文/数/表/记录）、内置 `print`。  
5. **验收**：`examples/structure/*` 与 `examples/keywords/*` 全部 `marqdo run` 通过。

### Phase II — 字节码 VM（正经化）

- AST → 线性字节码（栈式 VM 即可，与 clox / [GoAWK 经验](https://benhoyt.com/writings/goawk-compiler-vm/) 同类）  
- 控制流用跳转指令实现分支/循环  
- 同一套金样例必须 bit-同结果（回归）  
- 性能与可调试性（反汇编、栈跟踪）优于 Phase I

### Phase III — 语言完备（按需）

- 更完整表达式文法、错误模型、外联围栏 FFI  
- 生成 OKF 依赖目录  
- （可选）另一后端：编译到 Python/WASM——**另开 ADR**，不替代自研语义

---

## 4. 与 USTC Lab 的知识复用（学什么、不搬什么）

| 课程内容（摘要） | Marqdo 用法 |
|------------------|-------------|
| Lab1 词法/语法/AST | **直接对标** Phase I 前端；工具可自研递归下降，不必 Flex/Bison |
| Lab2 IR / 访问者 | Phase II 字节码 ≈ 轻量 IR；访问者可用于 AST→字节码 |
| Lab3 后端汇编 | **v1 不做**；解释器 VM 代替机器后端 |
| Lab4 优化 | Phase II 之后可选；解释器也可做常量折叠等 |

教材仍可读：龙书、陈意云《编译原理》、课主页参考书；解释器专项读 **Crafting Interpreters**（免费：[craftinginterpreters.com](https://craftinginterpreters.com/)）。

---

## 5. 实现语言（本路线图默认）

**参考解释器：Rust**（[ADR 0001](../adr/0001-implementation-language.md)）。

- 词法/语法：**自研 + GFM crate**，不用 Flex/Bison（[dependencies.md](../design/dependencies.md)）。  
- Python `spike/` 仅作风险探测与算法草稿。  
- Phase I 树遍历、Phase II 字节码 VM 均在 Rust 中实现。

---

## 6. 目录规划（正式代码，非 spike）

```
Cargo.toml
crates/marqdo/   # 或根包
  src/lex/ parse/ ast/ sema/ interp/ bytecode/ runtime/
tests/
examples/
spike/           # Python 存档，非运行时依赖
```

---

## 7. 近期里程碑（可执行顺序）

| 周序 | 交付 | 完成定义 |
|------|------|----------|
| M0 | 正式包 + CLI 空壳 + 金样例框架 | `marqdo run` 可调用；测试发现「未实现」 | ✅ |
| M1 | 词法·行分类 + 最小 AST | 单测覆盖注释/代码行 | ✅ |
| M2 | 解析：函数、输出调用、返回 | `hello` + `index` 金样例绿 | ✅ |
| M3 | 分支、循环、表格、绑定 | `branch`/`loop`/`collection` 绿 | ✅ |
| M4 | 导入与提升 | `with-import` 绿；**宣称 Phase I 达标** | ✅ |
| M5 | 字节码设计文档 + 原型 opcode | 核心金样例双后端一致；覆盖表见 [bytecode.md](../design/bytecode.md) | 进行中 |

**Phase I 完成前，不宣称「语言已实现」。**  
**仅 hello 特判通过，不算完成。**

---

## 8. 成功判据（产品级）

在仓库根或任意含 `index.mq.md` 的目录：

```bash
marqdo run examples/structure/nested-call.mq.md
# 或
marqdo run examples/structure/import/main.mq.md
```

输出与金样例一致；错误带位置；新增语法必须加金样例与单测。这才是「一门解释型语言」的最低合格线。
