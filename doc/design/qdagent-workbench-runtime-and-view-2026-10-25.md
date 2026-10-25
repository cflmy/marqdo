# qdagent 知识工作台所需的多模型运行与 View 嵌入支持

日期：2026-10-25。状态：Partial / in progress（M00–M06 脚手架与合同门禁已落地；F 矩阵仍有缺口，不得整文宣称 Verified；原生非兼容适配器仍 unavailable）。

源码基线：Marqdo `dc853fb`（2.1.2），qdagent `643dd31`，已同步远端。应用侧配套：[qdagent 多提供商与知识工作台](../../../qdagent/doc/design/multi-provider-and-knowledge-workbench-2026-10-25.md)。跨仓相对链接以兄弟仓库工作区布局为准。

沿用 [文档原生开发指导](document-native-development-guide.md)、[可执行知识规范](../next/Marqdo_Executable_Knowledge_Development.md) 与 [View 阅读体验设计](view-knowledge-reading-experience-2026-10-24.md)。本规格补充嵌入、调用身份和运行观测，不另造语言或第二套知识真相。

## 1. 框架目标与责任边界

让任何宿主应用可以按请求构造模型、追踪所有真实模型调用、以只读 View 展示知识文档，并从引用定位到版本一致的知识与来源。qdagent 的设置页、模型目录持久化、账单展示、会话导航、工作区授权和画像管理由应用实现。

| 能力 | Marqdo 拥有 | qdagent 拥有 |
|---|---|---|
| 多提供商 | 协议适配、模型 handle、能力协商、调用参数、归一化错误 | 提供商账户/端点/模型 CRUD、默认选择、凭证管理 |
| 模型路由 | Runtime 执行显式选择和允许的辅助模型策略 | 用户选择、工作区模型允许列表、路由授权 |
| 用量 | 每次请求/重试的身份、usage 来源与完整性、子任务传播 | 持久化账本、价格版本、费用估算、筛选/导出/预算策略 |
| 实时运行 | 逐事件输出、取消传播、背压、终态 | HTTP 会话鉴权、轮次映射、持久化事件重放、断线 UX |
| View | 文档语义、正文/目录/步骤、纯读取、稳定定位、可嵌入输出 | 容器/主题/深链/阅读历史/产品操作 |
| 知识引用 | 解析/歧义/版本/来源定位、验证多维事实 | 消息引用标识、授权、历史资料保存与入口 |
| 工作区 | 按显式 project 编译/查询，缓存及输出隔离 | 工作区 registry、root 解析、用户权限、文件管理 |
| 画像与笔记 | 通用文档渲染、来源关联、验证/编译原语 | 内容策略、提案/确认、生命周期、展示和上下文选择 |

不在语言核心添加 provider/workspace/profile 关键字；沿用 `.mq.md` 函数、结构化参数、扩展插件和 MLSP。不得让“可读”取决于用户手填 hash、node id、source span 等机器字段。

## 2. 2.1.2 已完成的基础与剩余缺口

### 2.1 当前可复用基础

| 源码 | 已有内容 | 本规格的增量 |
|---|---|---|
| [ext/ai/llm](../../ext/ai/llm.mq.md) | `create(model,base_url,api_key,backend,name)`、命名 handle、ask/stream | 无需重做基础 handle；增加隔离策略、协议能力、完整消息与调用身份 |
| [LLM plugin](../../plugins/llm/src/lib.rs) | 兼容 chat completions HTTP、SSE 文本读取、非流 usage 三项 | SSE usage/异常终态/每次调用观测、可取消持续输出 |
| [Agent](../../ext/ai/agent.mq.md) | Agent 唯一 loop、router_model、EFI、绑定/发布门闩 | 主/路由/修复/生成工作簿的模型及计量上下文传播 |
| [语义 IR](../../src/sema/ir.rs) | Artifact、KnowledgeStep、KnowledgeRef、SourceLocation、BoundCall | View 面向阅读的完整投影及 ID/版本映射 |
| [MLSP](../../src/mlsp.rs) | `artifact.inspect`、`link.resolve`、verify/bind/execute、document.extract | 只读 document/引用解析及 capabilities 版本握手 |
| [KnowledgePresentation](../../src/view/knowledge_present.rs) | schema v1，artifact nodes、依赖、generation/page token | 保留兼容，新增正文/目录/refs/source spans/结构化诊断 |
| [View output](../../src/view/output.rs) | `--no-exec` 在构建前跳过 capture | 将纯读取能力暴露给嵌入客户端，不复用会执行的 live 默认路径 |
| [document extract](../../src/document_extract.rs) | TXT/MD，partial，PDF/DOCX/OCR 显式不支持 | 能力查询及稳定来源定位；不要求本阶段重造 OCR |
| [数据资源](../../ext/data/db.mq.md) | SQLite migrate/exec/事务、唯一/索引声明 | 应用可直接承载 registry/账本/索引；不需另加产品专用数据库框架 |

2.1.2 已把知识辅助块移到正文后并折叠，跳过 module/type 卡片，显示“结构步骤非实际执行时序”。旧 View 截图中的最初顺序问题已有改进；但这不意味着已提供能直接放进 qdagent 右栏的独立阅读 SDK。

### 2.2 直接影响集成的事实

1. `llm.complete_result` 当前仍走 `openai.chat_completions`，Ollama 也是兼容路径；`backend` 名称不是任意原生协议适配器。
2. `plugins/llm::chat_completions` 从 `prompt` 构造一条 user 消息，没有完整消息列表契约；不能靠模型选择 UI 弥补历史丢失。
3. `usage_from` 缺字段填零；Agent `model_turn(stream=True)` 直接 `tokens=0`，最终 `token_cost=tokens`。现有输出无法支撑准确货币或流式用量显示。
4. 原生 LLM SSE parser 返回事件数组；`ext/ai` 有流事件机制，但应用所用 [web invoke](../../plugins/web/internal/httpx/routes_invoke.go) 在 `CallLibPath` 返回后才写 `{sse}`。需要打通实际增量路径，不能只换 Content-Type。
5. Agent 生成子工作簿的模板仍能构造默认 `llm.llm`；只传主 agent handle 不足以证明嵌套调用沿用了用户选定模型。
6. `build_file_view` 默认传 `execute=true`；live GET 路径使用它。嵌入阅读绝不能直接暴露这个入口。
7. Presentation 目前按 artifacts 投影，缺全量 prose/heading/step/ref 的统一阅读树；`render_knowledge_html` 另从 steps/refs 渲染，容易与 JSON 事实发生覆盖差异。
8. `source_digest` 当前由语义 artifact hashes 汇总，`semantic_generation` 是语义散列截短，`page_token` 来自原文件。它们不能直接分别冒充原文 revision、`.marqdo/CURRENT` generation 和跨请求完整版本。
9. `sse_accepts_packet` 对缺 token 的 legacy 包放行；正式嵌入客户端必须采用严格版本匹配，而不是继承这项兼容宽容。
10. `artifact.inspect/link.resolve` 每次走当前 semantic_project；现有入口不保证可以读任意历史 generation。历史引用需要明确的保留与解析合同。

## 3. 共用合同、版本与能力发现

合同名字固定为 `ModelSelectionV1`、`ModelCallUsageV1`、`RuntimeEventV1`、`DocumentViewV1`、`KnowledgeLocatorV1`；它们是这次扩展的 v1，与当前 KnowledgePresentation `schema_version:1` 不混用。新增 read model 不强行给已有 JSON 改含义。

新增 MLSP `capabilities`（拟议）：

```json
{
  "schema_version": 1,
  "runtime_version": "implementation-version",
  "features": {
    "llm.explicit_selection": 1,
    "llm.call_usage": 1,
    "runtime.event_stream": 1,
    "runtime.cancel": 1,
    "view.document": 1,
    "view.reference_resolve": 1
  },
  "adapters": [{"id": "openai-compatible", "available": true}],
  "extract_formats": [{"format": "text", "available": true}],
  "loaded_plugins": [{"name": "llm", "capabilities": ["call_usage_v1"]}]
}
```

只返回实际探测且加载可用的功能。不能仅从 CLI 版本推断 native plugin 符号存在；字段未知时应用降级或解释不支持。协议允许新增可选字段；删除/改变状态语义必须提升 major contract version。能力查询不调用供应商、不执行项目代码。

MLSP 继续使用现有 `{id,ok,result|error}` envelope；qdagent `/api/app` 转为自己的 `{ok,schema_version,data|error}`，不要要求它们共享同一层 envelope。所有合同提供成功、partial、unknown、unsupported、stale、cancelled、错误 fixtures 与 JSON Schema；应用和框架从相同样本跑消费测试。

## 4. M01：请求级模型身份与适配器

### 4.1 显式选择合同

应用将配置解析成可信 `ModelSelectionV1` 后传入 runtime，核心字段：

```json
{
  "schema_version": 1,
  "selection_id": "sel-1",
  "provider_id": "provider-1",
  "model_id": "model-2",
  "provider_revision": 3,
  "model_revision": 5,
  "adapter": "openai-compatible",
  "endpoint_ref": "endpoint-local-1",
  "credential_ref": "credential-local-1",
  "credential_version": 2,
  "upstream_model_id": "example-model",
  "parameters": {"max_output_tokens": 2048},
  "fallback_policy": "disabled"
}
```

endpoint/credential 引用由宿主 resolver 解析为调用局部配置；短期直接向现有 `llm.create` 传参数亦可，但这不是可序列化的 public snapshot。API key 不能出现在 events、debug JSON、生成工作簿、prompt、错误 URI 或持久化模型对象中。

当前显式 args 优先的能力保留；新增 strict selection 模式：字段缺失报错，不回退 process env/frontmatter。无凭证端点通过 `auth_mode:none` 显式表达，避免 falsy key 触发环境 fallback。legacy 命令行未传 selection 时仍可沿用原有 env 行为；本地便利兼容不扩散到应用请求。

每个运行绑定 selection snapshot，后续设置改变不修改它。主模型与 router_model 分开但同属受控 selection 集；manual 模式所有需要 LLM 的步骤固定选择，REUSE 无模型调用仍允许。fallback 必须由宿主明确授予、列出允许目标，不能因额度/网络错误静默转到另一个供应商。

### 4.2 子任务和工作簿传播

统一 `RunContextV1 = run_id,parent_run_id?,turn_id?,workspace_scope,selection_context,usage_sink,event_sink,cancellation_token`。进入 ask/step/plan、repair、summary、learn、子工作簿时显式继承；新 run/call id 关联父节点。

生成工作簿只包含不可重放滥用的运行上下文引用，不写密钥；执行时宿主注入/解析。独立导出工作簿在缺凭证/选择时清晰要求配置，不偷偷取默认模型。在任何 child/spawn/resume 路径遗漏 selection 都应由 strict 模式拒绝并给出定位诊断。

同时审查环境变量不只是凭证：project root、temp request、输出目录、event sink 也必须请求局部，不能在并发运行中依靠进程可变状态串联。

### 4.3 适配器与消息

适配器最小职责：验证 selection、报告能力、序列化请求、解析增量结果、usage 归一化、取消传输、错误归一化。可选 discovery/probe；无模型列表 API 仍允许手工配置。

V1 消息包含 role 和 content parts，parts 最低 text，附件通过受控引用；tool call/result 要保持配对及 role。不支持的 part/parameter 返回 `llm.unsupported_input/parameter`，不能悄悄转成空字符串。适配器决定协议格式，Agent 决定行为；qdagent 不拥有供应商协议代码。

阶段一验证两个不同兼容端点，分别至少两个模型，以及无密钥本地兼容服务。阶段二增加原生非兼容协议：每个适配器需具名能力、流式/非流式/错误/取消/usage fixtures 和独立接入说明后才能在 capabilities 中置 available。不能把所有厂商默认走兼容接口后统称全支持。

统一参数采用 `max_output_tokens,temperature?,stop?,reasoning?` 等语义键，能力不支持时明确拒绝；不随意透传客户端未知参数。实际 upstream model 若与请求 id 不同，在 result 中保留 requested/resolved 两者。供应商版本与价格随时变化，本文仅锁定内部合同；实施原生适配器时另附当时官方协议核验记录。

## 5. M02：用量、事件流与取消

### 5.1 ModelCallUsageV1

```json
{
  "schema_version": 1,
  "call_id": "call-1",
  "attempt_id": "attempt-1",
  "parent_call_id": null,
  "run_id": "run-1",
  "turn_id": "turn-1",
  "workspace_scope": "ws-1",
  "purpose": "answer",
  "selection_id": "sel-1",
  "provider_id": "provider-1",
  "model_id": "model-2",
  "requested_model": "example-model",
  "resolved_model": "example-model",
  "status": "succeeded",
  "usage_state": "final",
  "measurement_source": "provider",
  "input_tokens": 1200,
  "output_tokens": 300,
  "total_tokens": 1500,
  "cached_input_tokens": 800,
  "reasoning_output_tokens": null,
  "revision": 2,
  "provider_request_id": "opaque-provider-id",
  "started_at": "2026-10-25T01:00:00Z",
  "finished_at": "2026-10-25T01:00:03Z"
}
```

字段要求：tokens 非负整数或 null；`usage_state = pending|partial|final|unknown`；`measurement_source = provider|estimated|none`。估算可作为单独 estimate 字段返回，不能覆盖实测字段。供应商没有报告时用 null/unknown，传输失败不能自动解释成零消耗。

`status = started|succeeded|failed|cancelled|interrupted`，status 和 usage_state 独立。每个真实发出的 HTTP attempt 一个唯一 id；预检拒绝只产生 call-rejected 事件，不伪造已发送请求。框架重试必须可观测：所有 attempts 包括失败/有部分输出的 attempt；重复 delivery 的 usage revision 不算新请求。

累计 SSE usage 在同 attempt 内按 revision 发布完整快照，不让上层猜是 delta 还是 total。provider 总量与 input/output 不一致时保留原始读数及 `usage.inconsistent` 诊断，不自行强制改账；细分字段要标 inclusion 关系以避免缓存、reasoning 与总量重复相加。只保留脱敏的必要 provider usage 扩展字段，不把完整原始响应当 public usage。

纯知识复用用 run summary 中 `llm_calls=0`、`measurement_source=none` 表示，而不是伪造一条耗费为零的模型请求。框架输出 `call_count,known_tokens,unknown_call_count`，宿主据此显示覆盖程度。

### 5.2 事件流合同

```text
run.started
  → model.resolved
  → model.call.started
  → message.delta / tool.progress / usage.updated（零到多次）
  → model.call.finished（每个 attempt 各一条）
  → document.produced / citation.produced（仅有真实对象时）
  → run.completed | run.failed | run.cancelled | run.interrupted
```

`RuntimeEventV1` 公共字段：`schema_version,event_id,seq,run_id,parent_run_id?,turn_id?,workspace_scope,timestamp,type,payload`。seq 每个 run 单调；不同子 run 可以并发，宿主按所属 turn 另分配产品流序号。runtime terminal 事件仅一次；后续补全计量用有 revision 的 `usage.updated`，不能改变内容终态。

事件 sink 支持有界队列和背压；慢客户端不能无限堆积 token。丢弃可合并 progress 时记录 gap；内容与终态事件不可无声丢失。连接终止不会自动重跑；持久化/重放由应用，但框架需在实时回调后仍提供可核对的 final result/call summaries。

同一真实调用所有数据使用同一 run/call identity，不能流与最终结果各生成一套 id。工具参数/结果按 policy 脱敏，不把 secret、完整私有文件、模型内部推理链写入默认进度事件。

### 5.3 打通传输而不重造 Agent

需要同时打通 LLM provider reader → Agent event sink → host callback → web streaming handler → 浏览器。现有 `sys.stream_publish` 可以作为底座，但要验证它是否绑定正确请求且在 `CallLibPath` 返回前可写出；不能用进程全局事件广播混流。

可实施路径：新增 web 流式 invoke/job primitive，由框架注册 run handle 和 event sink；调用返回 handle 后宿主可订阅/取消。采用插件 ABI 增量接口及能力检测，旧 ABI 未支持时拒绝新 streaming 模式，保留旧完整返回。qdagent 只挂产品鉴权、event id 持久化和 HTTP 端点，不添加另一个推理循环。

SSE parser 必须处理任意 TCP 分块、UTF-8 跨块、CRLF、多行 data、注释 heartbeat、尾帧、usage-only 帧、[DONE]、连接在结束前关闭。未知可选事件可忽略且记诊断；已知事件无效、异常截断不能静默当成功完成。公共 framing 不包含供应商私有认证信息。

### 5.4 取消与重试

取消 token 传到 HTTP reader、循环、工具边界、子任务。取消后不得启动新模型调用/新工具；正在进行的不可抢占操作返回 `cancelling` 和是否仍运行，结束后才报告 cancelled。明确“停止生成”不能撤销已经完成的外部写入。

调用超时、连接错误、鉴权失败、限流、协议错误分开；错误字段包含 `code,retryable,retry_after_ms?,call_id,attempt_id,usage_state`。不要将原始包含密钥/完整 prompt 的错误直接传到 UI。限流/短暂网络错误按有界策略重试；已产生输出的重试不能接在同一条文字后假装连续，也不能把未知结果的外部工具操作自动重做。

费用价格由应用管理。框架保留 legacy `tokens/token_cost` 并标注 deprecated/单位为 token，新功能只消费 ModelCallUsageV1，避免破坏旧脚本的同时停止误用。

## 6. M03：无执行的嵌入式 View

### 6.1 新入口与纯度约束

新增内部公共服务 `analyze_document → project_read_model → render_fragment`；MLSP 暴露 `view.document`，可选 CLI `view document --json --no-exec` 与它共享实现。不要让新入口先进入 `build_file_view` 再隐藏按钮/清空输出。

读取允许：授权范围内原文/已编译快照/已有验证记录。读取禁止：运行模块入口、加载文档声明插件、执行外国代码、LLM、自动抓取 URL、写回、重新验证执行测试、隐式发布/重编译写盘。静态分析/内存投影允许；缺完整依赖时明确 partial。任何需要执行的行为必须另走显式 execute/verify 产品流程。

当前 `compile_project` 声明不加载插件/不求值/不抓取来源，可复用；缓存写入若需要，作为单独的宿主管理路径，不能把“读文档”隐含写项目目录。

### 6.2 DocumentViewV1 结构

请求由宿主传入受控 project 和文档定位，宿主必须已授权；框架仍做路径边界校验。

```json
{
  "method": "view.document",
  "params": {
    "project": "/authorized/project",
    "document": "knowledge/example.mq.md",
    "revision": "sha256:requested-source",
    "generation": "generation-id-or-null",
    "mode": "read",
    "node_id": "optional-node",
    "include": ["fragment", "outline", "nodes", "refs", "diagnostics"]
  }
}
```

```json
{
  "schema": "DocumentViewV1",
  "schema_version": 1,
  "document": {
    "id": "project-relative-document-id",
    "title": "知识标题",
    "source_revision": "sha256:raw-bytes-digest",
    "semantic_digest": "sha256:semantic-facts",
    "generation_id": null,
    "presentation_token": "opaque-token"
  },
  "execution": {"state": "not_run", "run_id": null},
  "render": {"format": "sanitized_html", "fragment": "<article class=\"mq-document\">…</article>", "renderer_version": "1"},
  "outline": [],
  "nodes": [],
  "refs": [],
  "diagnostics": [],
  "completeness": "complete",
  "capabilities": {"read_only": true, "has_semantics": true}
}
```

`generation_id` 来自实际编译投影；即时分析但未编译发布时为 null，同时给 semantic_digest。`source_revision` 是原始文件字节摘要，不能复用当前基于 artifacts 的 `source_digest`；`presentation_token` 覆盖 source revision、generation/semantic digest、依赖版本、renderer/schema、读取策略与请求范围。应用缓存还必须含用户授权范围。

`completeness = complete|partial`；语法损坏时允许返回原文与结构化诊断，但不能展示猜测出来的正常知识图。失败用 error envelope，不能空字符串伪装“文档无内容”。大文档 response 支持 section cursor 和 truncated 标识，限制要可配置并在响应中标明。

### 6.3 统一正文、目录、节点与引用

节点包含 `id,parent_id,kind,label,order,source_span,artifact_id?,ref_ids[],children[]`。kind 包括 heading/prose/table/code/observe/derive/call/iterate/narrate/navigation；正文树来自 parser/classifier/sema 的共同投影，不能只枚举 functions 后把每个步骤标 call。

outline 描述读者文档层级，不简单把函数和对象全铺成同一级。HTML 中节点带 `data-mq-node`，引用带 `data-mq-ref`；实现 payload 按需展开，说明作为正文。动态调用/无法解析引用保持原文并解释原因，不虚构执行边。

首批 renderer 共享 `src/view/document.rs` 的文档表面，standalone/full page 和嵌入模式调用同一 read model；只改变容器和导航外观，不产生第二种语义解释。替换当前 `KnowledgePresentation` 与 `render_knowledge_html` 双路径重复组装。

稳定 ID 的保证范围要写明：

- 同一 revision 再次读取，node/ref id 必须相同。
- 仅前插空行/叙述时，已有命名符号保持 symbol id；步骤 id 由结构身份映射，不能只靠行号。
- 同一行多个节点需要 column/offset/span 区分，不能都映射成 `stmt-N`。
- 匿名步骤重排/重复同名内容可能无法唯一映射；返回 unmapped/ambiguous，不能声称任意编辑下永恒稳定。
- source span 明确 1-based line/column 和 UTF-8 byte offset；JavaScript UTF-16 偏移不直接当 Rust 字节偏移。

旧 `#stmt-{line}` 可保留为当前版本兼容锚点，但不得作为跨版本唯一身份。

### 6.4 嵌入渲染、主题与访问性

fragment 不包含 html/body、应用导航、运行按钮、内联可执行 script、全局 CSS；CSS 以 `.mq-document` namespace 限定。主题暴露 `--mq-bg/surface/text/muted/border/accent/font-body/font-code`，宿主映射自己的设计 tokens；不把独立 View 的全局 `*`/body 样式注入 qdagent。

首批推荐 fragment + 结构化引用 + 宿主事件代理；无需为 iframe 和 React 各重做 parser。可选 iframe embedded shell 必须只读、无重复导航，宿主/子页通信校验 origin/source/nonce/schema，仅允许 open-reference/selection/ready 等已定义消息，不能从内容发任意运行指令。

所有文本/属性转义，允许的 Markdown HTML 白名单净化；链接 scheme 限制；外部资源不默认跨网加载；代码/公式不得执行。静态导出、独立页面、嵌入三者都测试恶意文本/闭合标签/危险 URL。不要把 KnowledgePresentation 内 JSON 的转义当作所有正文 XSS 防护。

标题/列表/表格用语义 HTML；说明链接有可访问名称，焦点可见，视觉排序和键盘顺序一致。折叠只隐藏技术细节，不隐藏理解结论所需的正文。320–720px 容器宽、暗色、200% 缩放和 print 都要能读，不出现遮住正文的关系连线。无 JS 时仍能读正文和普通链接。

## 7. M04：知识、来源与历史引用定位

### 7.1 KnowledgeLocatorV1

统一 locator：`project_id,document_id,source_revision,generation_id?,artifact_id?,node_id?,source_ref_id?,span?,relation`。relation 区分 cites/supports/derived_from/executes。普通链接导航、NarrativeRef 和 BoundCall 都能定位，但语义不同；点击说明永不成为 BoundCall 的执行。

新增 `view.reference_resolve`：接受 locator 或当前文档 ref_id，返回 `resolution = resolved|stale|unresolved|ambiguous|version_unavailable|unsupported`、目标 locator、候选（若有）、定位精度、诊断、读者标签。框架内部路径不作为浏览器 href，应用根据授权映射为 document/source route。

扩展 `artifact.inspect` 的版本参数，旧无版本调用仍查当前；新调用带 expected source/generation mismatch 时不能自动降到当前。来源读取接口可由宿主取实际文件，再调用框架 locator 映射；框架不自动 fetch 外部网页。`mq://` 必须按框架规则解析，不能交给浏览器或把 URI 直接当文件路径。

### 7.2 历史版本与保留责任

生成 artifact/generation 元数据不等于保存了原文历史。框架须输出可保存的 snapshot manifest：原文 revision、依赖 revision、source refs、artifact hashes、renderer/schema version；应用决定哪些源版本随引用保留及保留周期。已有 Git 或内容寻址存储可提供 snapshot resolver；不强制应用部署 Git。

source revision 定位到存储快照后，框架基于该快照解析；缺失任何必要版本时返回 version_unavailable/partial，不读取最新文件填充旧快照。如果只留 artifact 元数据、原始资料已删除，仍可展示历史元数据但明确“原文不可读取”。

知识改名、移动或 source anchor 改变可返回候选映射，只有唯一可靠匹配才自动导航；否则用户选择最新版/候选。保持旧引用与新引用两个对象，防止历史回答证据在后台悄悄改变。

### 7.3 验证和运行事实

节点详情至少携带 purpose/description、signature、applicability、provenance、effects、dependencies，以及 parse/resolve/contract/tests/evidence/execution 各自状态与记录来源。sourced 只能说“已关联来源”；static verify 成功不表示执行成功或适用业务正确。

真实执行证据独立结构 `run_id,artifact_digest,closure_digest,source_revision,generation_id,inputs_digest,executed_at,result_state,evidence_ref`，展示时匹配版本。旧运行结果不涂成当前文档的绿色通过。知识复用必须存在实际 bind/execute trace，不能凭 find 命中显示“已执行”。

View 不运行验证测试以填满状态。若用户点“验证”，应用走已有 verify/policy 门禁并记录新 evidence，之后读取才刷新详情。

### 7.4 错误与授权协作

| 内部结果 | 应用建议呈现 | 不能做 |
|---|---|---|
| unresolved | 无法定位该知识，可读原引用 | 跳转到第一个同名 artifact |
| ambiguous | 显示已授权候选和位置 | 自动选择或暴露不可见候选 |
| stale | 该知识/来源已变化，提供版本选择 | 静默替换旧引用 |
| version_unavailable | 回答所用版本不可用，可查看最新版 | 用最新版伪装历史证据 |
| partial | 部分依赖或来源缺失，说明范围 | 默认绿色通过 |
| permission denied（应用） | 无权访问 | 在错误中泄漏标题、路径、片段 |

用户权限归应用；框架服务在调用边界接受 scoped project/source resolver，不把传来一个 root 视为可遍历整个服务器。所有路径经过 canonicalize/symlink 检查；缓存的语义事实可共享，带原文的响应必须按权限复核，不能只靠 artifact id 缓存授权。

## 8. M05：工作区、笔记与导入所需的通用底座

不为 qdagent 画像加语言特殊节点。通用能力应支持普通文档元信息读取、来源范围、内容修订、diff/patch 的 expected digest、提取格式 capability。

document.extract 继续使用 DocumentSource→ExtractedDocument，报告 format/status/segments/source locator/diagnostics。TXT/MD 可支持；PDF/DOCX/OCR 没有适配器时明确 unsupported/failed，不能将空文本当成功。partial 保留已提取片段及原文范围、遗漏原因；发布政策由应用决定。

同一资料可产生多个知识单元，provenance 必须到具体片段，不能所有函数只指向文件根。原始资料 digest 改动的失效由现有 provenance/closure 保证；应用用户画像也可用普通来源引用，但不自动进入公共知识发布。

ProjectRoot/CompileOptions 显式传递，工作区 A/B 的 source id、输出 generation、缓存、临时目录必须隔离。避免每读一个节点就全项目编译：引入按 project snapshot 复用的分析服务，针对来源/依赖变化失效；并发同 snapshot 请求合并，不能分别写同一个 CURRENT。

上传 multipart/大小限制已有 web 底座，框架只提供安全提取和可消费结果；浏览器上传、作业生命周期、发布确认、权限和历史保存仍在 qdagent。多文件任务桥接采用结构化参数，禁止拼接用户文件名进 shell/JSON 或共用固定请求临时文件。

## 9. 实现落点与兼容策略

| 模块 | 改动方向 | 兼容要求 |
|---|---|---|
| `plugins/llm/src/lib.rs` | adapter boundary、消息数组、usage、event sink、取消 | 保留已有符号；新能力有 probe 和合同版本 |
| `ext/ai/llm.mq.md` 与 `ext/ai/llm/*` | strict selection、结构化 result、public snapshot | 无 selection 的旧调用仍可运行；不得回显密钥 |
| `ext/ai/agent.mq.md` | run context、子工作簿传播、逐调用观测 | 不复制 Agent loop，EFI/verify/publish_gate 不被绕过 |
| `plugins/agent/src/*` | 子任务/恢复上下文、cancel、event identity | resume 不重发已结算请求；不可恢复说明 interrupted |
| `src/host/*`、插件 SDK、web invoke | 请求局部 event/cancel 桥接 | ABI 旧宿主不伪支持；并发请求不混流 |
| `src/view/knowledge_present.rs` | DocumentView read model/兼容 adapter | 旧 presentation v1 不被新字段语义污染 |
| `src/view/document.rs,render.rs,html.rs,mod.rs,output.rs` | 共享纯读取、fragment、scope CSS、定位 | live/debug 与 read 明确分离；旧 no-exec 门禁保留 |
| `src/mlsp.rs` | capabilities、view/document/reference、版本 inspect | 使用现有 envelope；未知 feature 可探测 |
| `src/sema/ir.rs,provenance.rs` | span、snapshot/locator、依赖与来源失效 | 不改变核心语法，不把读请求变为执行 |
| `src/document_extract.rs` | capability 和片段定位完整性 | 保留不支持格式的显式失败 |

测试分层对应 plugin HTTP fixtures、Agent 假模型、MLSP read contracts、web 持续流集成、独立/嵌入 DOM 和浏览器交互。应用共享样本文件必须注明 schema 版本；两仓 PR 在各自文档索引相互链接对应实现说明。

迁移顺序为新增合同/接口 → 旧输出 adapter → qdagent 按 capability 接入 → 切默认 → 后续版本再考虑弃用。不要同时移除 legacy 行为且要求应用一次完成全链升级。发布包对 CLI/ext/native 插件做组合测试，缺插件给功能不可用诊断而非 unknown function 直到用户点击才暴露。

## 10. 交付切片与验收门槛

| 切片 | 前置 | 验收输出 | 支撑应用任务 |
|---|---|---|---|
| M00 合同冻结/探测 | 无 | schemas、fixtures、capabilities、版本和插件清单 | Q00、所有升级探测 |
| M01 多模型隔离 | M00 | 兼容适配器、strict handles、history、子任务选择一致 | Q01/Q02 |
| M02 调用观测与流 | M01 | 每 attempt usage、真流式、取消、错误/重放一致 | Q03/Q06 |
| M03 只读嵌入 View | M00，可与模型线独立 | DocumentViewV1、fragment、目录/节点、无执行证明 | Q04 |
| M04 引用与快照 | M03 | 历史 locator、来源详情、版本失效/歧义 | Q05 |
| M05 文档工作区底座 | M03/M04 + 已有 extract | 范围隔离、提取定位、并发与缓存 | Q07/Q08 |
| M06 集成与性能 | 全部所选切片 | 旧门禁 + 新 contracts + 浏览器与负载报告 | Q09 |

M01 支持多个兼容提供商不等待原生协议全做完；但原生适配器需另列完成状态。M03 不等待计费系统。M04 未完成历史快照前，应用可以显示当前文档，但必须把历史引用标“版本不可用”，不能宣布引用链闭合。

每个切片的 Definition of Done：实际接口可调用、失败合同覆盖、无副作用/隔离测试、文档示例由测试消费、版本能力可探测、qdagent integration fixture 通过。只有可运行的验收证据才可将 proposed 改为 verified。

## 11. 框架测试矩阵

| 编号 | 用例 | 断言 |
|---|---|---|
| F01 | 两端点各两模型、并行 A/B、无密钥本地端点 | 请求 payload/端点/鉴权正确，环境不串扰 |
| F02 | step/plan/router/repair/child/resume | strict selection 和 parent identity 完整，不退回默认模型 |
| F03 | multi-turn、system/tool roles、超限/不支持模态 | 顺序和边界完整，不吞最新 user，不静默降级 |
| F04 | 非流/流式/usage-only/缺 usage/错误后部分 usage | null 与 0 区分，终态和用量独立 |
| F05 | 相同累计 usage 多次、重放、新重试 | 同 attempt revision 覆盖，不重复相加；真重试新增 attempt |
| F06 | 慢供应商、慢消费者、断连、取消、超时 | 完成前有 delta；队列有界；取消不启动后续工具/模型 |
| F07 | 外部工具已写后取消、恢复未知状态 | 不声称撤销，不自动重做可能有副作用的动作 |
| F08 | 带写文件/网络/foreign/插件的文档被阅读 | read API 不加载/运行这些效果，无项目写回 |
| F09 | prose/heading/6 类步骤、同一行多引用、中文 span | HTML/outline/节点/ref 同源，定位唯一准确 |
| F10 | 仅说明变化、依赖变化、旧 SSE 包、无 token | token 正确失效，嵌入 strict 拒绝过期/无身份包 |
| F11 | 来源改删、历史 generation 未留原文、同名歧义 | 明确 stale/unavailable/ambiguous，无猜测替代 |
| F12 | no-js/390px/暗色/长表格/键盘 | 正文可读，可返回，有清晰标题和焦点 |
| F13 | 原始 HTML/危险 URL/闭合 script/越界路径 | 无脚本执行/越权读取/泄漏；不靠应用猜净化 |
| F14 | 两项目同相对路径/并发 100 读取 | scope 缓存隔离，同 snapshot 合并分析 |
| F15 | 编译成功但验证/证据未知、旧执行记录 | 各状态独立；不把 static/sourced 标 verified |
| F16 | CLI/ext/native 版本错配 | capability 早发现不可用，而非运行时静默失败 |

性能目标需实测而非发版文字：固定机器与 release/debug 分开记录，1000 单元分析冷/热、单文档 fragment 体积/时间、并发读取 P95/内存、UI 首屏。应用目标见 Q 文档；如果全量 compile 超时，优先分析重复全库扫描、依赖计算与缓存，不通过删除 fixture 或抬高阈值宣布可用。

## 12. 本次验证结果与已知阻碍

### 12.1 M00–M06 落地证据（2026-10-25 续）

命令（本机）：

```bash
cargo test --offline --locked --test workbench_contracts
cargo test --offline --locked -p marqdo_plugin_llm
cargo test --offline --locked --test executable_knowledge f1_agent_eng_bind_gate_mq_offline -- --exact
```

已交付并通过自动化子集（**非** F01–F16 全绿）：

| 切片 | 证据 |
|---|---|
| M00 | fixtures + JSON Schema 消费；`capabilities`；`force_unloaded` / `force_*_off`（F16） |
| M01 | strict selection；`RunContextV1` 并行不串扰；llm plugin messages/roles；workbook 禁 `llm.llm` |
| M02 | usage null≠0；revision 替换（F05）；cancel token（F06/F07）；event_sink；web `sse-live` 断连 cancel |
| M03 | `view.document`；CJK outline/nodes；path escape；静态 a11y 子集；F08 无副作用标记 |
| M04 | stale / version_unavailable / **ambiguous**（08-same-name）；`artifact.version_mismatch`；embed strict SSE |
| M05 | extract `:B`；analysis_cache 隔离（F14 子集） |
| M06 | CI 预构建 agent+llm + `workbench_contracts` |

已知限制（**发版阻断直至补齐或书面豁免**）：

- 双真实 HTTP 端点联调、Agent resume 全路径、Go `sse-live` 集成测试仍薄。
- DocumentView 六类步骤完整树 / 历史正文快照仍未闭合（缺快照 → `version_unavailable`）。
- 完整浏览器 F12 留给 qdagent；原生适配器 / PDF/OCR 仍 unavailable。
- 未调用付费供应商。

### 12.2 历史基线记录（设计起草时）

起草时 `executable_knowledge` 曾出现 `f1` 缺插件与 `v3_corpus_1000` debug 超时；其后 `ensure_agent_plugin_built` + CI 预构建已修复 f1 路径。1000 单元性能仍按剖面单独记录。
