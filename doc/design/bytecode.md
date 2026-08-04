# Marqdo 字节码（M5 原型）

| | |
|---|---|
| 状态 | 进行中（原型） |
| 日期 | 2026-08-04 |
| 相关 | [roadmap/interpreter.md](../roadmap/interpreter.md) · Phase I 树遍历 |

---

## 1. 目标

在保留树遍历后端（`--backend tree`，默认）的同时，提供栈式字节码 VM（`--backend bytecode`），使**核心金样例** stdout 与树遍历 bit 一致。

```
.mq.md → load/parse → Module AST
                         ├─ tree-walk
                         └─ compile → Chunk → VM
```

---

## 2. 指令集（草案 / 原型已实现子集）

| Op | 栈效应 | 说明 |
|----|--------|------|
| `Constant i` | → v | 常量池 |
| `True` / `False` / `None` | → b | |
| `Pop` | v → | |
| `GetLocal i` / `SetLocal i` | | 当前帧槽 |
| `GetGlobal i` / `SetGlobal i` | | 名字在常量池（文本） |
| `Add` `Sub` `Mul` `Div` `Negate` `Not` | | |
| `Equal` `NotEqual` `Greater` `GreaterEqual` `Less` `LessEqual` | | |
| `Jump a` / `JumpIfFalse a` / `JumpBack a` | | 相对/绝对跳转（原型用绝对 u16） |
| `Print` | v → | 弹出并写 stdout |
| `BuildList n` | v… → list | |
| `GetIndex` | list, i → v | foreach 用 |
| `Call f argc` | args… → ret | 用户函数（按形参序压栈） |
| `Return` | | 返回 / 结束帧 |
| `Interp n` | parts… → text | n 个交错 Lit/Var 标记后的拼接（简化：编译期展开为常量+拼接） |

原型实现以 `src/bytecode/` 为准；未列出的 Op 不存在于当前构建。

---

## 3. 编译约定

- 每个 Marqdo `Function` → 一个 `FnChunk`（含 `name`、`params`、`code`、`constants`）。
- 模块级 `Program`：函数表 + 入口名（`main` 或唯一无参 level-1）。
- 局部变量：函数体内首次赋值/形参分配槽位。
- 嵌套 `##`：编入同一 `Program`，按名 `Call`；查找规则对齐树遍历（子树优先，再顶层）。
- `and` / `or`：短接跳转。
- 分支 / while：`JumpIfFalse` + `Jump` / `JumpBack`。
- foreach：`BuildList` 或已有 list 全局/局部 → 索引循环。

---

## 4. 覆盖表（M5 原型）

| 金样例 | tree | bytecode |
|--------|------|----------|
| `structure/hello` | ✅ | ✅ |
| `structure/branch` | ✅ | ✅ |
| `structure/loop` | ✅ | ✅ |
| `structure/collection` | ✅ | ✅ |
| `keywords/print` | ✅ | ✅ |
| `keywords/bool-logic` | ✅ | ✅ |
| `structure/nested-call` | ✅ | ✅ |
| `structure/positional-call` | ✅ | ✅ |
| `structure/import/*` | ✅ | ✅（load 合并 AST 后编译） |
| `tests/errors/*` | ✅（非零退出） | 同 tree 语义（编译期/运行期诊断） |

核心金样例双后端已对齐；后续可扩展 Op、优化与更好的调试信息。

---

## 5. CLI

```bash
marqdo run FILE --backend tree          # 默认
marqdo run FILE --backend bytecode
marqdo run FILE --dump-bytecode         # 反汇编后仍可按 backend 执行
```

---

## 6. 非目标（本原型）

优化、寄存器分配、GC、与树遍历指令级调试对齐、独立 sema IR。
