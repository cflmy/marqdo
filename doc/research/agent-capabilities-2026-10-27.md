# 智能体能力调研与求道差距复核（2026-10-27）

| 项目 | 说明 |
|---|---|
| 状态 | Research / 设计输入；不是功能交付声明 |
| 调研日期 | 2026-10-27，Asia/Shanghai |
| 源码基线 | Marqdo `16f26d7d60ce9fbc62feb0e8e83f93951f7b53fe`；qdagent `68b757e02ebfb64833b5262b0c8a683512581e8b` |
| 框架设计 | [Agent 恢复、工具与资源协议](../design/agent-reliability-tools-resources-2026-10-27.md) |
| 产品设计 | [求道智能体能力与预览体验](../../../qdagent/doc/design/agent-capabilities-and-preview-2026-10-27.md)（同级仓库） |

## 1. 方法与结论边界

先以 `git pull --ff-only` 同步两仓，再追踪真实调用链与前端入口；对比公开官方文档，不以营销介绍、工具数量或旧设计中的 done 标签代替实现证据。本次是静态源码复核与官方资料研究，没有启动部署实例，没有调用付费模型，没有进行竞品账号实测。因此，下表“源码确认”仅代表代码路径可见，“设计判断”与“待实测”不能写成已复现或已交付。

主流能力的共同方向是：可恢复的执行、结构化工具与资源、按需上下文、可追溯结果、独立产物展示。不同产品和模型支持范围不同；这里不是市场占有率调查，也不声称所有智能体都具备全部能力。求道应保留文档原生、EKC、验证后沉淀的定位，吸收机制，不引入第二套 Agent Runtime。

## 2. 官方资料与可借鉴机制

以下页面均在调研日实际打开；MCP 特意引用固定 `2025-06-18` 版本，作为兼容基线，不宣称它是最新协议。厂商专有能力必须探测，不能因为端点兼容 `/v1` 就认为全部可用。

| 编号 | 官方来源 | 已核实内容 | 对求道的设计启示 |
|---|---|---|---|
| S01 | [OpenAI Agents：集成与可观测性](https://developers.openai.com/api/docs/guides/agents/integrations-observability) | MCP 接入与运行、模型、工具等 tracing | 统一 call/attempt/span 关联；产品只显示阶段摘要，详细事件可展开 |
| S02 | [OpenAI：Memory 与 Compaction](https://developers.openai.com/cookbook/examples/agents_sdk/building_reliable_agents_memory_compaction) | 区分当前任务上下文压缩、跨运行记忆、可审阅产物 | 历史不要无限拼接；完整产物不等于聊天全文 |
| S03 | [LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) | checkpointer 保存 thread 状态，store 支撑跨 thread 数据 | 恢复执行状态和用户长期知识是两种生命周期 |
| S04 | [LangGraph Interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) | 恢复时节点代码可能重新执行 | 检查点不能单独保证写操作只执行一次，必须记录效应与幂等键 |
| S05 | [Claude Tool search](https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool) | 工具定义按需进入模型上下文 | 工具多不代表每轮注入全部定义；检索后再校验权限 |
| S06 | [Claude Programmatic tool calling](https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling) | 在执行环境中编排工具、筛选结果 | 大批量只读处理可减少模型往返；后置于基础工具可靠性 |
| S07 | [Deep Agents overview](https://docs.langchain.com/oss/python/deepagents/overview) | 文件系统、上下文管理、子任务与可选规划能力 | 规划应按任务启用；简单问答不强制复杂工作流 |
| S08 | [Gemini Tools](https://ai.google.dev/gemini-api/docs/tools) | 检索、URL、文件搜索、代码执行等工具面 | 覆盖“找资料—读内容—计算—形成产物”，不能只有工程检索 |
| S09 | [Gemini File search](https://ai.google.dev/gemini-api/docs/file-search) | 文件检索与引用信息 | 命中要能定位原文，检索摘要不能代替源文件版本 |
| S10 | [MCP Tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) | schema、结构化结果、资源链接；annotations 有信任边界 | 工具结果包含短摘要和资源引用；注解不是权限凭证 |
| S11 | [MCP Resources](https://modelcontextprotocol.io/specification/2025-06-18/server/resources) | URI 资源、读取和应用决定的上下文注入 | “预览”“加入上下文”“执行”必须是独立动作 |
| S12 | [Claude Artifacts](https://support.claude.com/en/articles/17153992-what-are-artifacts-and-how-do-i-use-them) | 产物在会话旁独立打开，可继续查看与编辑 | 右栏应承载正文；不应只提供离开当前会话的入口 |
| S13 | [Mozilla PDF.js](https://mozilla.github.io/pdf.js/) | Web PDF 解析和渲染 | PDF 视觉预览和 OCR/知识提取分开交付 |
| S14 | [DOMPurify 官方仓库](https://github.com/cure53/DOMPurify) | DOM 清洗、标签/属性配置；插入前清洗 | 替换前端正则“清洗器”，配合同源资源代理和 CSP |

上述设计启示是本次推导，相关性能收益、成本下降比例不照搬厂商基准。工程路线不绑定某个最新模型或套餐。

## 3. 源码事实清单

下列 qdagent 路径相对该仓库；Marqdo 路径相对本仓库。基线 commit 已固定，后续实现需按符号再定位。

| 编号 | 当前实现证据 | 结论与限制 |
|---|---|---|
| F01 | qdagent `agent/qdagent.mq.md::契约提示`、`ask` | ASK 和 BUILD 都要求完整六段式 `.mq.md`，包含 main、验证和元数据；普通回答变长有明确结构性诱因。不能断言这是唯一原因 |
| F02 | 同文件 `ask` 的门禁修复分支 | `r2` 送入第二次门禁后，成功分支仍 `result = raw`；可能展示修复前内容，设计必须绑定同一 accepted candidate |
| F03 | 同文件 `ask` 返回表 | `gated_error` 路径仍可能保留 `status=ok` 且返回 `ok=True`；必须明确业务失败与输送成功的不同含义 |
| F04 | 同文件 `eng_tools` | 默认白名单是 `tool_find/reuse/bind/verify/summarize` 五个工程工具；不是整个框架只有五个工具，也不代表已覆盖网页、笔记、文件编辑 |
| F05 | `ext/ai/agent.mq.md` | 有 step/plan/run、READ、CALL、观察压缩、工作簿和恢复接口；应扩展这些原语，不另造循环 |
| F06 | `plugins/agent/src/resume.rs::resume_save` | JSON 直接 `fs::write`；现有恢复接口不等于具备原子检查点、工具效应对账及任意步骤自动续跑 |
| F07 | `src/runtime/`、`src/workbench/capabilities.rs` | 已有隔离执行、取消、事件回放与读通道；这一轮是 Agent 级恢复补足，不重新规划整个进程池 |
| F08 | qdagent `session-store.ts::resumeTurn` | 终态 completed/failed/interrupted 直接返回；已有事件续订，但终态消息补齐与中断后的执行恢复需分别处理 |
| F09 | qdagent `DesktopFrame.tsx::RightNotesPanel` | 实际只有“查看全部笔记”导航至 `/notes`，没有侧栏笔记正文 |
| F10 | 同文件 `RightFilesPanel`、`WorkspacePage.tsx::openFile` | 文件点击已调 `openTarget`；右栏取首个 workspace，根目录最多 40 个非目录项。不能笼统写“所有按钮都跳转” |
| F11 | `DesktopFrame.tsx` 的 reader/rightbar 条件 | `DocumentReader` 与右侧 tab 容器互斥，且使用两套开关/宽度状态；不是统一的资源浏览面板 |
| F12 | qdagent `NotesPage.tsx` detail | 剥离 frontmatter 后以 `whiteSpace: pre-wrap` 显示文本；没有 Markdown/Marqdo 正文结构渲染 |
| F13 | qdagent `DocumentReader.tsx` | 已有 DocumentView、引用版本传递、请求代数防迟到覆盖；`profile_item` target 未进入读取分支，格式不是通用资源分派 |
| F14 | 同文件 `sanitizeHtmlFragment` | 主要靠正则移除 script/事件/javascript URL；不能当作完整 HTML 白名单安全边界。未在本次做漏洞利用实验 |
| F15 | `src/workbench/document_view.rs` | 有只读文档视图，注释与实现明确避开执行路径；已解析语义应继续复用 |
| F16 | `src/workbench/capabilities.rs` | text/markdown 提取可用，pdf/docx/ocr 标 unavailable；提取能力与浏览器 PDF 预览能力须分别探测 |
| F17 | qdagent `lib/web_search.mq.md` | 既有网页搜索桥使用 legacy 脚本和固定 `last_web_search.json`；存在并发覆盖设计风险，不能直接作为新工具通道 |
| F18 | `plugins/agent/src/mcp_client.rs` | 已有 stdio connect/list/call/close；不能把本轮描述为“从零添加 MCP”，远程连接和资源能力需另验 |
| F19 | qdagent `compose_messages` / `ask` | 对话按角色标记拼成字符串注入 standing；会话语义、上下文预算和外部内容信任层级仍需强化 |
| F20 | qdagent `lib/workbench.mq.md::documents_read` | 临时文件名以秒和环境 RANDOM 构造，RANDOM 缺失又退到秒；设计仍需真正唯一临时文件。workspace 缺省 `.` 不应作为新授权资源接口规则 |

这些是本轮优先级依据。10-26 release notes 所列修复作为已有基础保留，但“Implemented（本仓）”不能替代浏览器、真实 HTTP、进程中断与模型输出的联合验收。

## 4. 能力覆盖与差距优先级

| 能力 | 当前基础 | 本轮目标 | 优先级 / 归属 |
|---|---|---|---|
| 自动纠错 | 一次文档门禁修复、plan patch | 分类错误、有限修复、正确结果替换、可恢复失败 | P0 / 两仓 |
| 执行恢复 | 任务隔离、取消、事件回放、JSON checkpoint | 传输续订与执行续跑分离；效应对账 | P0 / Marqdo 原语，qdagent UX |
| 回答长度 | 六段式文档、完整历史 | 直接回答、摘要+产物、显式详述三种表达 | P0 / qdagent 策略，Marqdo 输出通道 |
| 资源阅读 | DocumentView / Locator | 笔记、工作文件、引用、产物统一入口 | P0 / 两仓 |
| 侧栏预览 | 两套状态与导航入口 | 单一右栏，点击即预览，聊天不中断 | P0 / qdagent |
| 工具使用 | 五工具默认白名单、MCP/库原语 | schema 注册、按需发现、资源读取/搜索/写入/验证 | P1 / 两仓 |
| 联网研究 | legacy 搜索脚本 | search+fetch、来源时间、限量正文、可定位引用 | P1 / 两仓 |
| 上下文与记忆 | EFI pack、profile、history | 分层上下文、预算、可编辑记忆、资源 pin | P1 / 两仓 |
| 多格式 | 文档视图、文本提取 | text/md/mq/code/json/csv/image；PDF 后续 | P0→P1 / 两仓 |
| 长任务与计划 | plan 工作簿 | 里程碑、预算、暂停/继续、产物进度 | P1 / 两仓 |
| 多智能体与浏览器 | 不作为当前完成前提 | 有界委派、隔离执行、受控浏览器工具 | P2 / 框架主导 |
| 可观测评测 | usage、事件与既有测试 | 工具/恢复指标、实际任务语料、回归门禁 | P0 起持续 / 两仓 |

## 5. 采用与暂缓的决策

采用：有界失败恢复；工具结果摘要+资源引用；结构化参数；步骤效应账本；产物与聊天投影分离；资源预览与“加入上下文”分离；用户可检查的引用；按需工具发现。原有源码、知识、工作簿与验证证据仍是权威，衍生视图可重建。

暂缓：全自动跨站电脑操作、任意外部 MCP 自动信任、默认多智能体协作、所有 Office 格式保真编辑、每次问答都规划/运行代码。它们不能先于当前恢复、工具授权和预览闭环。

不采用：把长回答直接 substring 截断、把失败原文当最终成功、前端断流即重发任务、把所有文件当 HTML、用网页内容改写 system 指令、把预览等同于执行可执行文档。

## 6. 必须补做的动态验证

1. 在真实聊天中验证六段式输出与门禁失败/修复的最终消息和持久产物是否一致。
2. 丢弃最后一帧、断网、刷新、worker 崩溃分别注入，确认事件重放不重做工具副作用。
3. 使用真实 HTTP 入口验证右栏笔记、嵌套文件、历史引用、跨 workspace 资源；不只调用兄弟辅助函数。
4. 用恶意/畸形 HTML、路径与符号链接、过大文件验证预览隔离；no-exec 通过哨兵副作用测试。
5. 对无凭证模型、text-only provider、不可用转换器做明确降级；记录版本与环境，禁止以 mock 成功声称上线成功。

后续设计文档中的时间、轮数、字符数、成功率阈值均为拟定验收目标，需用该语料校准。
