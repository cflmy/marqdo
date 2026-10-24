# Marqdo 支撑求道知识沉淀与文档编译的框架缺口

| 项目 | 基线 |
|---|---|
| 日期 | 2026-10-24 |
| 框架源码 | `b8a4ddd`，Cargo 声明 2.1.1 |
| 消费者源码 | 工作区 qdagent，`7f8f19d` |
| 文档性质 | 当前源码复核、实测反例和增量建议；不重开已完成的 S0–S6 工作 |
| 配套文档 | [求道产品侧缺口](../../../qdagent/doc/gaps/03-knowledge-ingestion-and-sedimentation-2026-10-24.md) |
| 设计依据 | [文档原生开发指导](../design/document-native-development-guide.md)、[既有缺口路线](executable-knowledge-gaps-2026-10-23.md) |

## 1. 核心判断

Marqdo 已具备让求道采用文档原生知识的主要语言和执行基础。求道尚未接入的能力，不能统称为“框架不支持”。本轮框架侧优先工作是：**让生成知识的语言指导可信，让来源和验证真正约束发布与复用，让新语义制品进入默认 Agent 链路，并提供文档导入所需的通用适配接口。**

`knowledge` 编译的是已有源码和声明，不是把任意 PDF/DOCX 自动变成正确知识的转换器。上传、抽取、模型建模、语义编译、验证、发布是不同阶段。语义编译器保证程序和依赖的确定性；来源支撑、业务适用性以及领域审阅需要明确证据，不能由成功退出或 LLM 自述推导。

现有 [2026-10-23 路线](executable-knowledge-gaps-2026-10-23.md)的审计基线较早，之后已有多次实现提交。本文件按当前源码重新划分：已实现、有限子集、仍有断点；不照抄旧文档中的“无正文引用”“未知效应全为 pure”“两图互相覆盖”等历史判断。

## 2. 当前可直接复用的能力

| 能力 | 当前证据 | 给求道的边界说明 |
|---|---|---|
| 默认 Occlusion、`access`、显式遍历 | [核心表面](../design/core-surface.md)、[基线 01–04](../../tests/executable_knowledge/README.md) | 求道须迁移旧取元；无需增加新语法 |
| 函数、契约、表格、import、关联函数 | `src/parse/`、[check.rs](../../src/check.rs)、[sema/bind.rs](../../src/sema/bind.rs) | 支持编写可复用知识；不是所有知识都要塞进 main 或建类 |
| 项目身份、语义制品、绑定调用 | [project](../../src/project/mod.rs)、[symbols](../../src/sema/symbols.rs)、[IR](../../src/sema/ir.rs) | 已有 SymbolId/ArtifactId、BoundCall/DynamicCall；消费方不另造执行身份 |
| 语义图与检索图隔离 | [knowledge/compile.rs](../../src/knowledge/compile.rs) | 权威语义 `graph/` 与检索 `ekc/graph/` 已分开，旧覆盖问题不再作为未实现项 |
| 知识步骤、正文引用、view | `collect_named_steps`、NarrativeRef、现有集成测试 | 已含 observe/derive/iterate/call/narrate/navigation 子集；正文引用保持惰性 |
| 来源与适用期初版 | [provenance.rs](../../src/sema/provenance.rs) | 已解析 `sources` block list、region、valid_until；sourced 不等于事实已经验证 |
| 精确绑定与本地执行 | [mlsp.rs](../../src/mlsp.rs) 的 artifact.find/inspect/bind/execute | 已有摘要及 SymbolId 检查、按源行定位函数、pure 效应门槛；剩余失效问题见 M02/M03 |
| 局部修改 | MLSP artifact.patch / artifact.extract_pure | 已有摘要保护、措辞修改守卫、纯片段提取；不应要求模型每次重写整库 |
| 整代语义投影、锁、影响 | `write_semantic_projections`、[lock.rs](../../src/sema/lock.rs) | 已有 generations/CURRENT 与多类摘要；覆盖边界及实测失败见 M02/M06 |
| Agent 工作簿、技能、沙箱 | [ext/ai/agent](../../ext/ai/agent.mq.md)、[plugins/agent](../../plugins/agent/src/lib.rs) | 可复用 Runtime、CoW、工具协议和诊断修复；求道不另建 Agent loop |
| 文件上传/存储 | [storage.mq.md](../../ext/data/storage.mq.md)、[routes_media.go](../../plugins/web/internal/httpx/routes_media.go) | 已有上传路由、类型/大小验证和保存；不等于文档知识转换已存在 |

本轮离线执行 `cargo test --offline --locked --test executable_knowledge`：**25 passed / 1 failed / 0 ignored**。通过项包括双后端基线、绑定复用、正文引用、来源状态、局部补丁、100 单元和 CLI 冒烟。失败项为 `s5_closure_digest_moves_with_dependency`，单独重跑仍失败。存在大量有效基础，也存在需要先处理的失效缺口。

## 3. 框架缺口与可验收交付

### M01 · P1：上传底座尚未形成文档提取与来源映射协议

**现状。** Web 插件已注册 `web_upload_validate/save/app_upload`，`handleUpload` 接收 multipart 并存储字节。本轮在 `src/`、`plugins/`、`ext/` 的搜索未找到可供求道直接调用的 PDF/DOCX/OCR → 标准化正文 → 页段来源映射的完整链路。不要把 HTTP 文件保存或 `knowledge compile` 命令包装成已有文档转换器。

**建议。** 在扩展/插件层定义通用 `DocumentSource → ExtractedDocument` 协议，字段至少包括原件 URI/摘要、格式、提取器版本、段落/表格/页码定位、可提取文本、缺失部分、置信度及诊断。先支持 text/Markdown；PDF/DOCX/OCR 可通过独立插件或受控外部提取器接入。格式解析留在扩展，语言核心不引入 OCR 或模型业务。

知识生成由求道选模式、检索既有知识、调用模型并提供 patch；框架提供 compile/diagnostics/sandbox，而不是默默替作者推断领域真相。转换的外部程序必须服从宿主权限和资源边界。

**现有上传实现需补强的具体点。** `handleUpload` 先 `ParseMultipartForm(32 << 20)`、`io.ReadAll`，再按完整字节数做 Validate；32 MiB 是 multipart 内存阈值，不是请求总量硬限。应在解析/读取前限制请求与文件字节数，采用受限流式读取，并测试清理临时文件与失败路径。已有类型/路径验证应复用，不能将此描述为“没有任何上传校验”。

**验收。** 对文本、表格、跨页、坏编码、空文档、密码 PDF、扫描件、超限文件分别返回成功/部分/失败；每条抽取事实都可定位回原件；上传 `.mq.md` 不自动执行；相同文件及提取器版本输出可比较，重试不产生不同身份。

**责任。** Marqdo 提供提取与存储适配规范；求道负责上传界面、任务进度、知识库选择、发布政策与业务模型。

### M02 · P0：来源与依赖变化未完整进入摘要、锁与执行失效

**来源反例（实测）。** 2.1.1 下给函数声明 `sources: source.md#missing-anchor`，实际文件存在但锚点不存在，仍得 `evidence=sourced`；business bind 成功。修改 source.md 后 SourceRef.content_digest 变化，artifact.hashes.artifact 不变；旧 handle 继续执行。删除 source.md 后，旧 business handle 仍执行并返回原值。

**源码原因与边界。** [provenance.resolve_provenance](../../src/sema/provenance.rs)将 `#anchor` 存为 locator，但没有校验锚点；对来源文件计算哈希后，[IR artifact_for_function](../../src/sema/ir.rs)仍用 `digest(contract_summary(&contract))` 构造 evidence 摘要，没有包含来源内容和适用条件。`artifact.execute` 重编译并比较自身 artifact 摘要，但不重验 handle.policy 对应的证据状态。来源改变、来源删除、时间到期可以不改变函数实现，因此自身摘要检查不足。

**依赖反例（实测）。** [现有集成测试](../../tests/executable_knowledge.rs) `s5_closure_digest_moves_with_dependency` 中 `main` 调用 `d.rate()`，仅将被依赖函数返回值从 0.13 改为 0.14；测试读取到的 main closure 摘要未变。已重复验证。这里只确认该夹具失败：需要区分符号/模块同名选择、依赖边和闭包算法各自责任，不据此直接断言所有函数闭包都失效。

**其他源码缺口。** bind/execute 传递的是自身 `hashes.artifact`，没有把已有 `hashes.closure` 纳入句柄复核。`lock.impact` 从制品定义文件和依赖/正文边找反向影响，未把 `provenance` 的外部 corpus URI 纳入来源反向边。

**交付。**

1. evidence 摘要包含规范化来源 URI、定位、内容摘要、提取器/验证版本及适用性；锚点不存在返回不同于文件缺失的诊断。读取错误不能以空文件摘要冒充成功。
2. 修复上述闭包失败，并明确 type/module/function 的身份与测试选择。依赖递归/环用确定的 SCC 或等价闭包摘要规则；无关文件增删不应导致任意漂移。
3. 句柄记录并复核所需的依赖闭包、证据、政策及有效期；execute 重验 business/pure 等政策，不只相信 bind 时的状态。文件变化与时间变化都能使句柄失效。
4. 将 corpus → artifact → caller/answer 的影响关系纳入同一编译结果，来源改动能查询受影响对象；报告接口、实现、证据、依赖分别发生了什么变化。

**验收。** 仅修改来源、删来源、改锚点、时间越过有效期、改直接/传递依赖都拒绝不再有效的句柄；原有测试恢复通过。不得为让测试变绿删除断言或改成只检查命令成功。普通无业务来源要求的纯函数仍可按相应政策执行。

### M03 · P0：验证 API 尚不是可发布知识的完整判据

**现状。** `VerificationRecord` 已分 parse/resolve/contract/execution/tests/evidence/domain，但 `semantic_artifact` 的默认值仍包括 `contract=unchecked`、`execution=not_run`、`tests=unknown`、`domain=unreviewed`、`lifecycle=draft`。MLSP `artifact.verify` 主要返回编译记录并附 policy，没有实际执行对应测试或回写证据。不能把它宣传为已经验证业务事实。

`artifact.bind(policy=business)` 接受 sourced/verified，尚未检查 region 与本次输入、单位、前提、输入域等；`provenance` 用日期字符串比较有效期，没有完整日期格式校验。当前“business”至多表示来源已关联且未被判过期，不等于适用性或事实成立。unknown policy 字符串也未通过枚举校验形成明确拒绝。

`knowledge verify` 检查的是旧 EKC 结构一致性；`conflicts/stale` 是报告命令，`duplicate` 的默认行为也是报告。它们都有价值，但消费方不能从退出码 0 得出“候选可晋升”。求道正存在这种误用。

**交付。** 给调用方稳定的验证结果结构：验证对象/摘要、所选政策、每个检查的 passed/failed/unknown/skipped、证据记录、阻断原因、是否允许绑定或发布。测试运行器生成执行/断言事实；来源存在、引文一致、计算正确、领域结论分别报告。政策和结果字段应有版本化 schema，未知政策拒绝。

保留报告命令兼容性，提供清晰的机器裁决入口或明确 fail 选项；不要悄悄改变已有 CLI 的退出语义。发布政策可渐进：纯工具先证明程序行为，业务事实额外要求来源与适用条件，领域审阅按场景要求；不为所有函数强加人工审批。

**验收。** `sandbox failed`、模型声称“passed”、空证据、无效日期、地区不符、输入越界、测试未运行均不能被标成对应级别的 verified；缺证据与执行失败不同；实际通过的记录绑定内容摘要，修改后自动失效。

### M04 · P1：新语义复用协议尚未进入默认 Agent 学习与执行闭环

**现状。** [plugins/agent/eng.rs](../../plugins/agent/src/eng.rs) 的 `eng_reuse`、[knowledge/reuse.rs](../../src/knowledge/reuse.rs) 已输出 `discovery_only`、`bind_required`、`bind_targets`。这是真实进展，不再列为完全缺失。

但 [ext/ai/agent.mq.md](../../ext/ai/agent.mq.md) 的默认 `run` 仍以 route/resolve、agent-kb workbook、plan 为主；EFI 分支主要检查 create_allowed，未在该编排中找到 artifact.bind/execute 的完整消费。`run verify=True` 依据 `status==ok` 拼接 `verification="status: passed"`；随后可 maybe_learn/promote，并非调用 M03 所需的验证器。求道 BUILD 开启 `learn=True/promote=True` 后才做自己的 docgate，因此两层发布时序有冲突风险。

`build_step_context` 的 CALL 白名单只有名字协议；parent CALL 支持 ARGS 的能力与 step 工具路径不同。求道若要调用精确制品与输入，需要正式适配，不能在提示词中虚构已可调用的工具。

**交付。** 提供现有 Runtime 内的适配步骤：discover → inspect → bind → execute → record。执行工具使用宿主选择的 project/KB/policy，LLM 只提交候选及输入。REUSE 只有在绑定执行成功且有 trace 后计为实际复用；ADAPT 用现有 patch/extract 形成可审阅修改；CREATE 保留检索范围与原因。

提供产品可接入的发布前验证钩子或 candidate-only 模式：产品门禁失败时框架也不能先把技能标为可复用。知识文件、技能、episode 可以保留不同存储职责，但都关联同一版本化 artifact/evidence；不要新增另一套知识语义。

**验收。** 用真实 `agent.run` 编排和确定性假模型/工具响应验证，而不只直调 MLSP：已有知识被调用、无同义实现新增；错误来源/错输入被拒绝；验证失败无 promote；trace 对应符号、摘要、输入与输出。不得将 n-gram 高分或字符串 REUSE 当作成功。

### M05 · P0/P1：提供给模型的语言 Skill 与当前语法冲突

**源码事实。** [skills/marqdo/SKILL.md](../../skills/marqdo/SKILL.md) 前部 Hard rules 2/4 和 v0.2 表仍写“粗体是返回”、斜体是语句，旧 `[key](map)` 取元也仍在前部。后面虽有 v2.1 默认 Occlusion 段落，不能消除同一提示包的冲突。

[host/agent_rt.rs](../../src/host/agent_rt.rs) 的 `load_marqdo_skill/read_skill_pack` 加载 Skill/reference/examples；[agent.skill_brief](../../ext/ai/agent.mq.md) 默认只取前 1200 字符/80 行，deep 预算 4000 字符。因此旧规则可能优先进入窗口，新的说明反而被截掉。实际读取哪个包受 MARQDO_SKILL 和搜索路径影响，部署须记录真实来源，不能假定总在读工作区文件。

**交付。** 从现行核心表面、解析器与金样例维护一份版本化语法卡片，最前面即当前含义：`**…**` 代码、`*…*` 返回、`[描述](payload)`、`access`、显式 import/遍历、正文惰性。旧语法迁入独立迁移卡片，不继续与默认规则并列。模型可按需取契约、来源、表格、类/函数等卡片；语言核心规则和权限边界不可被预算截掉。

使用已有 MLSP syntax/schema/validate/repair 能力，但明确 validate 的范围：当前主要是契约/标签检查，不等于完整事实、所有变量/调用和业务验证。诊断附 span、相关源码、目标语法版本及建议修复范围；优先局部 patch，避免整篇重生成。

**验收。** `dump_step_context` 和 plan 上下文的脱敏快照中没有冲突规则；基础/deep 预算均保留必需语法。卡片中可执行示例在当前 CLI 双后端通过；同一批含旧取元、错误返回、表格几何误用、缺 import 的负例给明确诊断。记录实际 skill path/hash、CLI/Ext/schema/prompt 版本。

### M06 · P1：整代发布已存在，但扫描范围与消费者一致性仍需约束

**现状。** [IR](../../src/sema/ir.rs)已将语义投影写 generations，再原子替换 CURRENT；遵循 `semantic_generation_dir` 的读者可获得完整语义代。但随后还写根目录兼容镜像，[knowledge.compile](../../src/knowledge/compile.rs)另写 engineering.yaml / ekc 等检索投影；不能把语义代原子性推广为“所有消费者、所有索引都属于同一代”。并发构建共用 CURRENT.tmp、.tmp-sema-publish，需明确串行化或独占发布规则。

[project 扫描器](../../src/project/mod.rs)与 [EKC extract](../../src/knowledge/extract.rs)的排除集不同，后者遍历子目录时排除 data/docker，前者不排除 data。求道有多个原文、候选和运行目录，导入 `.mq.md` 后可能在不同消费者里出现不同可见性。已有 .marqdo 排除、深度/文件大小/数量限制值得保留，但尚不能替代显式源范围和信任范围。

**交付。** 项目配置声明 authored sources、corpus、draft/generated 的范围；语义、EKC、Agent/view 使用同一选择规则。发布 manifest 关联语义代、检索代、源码/证据摘要和 schema；消费者拒绝混代并能回退到旧完整代。多库使用独立项目身份，索引聚合不扩大执行文件范围。

**验收。** 暂存上传代码不会进入活动能力；data/ 路径不会因消费者不同而忽隐忽现；并发编译/失败注入期间读者只看到一套可用版本；旧索引可重建，源文档不受损。迁移期明确旧 graph 路径兼容期限。

### M07 · P1：来源粒度和知识类型不足以覆盖文档沉淀

**现状。** 来源主要来自模块 frontmatter，容易将一整篇资料附给该文件所有函数；SourceRef 有 uri/locator/digest，但缺少标准的“具体断言 → 原文片段/表格单元/页码 → 提取版本”关联。普通 EKC `KnowledgeKind` 有 Fact/Decision/Constraint 等，而语义 artifact 主要是 module/type/function；旧 `.md` Fact 的存在不等于可调用事实函数。作者写来源 inline list 时当前解析器会忽略，未知字段也缺少强诊断。

**交付。** 用现有 Markdown/frontmatter/契约表定义轻量知识约定，而不增加求道专属语法：可选的函数/断言级来源、适用范围、单位、负例、冲突/取代关系。明确知识内容类型与运行时类型的区别；事实可以是返回值、规则可以是函数、决策可以是声明性文档，类只用于真实模式复用。

由编译器生成身份、图边和 SourceRef，作者只维护实际内容与必要来源。为 plain note → draft fact/rule → verified callable 提供金样例和机器可读投影，不能仅把文本包在一个 print 中。字段不支持或位置错误时给诊断，避免“写了来源，编译却无声忽略”。

**验收。** 一文多来源、一事实多来源、原文冲突、缺失字段、单位不同、来源片段变化、声明性决策和可执行规则都能区分；无依据的信息保持 unknown；不要求每个纯工具函数带业务来源。

### M08 · P1/P2：生成式知识验收与真实复用评估不足

**现状。** 既有 26 项集成测试和 100 单元是有效基础，但不覆盖资料导入/模型沉淀全过程。现有 `eng_reuse` 协议测试证明 JSON 字段，不证明默认 Agent 调用了句柄；MLSP 直调测试证明本地纯函数复用，不证明模型按正确来源建库。测试 README、旧路线和示例中存在历史状态残留；此次 closure 测试失败也说明“done”必须和当前门禁分开。

**交付。** 两层评测：确定性 fixtures 验证协议和失败边界，固定任务集验证模型沉淀质量。每个样本保存原文、期望知识、可执行断言、来源定位、错误复用反例、实际上下文及版本；明确事实正确率与程序运行率不同。性能指标包括编译/检索延迟、上下文规模、修复轮次、真实 token、知识重复率和误晋升率。

**验收。** 用同一资料完成“首次建库→再次调用→原文更新→旧知识失效→修复发布”的完整故事；包括断网/缺插件/截断/OCR 错误/资料内指令/冲突来源。只统计有目标与结果 trace 的执行复用，不按 REUSE 决策估算零推理成本。核心发布门禁不能以 skip 计成功。

## 4. 建议的跨库接口边界

下表是待实现或待稳定的契约，不暗示现在已有同名函数/CLI。

| 契约 | Marqdo 保证 | 求道负责 |
|---|---|---|
| Document extraction | 原件摘要、标准正文、定位、诊断；扩展层实现格式转换 | 上传、任务、库范围、用户预览 |
| Knowledge draft | 当前语法卡片、检查/修复、可调用元数据 | 事实/规则选择、提示词、冲突与去重决策 |
| Verification report | 每个检查的真实状态、证据/制品摘要、阻断原因 | 项目政策、需要人工处理的歧义、发布编排 |
| Artifact handle | 唯一身份、依赖/证据失效、宿主能力边界 | 提交候选与输入、记录实际使用和解释 |
| Publish manifest | 一致的语义/索引代及可回滚读取 | 知识源码发布、迁移、当前活动版本 |
| Trace | 实际目标、输入、输出、版本与执行结果 | 回答来源展示、知识质量指标、运行记录关联 |

现有 `artifact.*`、sandbox、knowledge 等接口优先扩展；新增 façade 必须复用它们。工具协议、输出 schema 与提示词版本一起管理。首次本地试点可以使用固定目录与已授权项目政策；远程多租户的身份、凭据和隔离单独验收，不能把本地 MLSP 直接当公网授权协议。

## 5. 实施顺序

| 阶段 | 工作 | 完成条件 |
|---|---|---|
| F0 正确性基线 | M02/M03/M05：闭包反例、证据失效、验证状态、语法卡片 | 现有测试全过；来源变更/删除拒绝旧 business handle；提示词不含冲突语法 |
| F1 最小生产消费链 | M04 + 求道文本导入：candidate-only、验证钩子、绑定执行 trace | 同一资料第一次生成，第二次实际复用，门禁失败不学习发布 |
| F2 来源与更新 | M06/M07：统一源范围、来源定位/影响、发布 manifest | 改原文能找全受影响对象，发布失败旧版本仍可用 |
| F3 格式覆盖与评测 | M01/M08：PDF/DOCX/OCR 适配、资源限制、模型对照 | 文档可追溯，不完整提取显式报告，效果和失败分布有数据 |

框架不承担求道的登录界面、笔记列表、项目业务政策；求道不复制框架的 parser、binder、effect、lock 或执行器。无需为了这条主链先完成万能本体、全格式解析或远程授权平台。

## 6. 本轮可复核实验

### 6.1 当前门禁与失败复核

在框架仓库执行：

```bash
cargo test --offline --locked --test executable_knowledge
cargo test --offline --locked --test executable_knowledge \
  s5_closure_digest_moves_with_dependency -- --exact
```

第一条结果为 25 passed、1 failed；第二条仍在 `tests/executable_knowledge.rs:586` 报 `caller closure digest must move with dependency`。两次比较结果相同。这是既有测试的执行结果；本次未修改其实现或断言。

审计开始时 `target/release/marqdo --version` 与旧 debug 都是 2.1.0；上述构建生成的 debug 是 2.1.1。后续最小实验使用新 debug，不能把旧 release 输出充当当前源码能力证据。

### 6.2 来源变化与旧句柄复现

以下程序仅在新建临时目录写测试文件，通过本地 MLSP 调用，不联网、不调用模型。构建当前 debug 后在仓库根运行：

```python
import json
import subprocess
import tempfile
from pathlib import Path

binary = str(Path("target/debug/marqdo").resolve())
root = Path(tempfile.mkdtemp(prefix="mq-source-audit-"))

def rpc(method, **params):
    req = json.dumps({"method": method, "params": params}) + "\n"
    proc = subprocess.run([binary, "mlsp"], input=req, text=True,
                          capture_output=True, check=True)
    return json.loads(proc.stdout)

(root / "source.md").write_text("# rate\n\nrate = 0.13\n")
(root / "facts.mq.md").write_text(
    "---\nsources:\n  - source.md#missing-anchor\n"
    "applicability:\n  region: CN\n  valid_until: 2099-12-31\n"
    "---\n\n## rate\n\n*0.13*\n"
)
first = rpc("artifact.inspect", project=str(root), artifact_id="rate")["result"]
handle = rpc("artifact.bind", project=str(root), artifact_id="rate",
             policy="business")["result"]["handle"]
(root / "source.md").write_text("# rate\n\nrate = 0.14\n")
second = rpc("artifact.inspect", project=str(root), artifact_id="rate")["result"]
print("evidence:", first["verification"]["evidence"])
print("source changed:", first["provenance"][-1]["content_digest"] !=
      second["provenance"][-1]["content_digest"])
print("artifact changed:", first["hashes"]["artifact"] !=
      second["hashes"]["artifact"])
print("after edit:", rpc("artifact.execute", handle=handle, inputs={}))
(root / "source.md").unlink()
print("after removal:", rpc("artifact.execute", handle=handle, inputs={}))
```

本轮观察：`evidence=sourced`、`source changed=True`、`artifact changed=False`；两次 execute 均 `ok=true`、result 为 `0.13`。正确修复后，缺锚点应先阻断相应来源政策；用有效锚点重新绑定后，改/删来源也应拒绝旧句柄或要求重验。不能只修复示例中的缺锚点而遗漏来源内容失效。

### 6.3 求道旧取元风险

以下代码在默认 2.1.1 中，第一行输出整个 map，第二行输出 False：

```markdown
# main

`m` =

| ok | status |
|---|---|
| False | denied |

**old = [ok](m)**
**new = access(m, "ok")**
> print text=`old`
> print text=`new`
```

这是默认语义变化，不是框架需要回退的新语法错误。求道需迁移；框架应提供一致的迁移指导、诊断和版本信息，而不是在 Skill 中继续教授冲突的旧默认。

## 7. 审计限制与后续文档同步

本次验证是源码审查、仓库固定测试和临时输入实验；没有执行真实 LLM 的建库质量评估，没有测试线上 Web 上传或真实 PDF/OCR。M01 的“缺少转换链”限定在所审查的主代码区域，不否定外部工具可以接入。M02 的闭包根因仍需按身份、依赖图、摘要计算分层定位。

完成后应同步当前 Skill、README、测试 README 和路线进度：分别标记语言能力、协议子集、产品已接入、当前测试结果。优先把失败样例转成稳定回归门禁，再扩大“可验证知识”“业务复用”“上传编译”的能力声明。此次文档提交不表示缺口已修复。
