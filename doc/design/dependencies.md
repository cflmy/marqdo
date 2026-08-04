# Marqdo 实现依赖说明（Rust 参考解释器）

| | |
|---|---|
| 状态 | 已定 |
| 日期 | 2026-08-04 |
| 相关 | [ADR 0001](../adr/0001-implementation-language.md) · [interpreter 路线图](../roadmap/interpreter.md) |

---

## 1. 要不要 Flex / Bison？

### 结论：**不需要。** 主因不是「用了 Rust」，而是 **Marqdo 的语法形态 + 解释器并不绑定生成器。**

把两件事分开：

| 问题 | 答案 |
|------|------|
| 因为 Rust 才不用 Flex/Bison？ | **否。** Flex/Bison 生成的是 C（也可接 C++）。若实现改成 **C++**，**仍然不建议**用它们做 Marqdo 主前端。 |
| 因为是解释器才不用？ | **部分相关，但不是全部。** 解释器**需要**词法/语法阶段，但**不规定**必须用 Flex/Bison；手写递归下降是主流。即便将来做「编译到字节码/原生」，Marqdo 也因 Markdown 外壳而不适合整语言 Bison 化。 |
| 真正主因 | **Markup-as-Syntax**：源是 GFM 文档结构（标题/列表/强调/表格/行分类），不是 Cminusf 那种纯字符流文法。 |

| 工具 | 典型用途 | Marqdo 为何不用（与宿主语言无关） |
|------|----------|----------------------------------|
| **Flex** | 正则 → C 词法分析器 | 「词法」是 **行分类 + Markdown 标记**，不是经典 `id\|num\|op` 字符流 |
| **Bison** | Yacc 文法 → C LR 分析器 | 整体语法绑在 **GFM 块结构**上，硬写成单一 Bison 文法别扭且难与预览器对齐 |

[USTC 编译课 Lab1](https://ustc-compiler-2025.github.io/homepage/) 用 Flex/Bison，是因为对象语言是 **C 式源码的编译器**；换 C++ 写那个编译器也合理。  
Marqdo 换 **Rust 或 C++** 都不改变「主前端不该是 Flex/Bison」这一判断。

**解释型语言同样需要词法与语法分析**，要的是阶段，不是这两个具体程序。

---

## 2. Marqdo 前端应怎样做（替代 Flex/Bison）

拆成三层，比「一个大 lexer/parser」更贴合宪法：

```
.mq.md 文本
  │
  ├─① 行分类（自研）        无标记 → 注释；有标记 → 代码行
  ├─② GFM 块/行内（库）     标题、表、强调、列表结构
  ├─③ 行扫描补强（自研）    保留 '-' / '+'；frontmatter `> file.mq.md`
  └─④ Marqdo 语法（自研）   递归下降：Module/Fun/Stmt/Expr → AST
         └─ 表达式子语言     可手写，或仅对 Expr 用 pest/nom
```

| 阶段 | 做法 | 对应编译学 |
|------|------|------------|
| ①③ | Rust 手写扫描 | 词法 / 扫描器 |
| ② | `pulldown-cmark` 或 `comrak` | 复用 GFM，避免自研表格/强调 |
| ④ | **手写递归下降**（推荐主路径） | 语法分析 |
| Expr（可选） | `pest`（PEG）或 `nom`（组合子） | 仅迷你表达式，不是整门语言 |

**不推荐**整门语言上 LALRPOP / rusty_lr / Bison 等价物作为 v1 主前端：Markdown 外壳 + 行语义会让文法极度别扭。若将来有「纯文本 Marqdo 方言」，再另议生成器。

---

## 3. 详细依赖清单（Rust）

### 3.1 工具链（开发机）

| 依赖 | 用途 | 安装 |
|------|------|------|
| **Rust**（stable，建议 ≥ 1.75） | 编译实现 | [rustup](https://rustup.rs/)：`rustup default stable` |
| **Cargo** | 包与构建 | 随 rustup |
| **rustfmt / clippy** | 格式与静态检查 | `rustup component add rustfmt clippy` |
| **Git** | 版本管理 | 已有 |

**不要安装：** Flex、Bison、LLVM（v1 解释器不需要）。  
（日后若做原生后端再开 ADR；与当前解释器无关。）

### 3.2 Cargo  crate（建议写入根 `Cargo.toml` / workspace）

#### 必选（Phase I 树遍历解释器）

| Crate | 角色 | 说明 |
|-------|------|------|
| **pulldown-cmark** *或* **comrak** | GFM 事件/AST | 标题、列表、表格、emphasis/strong；Spike 已证明「库 + 行扫描」可行。二选一做 Spike 对比后钉死 |
| **thiserror** / **anyhow** | 错误类型 | 诊断与 CLI 报错 |
| **clap** | CLI | `marqdo run [path]` |
| **serde** + **serde_yaml** | 元信息 | 仅解析 frontmatter 中的 YAML **键值**；`> x.mq.md` 仍按**行**抽取（见 Spike） |

#### 强烈建议

| Crate | 角色 |
|-------|------|
| **camino** 或标准 `PathBuf` | 路径 |
| **indexmap** | 保序 map（模块导出等） |

#### 测试

| Crate | 角色 |
|-------|------|
| **rust 自带** `#[test]` | 单元测试 |
| **insta** 或手写金文件 | tests 期望 stdout 快照（可选） |
| **assert_cmd** / **predicates** | CLI 集成测试（可选） |

#### Phase II（字节码 VM，后加）

无强制新依赖；VM 以自研为主。可选：

| Crate | 角色 |
|-------|------|
| **num_enum** 等 | opcode 枚举（可选） |

#### 明确不引入（v1）

| 东西 | 原因 |
|------|------|
| flex / bison / lemon | 见 §1 |
| lalrpop / rusty_lr 作**整语言**前端 | 与 MD 外壳冲突；递归下降更合适 |
| llvm-sys / inkwell | 非解释器 v1 |
| 完整 Python 运行时嵌入 | 参考实现已定为 Rust，避免双运行时 |

### 3.3 表达式子语言（可选增强）

若手写 Expr 递归下降过痛，**仅**对 `` `a` + 1 == `b` `` 这类表达式引入其一：

| 选项 | 类型 | 何时用 |
|------|------|--------|
| **手写递归下降** | 无新依赖 | 默认；与全书一致、错误信息最好控 |
| **pest** | PEG 生成器 | 文法清晰、想要 `.pest` 文件时 |
| **nom** | 解析组合子 | 要零拷贝/细控时 |

整文件 `.mq.md` **不要**整份丢给 pest/nom。

---

## 4. 仓库布局（与依赖对应）

```
Cargo.toml                 # workspace 或单包
crates/marqdo/             # 或根包 marqdo
  src/
    main.rs / lib.rs
    lex/                   # 行分类、Token（自研，无 flex）
    parse/                 # 递归下降（自研，无 bison）
    ast/
    sema/
    interp/                # Phase I
    bytecode/              # Phase II
    runtime/
  tests/                   # gold.rs + structure|keywords|errors
public/                    # 用户可执行文档
spike/                     # Python 风险探测存档（可保留，非运行时依赖）
```

开发命令示例：

```bash
cargo build
cargo test
cargo run -- run tests/structure/nested-call.mq.md
```

---

## 5. 与 Python Spike 的关系

| | Python `spike/` | Rust 参考实现 |
|--|-----------------|---------------|
| 角色 | 已验证 S1–S5 风险 | **正式**词法/语法/语义/解释器 |
| 是否运行时依赖 | 否 | 是 |
| 算法可迁移 | 行扫描、`split_front_matter`、导入正则 | 用 Rust 重写同算法 |

---

## 6. 一句话对照

- **需要：** 词法（行分类+标记）、语法（递归下降→AST）、语义、解释器/日后 VM。  
- **不需要：** Flex、Bison、LLVM（v1）。  
- **外部库只需：** Rust 工具链 + GFM crate（pulldown-cmark/comrak）+ CLI/错误/测试若干；表达式可选 pest/nom。
