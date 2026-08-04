# 技术选型 v0（参考实现）

| | |
|---|---|
| 状态 | 草案（选型结论） |
| 日期 | 2026-08-04 |
| 目标 | 能跑通 `examples/*.mq.md` 的解析 + 求值，并用测试锁定行为 |

---

## 结论

**第一版参考实现采用 TypeScript（Node.js ≥ 20）+ 统一语系 Markdown 工具链 + Vitest。**

| 层 | 选择 | 理由 |
|----|------|------|
| 语言 | TypeScript | 与 GFM/MD 生态最贴；类型利于 AST；后续可编译到 WASM/CLI |
| Markdown 解析 | `remark-parse` + `remark-gfm` + `mdast` | 得到标题/列表/强调/表格/引用的标准 AST，避免手写 GFM |
| 运行 | Node.js ESM | CLI 简单；`fs` 读 `.mq.md` |
| 测试 | Vitest | 快；用 fixtures 对照 `examples/` 期望 stdout |
| 包管理 | pnpm | 锁文件清晰（亦可用 npm） |

**暂不采用：** Python 快速原型（后期两套实现成本高）、Rust 首版（MD 生态与迭代速度不如 TS 适合验证语法宪法）。语法稳定后再考虑 Rust 重写热路径或正式编译器。

---

## 架构草图

```
.mq.md
  → remark/GFM → mdast
  → marqdo lower：行首判别、标题树、列表符语义（- / + / 1.）
  → AST（函数、调用、输出、返回、分支、循环、表、绑定）
  → Pass1 收集定义 + frontmatter `>` 导入
  → Pass2 求值 → stdout / 返回值
```

包建议（仓库内）：

```
packages/marqdo/          # 或暂用根目录 src/
  src/parse.ts
  src/lower.ts
  src/eval.ts
  src/cli.ts
  tests/fixtures/
```

首版 CLI：`marqdo run examples/hello.mq.md` → 打印 `Hello World!`

---

## 测试策略

1. **金样例**：每个 `examples/*.mq.md` 对应一份期望输出（`.stdout` 或测试内快照）。  
2. **单元**：行首判别、`-`/`+` 消歧、`*` else vs `*ret*`、表格绑定。  
3. **排查顺序**：先 hello → index → branch → loop → collection → with-import。

---

## 里程碑

1. 脚手架 + `run` hello  
2. 函数 / 调用 / 输出 / 返回  
3. 分支 `+` 与循环 `-`  
4. 表格集合 + foreach  
5. frontmatter `>` 导入  

---

## 非目标（本选型文档）

- LSP、格式化、编译到原生  
- 完整表达式文法一次性做完（按测试增量加）
