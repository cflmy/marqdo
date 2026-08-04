# 示例与测试布局

| | |
|---|---|
| 状态 | 已定 |
| 日期 | 2026-08-04 |
| 原则 | **夹具即金样例**：`tests/{structure,keywords,errors}/**/*.mq.md` 由 `cargo test` / CI 跑通；`marqdo view tests` 也可浏览 |

---

## 1. 目录

只保留 **`tests/`**，用子目录区分类别（不再另设 `examples/`）：

```
tests/
  README.md
  gold.rs                   # 集成测试入口
  structure/                # 代码基础结构（Markup 架构）
    hello.mq.md
    nested-call.mq.md
    positional-call.mq.md
    branch.mq.md
    loop.mq.md
    collection.mq.md
    import/
      main.mq.md
      utils.mq.md
  keywords/                 # 关键字与内置名
    print.mq.md
    bool-logic.mq.md
  errors/                   # 期望失败（诊断文案稳定后再加金样例）
    …
```

面向访客的可执行介绍在 [`public/`](../../public/)（无 `errors/`），见 [user-site.md](user-site.md)。  
开发设计文稿在 [`doc/`](../)（单数）。

---

## 2. 分类职责

| 类别 | 测什么 | 不测什么 |
|------|--------|----------|
| **structure** | `#`/`##`、形参、`>` 调用、`*`/`**`、`+`/`-`、表、导入、位置/具名实参 | 关键字边角 |
| **keywords** | `True`/`False`/`None`、`and`/`or`/`not`、`print`/`input` 约定 | 复杂控制流（可极简） |
| **errors** | 未定义名、实参错误、语法错误的诊断 | 成功路径 |

未单独建夹、但应覆盖的其它测试（`src/**` 单测或日后 `tests/runtime/`）：

- **流水线调试**：`--dump-lines` / `--dump-ast` / `--trace-eval`  
- **CLI**：默认 `index.mq.md`、退出码  
- **view**：目录扫描、单文件页、执行区（见 [view.md](view.md)）  
- **模块图**：循环导入拒绝  

---

## 3. 金样例约定

每个可执行 `.mq.md` 在 `tests/gold.rs` 登记期望 stdout。  
`errors/` 内文件登记期望 stderr 子串与非零退出码。

---

## 4. 与 `view` / `public` 的关系

- `marqdo view tests`：开发者浏览金样例（含 errors）。  
- `marqdo view public`：用户文档站源；**不要**把 `tests/errors` 编进 `public/`。
