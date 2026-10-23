# 遮蔽语法与可执行知识：一次以证据为准的复核

| | |
|---|---|
| 状态 | **Research · Open（待决策）** |
| 日期 | 2026-10-23 |
| 复核对象 | `marqdo 2.0.0`（`target/release/marqdo`，2026-10-22 提交 `bcfc763`） |
| 触发 | 用户提出：`[]()` 遮蔽语法可能是「AI 时代编程语言的内化核心能力」的入口，并用「李白文档 + main 文档」的跨文档引用作为证据 |
| 方法 | 设计文档 + 源码 + CLI 实测 + `--dump-ast` + 语料统计；以**可复现行为**优先于文档声明 |
| 关联 | [occlusion-syntax.md](../design/occlusion-syntax.md) · [occlusion-description-and-search.md](../design/occlusion-description-and-search.md) · [bracket-as-label-evaluation.md](../design/bracket-as-label-evaluation.md) · [module-namespace.md](../design/module-namespace.md) · [objects.md](../design/objects.md) · [engineering-knowledge.md](../design/engineering-knowledge.md) · [change-m7-migration.md](../design/change-m7-migration.md) |
| 后续 | **[link-syntax.md](../design/link-syntax.md)** — 把本文的评判落成 v2.1 设计声明（类-函数调用 / 项目类索引 / LinkInfo / view / 图边） |

**目录**：0 摘要 · 1 问题/方法 · 2 `[]()` 的实现事实 · 3 两文档复现 · 4 逐条评判 · 5 三个概念混淆 · 6 遮蔽的价值与代价 · 7 工程规范硬证据 · 8 能力阶梯与提案 · 9 路线图与验收 · 10 结论 · 附录 A 复现 · B 证据索引 · C 术语 · D 局限

> **2026-10-23 补记（用户澄清）**：`[]()` **不新增语法**；`()` 就是已有代码，`#`/`##` 是类/函数，`类.函数` 是类调用。本文 §4.3/§5 的「呈现层」判断针对 **v2.0.0 实现**；原始设计缺的是类函数语义与跨文档类索引，不是新语法。设计声明见 [link-syntax.md](../design/link-syntax.md)。

---

## 0. 摘要

### 0.1 一句话结论

**你的方向是对的，但证据不支持「遮蔽语法已经是这个方向的核心」。**

- **可执行知识**是 Marqdo 值得追求的核心方向；
- **`[]()` 遮蔽**是**呈现层**语法，不是语义层能力：`[描述](实现)` 的值恒等于 `实现`，标签不参与绑定、作用域、类型、求值；
- **你给的两文档例子按现状不可执行**：`run` 退出 0、无输出，`check` 通过；`--dump-ast` 显示 `# main` 是空对象、`# 李白` 的方法体为空；
- 你的第三句话——**「工程规范做得不够好」**——**成立**，而且有可复现的硬证据；但根因不是「遮蔽还不够多」，而是「引用、解析、静态校验」这条闭环没有建立。

一句话建议：

> **把遮蔽降级为表层工效学，把「引用 / 解析 / 校验」升级为语言核心。**
> 知识要可执行，先给它**地址**、**解析器**和**验证器**，再谈链接长什么样。

### 0.2 五条判断，先给结论

| # | 你的判断 | 证据结论 | 判定 |
|---|---------|---------|------|
| 1 | Marqdo 已经有 AI 时代的东西 | 有 LLM / agent / MCP / WASM / catalog / EKC / view，但大多是**生态层**，不是语言核心 | ✅ 基本成立 |
| 2 | 接下来需要「内化」的核心能力 | 方向正确；缺的确实是核心，但不是遮蔽 | ✅ 成立 |
| 3 | `[]()` 遮蔽可以充当那个核心能力 | 它是透明的文档注解，**不增加可计算性**，也不提供跨文档解析 | ❌ 不成立（层级混淆） |
| 4 | 两文档例子证明可执行知识的威力 | 例子没有进入执行层；方法体为空；目标不是可调用的静态事实 | ❌ 作为证据不成立 |
| 5 | 设计已经到了这一步，只是工程规范不够好 | 默认仍是 Classic；只有 90/886 文件启用；`check` 不做名字解析（只查契约+标签）；知识图不从 Markdown 链接取边 | ⚠️ 后半句成立，前半句不成立 |

### 0.3 四个最重要的发现

1. **你的例子实际说明的是「叙述默认安全」，而不是「可执行知识」。**
   `[李白的出生年份](李白.李白的出生年份)` 在正文里是 Markdown 链接，不是代码；`--dump-ast` 里 `# main` 完全为空。`run` 的退出码 0 只表示「没有东西可执行」，不表示「链接执行成功」。

2. **`[]()` 的语义是恒等式 `[d](e) ≡ e`，它是「呈现层」，不是「语义层」。**
   它解决的是可读性、带空格函数名、循环来源、桩申报和 view 呈现；它不解决命名、寻址、解析、类型、跨文档引用。把这些说成「核心能力」，是把「读者看到什么」误当成「语言能做什么」。

3. **工程规范问题确实存在，而且是「契约没有闭环」，不是「文档不够多」。**
   README 的 `marqdo labels pending` 实测扫 0 个文件（真实值是 6）；README 说 v2.0.0 遮蔽，默认解析仍是 Classic；`check` 对未定义变量、未知函数实测放行；EKC 的运行图不从正文链接取边。一个以「知识-代码一致性」为旗帜的项目，自己的文档和代码先漂移了。

4. **迁移把「遮蔽」变成了「键名复述」。**
   全仓 231 处 `[label](access(x, "key"))` 形中，**204 处 label 与 key 完全相同（88%）**；设计文档承诺的 `label.provisional`（提醒「标签退化成键名」）在源码中不存在。语法税交了，信息增量没有。

---

## 1. 问题、方法与证据等级

### 1.1 要回答的三个问题

1. `[]()` 遮蔽语法**实际上**做了什么？它是不是「编程语言的核心能力」？
2. 用户给的两文档例子，是否真的展示了「可执行知识」？如果没跑起来，失败在哪一层？
3. 如果目标真是「可执行知识」，Marqdo 还缺什么？应该先补哪一块？

### 1.2 方法

- **读设计**：occlusion / label / module-namespace / objects / EKC 的 Accepted 文档；
- **读实现**：AST、解析模式、求值、字节码、`check`、`mlsp`、`label`、`knowledge::graph`；
- **实测**：在隔离目录 `/tmp/mq-probe` 复现用户的两份文档及其变体，记录退出码、stdout/stderr、`--dump-ast`；
- **统计**：全仓 `.mq.md` 与 `markup: occlusion` 的采用率、`access(` 迁移痕迹、`labels` 命中数。

### 1.3 证据等级

| 等级 | 含义 | 本文用法 |
|------|------|---------|
| **A** | 本机实测（命令 + 输出） | 结论的主要依据 |
| **B** | 源码事实（可定位到文件/行） | 解释行为为何如此 |
| **C** | 设计文档声明 | 用于对照「说的」与「做的」 |
| **D** | 规范推断（CommonMark / 语言设计） | 明确标注为推断，不冒充实测 |

> 本文的原则：**文档说过的，不算数；跑出来的，才算数。**

---

## 2. `[]()` 在实现里到底是什么

### 2.1 AST：它只是一个「带标签的表达式」

来源：[src/ast.rs:261](../../src/ast.rs#L261)

```rust
Occluded {
    label: String,
    payload: Option<Box<Expr>>,
}
```

- `label`：`[]` 里的自由文本，**不执行**，只被扫描关键词、检查反引号引用、做 T1 指纹；
- `payload`：`()` 里的表达式，**唯一执行**；
- `payload: None`：空载荷桩，值为 `None`。

### 2.2 求值：完全透明

来源：[src/interp.rs:1117](../../src/interp.rs#L1117)

```rust
// `<TAB>` M2: the label is inert prose; only the payload evaluates.
Expr::Occluded { payload, .. } => match payload {
    Some(inner) => self.eval_expr(module, fun, env, inner),
    None => Ok(Value::None),
},
```

字节码后端同样只编译 payload（[src/bytecode/compile.rs:831](../../src/bytecode/compile.rs#L831)）。

**形式化**：

> 对任意表达式 `e`、任意标签 `d`：`eval([d](e)) = eval(e)`。
> 标签对求值**零贡献**。

这不是「语义能力」，而是「语义透明的包装」。用 PL 的话说，它是 annotation / transparent wrapper / 文档宏，不是新的求值规则。

### 2.3 标签被用来做什么

| 用途 | 实现 | 说明 |
|------|------|------|
| 关键词检索 | `src/label.rs` + `marqdo labels` | 找「待开发 / 模拟 / 桩」等 |
| 结构诊断 | `label.missing`（error）、`label.empty_payload`（warning） | 空标签 / 空载荷 |
| 引用一致性 | `label.ref_dangling`（error） | **只认反引号实体**，纯口语不检查 |
| T1 过期检测 | MLSP `repair_apply` 的 before/after 指纹 | 只在一次修复会话内；`check` 不报 stale |
| 视图呈现 | `view` 的 `occl-card` | 标签作正文、payload 作 code chip（[src/view/render.rs:1227](../../src/view/render.rs#L1227)） |

注意最后两行：**「过期检测」是会话内、按修复动作触发的**；它不是 CI 级的 `label.stale`。设计文档把它叫「信任负债的代价」，但 CLI 的 `check` 并没有收这笔债。

### 2.4 解析模式：默认仍是 Classic

来源：[src/parse/expr.rs:14](../../src/parse/expr.rs#L14)

```rust
/// The default is `Classic` until M7 migrates the corpus and flips the gate
/// (otherwise every legacy index would silently change meaning).
pub enum Markup {
    #[default]
    Classic,
    Occlusion,
}
```

文件只有在 frontmatter 写 `markup: occlusion`（或 `标记: 遮蔽`）时才按遮蔽解析（[src/load.rs:292](../../src/load.rs#L292)）。同一进程可以按文件切换，import 也各自保持模式。

**这意味着**：

- 一个普通的 `.mq.md` 里写 `[a](b)`，默认含义仍是**取元**，不是遮蔽；
- 用户例子里的 `[]()` 在正文里连取元都不是，只是 Markdown 链接；
- README 的「v2.0.0 遮蔽语法」是**功能可用**，不是**默认语义**。

### 2.5 能力矩阵：它能做什么，不能做什么

| 能力 | 由 `[]()` 提供？ | 证据 |
|------|----------------|------|
| 求值（等于 payload） | ✅ | `interp.rs:1117` |
| 人话标签（不执行） | ✅ | `ast.rs:261` |
| 可读的嵌套取值 | ✅（写法上） | 设计 §1.1 |
| 带空格函数名调用 | ✅（`callee(args)`） | M1 / `m2` 实测 |
| 循环来源可见 | ✅（`遍历`） | M3 |
| 桩申报 / 债务盘点 | ✅ | `label.rs` / `labels` |
| view 呈现 | ✅ | `render.rs:1227` |
| **跨文档引用** | ❌ | `f4` 实测失败 |
| **名字解析 / 静态检查** | ❌ | `check` 对未定义变量放行 |
| **类型 / 契约求值** | ❌ | `check` 只查契约表/标签，不求值 |
| **知识实体 / 事实模型** | ❌ | `Fact` 是 frontmatter 元数据，不是 `[]()` |
| **知识图边** | ❌ | `knowledge::graph` 不从正文链接取边（边来自 import/call/tests/related/reuse/knowledge） |

### 2.6 一句话定义

> **`[]()` 是「给读者看的描述 + 给机器跑的表达式」的绑定语法。**
> 它让读者少读代码，让机器照常执行；它**不**让语言多算任何东西。

这就是为什么它是**表层工效学**，不是**语义核心**。

---

## 3. 用户两文档的逐层复现

### 3.1 原文

`libai.mq.md`：

```markdown
# 李白

这个文档存储了李白的信息

## 李白的出生年份

李白出生于*某年*
```

`main.mq.md`：

```markdown
# main
用户想要直到李白的出生年份，我们要告诉用户[李白的出生年份](李白.李白的出生年份)
```

### 3.2 实测结果

环境：`marqdo 2.0.0`，两份文件在 `/tmp/mq-probe` 下。

| 命令 | 退出码 | stdout | stderr | 解释 |
|------|--------|--------|--------|------|
| `marqdo run main.mq.md` | **0** | 空 | 空 | 没有任何语句执行 |
| `marqdo check main.mq.md` | **0** | `contract check OK: 无契约问题` | 空 | 契约检查通过，不代表程序可运行 |
| `marqdo run libai.mq.md` | **0** | 空 | 空 | 只构造对象，不调用方法 |
| `marqdo run main.mq.md --dump-ast` | 0 | `(object level=1 "main" params=[] @1:1)` | 空 | **`# main` 是空对象；链接不在 AST 里** |
| `marqdo run libai.mq.md --dump-ast` | 0 | 方法 `李白的出生年份` 下无语句 | 空 | **方法体为空，没有返回值** |

`main.mq.md` 的 AST（关键部分）：

```text
(object level=1 "main" params=[] @1:1
)
```

`libai.mq.md` 的 AST（关键部分）：

```text
(object level=1 "李白" params=[] @1:1
  (fun level=2 "李白的出生年份" params=[] @5:1
  )
)
```

### 3.3 失败在哪五层

这不是「一个拼写错误」，而是五个层次的连续缺口：

| 层 | 期望 | 实际 | 证据 |
|----|------|------|------|
| **1. 上下文层** | 正文链接由 `view`/`check` 解析 | 正文默认是叙述（注释），`run` 不解析、不进入 AST | `--dump-ast` 空 |
| **2. 索引层** | 项目类索引能找到类 `李白` | 没有类索引；裸类名不是绑定名 | `f4` 即使 import 也失败 |
| **3. 表达式层** | `李白.李白的出生年份` 是类调用 | 它被当成点号路径的**变量读取**；类函数语义缺失 | `f3`：`undefined variable` |
| **4. 分派层** | 类 `李白` 直接调用函数 | `# 李白` 是类型，`##` 被实现成实例方法；类调用被拒绝 | `n2`：`is an instance method; call it as var.李白的出生年份` |
| **5. 值层** | 方法返回年份 | 方法体为空（`李白出生于*某年*` 是带斜体的叙述，不是返回语句），返回 `None` | `n1` 输出 `None`；`--dump-ast` 空 |

> **关键**：即使把第 1 层补上（加 `**`），第 3/4/5 层仍然会失败。
> 这不是「写法不够甜」，是**语义模型不匹配**。

### 3.4 三个最小可运行改写

#### A. 顶层自由函数 + `>` 调用（跨文档，但不用遮蔽）

`k1_lib.mq.md`：

```markdown
## 李白的出生年份

*701*
```

`k2_main.mq.md`：

```markdown
---
import lb:k1_lib.mq.md
---

# main

**year = > lb.李白的出生年份**
> print text=`year`
```

实测：**输出 `701`，退出码 0**。

#### B. 对象 + 实例方法 + 遮蔽（跨文档，且用 `[]()`）

`lib2.mq.md`：

```markdown
# 李白

## 李白的出生年份

*701*
```

`n3.mq.md`：

```markdown
---
markup: occlusion
import lb:lib2.mq.md
---

# main

**li = > lb.李白**
**year = [出生年份](li.李白的出生年份())**
> print text=`year`
```

实测：**输出 `701`，退出码 0**；AST 里能看到 `Occluded { label: "出生年份", payload: Call(...) }`。

#### C. 纯本地遮蔽（证明它只是表达式包装）

```markdown
---
markup: occlusion
---

# main

**x = [二](1 + 1)**
> print text=`x`
```

实测：**输出 `2`，退出码 0**；默认 Classic 下同一文件会报 `footnote index needs list or map`。

### 3.5 结论

- **能跑的是 A/B 的「模块 + 对象 + 调用」系统**，不是「文档链接」；
- B 里的 `[]()` 只是把 `li.李白的出生年份()` 这行代码包了一层人话；
- 用户原始例子缺的正好是 A/B 的核心：**绑定、调用、值**；
- 所以它**不能**作为「可执行知识已经到位」的证据。它最多证明「可执行知识的语法外形可以长成链接」，而这一点设计文档已经说过。

---

## 4. 对用户判断的逐条评判

### 4.1 判断一：Marqdo 已经有 AI 时代的东西

**基本成立，但要区分层次。**

| 层 | 已有 |
|----|------|
| 语言核心 | 表达式、对象、模块、字节码、契约表 |
| 运行时 | stdlib、插件 ABI、WASM、沙箱、子任务 |
| 工具 | view、debug、catalog、EKC、MLSP |
| AI | ext/llm、ext/agent、MCP、Skill |

这些是**生态层**的 AI 能力。它们能证明 Marqdo 适合做 AI 的工程底座，但不能证明语言核心已经「内化」了知识。**生态丰富 ≠ 语言核心强。**

### 4.2 判断二：需要内化的核心能力

**成立。** 而且这是本文最同意的一句。

Marqdo 现在的强项是「文档即代码」的**呈现**和**执行**；弱项是「知识即实体」的**寻址**、**解析**、**验证**。后者才是「可执行知识」的承重结构。

### 4.3 判断三：`[]()` 可以充当那个核心

**要分两层说。**

- **v2.0.0 的实现**：不成立。它只做了「标签 + 表达式」；`##` 仍是实例方法，跨文档类名不解析，所以它只是呈现层。
- **原始设计**：成立，但前提是 `#`/`##` 真正成为类/函数，`类.函数` 可直接调用，项目类索引可用。这时 `[d](e)` 的语义来自 `e`（类调用），链接只增加 label/元数据，不是新语法。

`[d](e) ≡ e` 仍然成立。问题不是「链接要不要语义」，而是「`e` 能否解析成类/函数调用」。v2.0.0 的答案是不能；v2.1 要补的是这个解析，不是新语法。

**所以：当前实现是「读者的眼镜」；原始设计是「描述 + 地址（类/函数）」。补上类函数语义后，它才是核心。**

### 4.4 判断四：两文档例子证明可执行知识的威力

**作为证据不成立。** 见 §3：例子没有进入 AST，方法体为空，目标不是可调用的静态事实。

但**作为需求**它是成立的：它精确地指出了用户想要的东西——
> 「我在 A 文档里写下一个事实，在 B 文档里用一句人话引用它，B 能执行并拿到值。」

这个需求是真实的；只是 `[]()` 不是实现它的语法。

### 4.5 判断五：设计已经到了这一步，工程规范不够好

**后半句成立，前半句不成立。**

- 「设计到了这一步」：默认 Classic；90/886 文件启用；`lib/` 0 启用；真正带描述的遮蔽标签只有 6 处（其中 4 处在测试）；`check` 不解析名字；知识图不从链接取边。**这不是「到了」，是「有一个可用的原型」**。
- 「工程规范不够好」：**完全成立**，而且比你说的更严重——不是规范执行不严，而是**规范没有闭环**（见 §7）。

### 4.6 总评分

| 维度 | 评分 | 理由 |
|------|------|------|
| 愿景（可执行知识） | ★★★★★ | 方向正确，且与 OKF / literate programming 同构 |
| 对 `[]()` 的定位 | ★★☆☆☆ | 把呈现层当语义层 |
| 对现状的判断 | ★★☆☆☆ | 例子不可执行，默认未启用，采用率低 |
| 对工程规范的判断 | ★★★★★ | 有硬证据，且是当前最大瓶颈 |
| 建议的可执行性 | ★★★★☆ | 先补引用/解析/校验，再谈链接外形 |

---

## 5. 三个概念混淆（理论核心）

这一节解释「为什么这个例子看起来很美，实际却跑不起来」。

### 5.1 混淆一：呈现 vs 语义

| | 呈现层 | 语义层 |
|---|--------|--------|
| 关心 | 读者看到什么 | 程序能解析/验证/执行什么 |
| 例子 | 标签、注释、doc、view | 变量、作用域、类型、调用、模块 |
| 失败后果 | 读者误解 | 程序报错 / 静默错义 |
| **v2.0.0 的 `[]()`** | ✅ 主要在这里 | ❌ 只做透明传递 |

把呈现层说成核心，会导致资源投入到「更好看的语法」，而真正的解析器/校验器没人做。**用户例子跑不起来，正是这个错位的直接后果。**

> **原始设计**：`[]()` 不是纯呈现；`e` 是类/函数调用，语义在 `e`。v2.0.0 没实现类函数解析，所以它**看起来**才像纯呈现。

### 5.2 混淆二：变量读取 vs 类调用

`()` 里是**表达式**，在当前环境求值。`李白.李白的出生年份` 被解析成「变量读取」，所以报 `undefined variable`。

而用户想要的是**类调用**：`李白` 是类，`李白的出生年份` 是函数；解析器应在当前文档/import/项目类索引里找到它并调用。

| | 变量读取（当前） | 类调用（目标） |
|---|------------------|----------------|
| 含义 | 当前环境的一个变量 | 类命名空间里的函数 |
| 解析 | 运行时环境 | 当前文档 / import / 项目类索引 |
| 失败 | `undefined variable` | `link.unresolved` / `link.instance_only` |
| 例子 | `x` | `李白.李白的出生年份` |
| 用户目标 | 被当变量 → 失败 | 类调用 → 未实现 |

**`()` 是表达式槽；类调用是表达式的一种，不需要新的地址语法。**

### 5.3 混淆三：文档 vs 类型

用户的心理模型：

> `# 李白` 是文档标题，`## 李白的出生年份` 是文档的一节，`李白.李白的出生年份` 是这一节的地址。

Marqdo 的实际语义（[objects.md](../design/objects.md#L14)）：

> `#` 是对象/类型；`##` 是函数/方法；方法通过 `_type` 分派，需要实例。

| 用户以为 | Marqdo 实际 |
|---------|------------|
| `# 李白` = 文档 | `# 李白` = 类型 |
| `## 出生年份` = 一节/一个事实 | `## 出生年份` = 方法 |
| `李白.出生年份` = 事实地址 | 点号路径 = 类/函数；不使用 `self` 的函数可类调用，使用 `self` 的需实例 |
| 文档「存储信息」 | 文档「执行代码」；正文是注释 |

这不是小差别，是**文档模型 vs 类/函数模型**的差别。原始设计就是 `#` 类 + `##` 函数 + `类.函数` 类调用；不需要新语法，只需要把 `##` 从「实例方法」恢复为「类函数（`self` 时例外）」并建立跨文档类索引。

### 5.4 Markdown 链接三难

用户想要三件事同时成立：

1. **L**：它仍是普通 Markdown 链接（`[文本](目标)`，目标是一个 URL/路径）；
2. **X**：目标可执行（点一下/引用一下就运行）；
3. **S**：叙述默认安全（无标记的正文永远不执行）。

| 组合 | 是否可能 | 说明 |
|------|---------|------|
| L + S | ✅ | 当前状态：正文链接是惰性的 URL |
| X + S | ✅ | 只在显式代码标记内（遮蔽）；但它不再是「文档链接」 |
| L + X | ✅ | 但正文链接会变成代码入口，破坏 S |
| **L + X + S** | ❌ | **三者不可兼得** |

用户例子假设了 L + X + S 同时成立；这就是它「看起来该跑、实际不跑」的根源。

**结论**：要么接受「正文链接在 `run` 中是惰性的」，要么给「可执行引用」一个显式标记。Marqdo 可以选择前者 + `#`/`##` 类函数解析：**正文链接是 view 的知识调用，代码链接才是 `run` 的执行体**；这样不需要新语法，也不破坏叙述默认安全。

### 5.5 「内化」的五条判定标准

一个能力算不算「语言核心」，可以拿这五问去测：

| # | 问题 | 遮蔽 | 跨文档引用 |
|---|------|------|-----------|
| 1 | **可指称**：有稳定地址吗？ | ❌ 标签不是地址 | ⚠️ 路径是地址，但未解析 |
| 2 | **可绑定**：进入作用域/导入系统吗？ | ❌ 标签不绑定 | ⚠️ import 有，但事实无 |
| 3 | **可组合**：能作为值/参数/返回值吗？ | ✅ 透明传递 | ⚠️ 只能调用，不能当值 |
| 4 | **可验证**：工具链能证明它对吗？ | ⚠️ 只查反引号/空载荷 | ❌ `check` 不解析 |
| 5 | **可观察**：运行时/诊断能看到吗？ | ✅ view / 错误锚点 | ⚠️ 只有运行时错误 |

遮蔽在 5 问里只拿到 2 个 ✅、1 个 ⚠️；**它不是核心，是表层**。
跨文档引用拿到 0 个 ✅、3 个 ⚠️；**它有雏形，但不是能力**。

### 5.6 结论

> **在 v2.0.0 里，`[]()` 只完成了「描述 + 表达式」；在原始设计里，`[]()` 是「描述 + 类/函数调用」。**
> 缺的不是一种新语法，而是 `#`/`##` 的类函数语义与跨文档类索引；补上后，链接的语义来自调用，标签仍只是排版。

### 5.7 相关系统对照：它更像什么，不像什么

以下为高层对照（C/D），不是对各系统规范的逐条审计：

| 系统 / 传统 | 人话与代码的关系 | 跨文档引用 | 执行 | 对 Marqdo 的启示 |
|------------|----------------|-----------|------|-----------------|
| **Literate programming（Knuth）** | chunk 名 + 代码；weave/tangle | 文内 `<<chunk>>`，非跨文档事实 | tangle 后执行 | 遮蔽 ≈ chunk 名；chunk 名是宏地址，不是知识事实 |
| **Org-mode + Babel** | 散文 + 显式 `#+begin_src` | ID/链接用于导航 | 只在 src block 内执行 | **显式标记**是可执行的前提 |
| **Jupyter** | Markdown cell + code cell | 无原生文档间引用 | kernel 执行 | cell 边界是显式语法，不是「长得像链接」 |
| **Observable** | 散文 + 响应式 JS cell | cell 名互相引用 | 数据流执行 | 最接近「可执行知识」，但不是 Markdown 原生 |
| **Obsidian / Roam** | wiki link / backlink | ✅ 链接图 | ❌（插件另说） | **链接图 ≠ 执行图** |
| **Notion / Coda** | 文档 + 公式/按钮 | ✅ 跨文档引用 | ✅（专有） | 地址 + 公式模型，不是 Markdown |
| **OKF** | frontmatter + Markdown 链接 | ✅ 路径即 ID、链接即关系 | ❌（元数据） | 可借鉴「路径即身份」 |
| **Marqdo 现在** | `[label](code)`（代码内） | ❌ | ✅ payload | 表面强，类-函数调用缺 |

**结论**：

- 遮蔽更像 **literate programming 的 chunk 名**或 **Org-babel 的 prose label**：它服务于「读」和「组织」，不是「跨文档求值」；
- 用户的直觉更像 **Obsidian 的链接** + **Observable 的响应式引用**：那需要**地址**和**解析器**，不是把 `()` 换成代码；
- Obsidian 的链接图之所以有用，是因为它**不执行**；一旦链接执行，安全模型就完全不同了。Marqdo 必须在「链接图」和「执行图」之间划清边界。

---

## 6. 遮蔽语法的真实价值与代价

### 6.1 真实价值（这部分是肯定的）

1. **可读调用 / 带空格函数名**
   `[加一之后的值](加一 号(31))` 解决了 D8：`(` 是明确边界，空格不再切词。
   实测 `m2`：`[出生年份](lb.李白的出生年份())` → `701`。

2. **嵌套取值可读**
   `[水果类里苹果的第 1 项价格](at(at(at(货架, 水果类), 苹果), 1))`
   读者只看人话，机器看嵌套。这是真实的认知负担下降。

3. **循环来源可见**
   `- [从篮子里取出的每个果子](遍历 篮子 为 果)`
   比 `- [果](篮子)` 更能自解释「从哪来」。

4. **桩 / 债务申报**
   `[搜索接入，接口未接，待开发](None)`
   让「没接」变成可计数、可检索、可门禁的债务，而不是静默假功能。这是设计里最有价值的工程创新之一。

5. **view 呈现**
   标签作正文、payload 作 code chip；读者先读人话，需要时再看代码。

这些价值都是真的，但都属于**可读性 / 可审查性 / 工程治理**，不是「语言能算得更多」。

### 6.2 代价（这部分必须算清）

| 代价 | 说明 | 证据 |
|------|------|------|
| **同形歧义** | `[a](b)` 可能是 Markdown 链接、旧取元、遮蔽、调用 | 设计文档 §9 / M7 |
| **逐文件方言** | 同一仓库两种语义，靠 frontmatter 区分 | `expr.rs:14` / 90/886 |
| **Markdown 兼容裂缝** | `()` 里是代码，不是 URL；GFM 的链接目标会是代码/非 URL（能否成链接取决于括号/空格），不可导航 | D（CommonMark 推断） |
| **信任负债** | 标签是自然语言，可能说谎；只查反引号 | 设计 §2.2 / `label.rs` |
| **Agent 攻击面** | 标签可能误导 agent（「只读」标签 + 删除 payload） | `bracket-as-label-evaluation.md` §4.3 |
| **迁移语义空壳** | `[key](access(...))` 的标签常等于键，无信息增量 | 231 处形，204 处（88%）label=key；`label.provisional` 未实现 |
| **采用率低** | 90/886 文件启用；`lib/` 0；真正带描述的标签 6 处 | 统计 |

### 6.3 迁移留下的「语义空壳」

迁移工具把旧取元改成了：

```markdown
**timeout = [timeout_sec](access(self, "timeout_sec"))**
```

- 标签 `timeout_sec` 只是键名的重复；
- 设计文档 §1.6 明确说这是**反例**（「描述等于变量名，无信息增量」）；
- 设计文档 §9 承诺 `label.provisional`（hint）来提醒这种退化；
- **源码里没有 `label.provisional`**（`grep` 无结果）。

结果：对 231 处 `[label](access(x, "key"))` 的机械统计显示，**204 处 label 与 key 完全相同（88%）**。这不是「描述」，是「键名复述」；不是为了读者，是为了让语法通过。这不是「内化」，是「语法税」。

### 6.4 采用率：它不是默认，也不是主流

| 目录 | `.mq.md` 总数 | `markup: occlusion` | 占比 |
|------|-------------|---------------------|------|
| `lib/` | 53 | 0 | 0% |
| `ext/` | 31 | 8 | 26% |
| `examples/` | 90 | 7 | 8% |
| `public/` | 100 | 26 | 26% |
| `tests/` | 603 | 48 | 8% |
| **合计** | **886** | **90** | **10.2%** |

（排除 `target/`、`dist/`、`.marqdo/`；统计命令见附录 A。）

- `lib/` 是标准库，0 启用：说明「遮蔽」不是语言运行时的默认契约；
- 真正带**描述性标签**的遮蔽只有 6 处（`labels --json`），其中 4 处在测试；
- 90 个启用文件里，大多数是迁移后的 `[key](access(...))`，不是人写的「一句人话」。

**结论**：遮蔽目前是**一个可用的、局部的、实验性的表面**，不是「全语言的核心能力」。

### 6.5 对遮蔽的最终定位

| 选项 | 评价 |
|------|------|
| 删除遮蔽 | ❌ 可惜；它确实解决了 D8、嵌套可读、桩申报 |
| 把它当核心 | ❌ 层级错误；会继续把资源投到语法糖 |
| **保留为表层工效学，另建类-函数调用层** | ✅ **推荐** |

---

## 7. 工程规范的硬证据：文档-代码漂移

用户的第三句话「工程规范做得不够好」是对的。下面每一条都可复现。

### 7.1 总表

| 文档说 | 实际做 | 证据等级 | 证据 |
|--------|--------|---------|------|
| README：v2.0.0 遮蔽语法 | 默认仍是 Classic；需 frontmatter | A/B | `run e3` 报 `footnote index`；`expr.rs:14` |
| README：`marqdo labels pending` | 扫 0 个文件（真实 6） | A | 附录 A |
| README：`labels search --grep ...` | 扫 0 个文件（真实 2） | A | 附录 A |
| README：Markdown 链接天然长成知识图 | 运行图不从正文链接取边（来自 import/call/tests/...） | B | `knowledge/graph.rs` |
| README：AI 可验证语言 | `check` 只查契约+标签；未定义变量/函数实测放行 | A/B | `check.rs:1`；`e1/e2` |
| 设计：M0–M10 全部完成 | 默认未翻转；per-file 模式 | C/B | `change-m7:22`；`expr.rs:14` |
| 设计：506 处取元 + 44 处循环 | 实施记录 256 + 23（口径未说明） | C | `occlusion-syntax:318` vs `change-m7:11` |
| 设计：`label.provisional` | 未实现 | B | `grep provisional` 无结果 |
| 测试：项目称 605 全绿 | 没有跨文档 headline 场景 | B | `tests/occlusion.rs` 只有本地用例 |

### 7.2 `labels pending`：最危险的「假 0」

README 把这条命令当作「还有多少未完成」的默认口径：

```bash
marqdo labels pending                          # 内置口径：还有多少未完成
marqdo labels pending --count --baseline .marqdo/pending.json --fail-on-increase
```

实测：

```text
$ marqdo labels --count
6
$ marqdo labels pending --count
0
$ marqdo labels --grep 待开发 --count
2
$ marqdo labels search --grep 待开发 --count
0
```

原因（[src/main.rs:631](../../src/main.rs#L631)）：

```rust
// Accept the documented shapes `labels pending` / `labels search`
// as no-op words (the flags carry the real intent).
let paths: Vec<&PathBuf> = paths.iter().filter(|p| {
    let s = p.to_string_lossy();
    !(s == "pending" || s == "search")
}).collect();
```

它把 `pending`/`search` 从路径里过滤掉，但**没有回退到 `.`**。于是「文档里最常用的无路径盘点形式」会扫 0 个文件、返回 0。

**这是最能说明问题的一条**：一个以「不允许假装完成」为工程习惯的项目，自己的默认盘点命令会报告「没有未完成」。如果 CI 用这个无路径形式做棘轮，它会一直通过。

### 7.3 `check`：通过 ≠ 可运行

`check` 的文档写的是「静态契约互查」，这本身没错；但它的输出是：

```text
contract check OK: 无契约问题
```

对下面这个文件：

```markdown
# main

**x = > 不存在的函数 n=1**
> print text=`x`
```

`check` 退出 0，`run` 才报 `unknown function`。未定义变量同理。

`mlsp validate` 也只是 `check_module`（[src/mlsp.rs:199](../../src/mlsp.rs#L199)），所以 AI 的「validate」也拿不到名字解析错误。

**影响**：AI 生成一个「结构正确、名字全错」的程序，`check` 会说 OK；直到运行时才炸。这与 README 的「AI 可验证语言」叙事有距离。

### 7.4 默认 Classic：v2.0.0 的「遮蔽」是 opt-in

`expr.rs` 的注释写得很清楚：默认 Classic，等 M7 迁移完再翻转。但 M7 文档说「per-file 声明」，README 说「v2.0.0 遮蔽语法」。三者并不完全矛盾，但**读者会误解**：

- 「v2.0.0 有遮蔽语法」= 你可以用；
- 「默认是遮蔽」= ❌；
- 「全仓已迁移」= ❌（90/886）。

一个语言特性的「发布」到底是「可用」还是「默认」，必须在 README 里写清楚。否则用户按 README 写 `[a](b)`，得到的是取元，不是遮蔽。

### 7.5 知识图：链接没有变成边

README 的愿景 #4：

> `.mq.md` 之间的 Markdown 链接天然长成文档图 + 知识图 + 程序图（Executable Knowledge Graph）。

实际 `knowledge/graph.rs` 的边来自：

- module 的 `imports` → `depends_on`
- symbol 的 `calls` → `uses`
- tests / related / reuse / knowledge applies_to

**没有一条代码路径**从正文的 Markdown 链接提取边。`catalog` 会把依赖渲染成链接，但那是「图 → 链接」，不是「链接 → 图」。

所以用户例子里的 `[李白的出生年份](李白.李白的出生年份)` 不会成为任何图边；`marqdo catalog` 只会把它当作模块 `main` 的一段叙述。

### 7.6 测试没有覆盖 headline 场景

`tests/occlusion.rs` 覆盖了：

- 本地遮蔽值 = payload；
- Classic / Occlusion 双模式；
- 空标签 / 空载荷 / 反引号悬空；
- labels 检索；
- T1 会话内过期。

它**没有**覆盖：

- 跨文档 import + 对象方法 + 遮蔽标签；
- 正文链接的悬空检测；
- 用户两文档的「事实引用」场景；
- `labels pending` 的文档命令形式。

所以「605 全绿」不能证明「用户例子能跑」；它只证明了被测的部分能跑。

### 7.7 这不是「文档不够多」，是「契约没有闭环」

项目已经有大量设计文档、ADR、变更记录、研究文档。问题不是数量，而是：

1. **文档里的命令没有变成测试**（`labels pending`）；
2. **文档里的承诺没有变成诊断**（`label.provisional`、`label.stale` 在 CLI）；
3. **文档里的默认值没有和代码对齐**（v2.0.0 vs Classic）；
4. **文档里的 headline 例子没有金样**（两文档场景）；
5. **发布说明没有采用率/覆盖率指标**（90/886、6 个标签）。

对一个以「知识-代码一致性」为旗帜的项目，这五条是**自我指涉的失败**：它没有用自己的方法管理自己。

---

## 8. 如果要做成核心：能力阶梯与提案

### 8.1 能力阶梯

| 层 | 能力 | Marqdo 现状 |
|----|------|------------|
| **L0** | 叙述 + 本地表达式 | ✅ |
| **L1** | 变量 / 函数 / 对象 | ✅ |
| **L2** | 模块 / import / 点号路径 | ✅ |
| **L3** | **静态名字解析 / 类型检查 / 悬空引用** | ❌ 缺 |
| **L4** | **知识实体 / 稳定地址 / 图边** | ⚠️ 只有 EKC 元数据，未接执行 |
| **L5** | **可执行知识查询（`类.函数` → 值）** | ❌ 缺 |

**用户以为在 L5；证据显示在 L2 + 一个 L0.5 的呈现层。**

### 8.2 四个必需件

1. **类/函数语义**
   - `#` 是类，`##` 是函数；
   - 不使用 `self`/`自` 的函数是**类函数**，可 `类.函数` 调用；
   - 使用 `self`/`自` 的是**实例函数**，只能 `obj.函数` 调用。
2. **项目类索引**
   - 扫描项目根 `.mq.md` 的 `#`/`##`、签名、位置；
   - 跨文档解析 `类.函数`；同名 `link.ambiguous`；
   - 排除 `# main`；只在根内。
3. **类调用**
   - `类.函数` 零参隐式调用；带参 `类.函数(...)`；
   - 不构造实例；实例函数拒绝；
   - 正文链接由 `view`/`check` 解析；代码链接由 `run` 求值。
4. **验证**
   - `link.unresolved` / `link.ambiguous` / `link.instance_only` / `link.no_value` / `link.arg_mismatch` / `link.type_mismatch`；
   - LinkInfo；`check --json`；MLSP；
   - 图边。

### 8.3 语义提案：不新增语法，补类-函数调用

**不要**给 `()` 增加新语法。`()` 就是 `**` 代码；要补的是下面四条语义：

1. `#` 是类，`##` 是函数；不使用 `self`/`自` 的函数是**类函数**；
2. `类.函数` 是类调用；零参函数隐式调用，带参写 `类.函数(...)`；
3. 项目类索引跨文档解析；同名类 `link.ambiguous`；
4. 代码链接求值；正文链接由 `view`/`check` 解析，`run` 不执行；要终端输出用代码链接 + `print`。

| 场景 | 写法 | 语义 |
|------|------|------|
| 本地表达式 | `[描述](表达式)` | 值 = 表达式；标签 inert |
| 类调用 | `[描述](李白.李白的出生年份)` | 解析类/函数并调用 |
| 实例调用 | `[描述](li.李白的出生年份())` | 现有实例方法 |
| 正文链接 | 同上 | view/check 解析；run 不执行 |

不要新增协议前缀、别名、执行开关；不要靠「内容像路径」来猜语义——靠类索引和解析器。

### 8.4 知识文档约定

`type: Fact` 是 EKC 元数据，不是新语法；知识文档的「值」仍由 `##` 函数返回：

```markdown
---
type: Fact
id: libai.birth_year
title: 李白的出生年份
---

# 李白

## 李白的出生年份

| 返回 | 类型 | 说明 |
|------|------|------|
| 701  | number | 公元 701 年 |

*701*
```

- `# 李白` = 类；`##` = 函数；`*701*` = 值；
- 类函数不使用 `self`，可 `李白.李白的出生年份` 直接调用；
- `type: Fact` 只影响 catalog/EKC，不影响调用；
- 知识函数必须有返回值；无返回 → `link.no_value`。

### 8.5 用户例子的验收形态

`libai.mq.md`：

```markdown
# 李白

这个文档存储了李白的信息。

## 李白的出生年份

*701*
```

`main.mq.md`：

```markdown
# main
用户想要知道李白的出生年份，我们要告诉用户[李白的出生年份](李白.李白的出生年份)
```

期望：

- `view`：显示链接目标 `李白.李白的出生年份`、类/函数、值 `701`；
- `run`：无输出（正文链接不执行）；代码链接 + `print` 输出 701；
- `check`：`link.ok`（class/function/signature）；
- 类名不存在 → `link.unresolved`；
- 函数不存在 → `link.unresolved`；
- 同名类 → `link.ambiguous`；
- 无返回值 → `link.no_value`。

这是「描述 + 已有调用」，不是新语法。

### 8.6 安全边界：标签是不可信文本

- **策略/权限永远不看标签**，只看 payload / 解析后的类函数；
- agent 提示词要写：标签是自然语言，可能误导；以 payload 与类函数解析结果为准；
- `view` 必须同时显示标签与 payload（当前实现已显示 code chip，这是对的）；
- 对「标签说只读、payload 写文件」这类不一致，至少给 hint（设计文档 §4.2 提议过词重叠校验，但未实现）。

---

## 9. 路线图与验收

### 9.0 能力规划总表

| 能力 | 层次 | 现状 | 优先级 | 验收 |
|------|------|------|--------|------|
| 名字解析 / 类型检查 | L3 | 缺 | **P0** | `check --strict` 报未定义变量/函数 |
| 正文链接 / 类调用解析 | L3 | 缺 | **P0** | `check --links` 报 `link.*` |
| 类-函数调用 | L4 | 缺 | **P1** | `类.函数` 直接调用 |
| 项目类索引 | L4 | 缺 | **P1** | 跨文档解析类名 |
| 知识文档 / 事实 | L4 | 部分（EKC 元数据） | **P1** | `type: Fact` + 返回值 |
| 图边（链接 → 图） | L4 | 缺 | **P1** | `graph.json` 有 `calls` 边 |
| 执行 / 查询 | L5 | 缺 | **P1** | `run`/`view` 输出 701 |
| AI schema / 修复 | L5 | 部分 | **P2** | MLSP `link.resolve` |

> **顺序建议**：P0 先让「假成功」消失；P1 再让「类-函数调用」成立；P2 才谈 AI 闭环。不要在 L3 缺失时翻转遮蔽默认。

### 9.1 P0：先止血（1–2 周）

| # | 事项 | 验收 |
|---|------|------|
| P0-1 | 修 `labels` 路径回退：过滤 `pending`/`search` 后若无路径，默认 `.` | `labels pending --count` == `labels --count` == 6 |
| P0-2 | `check` 输出改为「contract check only」；新增 `--strict` 名字解析 | `check --strict e2` 报 `undefined variable`；`e1` 报 `unknown function` |
| P0-3 | 新增 `check --links`：解析正文 `[]()`，校验类/函数 | 用户例子的 `李白.李白的出生年份` 报 `link.*`（或 `link.ok`） |
| P0-4 | 把用户两文档加入金样：要么 `view` 显示 701，要么 `check` 精确报错 | 不允许「exit 0 + 空输出」被当成通过 |
| P0-5 | 修文档：v2.0.0 的「可用 ≠ 默认」；M7 的 per-file 模式；`labels` 命令；506 vs 256 口径 | README / 设计文档与实测一致 |
| P0-6 | `label.provisional` 落地，或迁移时不再生成「键名标签」 | 231 处 `access` 标签（204 处 label=key）有 hint 或被清理 |

### 9.2 P1：补类-函数调用（核心）

| # | 事项 | 验收 |
|---|------|------|
| P1-1 | 补齐 `#`/`##` 的类-函数语义：`类.函数` 直接调用；项目类索引解析跨文档 | 正文可写 `[描述](李白.李白的出生年份)` |
| P1-2 | 静态解析器：类 → 函数 → 契约 | `check` 报 `link.unresolved` / `link.type_mismatch` |
| P1-3 | 知识文档：`type: Fact` + 返回值 + 表/函数 | `run`/`view` 取值 701；`find`/`impact` 能用 |
| P1-4 | EKC 从链接取边；`catalog` 图与运行时解析同源 | `graph.json` 有 `calls` / `references` 边 |
| P1-5 | `view` 解析链接并显示 provenance | 点击/展开能看值与来源 |

### 9.3 P2：AI 原生闭环

| # | 事项 | 验收 |
|---|------|------|
| P2-1 | MLSP `link.resolve` / `link.check` / schema | AI 可查「这个链接指向哪个类/函数、类型对不对」 |
| P2-2 | agent 提示词：标签不可信；以 payload/类函数为准 | 注入「只读标签 + 删除 payload」测试 |
| P2-3 | 指标：采用率、解析率、check 覆盖率、stale 标签 | 发布报告可见 |
| P2-4 | 基准：AI 生成带类调用的文档 vs 纯代码 | 错误率/修复轮次/token 对比 |

### 9.4 总验收（一句话）

> **用户的两份文档：`view` 显示 701，`check` 返回 `link.ok`；加 `print` 的代码链接输出 701。**
> **不允许「退出 0、无输出、显示 OK」这种假成功。**

---

## 10. 结论

### 10.1 直接回答「遮蔽到底有什么用」

**v2.0.0 的 `[]()`**：

- 它让**读者**不必读复杂表达式（嵌套、长算式、循环来源）；
- 它让**带空格的函数名**有了明确边界；
- 它让**未完成的桩**变成可检索、可计数、可门禁的债务；
- 它让 **view** 能先显示人话、再显示代码；
- 它**不**提供类-函数调用、跨文档类索引、名字解析、LinkInfo、图边。

**原始设计还要加的四件事**（都不需要新语法）：

1. `#`/`##` 的类/函数语义；
2. `类.函数` 类调用；
3. 项目类索引跨文档解析；
4. view/LinkInfo/图边。

### 10.2 对你思路的最终评判

| 你的话 | 评判 |
|--------|------|
| 「可执行知识是核心」 | ✅ 对 |
| 「工程规范不够好」 | ✅ 对，且有硬证据 |
| 「`[]()` 遮蔽可以充当内化核心」 | ⚠️ **v2.0.0 实现**不成立（只是呈现层）；**原始设计**成立，前提是补类-函数语义与类索引 |
| 「两文档例子证明威力」 | ❌ 当前例子跑不起来（正文不解析、类调用未实现、函数无返回值） |
| 「设计已经到了这一步」 | ❌ 到了原型，没到核心；默认未启用，采用率 10% |

### 10.3 一句话建议

> **先把 `#`/`##` 的类-函数语义与项目类索引做进语言核心；`[]()` 只是「描述 + 已有调用」，不需要新语法。**

当 `[李白的出生年份](李白.李白的出生年份)` 能让 `view` 显示目标/签名/值、让 `check` 返回 `link.ok`、让 `run`（加 print）输出 701、让 EKC 建边时，「可执行知识」才真正展示威力。那时你会发现：**它不需要 `[]()` 变成新语法；它需要 `#`/`##` 真正成为类/函数。**

---

## 附录 A：复现命令

以下命令在 `marqdo 2.0.0` 下执行；`M=/workspace/marqdo/target/release/marqdo`。

### A.1 用户两文档

```bash
mkdir -p /tmp/mq-probe && cd /tmp/mq-probe
cat > libai.mq.md <<'EOF'
# 李白

这个文档存储了李白的信息

## 李白的出生年份

李白出生于*某年*
EOF
cat > main.mq.md <<'EOF'
# main
用户想要直到李白的出生年份，我们要告诉用户[李白的出生年份](李白.李白的出生年份)
EOF

$M run main.mq.md; echo "exit=$?"
$M check main.mq.md; echo "exit=$?"
$M run main.mq.md --dump-ast
$M run libai.mq.md --dump-ast
```

### A.2 遮蔽在代码里才执行

```bash
cat > p5_prose.mq.md <<'EOF'
# main

[运行一个副作用](print("PROSE-RAN"))
EOF
cat > p4_code.mq.md <<'EOF'
---
markup: occlusion
---

# main

**x = [运行一个副作用](print("CODE-RAN"))**
EOF
$M run p5_prose.mq.md --dump-ast   # 空对象
$M run p5_prose.mq.md               # 无输出，exit 0
$M run p4_code.mq.md                # CODE-RAN
```

### A.3 跨文档：三种写法

```bash
# A. 顶层函数 + > 调用
cat > k1_lib.mq.md <<'EOF'
## 李白的出生年份
*701*
EOF
cat > k2_main.mq.md <<'EOF'
---
import lb:k1_lib.mq.md
---

# main

**year = > lb.李白的出生年份**
> print text=`year`
EOF
$M run k2_main.mq.md                # 701

# B. 对象 + 实例方法 + 遮蔽
cat > lib2.mq.md <<'EOF'
# 李白
## 李白的出生年份
*701*
EOF
cat > n3.mq.md <<'EOF'
---
markup: occlusion
import lb:lib2.mq.md
---

# main

**li = > lb.李白**
**year = [出生年份](li.李白的出生年份())**
> print text=`year`
EOF
$M run n3.mq.md                     # 701

# C. 静态调用对象方法（应失败）
cat > n2.mq.md <<'EOF'
---
markup: occlusion
import lb:lib2.mq.md
---

# main

**year = [出生年份](lb.李白.李白的出生年份())**
> print text=`year`
EOF
$M run n2.mq.md                     # error: ... is an instance method
```

### A.4 静态检查的假通过

```bash
cat > e1.mq.md <<'EOF'
# main
**x = > 不存在的函数 n=1**
> print text=`x`
EOF
cat > e2.mq.md <<'EOF'
# main
**x = 未定义变量 + 1**
> print text=`x`
EOF
$M check e1.mq.md; echo "exit=$?" # 0，输出 contract check OK
$M run e1.mq.md;   echo "exit=$?" # 1，unknown function
$M check e2.mq.md; echo "exit=$?" # 0
$M run e2.mq.md;   echo "exit=$?" # 1，undefined variable
```

### A.5 `labels` 文档命令的假 0

```bash
cd /workspace/marqdo
$M labels --count
$M labels pending --count
$M labels --grep 待开发 --count
$M labels search --grep 待开发 --count
```

### A.6 采用率统计

```bash
cd /workspace/marqdo
for d in lib ext examples public tests; do
  t=$(find "$d" -name '*.mq.md' | wc -l)
  o=$(grep -rl '^markup: occlusion' --include='*.mq.md' "$d" 2>/dev/null | wc -l)
  echo "$d total=$t occlusion=$o"
done
$M labels --json

# 迁移形中 label 与 key 完全相同的比例
find lib ext examples public tests -name '*.mq.md' -print0 \
  | xargs -0 perl -ne 'while (/\[([^\]]+)\]\(access\(([^,]+),\s*"([^"]+)"\)\)/g) { $t++; $s++ if $1 eq $3; } END { print "total=$t same=$s\n" }'
# total=231 same=204
```

---

## 附录 B：证据索引

| 主题 | 文件 |
|------|------|
| AST 节点 | [src/ast.rs:261](../../src/ast.rs#L261) |
| 求值透明 | [src/interp.rs:1117](../../src/interp.rs#L1117) |
| 字节码透明 | [src/bytecode/compile.rs:831](../../src/bytecode/compile.rs#L831) |
| 默认 Classic | [src/parse/expr.rs:14](../../src/parse/expr.rs#L14) |
| frontmatter 模式 | [src/load.rs:292](../../src/load.rs#L292) |
| 标签检查 | [src/label.rs:1](../../src/label.rs#L1) |
| `check` 只查契约+标签 | [src/check.rs:1](../../src/check.rs#L1) |
| `mlsp validate` | [src/mlsp.rs:199](../../src/mlsp.rs#L199) |
| `labels` 路径过滤 | [src/main.rs:631](../../src/main.rs#L631) |
| EKC 图边 | [src/knowledge/graph.rs:1](../../src/knowledge/graph.rs#L1) |
| `type: Fact` | [src/knowledge/extract.rs:313](../../src/knowledge/extract.rs#L313) |
| 遮蔽渲染 | [src/view/render.rs:1227](../../src/view/render.rs#L1227) |
| 遮蔽设计总纲 | [occlusion-syntax.md](../design/occlusion-syntax.md) |
| 信任负债 | [occlusion-syntax.md:173](../design/occlusion-syntax.md#L173) |
| 标签漂移/安全面 | [bracket-as-label-evaluation.md:160](../design/bracket-as-label-evaluation.md#L160) |
| per-file 迁移 | [change-m7-migration.md:22](../design/change-m7-migration.md#L22) |
| 点号寻址 / 实例方法 | [module-namespace.md:117](../design/module-namespace.md#L117) |
| 对象 / 方法 | [objects.md:14](../design/objects.md#L14) |
| 知识类型 | [engineering-knowledge.md:31](../design/engineering-knowledge.md#L31) |
| 遮蔽测试 | [tests/occlusion.rs](../../tests/occlusion.rs) |

---

## 附录 C：术语

| 术语 | 本文用法 |
|------|---------|
| **遮蔽（occlusion）** | `[描述](实现)`；标签不执行，payload 执行；`[d](e) ≡ e` |
| **遮蔽（shadowing）** | 编程语言里「内层绑定遮住外层」；**本文不取这个义**，但中文同名，需在文档里区分 |
| **表达式** | 在当前环境求值的语法；`()` 里的东西 |
| **引用（reference）** | 在命名空间里指称实体；应由解析器/链接器处理 |
| **地址（address）** | 稳定 ID / 路径 / 锚点；引用的目标 |
| **解析（resolution）** | 把名字/路径映射到实体，并检查可见性/类型 |
| **呈现层** | 读者看到什么；标签、view、doc |
| **语义层** | 程序能解析/验证/执行什么；作用域、类型、调用、模块 |
| **L0–L5** | 本文的能力阶梯，见 §8.1 |

---

## 附录 D：局限

1. **GFM 渲染未在本机实测**：`[]()` 在 GFM 里渲染为 `<a href="...">` 是 CommonMark 规范推断（D），不是本机截图；但设计文档自己也说它「渲染成超链接」。
2. **未做性能/规模测试**：本文关注语义与工程规范，不评价 1000 标签文件的性能。
3. **未逐个审计 90 个 occlusion 文件**：采用率是机械计数；「label=key」统计是对 `access(...)` 形做的正则匹配（231/204），没有逐个人工判断语义。
4. **`type: Fact` 的运行时接入是提案**：当前代码只把它当 EKC 元数据；本文没有实现它。
5. **用户意图存在两种读法**（遮蔽 vs 引用）：本文两种都评估了；如果用户本意只是「遮蔽」，那么 §3 的第 3/4/5 层失败仍然成立，只是「引用」部分不适用。

---

## 附：一句话总结

> **遮蔽让读者不必读代码；引用让知识可以被找到。**
> 前者是排版，后者是语义。
> Marqdo 现在有一个不错的排版层，但还没有语义层的地址、解析与验证。
> 把这两件事分开，才是「可执行知识」真正变成核心能力的开始。
