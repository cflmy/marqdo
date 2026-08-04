# Marqdo 实现依赖说明（Rust 参考解释器）

| | |
|---|---|
| 状态 | 已定 |
| 日期 | 2026-08-04 |
| 相关 | [ADR 0001](../adr/0001-implementation-language.md) · [interpreter 路线图](../roadmap/interpreter.md) |

---

## 1. 要不要 Flex / Bison？

### 结论：**不需要。解释器也不依赖 Flex/Bison。**

| 工具 | 典型用途 | Marqdo 为何不用 |
|------|----------|-----------------|
| **Flex** | 从正则生成 C 词法分析器 | 我们的「词法」是 **行分类 + Markdown 标记**，不是 C 式 `identifier|number|…` 字符流；且目标实现是 **Rust**，不是 C 工具链 |
| **Bison** | 从 Yacc 文法生成 C LR 语法分析器 | Marqdo 表面是 **GFM 文档结构**；整体语法不适合写成单一 Bison 文法。解释器主流做法是 **手写递归下降** 或 Rust 生态的 PEG/组合子，而非 Bison |

[USTC 编译课 Lab1](https://ustc-compiler-2025.github.io/homepage/) 用 Flex/Bison 是为 **Cminusf 编译器** 教学服务的，和「从 `.mq.md` 解释执行」不是同一条产品路径。

**解释型语言同样需要词法与语法分析**，但是：

- 需要的是 **阶段**（lex → parse → AST），不是 **Flex/Bison 这两个具体程序**；
- Python、Ruby、Lua、Crafting Interpreters 的 jlox/clox，大量使用 **手写 lexer + 递归下降**；
- Flex/Bison 是「生成器」选项之一，不是解释器的充分或必要条件。

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
| **insta** 或手写金文件 | examples 期望 stdout 快照（可选） |
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
  tests/
examples/                  # 金样例（已有）
spike/                     # Python 风险探测存档（可保留，非运行时依赖）
```

开发命令示例：

```bash
cargo build
cargo test
cargo run -- run examples/index.mq.md
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
