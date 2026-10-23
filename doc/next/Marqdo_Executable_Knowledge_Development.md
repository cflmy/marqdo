# Marqdo 可执行知识系统详细开发规范

| | |
|---|---|
| 状态 | **目标实施规范；按文档原生指导修订交付与政策分级** |
| 日期 | 2026-10-23 |
| 适用版本 | v2.1 及后续兼容演进 |
| 真实意图来源 | [`009.md`](009.md)、[`Marqdo_Executable_Knowledge_Design.md`](Marqdo_Executable_Knowledge_Design.md)、[`Marqdo_Executable_Knowledge_Example.md`](Marqdo_Executable_Knowledge_Example.md) |
| 核心目标 | **代码即文档，文档即知识库，知识库可执行，可执行知识可确定性复用。** |

## 0. 文档地位与执行规则

本文把上述三份文档中的真实产品意图整理为可编码、可测试、可迁移、可验收的开发规范。2026-10-23 的[文档原生开发指导](../design/document-native-development-guide.md)进一步约束作者负担、AI 编辑与复用、场景分级和交付顺序；本文的内部结构不能反向要求作者重复填写注册表、工具 schema 或知识图。仓库中更早的链接、遮蔽、知识分层、类模型、OKF 或 EKC 文档若与上述方向冲突，不得据其改变本轮设计。

优先级从高到低如下：

1. `009.md` 中确认的产品目标与边界；
2. [文档原生开发指导](../design/document-native-development-guide.md)对作者体验、政策分级和交付排序的修订；
3. 本文的工程决策、数据结构和验收条件；
4. 两份附属文档中的概念设计与完整示例；
5. 其他旧文档，仅可作为历史背景，不是规范依据。

本文不宣称目标能力已经实现。未另行标日期的“当前”表格与 §2 缺口保留最初 v2.0.0 审计背景，不能继续当作 v2.1 的实时状态：默认 Occlusion、线程局部解析作用域、关联函数与零参路径等已经演进。当前实现判断见[2026-10-23 缺口审计](../roadmap/executable-knowledge-gaps-2026-10-23.md)及对应源码/测试；代码事实证明实现范围，不自动取代产品目标。任何阶段只有在对应的自动化测试、双后端一致性测试和迁移测试全部通过后，才可改标为“已实现”。CLI/API 示意与结构定义仍是目标协议，使用前须核对实现。

---

## 1. 产品目标与成功判据

### 1.1 目标链条

Marqdo 不是“能运行 Markdown 中几段代码”的解释器，也不是“给 RAG 提供更多 Markdown”的资料库。它要建立以下连续链条：

```text
代码即文档
  └─ 程序、说明、契约和示例共址
       ↓ 编译
文档即知识库
  └─ 内容拥有稳定身份、类型、关系、来源和适用条件
       ↓ 确定性绑定
知识库可执行
  └─ 知识不是重新解释的文本，而是可调用的编译制品
       ↓ 发现、校验、执行、追踪
可执行知识可复用
  └─ 人和 Agent 优先复用既有能力，不重复生成同义实现
```

### 1.2 产品级成功判据

最终系统必须同时满足以下条件：

- 人只维护 `.mq.md`、原始资料和显式配置，不手工同步另一份知识图；
- 同一份源码能够生成运行程序、语义制品、引用图、可读 view 和 Agent 接口；
- `[描述](载荷)` 在可执行位置与裸载荷取得同一值，在叙述位置默认不产生副作用；
- 跨文档引用最终绑定到稳定 `SymbolId`，不能只保留名字字符串；
- 模糊搜索只负责发现候选，真正执行前必须精确解析、校验契约和适用性；
- 代码、依赖、来源或验证状态变化时，影响范围能够被程序找出；
- Tree Interpreter 与 Bytecode VM 对同一已绑定程序产生一致结果和诊断；
- Agent 命中已有且适用的制品时，得到的是可调用句柄，不是让模型复制源码再写一遍；
- 系统从不把 `compiled`、`tested` 或 `source_present` 误报成“现实世界事实已证明正确”；
- 作者能从正文计算开始，按需要提取命名能力与跨文档复用，不先建立知识治理工程；
- 人和 AI 能按标题/步骤获得最小上下文、局部修改并查看语义影响，不必整篇重生成；
- 每个交付切片同时证明阅读、修改或复用的实际收益，而不只证明内部字段存在。

### 1.3 不在本轮目标内

- 不设计第二套链接专用表达式语言；
- 不增加 `knowledge:`、`call:`、`ref:` 等作者必须记忆的前缀；
- 不把全部知识强制建模为类或函数；
- 不在第一阶段实现通用 OWL/RDF 本体推理；
- 不承诺网络、时间、随机数、LLM 或外部进程可由源码哈希完全重放；
- 不用 LLM 替代确定性的解析、绑定、类型检查和权限检查；
- 不把自然语言标签当作授权、契约或事实正确性的依据；
- 不让 EKC、view、check、Agent 各自再次解析源码并形成不同理解。

### 1.4 作者成本与渐进严格

标题、叙述、表格、链接描述和执行语句是作者的主要工作面；符号 ID、步骤 ID、规范签名、效应闭包、工具 schema、图和摘要由工具生成。不能推导的领域条件、来源和许可由作者明确补充，不要求每个纯工具函数都填写事实证据。

本地探索允许无锁运行草稿，但不绕过宿主能力限制；项目自动复用按所选 policy 校验；严格业务再增加必要证据、版本锁和新鲜度；远程/多租户另有可信授权与隔离门槛。这是同一语法上的政策分级，不是新的语法模式。纯预览可复用已授予的项目/会话政策，不做逐链接重复确认。详情见[指导 §6](../design/document-native-development-guide.md#6-渐进严格不把治理门槛塞进每一行正文)。

自然语言标签是发现、解释和局部编辑的重要信息，但不证明 payload 正确。AI 可以选候选、组合、适配、重构及补测试；编译器确定实际绑定、类型和效应，宿主决定能力上限。不得把“不用模型决定语义”误读成“模型只能搜索”。

---

## 2. 初始实现基线与缺口背景（非 v2.1 实时清单）

### 2.1 可以复用的现有基础

| 位置 | 当前能力 | 本轮处理 |
|---|---|---|
| `src/ast.rs` | `Module`、`Function`、`Stmt`、`Expr::Occluded`、`Span` | 保留 Syntax AST；补正文结构、节点身份和语义映射，不直接塞入全部分析结果 |
| `src/parse/expr.rs` | Classic/Occlusion 两种 `[]()` 解释；payload 复用表达式解析 | 迁移后冻结为统一链接语义，移除运行时全局模式依赖 |
| `src/load.rs` | 递归 import、模块绑定、循环导入检测 | 拆分“源码装载”和“项目编译”；显式 import 仍是可执行代码的规范依赖 |
| `src/contract.rs`、`src/check.rs` | 渐进式契约、类型名、漂移和错位检查 | 接入统一 Sema；扩展为引用、effect、provenance、lock 诊断 |
| `src/interp.rs` | Tree 执行、实例方法、运行时契约 | 改为执行已解析引用；不在运行时重复猜目标 |
| `src/bytecode/` | Bytecode 编译与 VM、函数/实例方法调用 | 消费相同的 Bound IR，新增静态目标调用，不自行解析名字 |
| `src/knowledge/` | EKC、L0–L4、检索、reuse、impact、preflight | 保留用户接口；内部改成 Semantic Artifact/Graph 的投影与查询层 |
| `src/mlsp.rs` | `locate`、`validate`、`schema` 等 JSONL 接口 | 增加 artifact/link/verify/impact/bind 只读接口 |
| `src/view/` | AST 文档、执行捕获、注解 | 显示同一份 LinkInfo、验证状态、来源与依赖，不自行推断目标 |
| `src/artifact/` | Web/Endpoint/Prompt 元数据分类 | 避免命名冲突；迁为 `artifact::web` 或更名，不能冒充统一语义制品 |

### 2.2 必须修复的结构性缺口

1. `Expr::Occluded` 只有标签与 payload，没有上下文、解析目标、制品身份和引用策略。
2. 正文目前不会完整进入能做知识引用分析的结构化 AST。
3. Classic/Occlusion 用进程全局状态和 frontmatter 切换，同一形状可能静默表示索引或遮蔽。
4. `CallExpr.path` 在 Tree 和 Bytecode 中分别解释，绑定不是独立编译阶段。
5. 当前路径调用把对象下函数统一拒绝为实例方法，未根据 `self/自` 使用情况区分类函数和实例方法。
6. EKC 的符号、调用和图边主要由名字/文本提取形成，不能保证与实际执行目标一致。
7. 当前 fingerprint 更偏向相似性检索，不是接口、实现、传递依赖和证据的规范内容摘要。
8. `check` 主要做契约检查，没有项目级 Resolve、LinkInfo、effect、来源和 lock 校验。
9. Agent 的 REUSE/ADAPT/CREATE 可以发现相似条目，但没有“已绑定且可调用”的强制交接点。

### 2.3 两个必须避免的概念冲突

**Artifact 命名冲突**：当前 `src/artifact/mod.rs` 的 `ArtifactKind` 只是 Web/Endpoint/Prompt 分类。统一语义制品应命名为 `SemanticArtifact`，放入新的 `src/sema/`；现有模块迁到 `src/artifact/web.rs` 或改名为 `document_artifact`。

**Knowledge IR 重复真相源**：当前 `src/knowledge/ir.rs` 是 EKC 输出模型。它不得升级成第二套编译 AST。编译器先生成 `SemanticProject`，EKC 再无损投影成兼容的 catalog/graph/context pack。

---

## 3. 冻结的语言语义

### 3.1 `[]()` 的唯一表面形式

```text
[人类可读描述](机器可解析载荷)
```

`[]` 中的 label 是人话接口；`()` 中的 payload 使用所在上下文已有的 Marqdo 语法。二者共同形成一个命名步骤，但只有 payload 决定值、类型、依赖和副作用。

在可执行表达式上下文中：

```text
value([L](E)) = value(E)
type([L](E))  = type(E)
effect([L](E)) = effect(E)
```

label 不参与变量绑定，不改变控制流，不赋予权限，也不证明 payload 正确。编译器额外生成 `KnowledgeRef`/`KnowledgeStep`，因此透明不等于丢弃元数据。

### 3.2 上下文决定“是否执行”，目标种类决定“如何校验”

| 上下文 | 编译行为 | 运行行为 |
|---|---|---|
| `**...**`、`*...*`、调用实参、表格表达式 | 解析、绑定、校验并生成 BoundExpr | 正常求值 |
| 循环载荷 | 解析成既有 LoopSpec，生成命名步骤 | 按原循环语义执行 |
| 普通正文 | 解析导航或知识引用，生成 NarrativeRef | `run` 不执行；`view` 仅可按策略预览 pure 目标 |
| 标题、frontmatter、代码围栏、行内代码 | 不识别为可执行链接 | 保持原文本/原结构 |
| HTTP(S)、锚点、相对 `.md` 路径 | NavigationRef | 不执行 |

禁止通过“目标是函数”推导它在正文中可以执行。执行许可由上下文和 effect policy 共同决定。

### 3.3 一个语法，四种语义投影

| 示例 | `StepKind` | 核心依赖 |
|---|---|---|
| `[订单行金额](at(行, "金额"))` | `Observe` | 变量、字段 |
| `[折后价](小计 * 折扣)` | `Derive` | 输入表达式 |
| `[计算应付](定价.计算应付金额(...))` | `Invoke` | 目标 SymbolId、实参 |
| `[逐行处理](遍历 订单.行 为 行)` | `Iterate` | 集合、迭代变量 |
| 正文 `[免邮门槛](书店政策.免运费门槛)` | `Narrate` | 知识目标 |

`StepKind` 由已解析的 payload 和位置推导，不由 label 中的自然语言猜测。

### 3.4 类函数与实例方法

现有 `#`/`##` 表面语法保持不变，语义分析阶段明确区分：

- `##` 函数体直接使用 `self/自`，或调用同一类型中依赖接收者的方法：`InstanceMethod`，必须有实例接收者；
- 不使用接收者：`AssociatedFunction`，可由 `类.函数(...)` 调用，且不执行 `# 类` 的构造体；
- `#` 仍可承载类型/命名空间的兼容写法，但这不表示所有知识都是类；
- `Entity`、`Fact`、`Policy`、`Constraint`、`Dataset` 等是 Semantic Artifact kind，不强制增加源码关键字；
- 顶层自由函数和导入模块函数继续存在，不必包装成类。

接收者依赖必须在语义分析中递归确定。直接读取 `self/自` 是确定证据；调用同类型实例方法会传播接收者依赖；仅在叙述里出现“自”不算依赖。

### 3.5 零参路径

零参函数省略 `()` 只允许在以下明确的“需要值”位置：

- `[]()` payload 的最终路径；
- 赋值、返回、参数、条件或表格单元格中的值表达式；
- 正文知识引用的目标路径。

当语法位置允许函数值时，裸路径不做隐式调用；若语言当前没有一等函数值，则在引入一等函数前将此行为冻结为“路径解析到零参函数即调用”。带参数函数省略 `()` 必须报 `link.arg_mismatch`，不能返回函数对象或 `None`。

### 3.6 导航链接与知识引用消歧

识别顺序如下：

1. 明确 URI scheme（如 `https:`、`mailto:`）或 `#anchor` → Navigation；
2. 以 `./`、`../`、`/` 开头且指向文档/资源的路径 → Navigation；
3. 其余 payload 尝试使用 Marqdo 语法解析；
4. 解析成功 → Knowledge/Expression Reference；
5. 解析失败且符合普通 Markdown 目标 → Navigation；
6. 两种解释都成立 → 报 `link.ambiguous_syntax`，要求作者改为明确相对路径或合法表达式。

不能因为解析失败就无声地把预期知识引用降级成普通 URL。

---

## 4. 编译管线与模块边界

### 4.1 目标管线

```text
Source files
   ↓ SourceMap + Markdown-aware scan
SyntaxProject
   ├─ executable AST
   ├─ narrative nodes / links
   └─ contracts / metadata / source spans
   ↓ Declaration collection
ProjectIndex
   ↓ Name resolution
ResolvedProject
   ↓ type + contract + effect + provenance analysis
SemanticProject
   ├─ SemanticArtifact[]
   ├─ KnowledgeRef[] / KnowledgeStep[]
   ├─ DependencyGraph
   ├─ Diagnostic[]
   └─ HashManifest
   ↓ lowering
BoundProgram ──→ Tree Interpreter
             └─→ Bytecode compiler / VM
SemanticProject ──→ check / view / EKC / MLSP / lock / Agent
```

关键约束：运行器和工具链只能消费语义阶段已经确定的目标，不再用字符串搜索复制一套解析规则。

### 4.2 建议的源码目录

```text
src/
├─ syntax/                  # 后续从 parse/ast 渐进迁移；第一阶段可保持旧位置
├─ project/
│  ├─ mod.rs                # ProjectRoot、编译入口、缓存
│  ├─ discover.rs           # 文件发现与边界
│  ├─ module_id.rs          # ModuleId 规范化
│  └─ source_map.rs         # FileId、Span、源码摘要
├─ sema/
│  ├─ mod.rs
│  ├─ ir.rs                 # SemanticProject、Artifact、Ref、Step
│  ├─ symbols.rs            # 声明收集、SymbolId、索引
│  ├─ resolve.rs            # 名称解析与歧义诊断
│  ├─ classify.rs           # Type/Entity/Fact/Function/Capability...
│  ├─ types.rs              # CanonicalSignature、结构类型、单位
│  ├─ effects.rs            # EffectSet 推导
│  ├─ provenance.rs         # SourceRef 与证据解析
│  ├─ hashes.rs             # 四类 digest
│  └─ lock.rs               # links.lock 读写校验
├─ lower/
│  ├─ tree.rs               # Semantic IR → tree 可执行形式
│  └─ bytecode.rs           # Semantic IR → bytecode 输入
├─ knowledge/               # SemanticProject 的 EKC 投影/检索/复用
└─ artifact/
   └─ web.rs                # 现有 Web Artifact 分类，避免与 SemanticArtifact 混名
```

第一阶段不要求大规模搬文件。可以先新增 `project`/`sema`，让旧 `ast`、`parse` 成为输入；只有测试稳定后再做目录迁移。

### 4.3 公开编译入口

```rust
pub struct CompileOptions {
    pub project_root: PathBuf,
    pub entry: Option<PathBuf>,
    pub profile: BuildProfile,
    pub lock_mode: LockMode,
    pub effect_policy: EffectPolicy,
}

pub struct Compilation {
    pub sources: SourceMap,
    pub syntax: SyntaxProject,
    pub semantics: SemanticProject,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile_project(options: &CompileOptions) -> Result<Compilation>;
```

`run`、`check`、`view`、`knowledge compile` 和 MLSP 必须最终汇入此入口。针对内存源码的编辑器调用提供 `compile_overlay(project, overlays)`，不能另写简化语义。

---

## 5. 统一语义中间表示

### 5.1 标识类型

禁止在核心 API 中混用裸 `String` 表示文件、符号、制品和摘要：

```rust
pub struct FileId(u32);
pub struct ModuleId(String);
pub struct SymbolId(String);
pub struct ArtifactId(String);
pub struct RefId(String);
pub struct Digest(String);
```

序列化采用显式字符串格式；内存中保持 newtype，避免把显示名误当稳定身份。

### 5.2 SemanticArtifact

```rust
pub struct SemanticArtifact {
    pub artifact_id: ArtifactId,
    pub symbol_id: SymbolId,
    pub display_name: String,
    pub kind: SemanticArtifactKind,
    pub visibility: Visibility,
    pub source: SourceLocation,
    pub signature: Option<CanonicalSignature>,
    pub implementation: Option<BoundBody>,
    pub contract: ContractSet,
    pub effects: EffectSet,
    pub dependencies: Vec<ArtifactDependency>,
    pub provenance: Vec<SourceRef>,
    pub applicability: Applicability,
    pub hashes: ArtifactHashes,
    pub verification: VerificationRecord,
    pub lifecycle: Lifecycle,
}

pub enum SemanticArtifactKind {
    Module,
    Namespace,
    Type,
    Entity,
    Fact,
    Function,
    Capability,
    Constraint,
    Policy,
    Dataset,
    Document,
    ExternalCapability,
}
```

`SemanticArtifactKind` 是语义分类，不要求每一类都有新的 Markdown 标记。分类信息来源依次为：显式 metadata/type、结构位置与可检查契约、保守推导。无法确定时选择更一般的 `Function`/`Document`，不得根据函数名猜成 `Fact`。

### 5.3 KnowledgeRef 与 KnowledgeStep

```rust
pub struct KnowledgeRef {
    pub id: RefId,
    pub label: String,
    pub context: RefContext,
    pub payload: SyntaxNodeId,
    pub resolution: Resolution,
    pub expected: ExpectedContract,
    pub policy: VerificationPolicy,
    pub source: SourceLocation,
}

pub enum Resolution {
    LocalExpression,
    Navigation(NavigationTarget),
    Symbol {
        symbol_id: SymbolId,
        artifact_id: ArtifactId,
        binding: BindingKind,
    },
    Ambiguous(Vec<SymbolId>),
    Unresolved,
}

pub struct KnowledgeStep {
    pub id: ArtifactId,
    pub reference: RefId,
    pub kind: StepKind,
    pub inputs: Vec<DataDependency>,
    pub outputs: Vec<DataOutput>,
    pub calls: Vec<SymbolId>,
    pub effects: EffectSet,
    pub source: SourceLocation,
}
```

引用是“绑定关系”，步骤是“本次命名计算/叙述”。调用同一函数的两个位置拥有同一目标 `SymbolId`，但拥有不同 `RefId` 和 Step Artifact，便于回答“哪个句子受影响”。

### 5.4 规范签名

```rust
pub struct CanonicalSignature {
    pub receiver: ReceiverKind,
    pub params: Vec<CanonicalParam>,
    pub returns: CanonicalType,
    pub effects: EffectSet,
}

pub struct CanonicalParam {
    pub name: String,
    pub ty: CanonicalType,
    pub required: bool,
    pub default_digest: Option<Digest>,
    pub unit: Option<String>,
    pub constraints: Vec<Predicate>,
}
```

参数顺序、名称、是否必填、类型、单位、默认值和 receiver 都进入接口摘要。说明文字不进入接口摘要，但进入证据/文档摘要。

### 5.5 EffectSet

第一版固定以下 effect：

```text
pure
fs.read       fs.write
network.read  network.write
process.exec
plugin.call
llm.call
time.read
random.read
user.input
output.write
unknown
```

规则：

- 字面量、算术、纯集合操作为 `pure`；
- 内置/扩展注册时声明 effect；未声明的外部能力为 `unknown`；
- 函数 effect 是本体操作与所有调用依赖 effect 的并集；
- 递归 SCC 求不动点；
- `pure` 不与其他 effect 同时保存，出现其他 effect 即移除 `pure`；
- Narrative context 默认只允许 `pure` 的解析和元数据展示，不自动求值；
- lock 锁定代码不能消除运行环境 effect。

### 5.6 来源与验证记录

```rust
pub struct SourceRef {
    pub uri: String,
    pub locator: Option<String>,
    pub content_digest: Option<Digest>,
    pub retrieved_at: Option<DateTime>,
    pub relation: SourceRelation,
}

pub struct VerificationRecord {
    pub parse: CheckState,
    pub resolve: CheckState,
    pub contract: CheckState,
    pub execution: CheckState,
    pub tests: CheckState,
    pub evidence: CheckState,
    pub domain: CheckState,
    pub checked_at: Option<DateTime>,
    pub methods: Vec<VerificationMethod>,
}
```

禁止单个 `verified: bool`。API 可以派生 `reuse_readiness`，但必须保留每层状态与方法。

---

## 6. 稳定寻址与项目符号索引

### 6.1 项目边界

项目根解析顺序：

1. CLI 明确 `--project-root`/现有 `--fs-root`；
2. 从入口文件向上寻找 `.marqdo/project.toml`；
3. 从入口文件向上寻找 `.git`，但只作为默认边界；
4. 否则使用入口文件所在目录。

索引不得越过项目根，不跟随逃逸根目录的符号链接，不访问网络。`target`、`.git`、`.marqdo/cache`、`node_modules` 等生成目录默认排除，规则集中定义，不能由 EKC 与编译器各维护一份。

### 6.2 ModuleId

优先级：

1. frontmatter 显式 `id`；
2. 相对项目根的规范路径，移除 `.mq.md`；
3. 内置和扩展使用其注册的 package/module ID。

显式 `id` 在项目内必须唯一。fallback 路径移动会改变 ModuleId，属于可观察变更；需要保持身份时应先补 `id` 或生成 alias migration。

### 6.3 SymbolId

建议序列化格式：

```text
mq://<project-id>/<module-id>#/<kind>/<declaration-path>
```

示例：

```text
mq://bookshop/knowledge.pricing#/capability/定价/计算应付金额
mq://bookshop/knowledge.policy#/fact/书店政策/金卡折扣
```

`declaration-path` 使用源码声明层级，不使用数组序号；各段做 UTF-8 percent encoding。相同父级下同 kind 同名声明为硬错误。显示名称可以相同，只要规范地址不同。

### 6.4 ArtifactId 与 RefId

- 声明制品：`ArtifactId = SymbolId + artifact schema version`；
- 命名步骤：基于所属 SymbolId、规范 AST 路径和显式 source anchor 形成；
- 引用：基于 source ModuleId、source span anchor、目标 SymbolId 形成。

行号不能单独作为长期身份，因为插入文字会导致全文件漂移。第一版使用“所属声明 + 规范节点路径 + payload digest”，行列仅用于显示和重定位。

### 6.5 名称解析顺序

可执行代码：

1. 局部变量与显式接收者；
2. 词法父级声明；
3. 当前模块声明；
4. 显式 `use`；
5. 显式 `import` 绑定；
6. 内置和已注册扩展。

正文知识引用可以额外使用项目导出索引做发现，但唯一命中后必须写入具体 SymbolId/lock。代码路径默认不得依赖“扫描全项目后恰好唯一”，否则新增同名文件会改变运行结果。若产品决定允许代码短地址，也必须在首次绑定后要求 lock，且无 lock 构建发出诊断。

### 6.6 歧义和重命名

- 同一优先级两个候选 → `link.ambiguous`；
- 不允许根据目录遍历顺序、最近修改时间或搜索分数选择；
- 正式重命名由 `symbol-aliases.json` 或迁移命令生成旧→新映射；
- alias 只能用于给出迁移建议，严格模式下不能永久静默转发；
- alias 链禁止循环，默认最多一跳。

---

## 7. Markdown 与解析实现

### 7.1 两层解析而非正则扫描

第一层用 CommonMark/GFM 解析块与 inline，保留 source offset；第二层仅在 Marqdo 允许的节点和链接 destination 中调用现有表达式解析器。必须正确处理：

- destination 中的平衡括号；
- 转义的 `\]`、`\(`、`\)`；
- 中文标识符和 Unicode；
- 多个同一行链接；
- 行内代码与 fenced code 中看似链接的文本；
- 图片 `![alt](src)`；
- 普通 URL、锚点与相对文档链接；
- `**`、`*` 与 Markdown emphasis 的冲突；
- 表格单元格中的链接表达式；
- 正文链接与可执行表达式链接的不同上下文。

不得用简单正则从全文提取 `[]()`。

### 7.2 Syntax AST 的最小变化

短期保留 `Expr::Occluded` 以降低回归风险，但重命名计划如下：

```rust
Expr::Named {
    label: String,
    payload: Option<Box<Expr>>,
    syntax_id: SyntaxNodeId,
}
```

正文增加：

```rust
pub enum NarrativeInline {
    Text(String),
    Code(String),
    Link(NarrativeLink),
    Emphasis(Vec<NarrativeInline>),
}
```

`Module` 不应保存完整语义结果，只保留解析结构和 `SyntaxNodeId`。Semantic tables 通过 ID 关联，避免 AST 被工具专用字段污染。

### 7.3 Classic 迁移

统一语义启用前必须先迁移旧 `[键](集合)`：

```text
[键](集合)       → at(集合, "键")
[`键名`](集合)   → at(集合, 键名)
- [项](集合)     → 旧循环的规范新写法
```

迁移工具必须：

1. 在 Classic 模式解析旧文件；
2. 根据 AST 节点类型改写，不做正则替换；
3. 输出文件、行、旧 AST、新 AST 和文本 diff；
4. 默认 dry-run；
5. `--write` 前后分别以 Tree/Bytecode 运行可运行样例并对照结果；
6. 无法证明等价时拒绝自动改写并生成待办；
7. 完成仓库迁移后删除进程全局 `MARKUP_DEFAULT`，保留一个版本周期的只读兼容诊断。

不能让未声明模式的同一源码在升级后成功运行但产生不同值。

---

## 8. 绑定、类型、契约与适用性

### 8.1 绑定先于执行

每个 Call/Path/KnowledgeRef 在 lowering 前必须变成以下之一：

```rust
pub enum BoundTarget {
    Local(SymbolId),
    AssociatedFunction(SymbolId),
    InstanceMethod { method: SymbolId, receiver: BoundExprId },
    Imported(SymbolId),
    Builtin(BuiltinId),
    Extension(ExternalSymbolId),
}
```

Tree 和 Bytecode 都执行 `BoundTarget`，不再调用各自的 `lookup_function`/`resolve_call` 猜测相同名字。运行时只允许动态语言特性明确要求的 lookup，例如 `call_fn(text)`；这类动态调用必须带 `dynamic.invoke` effect/图边并降低静态可验证程度。

### 8.2 渐进式类型

保持 Marqdo 动态运行值，但在语义层建立渐进式类型：

- 有契约：以契约为主，并与推断参数双向互查；
- 无契约：从字面量、表达式、返回点和调用目标推断；
- 无法确定：`Any/Unknown`，不伪造精确类型；
- `number:CNY` 与 `number:ratio` 的运行存储可以相同，静态契约不可随意互换；
- Map/List/记录结构使用规范 schema；
- 参数、返回、字段、单位与枚举约束分别报告，不合并成笼统“类型错误”。

### 8.3 适用性

`Applicability` 至少支持：

```text
input predicates
unit / currency
region
valid_from / valid_until
audience / tenant
environment
required capabilities
```

第一版仅实现可以确定性验证的表格契约和 metadata；未支持的自由文本条件保存为 `declared_unchecked`，Agent 使用前必须看到该状态。不能因解析不了而丢弃条件。

### 8.4 失败关闭边界

以下情况禁止直接执行/复用：

- 目标未解析或歧义；
- 调用实例方法但没有接收者；
- 参数、返回类型或硬约束不匹配；
- 当前上下文禁止目标 effect；
- strict lock 下 digest 不匹配；
- 策略要求 evidence/domain verified 而状态不满足；
- 目标 deprecated/superseded 且策略禁止旧版本。

`knowledge.unverified` 在普通浏览可为 warning，但在调用方明确要求 verified 时升级为 error。

---

## 9. 指纹、锁定与增量构建

### 9.1 四类摘要

```rust
pub struct ArtifactHashes {
    pub interface: Digest,
    pub implementation: Digest,
    pub dependency_closure: Digest,
    pub evidence: Digest,
}
```

- `interface`：kind、receiver、规范签名、公开契约、effect 声明；
- `implementation`：去除无语义格式差异后的 Bound IR；
- `dependency_closure`：implementation 加所有静态依赖的 interface/implementation/closure，按 SymbolId 排序；
- `evidence`：规范化来源定位、内容摘要、验证方法和验证结果。

哈希输入必须带 schema version 和域分隔符，例如 `marqdo:artifact-interface:v1\0...`，防止不同对象的相同 JSON 偶然共用语义。

### 9.2 依赖环

先对调用图求 SCC：

- SCC 内节点先基于各自 implementation 生成 member digest；
- SCC digest 由排序后的成员 SymbolId/member digest 和外部依赖 closure 生成；
- 成员 closure 使用 SCC digest；
- 这样递归不会无限计算，也不依赖遍历顺序。

### 9.3 `links.lock`

默认路径 `.marqdo/links.lock.json`，规范结构：

```json
{
  "schema": 1,
  "project": "bookshop",
  "compiler": "marqdo/2.1.0",
  "links": [
    {
      "ref_id": "...",
      "from": "document.invoice-answer",
      "target_symbol_id": "mq://bookshop/knowledge.policy#/fact/书店政策/金卡折扣",
      "interface_digest": "sha256:...",
      "implementation_digest": "sha256:...",
      "dependency_closure_digest": "sha256:...",
      "evidence_digest": "sha256:...",
      "policy": "compatible"
    }
  ]
}
```

LockMode：

- `off`：只解析，不读写 lock；
- `check`：有条目则校验，无条目提示；
- `strict`：公共/跨文档引用必须存在且完全符合策略；
- `update`：显式更新，生成变更摘要，绝不由普通 `run` 偷偷改写。

### 9.4 缓存

`.marqdo/cache/sema/` 只保存可再生内容。缓存 key 至少包含：源码 digest、编译器版本、IR schema、项目配置、目标平台、扩展 registry digest。缓存损坏应回退重编译，不影响正确性。

文件扫描、索引和序列化必须排序；相同输入在不同机器、线程数和目录枚举顺序下产生相同摘要与 JSON。

---

## 10. 验证模型与诊断

### 10.1 七层状态

| 层 | 问题 | 可自动化程度 |
|---|---|---|
| Parse | 源码结构是否有效 | 确定性 |
| Resolve | 名字是否唯一绑定 | 确定性 |
| Type | 类型、参数、单位是否匹配 | 大部分确定性 |
| Contract | 前置条件/结构约束是否满足 | 声明范围内确定性 |
| Execute/Test | 受控执行和测试是否通过 | 取决于 effect/环境 |
| Evidence | 来源是否存在、内容是否变化、声明是否对应 | 部分确定性 |
| Domain | 现实业务/专业语境中是否正确、仍适用 | 人工或受标注的模型辅助 |

CLI 输出必须逐层展示。`check OK` 只能表示本次启用的检查没有 error。

### 10.2 诊断码

| Code | 默认等级 | 触发条件 |
|---|---|---|
| `link.unresolved` | error | 无可绑定目标 |
| `link.ambiguous` | error | 同优先级多目标 |
| `link.ambiguous_syntax` | error | 导航和表达式解释冲突 |
| `link.kind_mismatch` | error | 目标 kind 不可用于当前位置 |
| `link.instance_only` | error | 用类路径调用实例方法 |
| `link.arg_mismatch` | error | 参数数量/名称不匹配 |
| `link.type_mismatch` | error | 静态类型或单位不匹配 |
| `link.no_value` | warning/error | 值上下文目标无返回 |
| `link.effect_forbidden` | error | 当前上下文/能力策略不允许 effect |
| `link.target_changed` | warning/error | lock 目标或摘要改变 |
| `link.dynamic_unverified` | warning | 动态调用无法静态绑定 |
| `knowledge.source_missing` | warning/error | 声明来源不存在 |
| `knowledge.source_changed` | warning | 来源摘要变化未复核 |
| `knowledge.constraint_unmet` | error | 硬适用条件不满足 |
| `knowledge.not_yet_valid` | error | 尚未生效 |
| `knowledge.expired` | warning/error | 超出有效期 |
| `knowledge.unverified` | info/warning/error | 未达到调用方所需验证策略 |
| `knowledge.deprecated` | warning/error | 复用过时制品 |
| `label.ref_dangling` | error | label 中结构化反引号引用无目标 |
| `label.possibly_stale` | hint | 实现变化但 label 未复核 |

每条诊断必须包含 `code`、`severity`、source span、相关声明 span、候选目标（若有）、建议和所依据的 policy。JSON 结构向后兼容现有 `Diagnostic::to_json()`，新增字段不得把旧字段改义。

### 10.3 标签语义边界

label 与 payload 的自然语言一致性无法由类型系统完全证明。确定性检查仅覆盖：

- label 内反引号引用存在；
- payload 或目标变更后的 stale 提示；
- 明显单位/返回类型冲突；
- 显式词表规则。

LLM 语义复核是可选 provider，结果必须标为 heuristic，不能阻止离线编译，也不能覆盖确定性诊断。

---

## 11. 执行后端改造

### 11.1 Tree Interpreter

改造目标：

- `eval_call` 接受 `BoundCall`，不再按字符串顺序尝试 local/path/builtin/plugin；
- AssociatedFunction 直接用目标函数及所属模块执行，不运行对象构造体；
- InstanceMethod 保留 `_type`/继承链行为，但 method 已在语义层确定；
- `Expr::Named` 只透传 payload 值，同时向 trace 记录 `RefId`、ArtifactId、target digest；
- Host/Plugin 调用从 registry 获取签名和 effect；
- 运行时契约继续作为动态防线，不能因静态检查存在而删除。

### 11.2 Bytecode

新增或调整操作码：

```text
CallBound(symbol_index, argc)
CallMethodBound(symbol_index, argc)
TraceRefEnter(ref_index)
TraceRefExit(ref_index)
```

`Program` 增加 symbol/ref tables，disassemble 显示 SymbolId 和来源。旧 `Call(fid, argc)` 可以作为内部 lowering 结果保留，但编译阶段必须来自 BoundTarget，而不是 `resolve_call(name)`。

### 11.3 双后端一致性

每个 gold case 同时运行 Tree 和 Bytecode，比较：

- 返回值；
- stdout；
- 结构化错误 code 和 source span；
- 实际调用的 SymbolId 序列；
- KnowledgeRef trace；
- effect 拒绝结果。

仅文本错误措辞可允许微小差异；机器字段不得不同。

### 11.4 叙述预览

`view` 若显示正文 pure 引用的值，应通过单独的 PreviewExecutor：

- 默认仅允许 `pure`；
- 有输入的函数不自动调用；
- 有未知 effect 不调用；
- 限制步数、递归深度、输出大小和总时间；
- 超限只显示“可解析但未预览”，不影响页面其余部分；
- 预览结果标识输入、digest 和时间，不写回源码。

---

## 12. EKC、知识图与 Agent 复用

### 12.1 EKC 角色调整

EKC 从“再次阅读源码并猜测工程知识”调整为：

```text
SemanticProject
   ├─ project_symbols()  → L0/L1/L2
   ├─ capabilities()     → L3
   ├─ authored_knowledge → L4
   ├─ graph_edges()      → graph
   └─ search_documents() → lexical/vector optional index
```

短期兼容 `KnowledgeGraph`、现有 CLI 和 `.marqdo/catalog/**` 输出；内部字段逐步由 `SemanticArtifact` 填充。旧文本提取只为非 `.mq.md` authored knowledge 保留，不能覆盖编译器确定的符号和调用边。

### 12.2 图节点与边

节点：Project、Module、SemanticArtifact、KnowledgeStep、Evidence、Test、Execution、AgentEpisode。

核心边：

```text
contains
declares
references
calls
reads
derives
iterates
produces
depends_on
constrained_by
supported_by
tested_by
verified_by
supersedes
conflicts_with
executed_in
reused_in
```

`calls/references/depends_on` 必须来自 Bound IR；搜索命中不得直接生成事实图边。

### 12.3 Agent 状态机

```text
TASK
  → DISCOVER          模糊查找候选
  → RESOLVE           精确绑定 SymbolId
  → INSPECT           获取签名/effect/适用性/来源/状态
  → CONTRACT_CHECK
      ├─ ready        → BIND → EXECUTE → TRACE
      ├─ adaptable    → ADAPT → VERIFY → PROMOTE
      ├─ unusable     → 继续发现或 CREATE
      └─ ambiguous    → 请求消歧/失败关闭
  → CREATE            仅在 reuse budget 和 policy 允许时
  → VERIFY → PROMOTE
```

REUSE 的判据不是“搜索分数超过阈值”，而是候选已经精确绑定、输入满足契约、effect 获得许可、版本策略通过。搜索分数只用于候选排序。

`ready` 是对所选 policy、输入与上下文的判断，不是所有知识必须先获得的全局认证。AI 可以从候选中选择明确目标；歧义不能交给执行器按扫描顺序消解。ADAPT/CREATE 必须遵守任务写权限和预算，缺少业务决策时请求补充，而非擅自改变适用条件。草稿可被发现并在明确权限下探索执行，但不得冒充已通过严格政策的事实。

发现结果应按需包含标题/步骤、源码片段、输入角色、契约、效应、来源、验证缺口和相关失败经验。局部编辑须携带期望源码摘要，应用后检查语义 diff 与相关测试；不以重写整份文档作为默认修复方式。协议及受限试点范围见[指导 §5](../design/document-native-development-guide.md#5-ai-原生工作协议读步骤改步骤验证步骤)。

### 12.4 BoundArtifactHandle

```rust
pub struct BoundArtifactHandle {
    pub symbol_id: SymbolId,
    pub artifact_id: ArtifactId,
    pub interface_digest: Digest,
    pub dependency_closure_digest: Digest,
    pub accepted_effects: EffectSet,
    pub verification_policy: VerificationPolicy,
    pub expires_at: Option<DateTime>,
}
```

Agent 执行必须携带 handle 和显式输入；runtime 再次核对摘要和权限，防止发现后到执行前目标变化。handle 是能力引用，不包含可由模型修改后悄悄执行的源码副本。

上述结构表达绑定内容，不独自构成远程授权凭据。受信本地调用可先采用精确引用和摘要复核；不受信/远程调用必须另有服务端认可的主体、能力范围、防伪及撤销/期限机制，客户端自报 `accepted_effects` 不能扩大宿主许可。handle 由工具产生，不要求作者维护；是否检查 lock 和来源新鲜度取决于明确的 policy。

### 12.5 Promote 门槛

Agent 新建或适配的知识进入**共享自动复用的 ready 集合**前，按目标 policy 至少完成：

- Parse/Resolve/Type/Contract 通过；
- effect 已由静态绑定与宿主/扩展元数据推导；不能推导时保留 unknown 并按政策拒绝或要求明确能力许可，不要求业务作者抄写闭包；
- pure 逻辑有测试，外部 effect 有 mock/沙箱证明或人工批准；
- 来源型知识附 SourceRef；
- 生命周期从 `draft` 开始；
- 生成者、输入、工具版本、变更 diff 可审计；
- 未达到门槛的内容保持 draft/quarantine 状态；可在同一权威索引中过滤展示，不另造语义身份。

这是晋升政策，不是普通 `.mq.md` 文件被索引、查看或本地执行的前置注册流程。简单纯计算不强制现实来源；声称业务事实成立的知识不能用测试通过替代所需证据。

---

## 13. CLI、MLSP 与文件产物

### 13.1 CLI 演进

保留现有命令，增加能力而不是平行造新命令族：

```bash
marqdo check PATH --project-root ROOT --lock check --json
marqdo run FILE --lock strict --trace-knowledge trace.json
marqdo knowledge compile PATH
marqdo knowledge inspect SYMBOL
marqdo knowledge resolve EXPR --from FILE
marqdo knowledge verify SYMBOL --policy POLICY
marqdo knowledge impact SYMBOL_OR_PATH
marqdo knowledge lock [--check|--update]
marqdo migrate labels PATH [--write]
```

退出码：

- `0`：成功且无 error；
- `1`：程序/验证失败；
- `2`：CLI 使用错误；
- `3`：strict lock 不一致；
- `4`：effect/权限拒绝；
- `5`：项目索引或缓存损坏且无法重建。

### 13.2 MLSP 方法

新增：

| Method | 输入 | 输出 |
|---|---|---|
| `artifact.find` | query、kind、limit、project | 候选摘要；仅 discovery |
| `artifact.inspect` | SymbolId/ArtifactId | 签名、契约、effect、来源、hash、状态 |
| `link.resolve` | source file、payload/span | Resolution、候选、诊断 |
| `artifact.verify` | SymbolId、policy、可选输入 | 分层 VerificationRecord |
| `artifact.impact` | SymbolId/digest/path | 反向依赖与受影响步骤 |
| `artifact.bind` | SymbolId、expected digest、policy | BoundArtifactHandle；不直接执行 |
| `artifact.execute` | handle、inputs、capabilities | result、trace、更新后的状态 |

所有方法复用 `compile_project`，支持 overlay，不得自己调用 EKC 字符串搜索来完成 resolve。

### 13.3 生成目录

```text
.marqdo/
├─ project.json
├─ symbols.json
├─ artifacts/
│  └─ <artifact-id>.json
├─ refs.json
├─ graph/
│  ├─ nodes.json
│  └─ edges.json
├─ links.lock.json
├─ verification/
│  └─ <artifact-id>.json
├─ traces/
│  └─ <execution-id>.json
├─ catalog/                 # 兼容现有 EKC
├─ knowledge/               # 兼容现有 EKC
└─ cache/                   # 可删除、可重建
```

所有生成文件写入前先在临时文件完整生成和校验，再原子替换；编译失败不得留下“半新半旧”的项目图。

### 13.4 JSON schema 与版本

每类生成物必须有 `schema_version`，schema 文件纳入仓库。新增可选字段为兼容变化；删除、改义或类型变化必须提升 major schema。测试用 JSON 不比较键顺序，但生产输出按固定顺序生成以便 review。

---

## 14. View 与可解释性

### 14.1 每个引用的最小展示

View 至少展示：

- label 和 payload；
- 目标显示名、SymbolId、kind；
- 签名、单位、effect；
- 解析/契约/测试/证据状态；
- 当前值或“未执行”的原因；
- 来源与目标源码跳转；
- interface/implementation 是否与 lock 一致；
- 上游依赖和下游影响入口。

### 14.2 状态措辞

允许：

- “已解析到……”
- “契约检查通过”
- “测试于某版本通过”
- “来源存在但尚未完成领域复核”

禁止：

- 仅因函数能运行显示“知识正确”；
- 仅因 URL 可访问显示“来源已验证”；
- 仅因搜索命中显示“已复用”；
- 隐藏 effect 或 lock 变化。

### 14.3 可达性和静态导出

状态不能只靠颜色表达；HTML 使用文本、图标和可访问标签。`view output --no-exec` 仍展示解析与验证元数据，只隐藏运行值。静态导出不得为了显示知识值而放宽运行权限。

---

## 15. 安全、权限与可信边界

### 15.1 能力安全

沿用现有 HostCaps，并扩展到细粒度 EffectPolicy。编译期 effect 是预检查，运行时 capability 是最终授权。两者缺一不可。

正文链接、view 预览、`check` 和 `knowledge compile` 必须是只读路径，不得因为分析引用而触发：

- 文件写入；
- 网络写操作；
- 外部进程；
- 插件动态加载；
- LLM 请求；
- 用户输入等待。

### 15.2 路径与来源

- 项目文件和相对 SourceRef 规范化后必须仍在允许根内；
- symlink 逃逸按真实路径检查；
- 来源摘要读取大小有上限；
- 远程来源默认不在编译时抓取，只使用已有快照/显式 verify；
- URL 凭据、token 和查询机密不得进入 graph、lock 或 trace；
- trace 对参数和值执行现有 secrets/redaction 策略。

### 15.3 TOCTOU

解析、绑定与执行之间，用 source/hash 校验防止目标变化。严格执行时发现 handle digest 与磁盘当前制品不一致，应拒绝并要求重新绑定。

---

## 16. 迁移与兼容策略

### 16.1 版本阶段

| 阶段 | 默认行为 | 兼容行为 |
|---|---|---|
| 2.0.x | Classic 默认，Occlusion opt-in | 历史行为 |
| 2.1 preview | 新语义默认；全仓语料具有文件级 `markup: occlusion` | `--legacy-markup` / 显式 `markup: classic` 只读过渡 |
| 2.1 stable | 新语义默认；旧形状必须已迁移 | `--legacy-markup` 只读运行一个周期 |
| 2.2 | 删除全局模式与旧 parser 分支 | 仅保留明确错误和迁移建议 |

具体版本可随发布计划调整，但顺序不能颠倒。

### 16.2 不破坏的能力

- 现有显式 import；
- 实例构造与实例方法；
- 当前契约表；
- Tree/Bytecode CLI 入口；
- EKC 的 `find/reuse/impact` 用户命令；
- Web/Endpoint Artifact metadata；
- 无 `[]()` 文件的既有执行结果。

### 16.3 必须显式变化的能力

- `[键](集合)` 迁为 `access(集合, "键")`；
- 链接不再依赖进程级 markup 模式；
- `类.函数` 对无 receiver 函数成为合法 AssociatedFunction 调用；
- 正文知识链接被解析和诊断，但不自动执行；
- EKC 的 calls/edges 由 Bound IR 取代名字猜测；
- lock 更新从普通编译中分离。

### 16.4 迁移报告

```json
{
  "file": "...",
  "before_mode": "classic",
  "changes": [
    {
      "span": {"line": 10, "col": 4},
      "kind": "index_to_at",
      "before": "[银卡](折扣表)",
      "after": "at(折扣表, \"银卡\")",
      "semantic_check": "equivalent"
    }
  ],
  "manual_review": [],
  "tree_equivalent": true,
  "bytecode_equivalent": true
}
```

---

## 17. 分阶段开发计划

M0–M6 保留为工程能力与验收项目，不再作为顺序排期；部分迁移能力已经完成，状态见[现状审计](../roadmap/executable-knowledge-gaps-2026-10-23.md)。实际工作按[文档原生 S0–S6](../design/document-native-development-guide.md#7-交付切片每片都有文档作者与-ai-能看见的结果)形成可独立合并的垂直切片：早期联动 M1/M2/M3/M5 的安全子集，交付可读步骤和真实复用，不能等 M4 全部做完才让作者或 Agent 使用。100 单元生成器逐步增长，不阻塞最小故事；不得先大规模重构再等待最后一次性验收。

### M0：语义冻结与测试生成器

交付：

- 本文进入仓库并被文档索引引用；
- `tests/executable_knowledge/` gold case 结构；
- 100 单元项目生成器，固定随机种子；
- 解析上下文、导航链接、旧索引、正文安全的测试矩阵；
- 当前行为快照和预期变化清单。

完成条件：

- 测试可以明确区分“当前失败（expected fail）”与回归；
- 所有新诊断 code、JSON 最小字段和退出码冻结；
- 不依赖 LLM 或网络即可运行核心套件。

### M1：ProjectIndex 与稳定身份

交付：

- `project/`、`sema::symbols`；
- FileId/ModuleId/SymbolId；
- 项目范围、确定性扫描、duplicate/ambiguous 诊断；
- 显式 import 与正文项目发现索引；
- `marqdo knowledge inspect/resolve` 最小接口。

完成条件：

- 改变文件扫描顺序不改变绑定或输出；
- 同名李白示例报歧义，限定地址能解析；
- 同一源码连续构建的 SymbolId 完全一致；
- fallback 路径移动被 lock/impact 识别。

### M2：统一 Named/Ref IR 与类函数

交付：

- Markdown-aware narrative links；
- `Expr::Named`/兼容 lowering；
- KnowledgeRef/KnowledgeStep；
- AssociatedFunction/InstanceMethod 分类；
- Tree 和 Bytecode 使用 BoundTarget；
- 正文不执行测试。

完成条件：

- 两文档李白示例无需构造实例即可解析；
- 代码上下文可求得 `701`，正文上下文只解析；
- 使用 `self/自` 的方法经类路径调用报 `link.instance_only`；
- 两后端返回值、调用 SymbolId 和错误 code 一致。

### M3：SemanticArtifact、契约、effect、来源

交付：

- 完整 Artifact IR 和分类；
- CanonicalSignature、EffectSet、Applicability；
- SourceRef 和分层 VerificationRecord；
- 四类 digest；
- 扩充 `check --json`。

完成条件：

- 参数、类型、单位、适用条件和 effect 分别诊断；
- pure/外部 effect 传递闭包正确；
- 来源删除、内容变化与实现变化产生不同状态；
- 格式化空白不改变 implementation digest，语义变更会改变。

### M4：Lock、增量与影响分析

交付：

- links.lock check/update/strict；
- SCC closure digest；
- 原子写入、缓存和确定性序列化；
- 反向引用/依赖/证据影响查询。

完成条件：

- 金卡折扣变化能找到获取折扣、计算应付、结算入口和正文说明；
- interface/implementation/evidence 变化能分别展示；
- strict 模式拒绝未审阅的目标变化；
- 清空缓存后结果与增量构建完全一致。

### M5：EKC、View、MLSP 与 Agent

交付：

- EKC 消费 SemanticProject；
- view 的引用卡片、来源链和状态；
- MLSP 七个 artifact 方法；
- BoundArtifactHandle 和 Agent 复用 trace；
- draft/quarantine/promote 流程。

完成条件：

- 图中 calls 边与实际运行 trace 的静态目标一致；
- Agent 命中 ready 制品时执行 handle，不生成新函数；
- 只有模糊候选时不能直接执行；
- 不满足所选验证政策、已过相关有效期或 effect 不允许的制品不会被标记为该 policy 下的 REUSE ready；本地探索不冒充严格验证。

### M6：迁移默认值与发布

交付：

- 全仓 Classic 语法迁移；
- 默认切换、兼容 flag、弃用说明；
- public/examples/tutorial 更新；
- schema 文档、升级指南和 release notes；
- 100 单元基准与 Agent 对照实验。

完成条件：

- 全部现有 gold tests 加新套件通过；
- 仓库不再依赖全局 Markup 状态；
- 迁移前后无未说明的输出变化；
- 性能、正确性和复用指标达到发布阈值。

---

## 18. 文件级实施清单

下表是目标职责映射，不是“所有文件尚未创建”的任务表。`src/project/`、`src/sema/` 和部分 API 已存在，先检查对应缺口与测试，再增量修改；不要依照旧“新建”措辞重复搭建模块或移走仍被消费者使用的能力。

| 文件/模块 | 修改 |
|---|---|
| `src/ast.rs` | 增加 SyntaxNodeId；`Occluded` 迁为 `Named`；承载 Narrative AST 的最小引用 |
| `src/parse/prose.rs` | 用 Markdown event/span 构建正文 inline/link，不执行 |
| `src/parse/expr.rs` | 统一 `[]()`；隔离并最终删除 Classic 分支和全局状态 |
| `src/load.rs` | 保留单模块加载；项目构建改由 `project` 编排；frontmatter mode 进入迁移诊断 |
| `src/project/*` | 新建项目发现、SourceMap、ModuleId、缓存 |
| `src/sema/*` | 新建符号、解析、类型、effect、来源、hash、lock、统一 IR |
| `src/check.rs` | 成为 SemanticProject 诊断聚合器；保留当前契约规则 |
| `src/contract.rs` | 输出规范 CanonicalType/Predicate；运行时校验复用同一声明 |
| `src/interp.rs` | 从 BoundProgram 执行；AssociatedFunction；Ref trace |
| `src/bytecode/compile.rs` | 删除独立名字绑定职责；从 BoundCall 生成 opcode |
| `src/bytecode/vm.rs` | 执行 bound symbol/ref table；统一 trace 与诊断 |
| `src/artifact/mod.rs` | 迁移现有 Web 分类，释放概念命名 |
| `src/knowledge/extract.rs` | `.mq.md` 改从 SemanticProject 投影；普通 `.md` 仍可提取 authored knowledge |
| `src/knowledge/fingerprint.rs` | 相似性 fingerprint 保留作检索；规范 digest 改用 `sema::hashes` |
| `src/knowledge/graph.rs` | 从已绑定依赖生成边；移除“第一个同名符号”策略 |
| `src/knowledge/reuse.rs` | 搜索分数只做 discover；新增 readiness/bind gate |
| `src/mlsp.rs` | 加 artifact API；dispatch 共享 project compiler 与 overlay |
| `src/view/*` | 消费 LinkInfo/VerificationRecord；PreviewExecutor 隔离执行 |
| `src/main.rs` | CLI 参数、lock/trace/inspect/resolve 子命令和退出码 |
| `tests/executable_knowledge/*` | 正负例、双后端、迁移、lock、effect、图、Agent 测试 |
| `examples/executable-knowledge/*` | 以书店示例替换占位目标，标注每阶段可运行状态 |

实现时每个 PR 只覆盖一个可验收切片；不要在同一提交混合语法迁移、IR 重构和 UI 大改。

---

## 19. 自动化测试规范

### 19.1 单元测试

- SymbolId/ModuleId 规范化、Unicode、路径移动；
- Markdown destination 平衡括号与转义；
- 导航/知识引用消歧；
- AssociatedFunction/InstanceMethod 分类和传播；
- CanonicalSignature/类型/单位；
- EffectSet 合并与 SCC；
- 四类 digest 的稳定性；
- lock policy；
- VerificationRecord 状态提升/降级；
- JSON schema round-trip。

### 19.2 集成正例

- 李白两文档事实；
- 书店政策→定价→结算→客服说明；
- 显式 import 的类函数；
- 本地表达式遮蔽；
- 正文 pure 引用 view 预览；
- 实例继承方法保持行为；
- 表格/循环/取元形成 KnowledgeStep；
- 中文/英文双名与扩展能力；
- Tree/Bytecode 完全一致。

### 19.3 集成负例

至少覆盖：

- 目标删除/改名；
- 同级同名；
- 文档发现唯一但代码缺显式 import；
- 漏参、多参、错误具名参数；
- number/text、CNY/ratio 错配；
- 类路径调用实例方法；
- 零参省略作用于有参函数；
- 正文调用网络、写文件、支付或 LLM；
- 来源删除、来源内容变化、有效期过期；
- 实现未变但传递依赖变化；
- lock 的 target/interface/implementation/evidence 分别变化；
- 递归依赖和循环 import；
- symlink 越界；
- label 与代码明显冲突；
- 动态调用无法验证；
- 缓存损坏；
- 目录扫描顺序扰动。

### 19.4 100 单元确定性夹具

生成器固定包含：

| 类别 | 数量建议 |
|---|---:|
| 正常跨文档调用链 | 25 |
| 同名/限定名/导入消歧 | 15 |
| 签名/类型/单位错误 | 15 |
| 来源/有效期/验证状态 | 10 |
| effect 与正文安全 | 10 |
| 依赖变化/lock/impact | 15 |
| 递归、循环和动态调用 | 10 |

每次生成额外随机打乱文件创建顺序。断言绑定结果、诊断集合、digest 和图边与顺序无关。

### 19.5 属性与模糊测试

- `[L](E)` 与 `E` 在执行上下文的值/type/effect 等价；
- 格式化和无语义叙述变化不改变 implementation digest；
- 任一 Bound call 对应且仅对应一个静态 target；
- graph 的静态 calls 边覆盖所有非动态运行调用；
- lock update 后 strict check 通过；
- parser 对任意输入不 panic；
- view/check 对正文链接永不触发外部 effect。

---

## 20. 基准、实验与发布门槛

### 20.1 编译器指标

- 冷构建耗时、峰值内存；
- 单文件变更增量耗时；
- 1k/10k/100k artifact 索引规模；
- 符号解析准确率；
- 影响分析 precision/recall；
- 两后端一致率；
- 缓存命中率；
- 生成物确定性。

### 20.2 Agent 复用指标

在相同模型、任务、工具、token 预算和超时下比较普通 Markdown+RAG、当前 EKC、Semantic Artifact 方案：

- 已有能力发现率；
- 正确绑定率；
- 不适用知识误用率；
- 重复函数新增率；
- REUSE/ADAPT/CREATE 分布；
- 首次可运行成功率；
- 总 token、工具调用、修复轮次和耗时；
- 回答能否追溯到 SymbolId、digest、输入和来源。

同时记录作者手写声明/重复字段/配置数量、阅读定位准确率、语义步骤修改范围、无关 diff、并发补丁冲突和首次写作成本。比较时各方案应获得同等现有能力与输入信息，不向新方案额外提供答案；将编译和人工准备成本计入总成本。先通过正确性与权限硬门槛，再讨论 token 和速度收益，参见[指导 §8](../design/document-native-development-guide.md#8-验证-marqdo-优势而不只验证组件存在)。

### 20.3 发布门槛

- 核心确定性套件 100% 通过；
- 双后端目标用例 100% 一致；
- 正文安全测试 0 次外部 effect；
- 符号歧义无静默选择；
- 迁移工具对已自动改写样例 100% 语义等价；
- graph calls 与 bound calls 一致；
- Agent 不适用知识误用率不得高于现有基线；
- 性能退化超过预设阈值必须有明确批准与记录。

不要只报告平均值；保存失败样本、任务定义、环境、置信区间和版本摘要。

以上按发布宣称的能力范围执行。语法兼容、本地确定性复用、严格业务复用、远程执行分别有独立证据，不以一个总体 `done` 替代。M0–M6 保留为工程能力检查项，实际交付顺序采用[修订路线 S0–S6](../roadmap/executable-knowledge-gaps-2026-10-23.md#7-依赖顺序与可合并切片)：早期完成一篇文档的读/执行/复用闭环，再增加治理深度；任何已开放执行路径都不能延后安全门禁。

---

## 21. 书店示例的最终验收链

以附属示例为系统级验收：

```text
corpus/pricing-policy.md
  └─ supported_by
knowledge/policy.mq.md
  ├─ 书店政策.金卡折扣
  ├─ 书店政策.银卡折扣
  ├─ 书店政策.免运费门槛
  └─ 书店政策.基础运费
       ↓ calls / depends_on
knowledge/pricing.mq.md
  └─ 定价.计算应付金额
       ├─ 计算商品小计
       ├─ 获取折扣
       └─ 计算运费
            ↓ reused_in
apps/checkout.mq.md + apps/assistant.mq.md
```

必须验证以下全过程：

1. 项目编译产生稳定 SymbolId；
2. 所有 `[]()` 产生 Ref/Step；
3. `60 × 2 × 0.90 = 108` 由 Tree/Bytecode 同样计算；
4. assistant 正文不在 `run` 中执行；
5. view 可显示门槛、折扣、来源和验证层级；
6. 修改金卡折扣到 `0.88` 后 implementation 和 closure 改变；
7. 未更新来源时显示 evidence 不一致；
8. impact 找到所有下游引用；
9. strict lock 拒绝静默使用新版；
10. 审阅并 update lock 后 Agent 可重新 bind/execute；
11. 输入数量为 0 时在执行前报 `knowledge.constraint_unmet`；
12. 在正文引用支付能力时拒绝 effect，而不是发起支付。

---

## 22. 开发过程约束

1. **一套语义事实源**：parser 只认语法，sema 决定身份和含义，runtime 执行 bound target，EKC/view/Agent 消费结果。
2. **先确定性，后智能化**：解析、绑定、契约、effect、hash、lock 不使用 LLM。
3. **先负例，后演示**：每新增一种成功路径，至少增加一个歧义/权限/变化负例。
4. **不静默兼容**：旧写法可能改义时必须迁移或报错。
5. **不伪造可信度**：未知就是 unknown，待复核就是 pending。
6. **不复制源码复用**：复用结果是 SymbolId/handle/trace，不是代码片段。
7. **生成物可删除**：`.marqdo` 投影永远可以从源码和显式证据重建。
8. **产物可审计**：每个值知道由哪个符号、版本、输入和 effect 得到。
9. **变化可传播**：实现、依赖、证据和适用性分开变更、分开报告。
10. **文档与代码共同验收**：示例未能由当前版本运行前，必须继续标为目标而不是已实现。

---

## 23. 最终定义

Marqdo 的本轮开发不以“链接语法更漂亮”为终点，而以以下能力闭环为终点：

> 一份由人维护的可读文档，被编译为具有稳定身份、明确契约、可追踪来源、受控副作用和版本摘要的语义制品；人或 AI 可以先发现它，再精确绑定它，在满足适用条件与权限后直接执行它，并把实际复用关系写回可审计的知识图。

`[]()` 是这条链路统一、低负担的作者入口；Semantic Artifact 是编译器的统一事实；Verified Knowledge Reference 是从文档到执行的确定性桥梁；EKC、view、MLSP 和 Agent 是同一语义结果的不同投影与消费者。

只有当“发现已有知识 → 确定绑定 → 验证适用 → 执行既有制品 → 追踪复用结果”能够在不重新生成同义实现的情况下完成，Marqdo 才真正达成：

**代码即文档，文档即知识库，知识库可执行，可执行知识可复用。**
