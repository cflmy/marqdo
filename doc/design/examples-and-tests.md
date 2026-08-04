# 示例与测试布局

| | |
|---|---|
| 状态 | 已定 |
| 日期 | 2026-08-04 |
| 原则 | **示例即金样例**：`examples/**/*.mq.md` 由 `cargo test` / CI 跑通；`marqdo view` 也可直接浏览 |

---

## 1. 目录

```
examples/
  README.md                 # 本树说明
  structure/                # 代码基础结构（Markup 架构）
    hello.mq.md
    nested-call.mq.md       # 原 index：嵌套函数 + 调用
    positional-call.mq.md   # 位置实参
    branch.mq.md
    loop.mq.md
    collection.mq.md
    import/
      main.mq.md            # 原 with-import
      utils.mq.md
  keywords/                 # 关键字与内置名
    print.mq.md
    bool-logic.mq.md        # True/False/None、and/or/not
  errors/                   # 期望失败（诊断文案稳定后再加金样例）
    .gitkeep
```

Rust 集成测试：`tests/gold.rs` 扫描约定路径，比对 stdout。

---

## 2. 分类职责

| 类别 | 测什么 | 不测什么 |
|------|--------|----------|
| **structure** | `#`/`##`、形参、`>` 调用、`*`/`**`、`+`/`-`、表、导入、位置/具名实参 | 关键字边角 |
| **keywords** | `True`/`False`/`None`、`and`/`or`/`not`、`print`/`input` 约定 | 复杂控制流（可极简） |
| **errors** | 未定义名、实参错误、语法错误的诊断 | 成功路径 |

未单独建夹、但应覆盖的其它测试（放 `tests/` 单测或日后 `examples/runtime/`）：

- **流水线调试**：`--dump-lines` / `--dump-ast` / `--trace-eval`  
- **CLI**：默认 `index.mq.md`、退出码  
- **view**：目录扫描、单文件页、执行区（见 [view.md](view.md)）  
- **模块图**：循环导入拒绝  

---

## 3. 金样例约定

每个可执行 `.mq.md` 在 `tests/gold.rs`（或旁路 `.out` 文件）登记期望 stdout。  
`errors/` 内文件登记期望 stderr 子串与非零退出码。

Frontmatter 可写：

```yaml
---
title: …
expect_stdout: |
  line1
  line2
---
```

（可选；v0 先以测试代码内嵌期望为主。）

---

## 4. 与 `view` 的关系

`marqdo view examples` 按上述文件夹生成侧栏索引；`structure` / `keywords` 为一级分组。
