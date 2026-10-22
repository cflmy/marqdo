# Marqdo 开发文档

本目录存放各阶段设计、选型与路线图。

## 目录约定

| 路径 | 用途 |
|------|------|
| [`research/`](research/) | 外部标准与调研 |
| [`design/`](design/) | 语言设计（语法映射等） |
| [`adr/`](adr/) | 架构决策记录 |
| [`roadmap/`](roadmap/) | 实现路线与里程碑 |

## 当前文档

| 文档 | 摘要 |
|------|------|
| [design/markdown-mapping.md](design/markdown-mapping.md) | **v0.2 语法宪法（现行运行时）** |
| [design/markdown-mapping-v0.3.md](design/markdown-mapping-v0.3.md) | **v0.3 草案：叙述统一 · 粗体代码 · 斜体返回 · 形参体推断** |
| [adr/0005-prose-unified-markup.md](adr/0005-prose-unified-markup.md) | **Proposed：采纳 mapping v0.3** |
| [roadmap/markup-v0.3.md](roadmap/markup-v0.3.md) | **Markup v0.3 实现路线（M0–M5）** |
| [design/objects.md](design/objects.md) | **`#` 对象 / `##` 函数与方法** |
| [design/ai-skill.md](design/ai-skill.md) | **AI 接入 Skill**（[`skills/marqdo/`](../skills/marqdo/)） |
| [design/code-vs-comment.md](design/code-vs-comment.md) | 叙述 / 可执行 / 外联 |
| [design/return-hr-and-code-surface.md](design/return-hr-and-code-surface.md) | 语句/返回/输出决议 |
| [design/generated-yaml-manifest.md](design/generated-yaml-manifest.md) | 生成式 OKF 清单（早期方向） |
| [design/okf.md](design/okf.md) | **Marqdo OKF 实现规范**（catalog v0 + 任务知识包） |
| [design/engineering-knowledge.md](design/engineering-knowledge.md) | **Engineering Knowledge Compiler（EKC）**：L0–L4、reuse、EFI |
| [design/catalog-cli.md](design/catalog-cli.md) | **`marqdo catalog` / `sync` OKF 清单命令** |
| [design/tech-stack.md](design/tech-stack.md) | 实现栈对比 |
| [adr/0001-implementation-language.md](adr/0001-implementation-language.md) | **Accepted：Rust 参考解释器；不用 Flex/Bison** |
| [design/dependencies.md](design/dependencies.md) | 依赖详单（无 Flex/Bison） |
| [design/keywords.md](design/keywords.md) | **英文最小关键字 / 内置函数** |
| [design/keywords-i18n.md](design/keywords-i18n.md) | **内核中英双名（无需导入；少而精）** |
| [design/module-namespace.md](design/module-namespace.md) | **模块命名空间：导入绑定库名、`库.成员` / `use`（M1+M2 已落地）** |
| [design/stdlib.md](design/stdlib.md) | **标准库：内置 + `lib/` + 错误约定** |
| [design/stdlib-i18n.md](design/stdlib-i18n.md) | **标准库中英命名：靠库文件名区分，无 lang 开关** |
| [design/stdlib-modules.md](design/stdlib-modules.md) | **L1：文件/系统/时间/JSON/网络/数学/外联/插件** |
| [design/stdlib-mid.md](design/stdlib-mid.md) | **中层 Mid M1–M6（已落地）** |
| [design/stdlib-mid2.md](design/stdlib-mid2.md) | **中层 Mid2：datetime / url / toml / html / fs++** |
| [design/stdlib-datetime.md](design/stdlib-datetime.md) | **`lib/datetime`（Mid2 M7）** |
| [design/stdlib-url.md](design/stdlib-url.md) | **`lib/url`（Mid2 M7）** |
| [design/stdlib-toml.md](design/stdlib-toml.md) | **`lib/toml`（Mid2 M8）** |
| [design/stdlib-html.md](design/stdlib-html.md) | **`lib/html`（Mid2 M8）** |
| [design/stdlib-fs-mid2.md](design/stdlib-fs-mid2.md) | **`lib/fs` Mid2 M9 加厚** |
| [design/stdlib-stats-log.md](design/stdlib-stats-log.md) | **`lib/stats` + log.fields（Mid2 M10）** |
| [research/stdlib-gaps-after-mid.md](research/stdlib-gaps-after-mid.md) | **Mid 之后缺口盘点**（对照 Python/Go） |
| [roadmap/stdlib-mid.md](roadmap/stdlib-mid.md) | **Mid M1–M6 路线（done）** |
| [roadmap/stdlib-mid2.md](roadmap/stdlib-mid2.md) | **Mid2 M7–M10 路线（规划）** |
| [design/stdlib-math.md](design/stdlib-math.md) | **数学库（数值/随机/作图/轻量求解）** |
| [design/stdlib-foreign.md](design/stdlib-foreign.md) | **外联胶水（\`\`\`lang 具名块）— v1 已落地** |
| [design/foreign-kernel.md](design/foreign-kernel.md) | **外联内核：持久内核 / JSON 桥 / 状态桥 / 安全** |
| [design/notebook-session.md](design/notebook-session.md) | **文档会话：cell / kernel / step / journal / 状态面板** |
| [design/notebook-syntax.md](design/notebook-syntax.md) | **文档原生 notebook 语法：内核表 / 会话表** |
| [research/foreign-notebook-decisions.md](research/foreign-notebook-decisions.md) | **外联 / notebook 决策记录（逐问逐答）** |
| [roadmap/foreign-v2.md](roadmap/foreign-v2.md) | **外联内核 v2 路线（F0–F4）** |
| [roadmap/notebook-session.md](roadmap/notebook-session.md) | **文档会话路线（D0–D5）** |
| [design/ext-abi.md](design/ext-abi.md) | **原生插件 C ABI v1**（`lib/plugin`） |
| [design/ext-llm.md](design/ext-llm.md) | **官方扩展 `ext/llm`（OpenAI 兼容）** |
| [design/ext-agent.md](design/ext-agent.md) | **官方扩展 `ext/agent`（文档驱动：单步 / 多步工作簿 + 自写回；废弃 thin TOOL 循环）** |
| [design/ext-agent-plan.md](design/ext-agent-plan.md) | **`## 多步` / `## plan` 锁定设计**（工作簿文件、subtask 执行、修订循环、`step` 默认写回） |
| [design/ext-cli.md](design/ext-cli.md) | **`marqdo ext list/add/remove` 官方扩展安装器** |
| [design/ext-cdn.md](design/ext-cdn.md) | **扩展 CDN（R2 / ext.marqdo.com）与分开发版** |
| [design/ubuntu-ppa.md](design/ubuntu-ppa.md) | **Ubuntu PPA 打包与上传** |
| [design/ext-web.md](design/ext-web.md) | **官方扩展 `ext/web`（页面/库表/绑定/后台；ABI + CLI；设计锁定）** |
| [design/ext-web-go-rewrite.md](design/ext-web-go-rewrite.md) | **`plugins/web` Go 全量重写**（多语言 ABI · 网页制备完备 · anlian 验收） |
| [adr/0004-web-plugin-go.md](adr/0004-web-plugin-go.md) | **Accepted：web 原生插件实现语言为 Go** |
| [research/ext-web-customization-limits.md](research/ext-web-customization-limits.md) | **`ext/web` 定制限制踩坑盘点**（路由/壳 CSS/鉴权/样式/部署） |
| [design/ext-web-customization/](design/ext-web-customization/) | **对应解决设计**（C0–C4：前缀可让出、shell_css、gate、布局…） |
| [design/web-assets-and-images.md](design/web-assets-and-images.md) | **W8：favicon / Head 资源表 / 图片装配**（一次补齐） |
| [design/web-asgi-servers-and-marqdo.md](design/web-asgi-servers-and-marqdo.md) | **Daphne/Uvicorn 等 vs Marqdo**：不能直接托管；反代 + axum `listen` |
| [design/call-arguments.md](design/call-arguments.md) | **调用：具名 + 位置实参** |
| [design/examples-and-tests.md](design/examples-and-tests.md) | **tests/ 金样例目录布局** |
| [design/view.md](design/view.md) | **`marqdo view` / `view output` 黑白极简 + 静态文档** |
| [design/user-site.md](design/user-site.md) | **用户站：`public/` → `gh-pages`** |
| [design/view-debug.md](design/view-debug.md) | **`marqdo debug` 独立调试页** |
| [design/vscode-extension.md](design/vscode-extension.md) | **VS Code 扩展：分支 `vscode-extension`（main 不跟踪源码）** |
| [design/vscode-extension-commit.md](design/vscode-extension-commit.md) | **扩展提交约定（禁止在 main 提交 `vscode-extension/`）** |
| [design/bytecode.md](design/bytecode.md) | **M5 字节码 / 双后端** |
| [design/pipeline-debug.md](design/pipeline-debug.md) | **流水线 `--dump-*` 可视调试** |
| [dev-setup.md](dev-setup.md) | 开发环境配置（Rust M0） |
| [roadmap/interpreter.md](roadmap/interpreter.md) | 正经解释器路线图 |
| [roadmap/next-phase.md](roadmap/next-phase.md) | **下一阶段：input / 诊断 / 标准库 / view 调试** |
| [roadmap/okf-near-match.md](roadmap/okf-near-match.md) | **规划：OKF 相近任务命中**（精确 sig 之外的可选软复用） |
| [roadmap/agent-streaming.md](roadmap/agent-streaming.md) | **流式输出**（SSE + `complete`/`step` stream；S0–S1 已落地） |
| [roadmap/object-inheritance.md](roadmap/object-inheritance.md) | **对象继承**（`# 子类 = > 父类`；I0–I2 已落地） |
| [roadmap/tables-maps-footnotes.md](roadmap/tables-maps-footnotes.md) | **表格语义**（竖表 / 横表字典 / 脚注取元 / 行向记录 / **单元格表达式 T5**） |
| [design/table-cell-expressions.md](design/table-cell-expressions.md) | **表数据单元格 = 表达式**（同调用实参值语法；Accepted） |
| [roadmap/browser-wasm.md](roadmap/browser-wasm.md) | **浏览器 WASM 路线 C（C0–C5 Completed）** |
| [roadmap/browser-wasm-d.md](roadmap/browser-wasm-d.md) | **路线 D：作者零 JS 前端（Completed）** |
| [roadmap/browser-wasm-e.md](roadmap/browser-wasm-e.md) | **路线 E：前端能力补齐（Completed）** |
| [roadmap/browser-wasm-f.md](roadmap/browser-wasm-f.md) | **路线 F：媒体/文件/Canvas 等（Completed，F1–F5；v0.3.3）** |
| [design/browser-marqdo-wasm.md](design/browser-marqdo-wasm.md) | **浏览器 Marqdo（WASM）设计** |
| [roadmap/ext-quantum.md](roadmap/ext-quantum.md) | **规划：`ext/quantum` 量子模拟**（基础门·电路·ABI·`ext add`） |
| [research/okf-and-marqdo.md](research/okf-and-marqdo.md) | OKF 调研 |
| [research/agent-frameworks-and-marqdo.md](research/agent-frameworks-and-marqdo.md) | 主流智能体框架 vs Marqdo 优势 |
| [research/agent-framework-gaps-after-a4.md](research/agent-framework-gaps-after-a4.md) | **`ext/agent` A0–A4 之后缺口盘点** |
| [research/agent-framework-2026-09.md](research/agent-framework-2026-09.md) | **2026-09 刷新：宪法优先 Wave B** |
| [research/ai-agent-sandbox.md](research/ai-agent-sandbox.md) | **AI 扩展沙盒调研**：Wasm / OS / CoW / symlink 对比与 Marqdo 分层推荐 |
| [design/ext-ai-sandbox.md](design/ext-ai-sandbox.md) | **`ext/ai/sandbox` 设计锁定**：CoW · 能力表 · agent ABI（非 stdlib） |
| [roadmap/ext-ai-sandbox.md](roadmap/ext-ai-sandbox.md) | **`ext/ai/sandbox` 实现里程碑** S0–S4 |
| [research/ai-agent-llm-optimize-2026-10.md](research/ai-agent-llm-optimize-2026-10.md) | **2026-10 调研**：代码即文档×OKF×沙盒下的 Agent/LLM 优化与问题分析 |
| [design/ext-ai-optimize.md](design/ext-ai-optimize.md) | **Accepted**：Agent/LLM 优化 O1–O5 locked |
| [roadmap/ext-ai-optimize.md](roadmap/ext-ai-optimize.md) | **O1–O5 实现路线**（沙盒接入→证明晋升→路由/成本→lint→隔离） |
| [research/ext-hosting-gaps.md](research/ext-hosting-gaps.md) | **宿主集成缺口**：同域 SSE 中继 · HTTP→`##` · MCP Server · 副端口 · 跨模块 db/corpus |
| [design/ext-hosting-fill.md](design/ext-hosting-fill.md) | **宿主缺口补齐设计**（`ext/web` proxy/invoke · `ext/ai` mcp_server） |
| [roadmap/ext-hosting.md](roadmap/ext-hosting.md) | **宿主集成 H0–H4**（规划） |
| [design/occlusion-tags-and-stubs.md](design/occlusion-tags-and-stubs.md) | **遮蔽描述与桩**：描述自由文本 + 内置词检索（非语法）· 内部语法定稿 · 债务可结算 |
| [design/occlusion-syntax.md](design/occlusion-syntax.md) | **遮蔽语法总纲（实施依据）**：`[描述](实现)` 的目的/语义/新鲜度校验 + M0–M8 计划 |
| [design/label-freshness-and-repair.md](design/label-freshness-and-repair.md) | 标签过期检测：L1 引用一致性 · 指纹台账 · LLM 复核 · MLSP 修复回路 |
| [design/warning-fatigue-control.md](design/warning-fatigue-control.md) | 防警告疲劳：stale 是变更报告而非扫描报告 · 分级/聚合/会话末汇总 |
| [design/staleness-providers.md](design/staleness-providers.md) | 变更检测不依赖 git：T0/T1 零依赖 + T2/T3 可选 provider |
| [design/t1-session-staleness.md](design/t1-session-staleness.md) | 会话内陈旧检测：为何模型在结构上收不到重复警告 |
| [tutorial/15-常见陷阱.md](tutorial/15-常见陷阱.md) | **使用向陷阱手册**：行首数字、跨函数作用域、空格函数名、导入路径、`table.put` 非原地（1.5.2 实测） |
| [research/qdagent-refactor-defects-2026-10.md](research/qdagent-refactor-defects-2026-10.md) | **qdagent 重构期缺陷实录 D1–D7**（沙箱 profile 不生效 · 全角 CALL 静默丢失 …） |
| [roadmap/ext-agent-optimize.md](roadmap/ext-agent-optimize.md) | **`ext/agent` A0–A4 done · Wave B Active** · 后续优化见 [ext-ai-optimize](roadmap/ext-ai-optimize.md) O 波次 |

金样例与集成测试：[`tests/`](../tests/)。面向访客的可执行文档：[`public/`](../public/)。Spike 存档：[`spike/`](../spike/)。

## 写作原则

1. 先宪法与路线图，再实现。  
2. 金样例不过 = 未完成。  
3. 示例统一 `.mq.md`。
