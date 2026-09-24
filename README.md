# Marqdo

**代码即文档。文档即知识库。知识即可执行。**

Marqdo 把 Markdown **标记当作编程语法**：同一个 `.mq.md` 文件，既是给人读的文稿，也是给机器跑的程序。写文档的过程就是在写可执行逻辑——大型项目里，说明、示例与实现不再分叉成三份真相。它不是「又一个 Markdown 编程语言」，而是**文档原生的编程运行时（Document-native runtime）**——一门关于**可执行知识（Executable Knowledge）**的语言。

欢迎访问 [marqdo 官方网站](https://www.marqdo.com/) 阅读更多特性与可执行文档。

语法宪法：[doc/design/markdown-mapping.md](doc/design/markdown-mapping.md) · 对象：[objects.md](doc/design/objects.md) · 调用实参：[call-arguments.md](doc/design/call-arguments.md) · 用户站：[user-site.md](doc/design/user-site.md) · 浏览：[view.md](doc/design/view.md) · 调试：[view-debug.md](doc/design/view-debug.md) · OKF 清单：[catalog-cli.md](doc/design/catalog-cli.md) · VS Code 扩展：[vscode-extension.md](doc/design/vscode-extension.md)（分支 **`vscode-extension`**；提交约定 [vscode-extension-commit.md](doc/design/vscode-extension-commit.md)） · **AI Skill**：[skills/marqdo/](skills/marqdo/)（说明 [ai-skill.md](doc/design/ai-skill.md)） · 官方扩展：[ext-llm.md](doc/design/ext-llm.md)（`ext/llm`）· [ext-agent.md](doc/design/ext-agent.md)（智能体开发框架）· [ext-cli.md](doc/design/ext-cli.md)（`marqdo ext list/add/remove`） · 原生插件 ABI：[ext-abi.md](doc/design/ext-abi.md)（`lib/plugin`） · 金样例：[tests/](tests/) · 用户文档：[public/](public/) · 变更：[CHANGELOG.md](CHANGELOG.md)

---

## 核心优势：可执行知识（Executable Knowledge）

> **Knowledge That Can Run —— 知识即可执行。**
> Marqdo 不是「用 Markdown 写程序」这么简单：它让一个 `.mq.md` 文档**同时**是文稿、知识与程序——同一份结构，同时服务于人、解释器、工具与 AI。

```text
         .mq.md（同一份文件）
        ┌─────────┼─────────┐
       文稿      知识       程序
        └─────────┼─────────┘
                  ▼
           Marqdo 运行时
                  ▼
        可执行知识（Executable Knowledge）
```

| 真正的核心优势 | 说明 |
|------|------|
| **知识-代码一致性**<br>Knowledge-Code Coherence | 说明、数据、示例与实现**同一份源文件**：文档里的表格就是程序的数据，文档里的调用就是真实执行。「文档过期」不再是悄悄漂移，而是可被机器发现的**知识不一致**——知识陈述本身可以被测试。 |
| **默认安全，AI 友好** | 无标记文字默认是叙述（注释），只有显式标记才进入执行。AI 自由发挥**不会污染**文档与代码的边界——这比「自然语言编程」安全得多，是 AI 原生语言的重要前提。 |
| **multi-view artifact**<br>`.mq.md` 双后缀即开放接口 | `.mq` 告诉解释器「可执行」，`.md` 告诉整个 Markdown 生态「仍是文档」。GitHub 直接读、VS Code 直接改、搜索引擎直接索引、LLM 直接理解、Marqdo 直接运行。**Marqdo 不占有知识，只是给 Markdown 增加执行语义。** |
| **表格即数据结构** | 程序的数据结构直接就是文档的数据结构：单列竖表 → `List`；横表（1 数据行）→ `Map`；横表（多数据行）→ `Map` of `List`（列向聚合）；首列写 `@` / `行` / `row` → 行向记录 `List` of `Map`。取元 `[键](集合)` 也是 Markdown 链接形。这是 literate programming 的下一步。 |
| **语义干净** | **返回是架构，打印只是函数**：`*…*` 改变控制流，`print` 只是副作用函数；分支与循环长在 Markdown 结构上（`1.` `2.` 递增 = 分支，`-` = 循环），不是所有列表都被粗暴当成 if。 |
| **AI 的可靠上下文与长期记忆** | 一个 `.mq.md` 同时是 documentation + schema + code + examples + knowledge + executable behavior；AI 不必再拼装 README + OpenAPI + JSON Schema + prompt。整个工程天然就是 AI 的**世界模型**——比 RAG 更接近 Executable RAG。 |
| **完整工程，不是玩具** | Rust 正经解释器（词法 / 语法 / 语义 / 树遍历 + 字节码后端）、stdlib、模块与对象、debugger、viewer、catalog、WASM、原生插件 ABI、VS Code 扩展、agent / LLM / MCP。 |

---

## 愿景与期许

不把「AI 能写 Marqdo」当终点，而是 **「AI 能可靠地生成可验证的 Marqdo」**：

1. **AI 可验证语言**：`LLM → Marqdo AST → 语义检查 → 结构化诊断 → LLM 修复 → 合格程序`。让类型与语义检查成为 AI 编程闭环里的强约束，堵住「语义看似合理、类型实则不对」的生成。
2. **Marqdo Schema / Language Server for AI**：机器可读的语言 Schema（`get_syntax` / `validate` / `repair` …），AI 不必读千行文档猜语法，而是 discover → construct → validate → execute → inspect。
3. **程序即 AI 长期记忆**：`.mq.md` 工程 = 可执行知识库 = AI 的世界模型（Executable Knowledge Representation）。
4. **可执行知识图**：`.mq.md` 之间的 Markdown 链接天然长成文档图 + 知识图 + 程序图（Executable Knowledge Graph）。
5. **知识可测试**：文档中的示例表自动执行；知识与执行结果不一致会被机器报告——Knowledge-Code Coherence 从理念变为日常。
6. **可靠性基准**：让同一个 AI 分别用 Python / TypeScript / Marqdo 完成真实任务，测量生成长度、语法 / 语义错误率、修复轮次、token 消耗与人类阅读 / 修改成本——用实验数据证明「文档型语法降低 LLM 生成熵」。

---

## 为什么重要

| 痛点 | Marqdo 的答案 |
|------|----------------|
| 文档过期、示例跑不通 | **一份源文件**：叙述、结构与执行结果同源 |
| 知识散落在 Wiki / 注释 / 脚本里 | **`.mq.md` 即知识单元**，可浏览、可运行、可检索 |
| 人写给人看，AI / 工具另起一套 | **人机同读同写同构**：标记语言对人类友好，对解释器确定 |
| 大仓里「文档站」与「代码仓」脱节 | `view` 文档站 + `debug` 调试面 + `catalog` 清单，同一棵树长大 |

这不是「再做一个 Markdown 渲染器」，而是：**把文档升格为可解释的程序与可生长的知识库**，让协作在大型项目里依然站得住。

---

## 语法速览（v0.3）

| 标记 | 含义 |
|------|------|
| 无标记行 | 叙述（注释）；行内 `` `名` `` 可声明 |
| `**…**` | **代码**（赋值 / 调用 / 表达式） |
| `*…*` | **返回值**（整行、`返回*n*`、或表达式形） |
| `****` / 整行 `**` / `*None*` | **空返回**（`None`），并结束本函数体 |
| `> print …` 等 | 输出等副作用 = **普通函数**，非架构标记 |
| `#` | 对象 / 类型（构造体；`# main` = 入口） |
| `##` … | 函数 / 方法（按标题深度嵌套） |
| `` > `obj`.method … `` | 方法调用（`obj` 为带 `_type` 的 map） |
| 叙述 `` `名` `` / `` `名`=默认 `` | 形参由体推断（`` + `名` `` 仍可用） |
| `1.` … | 体内 `1.` `2.` … = 分支（`N. *` = else） |
| `-` | 循环（`` - `条件` `` 或 `` - [项](集合) ``） |
| 表格 | 集合（几何即类型，见下）；取元优先 `[键](集合)` |
| 单独一行 `---` / `***` | **叙述分隔**（跳过；不是函数收束） |

```markdown
# main

> 问候 谁=World

## 问候

向`谁`问好。

**print text="Hello " + 谁**
*None*

## 加一

对于输入变量`n`，执行**n=n+1**接着返回*n*。
```

**表格几何即类型**：单列竖表 → `List`；横表（1 数据行）→ `Map`；横表（多数据行）→ `Map` of `List`（列向聚合）；首列写 `@` / `行` / `row` → 行向记录 `List` of `Map`。详见 [tables-maps-footnotes.md](doc/roadmap/tables-maps-footnotes.md)。

完整约定：[markdown-mapping-v0.3.md](doc/design/markdown-mapping-v0.3.md)。

---

## 理念

1. **叙述默认安全**（无标记 = 注释）。  
2. **返回是架构；打印只是函数。**  
3. **代码即文档即知识库**：文稿可执行，执行可回看结构，结构可导航与调试。  
4. **对齐 [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)（Open Knowledge Format）方向**：清单由工具从 `.mq.md` **自动生成**，不是手填配置——见下节与 [catalog-cli.md](doc/design/catalog-cli.md)。  
5. **正经解释器（Rust）**：词法/行分类 → 语法 → 语义 → 树遍历（及字节码）；**不用 Flex/Bison**。见 [路线图](doc/roadmap/interpreter.md)。  
6. **语言核心保持小**：Language（标记映射 / AST / 语义 / 类型）、Runtime（stdlib / 插件 ABI / WASM / agent）、Document（知识 + 代码 + UI + AI）三层分明——生态功能不淹没语言身份。

---

## OKF 风格清单（自动生成）

从源码派生 YAML / Markdown 知识包（勿手改生成物）：

```bash
marqdo catalog [PATH] -o OUT_DIR
marqdo sync [PATH] -o OUT_DIR          # catalog 的别名
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | 工程根或含 `.mq.md` 的目录 |
| `-o` / `--out` | `.marqdo` | 输出目录 |

产物示例：`catalog.yaml`、`index.md`、`modules/*.md`（`type: Marqdo Module` 等）。设计见 [generated-yaml-manifest.md](doc/design/generated-yaml-manifest.md) · 调研 [okf-and-marqdo.md](doc/research/okf-and-marqdo.md)。

智能体框架横向调研（LangGraph / CrewAI / 厂商 SDK 等 vs Marqdo 文档驱动优势）：[agent-frameworks-and-marqdo.md](doc/research/agent-frameworks-and-marqdo.md) · 优化路线 [ext-agent-optimize.md](doc/roadmap/ext-agent-optimize.md)。

```bash
cargo run -- catalog public -o .marqdo
# 或已安装二进制时：
marqdo catalog public -o .marqdo
```

---

## 现状（v1.1.1）

- **AI 栈（v1.1.1）**：`ext/ai/llm` 语义 `ask`/`stream`/`collect` · Artifact Metadata Binding（`${env|arg|sys|secret}`）· Agent Skill Compilation + Adaptive Routing · MCP stdio 客户端 + resume 检查点 · 可选 `plugins/llm` Transport ABI
- **AI 原生闭环（v1.1.0）**：核心表面守卫（19 构造，单边改动 CI 必红）· **渐进式契约**（文档内嵌 `参数`/`返回`/`字段` 表 + 四边界校验 + `marqdo check` 防漂移）· **MLSP for AI**（`marqdo mlsp`：`locate`/`syntax`/`validate`/`repair_targets`/`repair_apply`/`schema`，AI 按需查语法而非背诵）· **结构化诊断**（`run --json`，错误即数据）；验证见 [perf-validation-report-2026-09-23.md](doc/roadmap/perf-validation-report-2026-09-23.md)
- 映射与解释器：**v0.3 语法宪法**（`**` 代码 / `*` 返回 / 叙述声明）；Phase I 树遍历 + 字节码后端；金样例在 `tests/` 与 `tests/markup-v03/`  
- **对象**：`#` = 类型/构造，`##`+ = 函数/方法；见 [objects.md](doc/design/objects.md)  
- **`marqdo view`**：文档浏览器（Structure + 函数大纲/搜索 + Execution + Variables 浮窗）  
- **`marqdo debug`**：独立调试页（断点 / 单步 / locals；默认端口 7430；页面 favicon / 品牌使用官方 Logo）  
- **`marqdo catalog` / `sync`**：OKF 风格 YAML + 模块概念页  
- **`marqdo version --check`**：与 GitHub 最新 release 对比  
- 标准库：**内置于二进制**（v0.1.2+）；磁盘 `lib/` 或 `MARQDO_LIB` 可覆盖。模块含文本、表、**浏览器效应**、文件、系统、时间、JSON、网络、数学、外联、插件、**自写回**、**子任务**；**中层 Mid M1–M6** + **Mid2 M7–M10**（datetime/url/toml/html/fs++/stats/log.fields 等，见 [stdlib-mid.md](doc/design/stdlib-mid.md) · [stdlib-mid2.md](doc/design/stdlib-mid2.md)）  
- **官方扩展库 `ext/`**（**非** stdlib，本版收口）：
  - **`web`**：W0–W7 + P3 + **W8**；**定制 C0–C4**；**宿主 H1–H2**（`proxy`/`invoke`，见 [ext-hosting.md](doc/roadmap/ext-hosting.md)）；**表驱动引言**（`compose_intro` / `引言装配`）；原生层默认 **Go `libweb`**（[ADR 0004](doc/adr/0004-web-plugin-go.md)，`scripts/build-web-plugin.sh`）；示例 [web-site](examples/web-site/) · [web-site-zh](examples/web-site-zh/) · [marqdo-blog](examples/marqdo-blog/) · [anlian-mq](examples/anlian-mq/)；生产路径见 [web-asgi-servers-and-marqdo.md](doc/design/web-asgi-servers-and-marqdo.md)
  - **`quantum`**：Q0–Q7 + Q8a/b 主题 SVG；示例 [quantum-entanglement](examples/quantum-entanglement/)
  - **`linalg`**：L0–L6 + 公式文档面（中缀 / `declare` / 展示化简）；示例 [linalg-transpose](examples/linalg-transpose/) · [linalg-svd](examples/linalg-svd/) · [linalg-least-squares](examples/linalg-least-squares/)
  - **`agent`**：A1–A4 + **MCP Server/Client stdio** + Skill Compilation + Adaptive Routing + resume；宿主缺口见 [ext-hosting.md](doc/roadmap/ext-hosting.md) · [agent-framework-gaps-after-a4.md](doc/research/agent-framework-gaps-after-a4.md)
  - **`llm`**：语义 `ask`/`stream`/`collect`（OpenAI/Ollama；可选原生 `plugins/llm`）
  - 安装：`marqdo ext list` / `add …` / `remove`（[ext-cli.md](doc/design/ext-cli.md)；默认 `~/.marqdo/ext`）。原生插件先 `cargo build -p marqdo_plugin_*` 再 `ext add`
- **原生插件 ABI**：[`include/marqdo_abi.h`](include/marqdo_abi.h) · [ext-abi.md](doc/design/ext-abi.md)；`plugins/{demo,agent,web,quantum,linalg,llm}`（`web` 为 Go；其余为 Rust）  
- **用户静态站**：`public/` → `view output` → CI 发布 [gh-pages](https://cflmy.github.io/marqdo/)  
- **VS Code 扩展**：分支 **`vscode-extension`**（`main` 不跟踪源码；见 [vscode-extension-commit.md](doc/design/vscode-extension-commit.md)）；Release 附带 `.vsix`  
- **浏览器 Marqdo（WASM）**：`marqdo wasm build` + 官方 bridge 自启（作者零业务 JS；桥内可含列表/路由/storage/ws/文件/Canvas/音频/Observer/拖放）；`lib/browser` + GFM；`web.client_embed`；示例 [browser-hello](examples/browser-hello/) · [browser-app](examples/browser-app/) · [browser-media](examples/browser-media/) · [web-client-site](examples/web-client-site/)（[ADR 0002](doc/adr/0002-browser-marqdo-wasm.md) · [D](doc/roadmap/browser-wasm-d.md) · [E](doc/roadmap/browser-wasm-e.md) · [F](doc/roadmap/browser-wasm-f.md)）
- 选型：[ADR 0001 — Rust](doc/adr/0001-implementation-language.md) · [ADR 0002 — 浏览器 WASM](doc/adr/0002-browser-marqdo-wasm.md)（C0–C5 完结，见 [roadmap/browser-wasm.md](doc/roadmap/browser-wasm.md)）· [ADR 0003 — 异步效应](doc/adr/0003-browser-async-effects.md)

### 如何使用最新 Marqdo（v1.1.1）

**安装解释器（任选其一）**

| 方式 | 适用 | 命令 / 链接 |
|------|------|-------------|
| **Ubuntu PPA** | Ubuntu 24.04+（`noble` 等） | `sudo add-apt-repository ppa:cflmy/marqdo && sudo apt update && sudo apt install marqdo` |
| **GitHub Releases** | Windows / Linux 预编译 zip | https://github.com/cflmy/marqdo/releases |
| **源码** | 需本机 Rust | 见下方 `git clone` + `cargo build --release` |

**安装官方扩展**（与 CLI 可分开发版）：`marqdo ext add …` 默认下载顺序为 **CDN** [`https://ext.marqdo.com`](https://ext.marqdo.com) → GitHub Releases → [`proxy.cflmy.top`](https://proxy.cflmy.top) 镜像。

```bash
# 1) 源码安装（跟 tag 或 main）
git clone https://github.com/cflmy/marqdo.git && cd marqdo
git checkout v1.1.1   # 或留在 main
cargo build --release
export PATH="$PWD/target/release:$PATH"
marqdo version
marqdo version --check

# 2) 跑解释器 / 文档站 / catalog
marqdo run tests/structure/hello.mq.md
marqdo run tests/structure/hello.mq.md --backend bytecode
marqdo view public --no-open
marqdo debug public --no-open
marqdo catalog public -o .marqdo

# 2b) AI 原生闭环（v1.1.0+）：契约检查 / 结构化诊断 / MLSP 查询
marqdo check tests/contracts/ok_contract.mq.md        # 静态契约防漂移
marqdo run tests/structure/hello.mq.md --json # 结构化诊断（错误即数据）
printf '{"id":1,"method":"syntax","params":{"query":"返回"}}\n' | marqdo mlsp

# 3) 安装官方扩展（Release / CDN 用户：无需本机 Rust/Go）
marqdo ext add web      # 或：网页 — 自动下载 L1 + 预编译 native（Go libweb）
marqdo ext add agent    # 或：智能体
marqdo ext add quantum  # 或：量子
marqdo ext add linalg   # 或：线性代数
marqdo ext add llm      # 或：大模型（.mq.md + 可选 native llm）
# 开发者：cargo build --release -p marqdo_plugin_{agent,quantum,linalg,llm}
#         bash ./scripts/build-web-plugin.sh   # 需 Go + cgo → libweb
# 离线：解压 Windows zip（已含 ext/native）或手动放下 native zip；MARQDO_EXT_NO_DOWNLOAD=1

# 4) 动态站示例（扩展装好后）
marqdo run examples/web-site-zh/index.mq.md   # 表驱动引言 Demo
marqdo run examples/marqdo-blog/index.mq.md
marqdo run examples/anlian-mq/index.mq.md   # W-G14 验收站
# 浏览器打开终端打印的 listen 地址；/favicon.ico 与 logo 装配见 W8

# 5) 浏览器 WASM（可选）
marqdo wasm build
# → dist/wasm/ … 见 examples/browser-hello/

# 6) Releases：Windows exe/zip/vsix；Linux CLI zip + native `.so` zip
#    https://github.com/cflmy/marqdo/releases/tag/v1.1.1
```

开发期也可用 `cargo run -- …` 代替已安装的 `marqdo`：

```bash
cargo run -- run tests/structure/hello.mq.md
cargo run -- view public --no-open
cargo run -- view output public -o public
powershell -File ./scripts/build-public.ps1
```

**发布包**：GitHub Releases 的单独二进制 **已内置**官方 `lib/`。Ubuntu 用户优先 **PPA** 装 CLI。**扩展库**用 `marqdo ext add …`（CDN → GitHub）；设计见 [ext-cdn.md](doc/design/ext-cdn.md) · [ubuntu-ppa.md](doc/design/ubuntu-ppa.md) · [ext-cli.md](doc/design/ext-cli.md)。

文档：用户站 [public/](public/) · 设计 [doc/](doc/) · OKF / catalog [catalog-cli.md](doc/design/catalog-cli.md) · 调试 [view-debug.md](doc/design/view-debug.md) · 变更 [CHANGELOG.md](CHANGELOG.md)
---

## 命名

**Marqdo** = Marq（Markdown）+ do（执行）

## 许可

[Apache License 2.0](LICENSE)
