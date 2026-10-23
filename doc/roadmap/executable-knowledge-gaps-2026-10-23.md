# Marqdo 现状与规划缺口：从可执行文档到可复用知识

| | |
|---|---|
| 状态 | 现状审计与下一阶段规划；不是已实现能力声明 |
| 审计基线 | `main` / `434be9f`，2026-10-23；后续改动须重新核验对应判断 |
| 产品意图 | [009.md](../next/009.md) · [详细开发规范](../next/Marqdo_Executable_Knowledge_Development.md) · [完整示例](../next/Marqdo_Executable_Knowledge_Example.md) |
| 目标 | **代码即文档，文档即知识库，知识库可执行，可执行知识可复用** |
| 方向修订 | 2026-10-23：按[文档原生开发指导](../design/document-native-development-guide.md)重排交付，G01–G12 保留为缺口证据，不等于作者必须经历的治理流程 |

## 1. 本文解决什么问题

仓库已经拥有语言运行时、双后端、标准库、扩展、文档站、EKC、Agent 和一批语义制品结构。现有[详细开发规范](../next/Marqdo_Executable_Knowledge_Development.md)定义了目标架构和 M0–M6，但它主要写于目标能力尚未落地之时；现在照它逐项勾选，容易把“结构存在”“CLI 可调用”“目标闭环成立”混为一谈。本文只补**现状与目标之间尚未被规划成可验证闭环的工作**。语法和产品目标仍以前述意图文档为准；旧文档中的 Classic 链接取元、单体知识类、或另一套知识索引不应反向规定新语义。

本轮判断以源码、可定位的测试和文档状态为证据。`done` 表示有可运行入口和回归测试；`partial` 表示有结构或单一路径，仍不满足产品判据；`design` 表示主要存在于文档和目标示例。**不能把能解析、能运行、能检索、能验证事实视作同一个结论。**

### 1.1 对上一版规划排序的修正

上一版把内部工程完整度放在主要位置，直到 S6/S7 才验收 Agent 复用和作者体验，容易造成“先建治理平台，再证明文档有用”的偏差。本版改为从可读步骤、确定执行和早期本地复用向外扩展；每片同时交付源码示例、诊断和消费者可见结果。工程基础按切片所需完成，不降低已开放路径的安全要求。

P0/P1/P2 分类改为缺口领域，不再按编号整组串行：G06 的未知效应安全下限必须进入最早执行试点，G09 的本地复用也不再等锁和远程授权全部完成。复杂业务的证据、严格锁和远程凭据按使用场景增加，不能强加给每个单文件程序。每片还须记录作者重复声明数、局部修改范围和实际复用结果，指标见[指导 §8](../design/document-native-development-guide.md#8-验证-marqdo-优势而不只验证组件存在)。

## 2. 设计思想落实为四个可证伪承诺

| 承诺 | 必须由什么证明 | 当前判定 |
|---|---|---|
| 代码即文档 | 一份 `.mq.md` 的叙述、契约、示例、可执行语句由同一解析结果驱动；改变语义会让说明/示例失配可见 | 运行与 view 已有基础；正文引用尚未进入完整语义 AST（partial） |
| 文档即知识库 | 可寻址的事实、规则、步骤、来源、适用范围和关系由编译器生成；不同消费者读取同一身份和图 | 项目索引与 Artifact 投影已起步；双图和占位字段仍存在（partial） |
| 知识库可执行 | 引用先绑定、检查参数/契约/效应/版本，再由运行时执行**绑定的目标**；正文默认不触发副作用 | CLI 可解析、运行时可执行，但两条路径未闭合（partial） |
| 可执行知识可复用 | Agent 发现候选后取得有条件、可失效的句柄，实际调用既有制品；trace 记录符号、摘要、输入和结果 | MLSP 有初版 handle；EKC/Agent 尚未统一到此协议（partial） |

这四条有语义依赖，但应在受限故事中逐步贯通，不是四个必须全部做完才能进入下一层的大项目。阅读体验和检索召回不能代替绑定；绑定不能证明知识内容正确；执行成功不能证明已经复用既有知识。

## 3. 仓库能力盘点与规范 M0–M6 的真实进度

| 既有里程碑 | 可复用成果 | 仍未满足的完成条件 |
|---|---|---|
| M0 语义与测试 | Occlusion 已为默认；[语法测试规程](../next/Marqdo_Executable_Knowledge_Testing.md)和部分金样存在；标准库、扩展库已经迁移 | `tests/executable_knowledge/` 系统夹具、100 单元生成器、导航/正文/效应负例矩阵未形成独立门禁 |
| M1 项目身份 | [`src/project/mod.rs`](../../src/project/mod.rs)确定项目根和扫描顺序；[`src/sema/symbols.rs`](../../src/sema/symbols.rs)产生 ModuleId/SymbolId；CLI `knowledge inspect/resolve` 可用 | 身份保留、移动/重命名、symlink/重复扫描、大项目增量与错误聚合还需完整策略和测试 |
| M2 统一链接与调用 | 裸零参路径和关联函数已有 Tree/Bytecode、CLI 金样；`Expr::Occluded` 已承载标签和 payload | [`src/ast.rs`](../../src/ast.rs)仍无统一 Named/正文引用节点；[`src/sema/ir.rs`](../../src/sema/ir.rs)只提取执行体部分调用；运行时仍独立按名称寻找调用目标 |
| M3 语义制品 | SemanticArtifact、EffectSet、SourceRef、VerificationRecord、JSON 投影已存在 | 适用性、来源证据、单位、真实验证状态、完整效应闭包与规范摘要仍缺；字段存在不代表语义已实现 |
| M4 锁与影响 | `knowledge lock/impact` 可用；[`src/sema/lock.rs`](../../src/sema/lock.rs)支持原子写入和反向依赖查询 | 锁只比较 artifact 摘要；目标、接口、实现、证据、传递闭包变更未独立报告；正文/证据边不在影响闭包内 |
| M5 消费者 | view 显示部分语义信息；MLSP 提供 `artifact.*`；EKC/Agent 有各自检索与复用能力 | 同一图/同一绑定尚未贯通；Agent 对 ready 的判定与执行句柄未被强制；trace 不足以证明实际复用 |
| M6 迁移与发布 | 全仓新语法迁移、`--legacy-markup` 入口、v2.1.0 发布已完成 | 发布不等于 M0–M5 已完成；目标示例仍注明“尚不能完整运行”，100 单元与 Agent 对照实验未过 |

`examples/executable-knowledge/README.md` 是目标示例，仍明确说明类函数、统一 `Named<T>` 和步骤图的完整运行是设计目标。它不得作为“产品闭环已完成”的发版证据。

### 3.1 相邻路线如何进入本规划

| 仓库区域 | 当前路线状态 | 与可执行知识主线的连接点 |
|---|---|---|
| Interpreter / 标准库 | Tree、Bytecode、`lib/` 与中英库有大量金样；[Mid2](stdlib-mid2.md)已完成 | 作为 BoundCall、效应声明、签名与迁移回归的基础；不为本路线重做通用库 |
| Web / 浏览器 | [Web](ext-web.md) W 波与 [WASM](browser-wasm.md) C 波已落地 | 作为 view/静态投影和宿主能力消费者；知识引用不能在静态导出或浏览器预览中偷跑服务端 effect |
| Agent / LLM / 沙盒 | [Agent](ext-agent-optimize.md) Wave B 有局部待项；[优化 O 波](ext-ai-optimize.md)已完成 | 工作簿、OKF、沙盒可复用；S2 接本地 bound handle，S4/S5 增强政策；不另建平行知识身份 |
| 外联 / Notebook | [外联 F4](foreign-v2.md)和[会话 D4–D5](notebook-session.md)尚有余项 | 外部内核、持久状态和重放需显式建模为 effect/环境依赖；不能把会话结果当纯制品缓存 |
| 领域扩展 | [Quantum Q8c](ext-quantum.md)等剩余项属领域体验；[宿主 H4b](ext-hosting.md)为可选 HTTP 能力 | 不阻塞语义主链；但扩展元数据与插件调用必须参加 G06/G10 的 effect 和兼容性门禁 |

这些路线各有自己的完成定义。本文不将其余项全部吸收到下一版核心范围；只规定它们与统一语义层交互时必须遵守的契约。

## 4. 语义一致性缺口：消除错误的语义真相

### G01 两份图与同名输出路径

**证据。** [`src/knowledge/compile.rs`](../../src/knowledge/compile.rs)先调用 `compile_project` 并写语义投影，随后重新用 `extract_graph` 从源码提取旧 EKC 图。后者的 `write_projections` 再写 `graph/edges.json`；[`src/knowledge/graph.rs`](../../src/knowledge/graph.rs)的调用边按短名选首个匹配符号。由此“语义边”和“检索边”既不是同一事实，也会争用同一个文件名。

**规划缺口。** 详细规范要求 EKC 是 `SemanticProject` 投影，却未给出**并行迁移期间的文件命名、读取优先级、冲突检测与废弃时点**。只新增语义图不解决消费者继续读旧边的问题。

**决策与交付。** 明确一个权威 `SemanticProject`；图边按稳定 `SymbolId/ArtifactId` 生成。旧检索索引仅含查询优化数据，不再自行确定 calls/uses。若迁移期保留旧 JSON，放入独立版本化路径并声明其非执行依据。`knowledge compile`、`view`、`check`、`impact`、MLSP、Agent 对同一引用应报告同一目标及诊断。

**验收。** 制造两个模块同名函数并让调用只 import 其中一个；语义图、CLI resolve、Tree/Bytecode trace、view 和 Agent handle 必须指向同一 ID。编译两次输出字节一致；生成物中没有同名文件被后续投影覆盖。

### G02 绑定没有成为执行的前置条件

**证据。** [`src/main.rs`](../../src/main.rs)的 `knowledge resolve` 走项目索引；[`src/mlsp.rs`](../../src/mlsp.rs)的 `link.resolve` 通过 artifact 名字/后缀匹配；[`src/interp.rs`](../../src/interp.rs)仍有自身的 `eval_path_call` 和函数查找；[`src/bytecode/`](../../src/bytecode/)也有独立调用实现。`compile_project` 目前只将同模块无接收者调用识别为静态依赖，其余记为 `external_or_dynamic`。这几条路径存在真实分叉。

**规划缺口。** 现有规范提出 Bound IR，但缺少**最小可合并的执行切面**以及动态调用的明确边界。不能先宣称图边覆盖执行调用，再等待后端改造。

**决策与交付。** 先让显式 import、普通函数、关联函数这三个静态路径通过唯一 resolver，生成 `BoundCall { target, signature, source_span, required_effects }`。Tree 和 Bytecode 只消费 BoundCall；确需动态解析的路径使用独立 `DynamicCall`，在图/诊断中显式标记，不能假装已绑定。`run/check/knowledge resolve/MLSP` 使用相同 import 范围和歧义规则。

**验收。** 同名跨模块、重命名、缺 import、实例方法误用、动态目标分别有稳定诊断；任一静态 trace 中执行的 SymbolId 与编译图边一致。错误不得因后端不同而变成另一目标。

### G03 `[]()` 只有标签，还没有完整知识步骤

**证据。** [`src/ast.rs`](../../src/ast.rs)保留 `Expr::Occluded`；[`src/parse/prose.rs`](../../src/parse/prose.rs)扫描粗体、斜体和反引号，但没有完整 Markdown 正文链接 AST。[`src/sema/ir.rs`](../../src/sema/ir.rs)当前 KnowledgeRef/KnowledgeStep 主要由调用点提取，步骤 `kind` 为 `call`；取元、推导、遍历、正文、导航尚未同等进入图。目标示例列出的 observe/derive/iterate/narrate 不能由现有投影证明。

**规划缺口。** 缺少正文 inline 的位置、逃逸、代码围栏、标准 URL 与知识引用的**一套无副作用解析协议**，以及 `[]()` 在表达式、循环和正文的统一节点身份。

**决策与交付。** 保留现有 payload 表达式解析器；为上下文建立稳定 SyntaxNodeId 与 `Named<T>`/等价 lowering。明确 `NavigationLink`、`NarrativeRef`、`ExecutableNamed`、`LoopNamed` 的互斥分类及 span。正文引用只解析和验证；主动纯预览可使用项目/会话既有政策，不要求逐链接确认；外部 effect 仍需明确执行上下文与宿主能力许可。各类型步骤的输入、输出、依赖和角色由编译器推导；标签是阅读、检索和局部编辑的上下文，但不参与授权和寻址。作者不手工填写步骤 ID 或生成图。

**验收。** 一份文档同时含 URL、锚点、带空格/嵌套括号载荷、围栏、行内代码、取元、计算、循环、正文引用；生成的 ref/step 数、分类和跨度稳定，正文 `run/check/view static` 不发生外部 effect。

### G04 生成物写入不是整代快照

**证据。** [`src/sema/ir.rs`](../../src/sema/ir.rs)将语义文件写到临时目录后，逐个删除并移动 `project.json`、`symbols.json`、`refs.json`、`artifacts/`、`graph/`、`verification/`；[`src/knowledge/compile.rs`](../../src/knowledge/compile.rs)再写旧投影。这不是消费者可原子读取的整代快照，构建中途可能看到混合版本。

**规划缺口。** 规范写了“原子写入”，未规定**整代提交、读者快照、失败恢复和旧版本保留**。

**决策与交付。** 使用带 schema/build-id 的代目录与原子指针或等价快照机制；所有消费者一次选择一代，读完整个结果后才释放。生成物可删除重建；崩溃后只能看到旧完整代或新完整代。

**验收。** 在写每个投影文件时注入故障并并发读取；不得出现 `project` 属于 A 代而 `edges` 属于 B 代，失败后旧代仍可读。

## 5. 可信执行缺口：按场景落实验证门槛

### G05 来源、适用性和验证状态仍是占位投影

**证据。** [`src/sema/ir.rs`](../../src/sema/ir.rs)给函数固定 `"no explicit applicability constraints"`，`SourceRef` 主要指向定义它的源码，`evidence` 摘要来自契约文本；验证记录固定 `contract=unchecked`、`tests=unknown`、`evidence=unverified`、`domain=unreviewed`。现有 [`src/knowledge/lifecycle.rs`](../../src/knowledge/lifecycle.rs)的 `verify` 是另一套 EKC 检查。因此“有 SourceRef”并不表示政策原文被关联或核验。

**规划缺口。** 规范列出字段和七层状态，但没有定义**作者如何写来源/适用性、如何证明测试确实覆盖此制品、谁可将 domain 从 unreviewed 提升、何时失效**。不确定现实真假的结论必须保留 unknown。

**决策与交付。** 用现有 Markdown/frontmatter/契约表表达可选的 `source`、有效区间、地域/单位/输入前提，形成可解析 schema；把编译、静态检查、实跑测试、来源内容、人工领域审阅作为独立证据事件。定义状态迁移、撤销和失效规则；生成物记录证据文件/测试运行摘要，不允许 CLI/Agent 自报 `verified`。

**验收。** 政策原文改变而函数未改时 evidence 状态下降；来源删除、过期、输入不满足分别给不同诊断；无来源的纯工具函数可运行，但不能被宣称为已证实业务事实。

### G06 效应识别存在“未知即纯”的漏洞

**证据。** [`src/sema/ir.rs`](../../src/sema/ir.rs)的 `effects_for_call` 依据名字前缀猜测 `fs/net/llm` 等，其余返回 `pure`；`direct_effects` 只汇总直接调用。未知插件、别名、包装函数和传递调用可能因此被标作纯。[`src/mlsp.rs`](../../src/mlsp.rs)以该集合做 `artifact.execute` 的效应拒绝。

**规划缺口。** 详细规范定义 EffectSet，却没有给出**跨宿主函数、标准库、扩展 ABI、动态调用的权威效应注册表和未知效应策略**。

**决策与交付。** 内核/标准库/插件元数据声明 effects，绑定后做调用图闭包；动态或缺元数据为 `unknown`，需要显式能力授予，不得降为 pure。`view` 静态、`verify pure`、Agent ready 门槛使用同一集合。实际宿主能力限制仍在执行时强制。

**验收。** 用自定义名字包住写文件/网络/插件调用；静态 effect 必须传递到入口，默认 pure 策略拒绝执行。循环调用图收敛；未知动态调用拒绝 pure 证明。

### G07 绑定句柄未表达可复用的授权与适用性

**证据。** [`src/mlsp.rs`](../../src/mlsp.rs)的 `artifact.bind` 主要检查目标唯一及可选 digest，返回 `artifact_id/digest/project/policy/effects`；它没有要求 ready、契约、适用性或来源状态。`artifact.execute` 重编译比对摘要后按 `display_name` 最后一段调用解释器；调用者提交的 `allow_effects` 会变为宿主能力。现阶段这是实验性执行接口，不是可信复用协议。

**规划缺口。** 缺少句柄的**发行者、主体、策略、输入约束、能力上限、过期/撤销、并发源变更**模型；也缺少“绑定到精确嵌套函数”在 runtime 的保证。

**决策与交付。** `bind` 必须验证 SymbolId、签名、输入、已声明适用性、所选验证政策及 effect 上限；政策要求锁时检查 lock。`execute` 复核同一目标与状态并按绑定符号调用。受信本地调用先采用精确绑定引用和摘要复核，不强制搭建签发服务；远程/多租户/不受信调用再加入可信发行者、主体、范围、期限/撤销及防伪机制。客户端参数不是远程授权来源；运行时能力上限始终由宿主强制。响应如实显示政策和未知状态，作者不手写 handle。

**验收。** 本地先覆盖改参数、改依赖、改效应策略、同名嵌套函数和并发源码变更，不得错调用或越权；严格政策增加来源与锁变更，远程增加伪造/过期/撤销测试。`ready` 必须注明相应 policy 与条件，不是全局永久认证。

### G08 锁定与影响分析未覆盖知识变化

**证据。** [`src/sema/lock.rs`](../../src/sema/lock.rs)的 lock 文件虽然写出 interface/implementation/evidence/dependencies，但检查时只比较 `artifact` 摘要。该摘要在 [`src/sema/ir.rs`](../../src/sema/ir.rs)只组合本地 interface/implementation/evidence，没有传递依赖闭包；impact 的边也只来自已识别的本地调用。`knowledge impact` 与语义 impact 是两套命令/图。

**规划缺口。** 未定义**锁的审阅单位、变更分类、依赖 SCC、证据变更如何传播到正文、升级冲突处理**。

**决策与交付。** 分别持久化 target/interface/implementation/evidence/closure 摘要，SCC 按确定规则求闭包；strict 对未审阅目标变化失败。impact 用同一图沿 calls/depends_on/supported_by/narrates 反向传播，并说明为何被影响。每次 lock update 留下人可审阅的差异报告。

**验收。** 本地函数源码不变而下游税率规则变了时 closure 变化；只改描述不混同事实证据变化；来源变更影响客服说明；同一个查询在 CLI、MLSP、view 展示相同受影响 ID 集合。

## 6. 作者与生态缺口：从早期切片持续验证

### G09 Agent 仍可能把候选当作可执行知识

**证据。** [`src/knowledge/reuse.rs`](../../src/knowledge/reuse.rs)、[`plugins/agent/`](../../plugins/agent/)和 `ext/ai/agent.mq.md` 有各自的检索、near-match、工作簿/Skill 固化流程；[`src/mlsp.rs`](../../src/mlsp.rs)另有 artifact handle。现有 [Agent 路线图](ext-agent-optimize.md)主要跟踪工作簿、路由和评测，没有把“发现→精确绑定→验证→调用既有制品→记账”设成强制路径。

**规划缺口。** 需要明确 **REUSE、ADAPT、CREATE 的进入条件与失败后分流**，以及“避免重复生成”用什么客观指标衡量。

**决策与交付。** near/soft-match 只能供候选发现；Agent 可以选择并提交精确候选，REUSE 前必须经同一 resolver 和 policy 得到 bound handle。不可用则记录原因，按权限进入 ADAPT/CREATE 或请求补充决策。新草稿可被发现、查看和在本地明确权限下试跑，进入共享自动复用的 ready 集合前按政策验证晋升。先提供含步骤、源码、约束、负知识和验证状态的最小上下文卡，再支持带期望摘要的局部补丁和语义 diff，避免整篇再生成。trace 包含任务、候选、选定 SymbolId、所有相关摘要、输入摘要、policy、实际执行结果及因果依赖；私密输入只存可审计脱敏摘要。

**验收。** 已有适用能力时实际调用而不新建同义函数；同名未消歧、违反所选政策的过期/未验证候选、权限不足不能自动执行。局部修改不覆盖并发编辑；只改措辞却改变签名必须可见。同样任务与预算下报告重复实现率、错误复用率、发现率、成功率、编辑范围、作者维护成本及失败样本。

### G10 标准库、扩展库与发布物缺少跨层一致性门禁

**证据。** [语法测试规程](../next/Marqdo_Executable_Knowledge_Testing.md)覆盖了部分零参路径金样；仓库有大量 `lib/`、`ext/`、`public/` 和示例。`.github/workflows/` 目前只有 release/pages，没有对每次提交运行可执行知识项目夹具的 CI；[目标示例](../../examples/executable-knowledge/README.md)仍不可端到端运行。现有 [lib/ext 路线图](lib-ext-code-as-docs.md)的历史“链接取元”已被 v2.1 默认遮蔽语义取代，`done` 不代表今天的知识引用集成已完成。

**规划缺口。** 对新语义没有固定的**全仓兼容矩阵、跨平台发布前检查、示例可运行状态标签、回滚规则**。

**决策与交付。** 分层门禁：parser/sema → Tree/Bytecode → 标准库中英镜像 → 扩展插件装载 → CLI/MLSP/view/静态导出 → 一体化书店/订单示例。每个负例也断言诊断码和副作用计数。每次语义变化先迁移 `lib/ext` 并保持 stdout 与 effect，不通过扩展旧语义来掩盖库的过时写法。CI 在 main 的 PR/push 执行无网络核心套件；需要插件/浏览器的平台测试单列结果；发布前检查包内标准库/扩展与二进制版本匹配。

**验收。** `cargo test --locked`、gold 单线程、可执行知识项目夹具、CLI JSON schema 和每个平台打包后冒烟通过；文档示例一旦标为可运行，CI 每次执行。没有 `skip` 可以充当核心能力成功证据。

### G11 文档状态本身缺少统一真相

**证据。** [`doc/README.md`](../README.md)仍将较早 mapping 称为“现行运行时”，将 [详细开发规范](../next/Marqdo_Executable_Knowledge_Development.md)的 v2.0.0 基线直接当下一阶段实施依据；多个旧路线图的 `done` 和新语义目标相互交错。README 的 v2.1 链接/投影描述可能被误读为完整可执行知识系统已交付。

**规划缺口。** 缺少文档版本、优先级、状态和实现证据之间的**维护协议**。这会让人和 Agent 从旧文档学回被迁移掉的语义。

**决策与交付。** 文档顶部标明“目标规范 / 已实现 / 历史兼容 / 被取代”及适用版本；在 `doc/README.md` 提供唯一入口。每项能力把规范段落、代码位置、测试、示例、发布版本关联起来；任何 `done` 必须列出验收命令及结果。旧内容可以保留历史价值，但搜索结果与 Skill/教程默认指向当前规范。制定文档可运行片段的抽取或同步测试，降低“文字与实现分叉”。

**验收。** 新贡献者只读 README、doc 索引和 Skill 能写出默认语义的运行样例；旧取元写法在当前教学材料不会以默认形式出现；变更语法时 CI 能发现核心规范与构造卡片失配。

### G12 项目规模、缓存与权限边界尚无可量化 SLO

**证据。** [`src/project/mod.rs`](../../src/project/mod.rs)每次扫描目录；[`src/sema/ir.rs`](../../src/sema/ir.rs)重新读取并解析项目文件；view 和 MLSP 每次查询也可重新编译。`ProjectRoot::source_files` 特别处理 symlink，但没有明确的路径环/别名去重、资源上限与增量失效协议。当前规范虽列 1k/10k/100k 指标，却没有基线值和可接受阈值。

**规划缺口。** 项目发现、Symlink、外部 import、生成目录、并发编辑、overlay、缓存一致性和诊断聚合缺少一份可测试的**规模与信任边界规格**。

**决策与交付。** 先定义约定式项目发现、源文件唯一性、遍历深度/文件大小上限、外部来源许可和增量失效键；单文件与普通项目免额外注册，只有边界无法由约定表达时才要求显式 manifest。再建立 100/1k/10k 项目生成器、冷/热构建计时和内存基线；超过阈值的操作降级为明确诊断，不能静默跳过或返回旧图。MLSP overlay 要么被安全纳入快照，要么显式拒绝且不宣称即时编辑验证。

**验收。** 扫描顺序与重复路径不改变身份；symlink 环不挂起；修改单文件只失效正确下游；缓存损坏后结果与全量构建一致；目标机器上的时间与内存阈值由基准结果确定后固化进 CI。

## 7. 依赖顺序与可合并切片

| 切片 | 前置 | 作者 / AI 可见交付及工程关联 | 可复核的完成证据 |
|---|---|---|---|
| S0 作者基线与最小语料 | 无 | 六类实际写作/复用任务、当前语法入口、无网络最小门禁（G10/G11） | **已落地**：[tests/executable_knowledge/](../../tests/executable_knowledge/README.md) + `cargo test --test executable_knowledge`；Bytecode 裸名同名误连记为基线缺陷待 S2/G02 |
| S1 可读步骤与唯一目标 | S0 | 一份文档的源码/步骤/view 联动；统一 resolver、图与上下文节点（G01/G03） | **已落地子集**：G01 双图分离；G03 步骤 kind + `steps.json` + view 导航；同名 import resolve；正文 NarrativeRef 仍待 |
| S2 确定执行与早期 AI 复用 | S1 | BoundCall 双后端、最小上下文卡、本地 handle 与现有 Agent 试点（G02/G06/G07/G09 的安全子集） | **已落地**：G02 共享 `resolve_for_exec` + Tree/Bytecode + `bound_calls.json`/`dynamic_calls.json`（`10-boundcall`）；G06 效应；G07 MLSP bind/execute；eng_reuse `bind_required`/`bind_targets` |
| S3 局部编辑与库可读性 | S1–S2 | 摘要保护的局部补丁、语义 diff、受限纯表达式提取；lib/ext 示例优化（G03/G09/G10/G11） | 无无关改写；提取等价；声明变更可见；不为旧库增加兼容语法 |
| S4 业务知识与适用性 | S2；与 S3 可交错 | 书店契约、来源/验证事件、传递效应及政策、负知识（G05/G06/G07/G09） | 分清输入不适用、来源过期和测试失败；纯工具免虚构来源 |
| S5 变化传播与严格复用 | S1–S2/S4 | 分类摘要、SCC 闭包、整代快照、impact、严格 handle 复核（G04/G08） | 依赖/证据变化影响正确；文案不混同实现；故障注入无混代 |
| S6 生态验收与发布 | 按发布声明选择 S1–S5 范围 | 全仓兼容矩阵、可运行端到端示例、平台冒烟、作者/AI 指标（G10/G11/G12） | 发布声明逐条有证据；未支持路径明确；无 skipped 核心成功证据 |

每切片完成时同时更新 [详细开发规范](../next/Marqdo_Executable_Knowledge_Development.md)的实现状态、相关示例及本表；只改文档状态不算完成。S1/S2 先用**无 LLM、无网络、无插件**测试证明确定性协议，随即接现有 Agent 验证实际复用；不等所有治理工程结束。G10/G11 是每片持续门禁，100 单元语料逐片增长；G04 的并发一致性必须在开放并发消费前完成，不能因为列在 S5 就忽略前置风险。

G12 大规模优化与远程凭据服务按场景安排；远程执行若开放，其授权测试就是硬前置。H4b MCP HTTP、dense embedding、更多语言糖和全面本体推理均不阻塞本地链。任务拆分、模块落点和作者验收详见[指导 §7](../design/document-native-development-guide.md#7-交付切片每片都有文档作者与-ai-能看见的结果)。

## 8. 横向验收场景与发布条件

### 8.1 必须固定的四个故事

1. **两文档事实**：李白出生年份定义、同名冲突、显式 import、正文惰性引用、来源变更、锁拒绝；验证“文档即知识库”。
2. **订单定价**：来源政策→税率/折扣→多步计算→正文解释；对每个 `[]()` 生成步骤，修改上游后 `impact` 找全下游；验证“知识可执行”。
3. **权限反例**：取一个无明显危险词的用户函数，内部经标准库/插件包装写文件或请求网络；静态未知或效应传递阻止静态预览和纯策略执行；验证默认安全。
4. **Agent 已有能力复用**：先存在 ready 制品，再给相似需求；Agent 发现后精确绑定并执行，trace 中有 SymbolId、摘要与结果，仓库不增加同义函数；再加入过期/不适用候选并验证拒绝。

每个故事覆盖成功与拒绝、Tree/Bytecode（执行适用时）、CLI/MLSP、view、生成图及清缓存重建。旧版本迁移另保存输入、输出、诊断和 effect 的对照；不以“测试通过”掩盖输出语义改变。

### 8.2 完整可执行知识声明的门槛

- 100 单元确定性夹具和四个故事在 CI 全绿；正文静态路径外部 effect 计数为零。
- 静态调用图与运行 trace 的 target ID 一致；两后端的值、目标、诊断一致；动态调用在结果中明确标记。
- 任何模糊/同名、unknown effect、无效适用性、过期 lock、缺失必需证据都不能被自动标为 ready。
- Agent 在已有适用能力的任务上实际执行 bound handle，且错误复用率、重复实现率与当前 EKC 基线同时报告；保存数据和失败样本。
- `lib/ext/public/examples` 当前语义全仓通过，安装包中的 CLI、标准库、扩展库版本和示例行为一致。
- README、教程、Skill 与目标示例明确区分已实现、实验性与设计态；运行示例由 CI 执行。

这些是完整链的门槛，不要求每次修补版等完整链全部完成。受限本地复用可在 S1/S2 对应正负例和兼容门禁通过后按精确范围发布；严格业务复用增加 S4/S5，远程执行另验授权和隔离，见[分层声明](../design/document-native-development-guide.md#83-发布声明分层)。未完成项不得被笼统“可执行知识已实现”覆盖。目前仍应表述为：**v2.1 已提供文件级新语法、项目索引与语义制品初版；知识验证和 Agent 复用闭环仍在建设。**

## 9. 明确的非目标

- 不要求作者为每个函数写一份重复的机器配置；优先从现有 Markdown、契约表和调用解析推导。
- 不要求每个取元、循环或纯工具函数都附现实世界证据；只有声称业务事实成立或需要高信任复用时才提高证据门槛。
- 不以 LLM 的模糊判断替代符号绑定、类型检查或权限授予；LLM 可选择精确候选、组合、编辑、适配和补测试，编译器校验实际目标与语义，宿主执行权限上限。
- 不把 `Type`、`Entity`、`Fact`、`Function` 强行折叠为“类”；兼容现有 `#`/`##` 表面，语义类型在 IR 中清楚区分。
- 不把 v2.1 的发布回退为未完成，也不把历史路线图的 `done` 自动推广为本路线的完成。
