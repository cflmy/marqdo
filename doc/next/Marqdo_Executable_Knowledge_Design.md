# Marqdo 可执行知识体系：统一语义制品与知识引用设计规范

> 状态：设计提案（非当前版本的已实现功能）  
> 基线：仓库 main，2026-10-23；README 标记 v2.0.0；跨文档类调用与 LinkInfo 为 v2.1 目标  
> 宗旨：**代码即文档，文档即知识库，知识即可执行。**  
> 核心判据：当 AI 遇到任务时，能否先发现已有知识，并以确定性绑定复用，而不是再生成一份“看起来相同”的知识或实现？

## 1. 决策摘要

保持 `[描述](表达式或知识目标)` 的 Markdown 形态，不引入新的前缀或第二套表达式文法。`[]` 是面向读者的陈述，`()` 内的内容进入统一表达式解析器。语言内部建立稳定的 **Semantic Artifact（语义制品）** 和 **Verified Knowledge Reference（可验证知识引用）** 两种核心抽象。前者承载身份、语义、实现、依赖、契约与来源；后者是一个经编译期解析、受上下文约束的引用。它们不必对应新语法标记，而是 `.mq.md` 经语义分析得到的统一中间表示。

**三项不变式：**

1. **表达式透明性**：在求值上下文中 `[L](E)` 与 `E` 取得同一类型和值；`L` 不进入符号绑定或改变结果。引用额外产生诊断与溯源元数据。
2. **文本安全性**：叙述中的知识链接在 `check/view` 中解析；`run` 不因正文出现链接而隐式执行有副作用的函数。导航链接（HTTP、相对文档路径）仍属于普通 Markdown 链接。
3. **知识可信边界**：`resolved`、`compiled`、`tested`、`source_verified`、`domain_validated` 是不同状态。能执行不等于事实真；有来源不等于来源仍然适用。没有证据时允许展示，但不得向 agent 宣称为已验证知识。

### 1.1 不是要做的事

- 不将 Markdown 转成一个独立于解释器的新运行语言。
- 不把所有知识都强制构建为类，也不等同“知识 = 函数”。
- 不把自然语言标签作为权限或事实正确性的证明。
- 不把符号名的字符串匹配当成稳定知识复用。
- 不要求所有调用都锁版本：本地临时表达式可无锁，跨文档公共知识/关键业务则可选或强制锁定。

## 2. 以真实仓库为准的兼容边界

| 语法或能力 | 当前已支持 | 目标设计/注意事项 |
|---|---|---|
| `.mq.md` 和 Markdown 叙述 | 是 | 原有叙述默认安全 |
| `#` 对象/类型，`##` 函数/方法 | 是 | 目标：不依赖 `self` 的函数可通过类命名空间直接调用 |
| `**表达式**`、`*返回*` | 是 | 复用现有表达式解析器，不另造链接表达式 |
| `+ \`参数\``/叙述推断参数 | 是 | 签名必须能静态抽取并生成规范化形式 |
| `1.` 条件分支、`-` 循环 | 是 | 新版链接循环形式须通过模式迁移；旧式依赖 Classic |
| Markdown 表格→List/Map/列向表/行向记录 | 是 | 数据语义与契约表应从上下文区别 |
| Frontmatter `import` | 是 | 可作为当前跨文档路径的兼容实现 |
| `[描述](本地表达式)` | v2.0.0 已有，主要需 `markup: occlusion` | 统一后免模式开关；旧 `[键](集合)` 必须避免静默变义 |
| `类.函数` 无实例直接调用 | 未按目标实现 | **新能力**：区分类函数/实例方法、隐式零参调用 |
| 跨文档项目符号索引 | 尚缺 | **新能力**：可控作用域、同名消歧、增量索引 |
| LinkInfo/知识引用图/links.lock | 目标设计 | **新能力**；与已有 EKC 统一，拒绝第二套知识图 |
| 双执行后端（树遍历、字节码） | 是 | 引用语义必须在两个后端一致 |

项目示例分为：**V2-目标示例**（体现最终范式）与**V2.0-兼容示例**（现有调用方式）。所有目标示例是规范性设计，不宣称已在 v2.0.0 执行验证。

## 3. 知识不是代码字符串：规范化语义模型

### 3.1 Semantic Artifact

每个可引用实体编译为 `Artifact`，而非仅落入 AST 一个节点。

```text
Artifact {
  symbol_id: StableSymbolId,           // 跨增量构建稳定的身份（命名空间 + 显式/规范化路径）
  kind: Type | Entity | Fact | Function | Capability | Constraint | Policy | Dataset,
  visibility: Local | Module | Project | Exported,
  source_span: SourceSpan,
  signature: CanonicalSignature?,      // 参数、返回值、默认值、单位与结构类型
  ir: CompiledIR?,                     // 运行逻辑或规范化常量；普通叙述可无
  effects: EffectSet,                  // pure / fs.read / fs.write / network / llm / time / random ...
  dependencies: [SymbolId],
  contract: [Predicate | Schema | Condition],
  provenance: [SourceRef],
  hashes: { interface, implementation, dependency_closure, evidence },
  verification: VerificationRecord
}
```

区分以下四个概念：

- **Identity（身份）**：目标属于哪一知识单元；名称调整可使用迁移映射，不允许悄悄重绑定到不同实体。
- **Content（内容）**：符号当前定义和可执行 IR；代码变更指纹改变。
- **Applicability（适用性）**：参数、业务前提、单位、时间/地区/场景等约束；匹配才可复用。
- **Evidence（证据）**：来源、测试与验证结果。证据改变不等同于程序改变，但需要重新评估可信状态。

### 3.2 Knowledge Reference

```text
KnowledgeRef {
  label: String,
  payload_ast: Expr,
  context: Narrative | Code | Return | Argument | Table | Heading,
  resolution: ResolvedSymbol | LocalExpression | NavigationURL | Unresolved,
  target_symbol_id: SymbolId?,
  target_digest: Digest?,
  expected_signature: Signature?,
  required_verification: VerificationPolicy,
  source_span: SourceSpan
}
```

`[描述](E)` 在运行语义上是透明的，但其静态解析必须构建 KnowledgeRef。在调用点将地址绑定到 `SymbolId`，不能每次由 AI 重新解释描述。

### 3.3 类型 / 知识实体 / 计算能力分别建模

- `# 诗人` 是 Type；`# 李白 = > 诗人` 在现有语法中仍是**继承/特化**。作为 v2.1 过渡方案可视为“单体知识类”，但不得默认为真正的 `instance_of`。
- 事实可以由 `## 出生年份` + 返回值 + 来源表承载；编译器推导为 Fact Artifact。是否是事实要根据可检查的声明/契约，不应仅凭函数名字猜测。
- 业务函数如 `## 计算应付` 是 Function/Capability；不应因为带中文说明就被误识别成事实。
- 未来需要大规模实体库时，Entity/Fact 应支持表驱动注册，不为每个实体强制生成 Rust 或 Marqdo 类。

## 4. `[]()` 的确定语义

### 4.1 一个表面形式、三种解析结果

| 表面形式 | 静态解释 | `run` | `check` / `view` |
|---|---|---|---|
| `**总额=[应付](数量*单价)**` | 本地表达式遮蔽 | 求值 | 保存标签、表达式和诊断 |
| `**总额=[计算总额](Pricing.计算总额(订单))**` | 知识能力调用 | 绑定后调用 | 检查签名、效应、依赖和版本 |
| `文中参见[税率](Policy.税率)` | 叙述知识引用 | 不执行 | 解析、展示受信任/未验证状态，纯目标可选择显式求值 |
| `[项目文档](./guide.md)` | 导航链接 | 不执行 | 保持 Markdown 导航 |

语法上的“同权”是 `()` 与 `**...**` 共用表达式文法；**不是正文和代码拥有相同的执行许可**。 `view` 对具外部效应的目标默认只展示元数据，必须明确授权才允许运行。

### 4.2 最小语义规则

```text
parse_bracket(label, payload, context) -> KnowledgeRef

context ∈ executable_context:
    verify_resolved_target(); evaluate(payload_ast)
context ∈ narrative_context:
    resolve_only(); never_schedule_effects()
payload ∈ ordinary_navigation:
    preserve_as_markdown_link()
```

### 4.3 唯一性与优先级

1. 当前词法作用域的变量/接收者；
2. 当前文档的声明；
3. 显式 `import` 命名空间；
4. 项目导出索引（受 `--fs-root` 和导出策略约束）；
5. 内置/已安装扩展。

如果两个候选都满足同一级优先规则，报 `link.ambiguous`；不能随文件顺序自动挑一个。解析失败必须有诊断，不能把知识目标悄悄解释成字面字符串或导航路径。

### 4.4 类函数 vs 实例方法

- 目标函数使用 `self/自` → 实例方法，必须有显式接收者。
- 函数不使用接收者 → 可由 `类.函数` 调用；不运行类的构造体。
- 路径解析到零参函数时可以省略 `()`，但**函数作为一等值的语义必须避免与隐式调用冲突**。建议只有被 `[]()` 用作知识目标、或显式值需求时才触发零参隐式调用；直接 `类.函数` 在函数值场景需明确消歧。最终以一致的形式化语义测试冻结。

### 4.5 版本策略与效应

`interface_digest` 代表签名兼容性，`implementation_digest` 代表实现变化，`dependency_closure_digest` 代表传递依赖；`evidence_digest` 代表证据状态。既存调用在接口未变时不必全部报错，但“严格复现”任务应使用 lock 来禁止目标悄然变化。

纯知识事实允许受控求值与缓存；依赖网络、时间、随机数、外部模型的调用即使锁了源码也不天然确定。必须记录输入快照、环境与效应，并明确“不保证结果复现”的状态。

## 5. 静态分析、来源与可信性

### 5.1 五层验证（必须分层报告）

1. **Parse**：Markdown 结构/语法可解析。
2. **Resolve**：目标存在，歧义可消除。
3. **Type/Contract**：签名、入参与结构类型、单位、前置条件匹配。
4. **Execution/Test**：纯函数或受控沙箱完成运行与测试。
5. **Evidence/Domain**：事实来源可访问，内容匹配，满足适用范围；复杂自然语言事实可能需要人工或 LLM 辅助复核，复核结果必须标识非确定性。

严禁将 `check OK` 展示成“现实知识已被证明正确”。

### 5.2 诊断码草案

| Code | 默认级别 | 含义 |
|---|---|---|
| `link.unresolved` | error | 找不到可绑定的目标 |
| `link.ambiguous` | error | 目标符号不唯一 |
| `link.kind_mismatch` | error | 把类型/实例/函数错当为可执行知识 |
| `link.instance_only` | error | 类调用命中实例方法 |
| `link.arg_mismatch` | error | 参数与签名不符 |
| `link.effect_forbidden` | error | 当前位置不允许此类副作用 |
| `link.target_changed` | warning/error | 被锁定或要求稳定的目标发生变更 |
| `knowledge.source_missing` | warning/error | 来源缺失 |
| `knowledge.source_changed` | warning | 来源指纹变化尚未复核 |
| `knowledge.constraint_unmet` | error | 当前输入不满足适用条件 |
| `knowledge.unverified` | info/warning | 可用但未得到所需级别的验证 |
| `label.ref_dangling` | error | 标签中反引号引用无法解析（保留已有规则） |
| `label.possibly_stale` | hint | 代码变更但描述未更新，需要人工检查 |

标签里提到的自然语言不能被静态类型系统完全证明。所有自动语义比对必须报告置信限制。

## 6. 编译器实施架构

```text
Markdown Source
    │
    ├─ CommonMark/GFM-aware parse + Marqdo lexical context
    ▼
Syntax AST (prose + code + table + heading + bracket)
    ▼
Symbol Index (module, class, function, fact, export)
    ▼
Resolution & Type / Contract / Effect Analysis
    ▼
Semantic Artifact IR + KnowledgeRef IR
    ├──────────────┬──────────────────┬───────────────────────┐
    ▼              ▼                  ▼                       ▼
Tree Interpreter   Bytecode VM        View / Debug            EKC/Catalog/MLSP
                                      │                       │
                                      └──────────┬────────────┘
                                                 ▼
                                       Agent deterministic reuse
```

**实现优先原则**：统一在语义分析层生成符号与引用元信息。不要在 view、EKC、Agent 中分别重新解析 `.mq.md` 来猜同一段代码代表什么。

### 6.1 Markdown 链接兼容性

`[描述](含空格或括号的表达式)` 不是任意标准 Markdown 渲染器都可识别的合法 URL 目标。Marqdo 必须定义其**源码级别的上下文语义和渲染行为**：先保留有意义的源码跨度，再由专属 view 转为原生可读的标签及悬浮信息；不能假设 GitHub Markdown 会在所有表达式上正确渲染。代码围栏、行内代码、转义以及普通 URL 链接不能被误识别成可执行引用。

### 6.2 旧语法迁移

Classic 的 `[键](集合)` 曾用于取元，`- [项](集合)` 曾用于循环。默认统一为遮蔽含义前，需要 AST 辅助迁移：

```text
[key](table) → [可读描述](at(table, key))
- [item](list) → - [逐项遍历](遍历 list 为 item)
```

避免靠简单正则批量替换；不能把旧代码成功运行但产生不同值视为兼容。迁移应生成审计报告、变更 diff 与双后端对照测试。

## 7. EKC 与 Agent 复用协议

当前 EKC 已有 Symbol、Capability、Knowledge、Context Pack、REUSE→ADAPT→CREATE。将这些能力变为统一 Semantic Artifact 的**投影/索引**，不要再建第二套符号管理器。

Agent 必须按以下状态机处理：

```text
TASK → DISCOVER → RESOLVE → CONTRACT_CHECK
                       │          │
                       │          ├─ verified + applicable → REUSE → EXECUTE → TRACE
                       │          ├─ adaptable             → ADAPT → VERIFY → PROMOTE
                       │          └─ absent/unusable       → CREATE → VERIFY → PROMOTE
                       └─ ambiguous → REQUEST_DISAMBIGUATION / FAIL_CLOSED
```

- **Discovery 是召回**，可以模糊检索，不等于可信复用。
- **Resolve 必须精确**，让自然语言意图最终绑定到具体 `SymbolId`。
- **Reuse 是编译器行为**：Agent 持有可验证引用，并使用明确的输入与契约，不复制代码段。
- **Promote 是门槛**：新生成内容必须通过所需静态检查、测试与来源规则才可进入已验证知识索引。
- **Trace 可审计**：保存“这次回答具体用了哪些符号、哪些版本和哪些输入”，不是仅存 LLM 的解释文本。

建议新增的 MLSP 读接口：`artifact.find`、`artifact.inspect`、`link.resolve`、`artifact.verify`、`artifact.impact`、`artifact.bind`；后者在编译器确定性解析后返回可调用句柄/引用描述，不应凭自然语言标签直接执行。

## 8. 里程碑与硬验收

### M0 语义冻结

明确新语义与 Classic 的区别，形成机器可读 gold cases，涵盖 URL、嵌套括号、空标签、取元、循环、代码围栏、换行、标题、叙述链接与作用域。所有未经标注的旧语义变化不得静默放行。

### M1 符号身份与项目索引

构建导出符号表、稳定 `SymbolId`、同名诊断。验收：同仓两个 `李白` 定义应报歧义；限定名可解析；改变目录扫描顺序不影响绑定结果。

### M2 统一 Bracket IR 与类函数调用

语法识别→表达式解析→目标绑定→双后端执行。验收：含 `self` 与不含 `self` 的函数正确分派，构造不产生意外副作用；正文不自动执行。

### M3 Semantic Artifact / 验证

签名、effect、contract、provenance、验证记录与指纹。验收：接口修改、依赖修改、证据变化、适用条件不匹配得到不同诊断。

### M4 EKC、view、agent 集成

整个工具链共享 SymbolId/LinkInfo，生成 links graph、可查看调用与来源、可追踪复用。验收：图边与实际编译的调用目标完全一致；Agent 命中已有能力时不得偷偷生成新实现。

### M5 基准实验（对照 Python/Notebook/普通 Markdown+检索）

同一模型、同一任务集、相当的工具与提示预算；分别统计可执行成功率、知识错误引用率、重复实现比例、文件改动数、调用/修复次数、总 tokens、端到端时延、可复核程度。公开任务、生成器、冻结环境、置信区间和失败样本；不要使用只在 Marqdo 方案可解的夹具，也不要把编译器固有优势等同整个 Agent 的领先。

## 9. 设计原则冻结

1. **源码可读，编译结果确定**。
2. **知识是制品，不是文本片段**。
3. **检索可以模糊，绑定必须确定**。
4. **知识能够执行，但执行不等于真实**。
5. **知识可以复用，但必须满足契约与适用条件**。
6. **显示文字可自由，语义判断不依赖显示文字**。
7. **默认保持叙述安全，副作用需要明确执行上下文**。
8. **保持一种语法，一个语义解析器，一套符号索引**。
9. **双后端与 view/EKC 使用同一份语义中间表示**。
10. **失败必须可观察，不能因为链接看起来优雅而隐去错误**。

## 10. 来源依据

仓库：<https://github.com/cflmy/marqdo>。重点阅读：

- `README.md`（v2.0.0 现状与 v2.1 目标）
- `doc/design/markdown-mapping-v0.3.md`（基本语法）
- `doc/design/objects.md`（类、实例、自/self、继承）
- `doc/design/link-syntax.md`（现有链接设计声明）
- `doc/design/occlusion-syntax.md`（遮蔽与描述新鲜度）
- `doc/design/knowledge-layers.md`（知识分层、过渡型单体类）
- `doc/design/engineering-knowledge.md`（EKC 与复用流程）
- `.cursor/skills/marqdo/reference.md` 与 `examples.md`（参考语法；其部分 Classic 旧写法必须经过迁移）

附录项目：`Marqdo_Executable_Knowledge_Example.md`。其中对未实现特性逐项标注，所有业务数据均为虚构演示数据。
