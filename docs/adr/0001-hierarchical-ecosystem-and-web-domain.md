# ADR 0001：分层生态与 Web 原型设计领域实施规范

- 状态：已采纳为实施输入（2026-11-26）；规范正文已合并进现行文档，本文件仅作历史实施记录。
- 现行权威：`docs/ARCHITECTURE.md`、`docs/DEVELOPMENT.md`、`docs/MIGRATION.md`、`docs/REPOSITORY-TASKS.md`、`AGENTS.md`、`components.toml` 与各仓库的合同文档。
- 本文不是第二份现行规范；两者不一致时，以现行文档与 Git tree/catalog 为准。
- 关联：Web 领域唯一作者合同为 `marqdo-ext-web` 的 `docs/WEB-DESIGN.md`；本 ADR 中的实施工单编号（ECO/STD/WEB）是计划编号，远程工单号以各仓库 Issue 为准。

---

# Marqdo 分层生态与 Web 原型设计领域：架构及开发实施规范

**日期：** 2026-11-26  
**适用对象：** Marqdo 生态维护者、领域维护者、组件开发者、人类及 AI Coding Agent。  
**性质：** 待维护者批准的实施规范；其中“必须”是验收条件，“建议”是默认实现选择，“可选”不是首版发布门槛。  
**目标仓库：** `marqdo`、`marqdo-stdlib`、`marqdo-ext-web`、其功能子仓库，以及确有公共边界缺口的现有组件。  
**阅读前提：** 无需任何此前讨论。本文同时给出动机、设计合同、实现步骤、工单、示例和开发结束条件。

## 0. 执行摘要

Marqdo 以 Markdown 为人类和 AI 共同阅读的知识及程序载体，以独立语言核心提供计算语义。随着生态扩张，必须把领域实现、组件治理和生态集成分离，避免一次局部缺陷导致整个生态重构。与此同时，Web 领域应当发挥 Marqdo 在结构化设计、精确引用、组件分工和知识复用方面的优势，而不是重新实现成熟前端框架。

本规范确立两项互相独立的决策：

1. **仓库治理从一级扁平子模块升级为有限深度的分层生态。** 主仓库管理领域与独立组件；领域聚合仓库管理本领域的组件组合；功能仓库实现功能、负责缺陷。最多两级 gitlink（主仓库→领域→功能），除非新的 ADR 有充分证据。目录层级不等于运行时依赖层级。
2. **Web 定位为文档原生、可交互的前端设计原型及工程交付。** GFM 表格描述组件间的关系，而不枚举全部 DOM；普通 Markdown 描述内容与意图，链接表达精确的组件引用和设计依赖，有限场景表达交互原型。成熟框架负责生产级前端实现。HTTP、数据库、认证、浏览器运行时等仍是独立的通用能力，不为原型工具强制安装。

这两项工作可以并行，但 Web 领域正式接入主仓库需要递归治理基础已经完成。任何子项目都应能不检出主仓库而独立开发。本文不要求所有候选领域、标准库和扩展库一起发版。当前目标是建立真实、可终止的实施闭环。

**最重要的排他原则：** 一个功能一个权威实现；一个组件一个治理归属；一个公共边界一套现行合同；一个发布一组精确的源码和制品身份。主仓库和领域聚合仓库不实现子组件业务。组件之间只通过声明的公开接口协作，不依赖彼此的物理 checkout 位置。

## 1. 审计基线与事实边界

### 1.1 现有仓库与提交

本次依据可访问的 GitHub 镜像及其固定提交，不将镜像状态当作自建远端最新状态。实际实施时必须先到 `git.cflmy.cn/cflmy` 盘点真实仓库、分支、标签和 CI；`ci.cflmy.cn` 是现有工程文档声明的 CI 权威。

| 项目 | 本次固定依据 | 意义 |
| --- | --- | --- |
| [生态主仓库](https://github.com/cflmy/marqdo/tree/451000e9ad5d7ae23164d6fcbde3d3e6222a6b8c) | `451000e9ad5d7ae23164d6fcbde3d3e6222a6b8c` | 已转换为生态主仓库，当前是直接子模块模式；标准库及扩展仍为 planned |
| [主仓库锁定的 CLI](https://github.com/cflmy/marqdo-cli/tree/452e9a32bd9e40dba427d4180691c58bab74449e) | `452e9a32bd9e40dba427d4180691c58bab74449e` | RC.4 发行基线；不是 CLI 镜像分支 HEAD |
| [CLI 镜像 master](https://github.com/cflmy/marqdo-cli/tree/4bbefc92449ee480c8a2c89b0308feb4cda8d0e7) | `4bbefc92449ee480c8a2c89b0308feb4cda8d0e7` | 审计 Rust 依赖与标准库打包代码；不可替代主仓库锁定制品 |
| [旧版实现](https://github.com/cflmy/marqdo/tree/1c55baabd570e18b9da8f9057cef2f848e176690) | `1c55baabd570e18b9da8f9057cef2f848e176690` | 历史 Web Artifact、布局和 Browser WASM 设计参考；不恢复旧语法及 ABI |

主仓库 Git tree 实际含如下六个 gitlink：

| 路径 | 锁定提交 |
| --- | --- |
| `cli` | `452e9a32bd9e40dba427d4180691c58bab74449e` |
| `tests/conformance` | `015ff8ac15417b7ebbdb31ce2be2dab5362a1eba` |
| `tools/knowledge` | `d8fa201b2859e922afb797c1b87cd7638382b12b` |
| `tools/lint` | `111e085f2445a9ba97808a15ad5a50645b8b2007` |
| `tools/prove` | `d8c340b6e2e436329c299db5cfc545f14a18fdfe` |
| `tools/view` | `04f3a0e9e5dcbf81166c54a993a44c27fa8ed642` |

这里只核实了主仓库记录的 gitlink，不证明所有自建子仓库提交仍可获取或功能已通过验收。不得重建现有组件来满足目录命名。官网、编辑器、qdagent 等继续为独立产品；本工作不更改它们的开发体系。

### 1.2 必须处理的真实架构缺口

| ID | 当前代码证据 | 所需变更 | 所有者 |
| --- | --- | --- | --- |
| G01 | `scripts/check-layout.py` 的 `RELEASE_ROLES` 无 domain；路径按 kind 固定为一级 | 引入领域治理角色，递归验证每一级直属清单、gitlink、远端 | 主仓库 |
| G02 | 同脚本只校验单一 `components.toml` 与 `git ls-tree` | 检出后读取各聚合仓库固定提交的 catalog；全局 ID 唯一及依赖 DAG 校验 | 主仓库 |
| G03 | `.woodpecker/eco.yml` 使用 `git submodule update --init` | 受信远端、逐层验证、递归 checkout 与候选身份记录 | 主仓库 |
| G04 | `scripts/integration.sh` 写死六个 active 路径，新增组件即报 UNVERIFIED | 保留现有测试的效果，迁移成声明驱动的组件验收入口与领域集成步骤 | 主仓库/各组件 |
| G05 | 同脚本从 `tools/view/scripts/frozen-beta.sh` 取 CLI 归档 | 由生态构建输入统一选定并验证 CLI 归档；不得让任一消费者成为核心发行权威 | 主仓库 |
| G06 | CLI `tools/pack-stdlib.py`、`tools/release/package.py`、`crates/cli/tests/stdlib_product.rs` 依赖仓库内 `stdlib/` | 以明确的外部 Bundle 输入替换源码树依赖，先完成发布和测试解耦再删旧源码 | CLI/Stdlib |
| WG01 | RC.4 Analysis 包含完整 source、document AST、units、anchors、functions、references，但只覆盖静态加载集合 | Web Design 显式追踪设计链接，针对入口调用公共 Analysis；不把 link 当 import | Design |
| WG02 | 旧 Web 与 Browser 依赖历史语法及大插件；新 Runtime 仍依赖带原生平台依赖的 Host | 本期不迁移 WASM、不重新实现语言，不要求新版支持旧 plugin 回调 | Web/Browser |

根仓库原规范明确 `stdlib/` 与 `extensions/` 为分类目录、只允许直接子模块；本文有意替换这一项决策。其他关于公共协议、标准库/扩展库分离、工单责任、独立构建、发布证据的现行纪律继续有效。落地时更新 `docs/ARCHITECTURE.md`、`docs/DEVELOPMENT.md`、`docs/MIGRATION.md`、`AGENTS.md` 和脚本，不并列维护两个“当前架构”。

### 1.3 旧 Web 与实验证据的正确使用

历史参考：[ADR 0007](https://github.com/cflmy/marqdo/blob/1c55baabd570e18b9da8f9057cef2f848e176690/doc/adr/0007-web-document-endpoint.md)、[Web Artifact](https://github.com/cflmy/marqdo/blob/1c55baabd570e18b9da8f9057cef2f848e176690/doc/design/ext-web-artifact.md)、[递归布局](https://github.com/cflmy/marqdo/blob/1c55baabd570e18b9da8f9057cef2f848e176690/doc/design/ext-web-layout-paradigm.md)、[Browser WASM](https://github.com/cflmy/marqdo/blob/1c55baabd570e18b9da8f9057cef2f848e176690/doc/design/browser-marqdo-wasm.md)。保留“先划分布局、再递归细化至可读内容”的思想，不保留历史 Artifact metadata、旧式动态插件加载、旧 Web 五列表格的兼容层。

此前的独立科研任务管理台原型（`marqdo-web-poc.zip`）真实测试了 Markdown 表格提取、结构化 DOM、CSS 响应式、交互、SQLite 持久化；记录 9/9 通过。但业务逻辑与动态列表由手写 JS 替代，服务端为 Python，浏览器网络通过测试适配器；没有真实 Marqdo 执行或 WASM。它是风险探索材料，不是正式 Web 产品源码，也不能作为本规范任何生产级验收通过的证据。可以重用测试案例和设计素材，禁止直接把该专用宿主当通用渲染器发布。

## 2. 架构原则：区分三种层级

### 2.1 语言与产品层级（不改变）

| 层 | 权威所有者 | 允许 | 禁止 |
| --- | --- | --- | --- |
| L0 语言核心 | CLI | Markdown 映射、AST、IR、Value、Memory、求值 | Web/AI/数据库专属关键字 |
| L1 平台能力 | CLI Capability Host | 权限、预算、通用资源/扩展机制 | 网页布局和业务协议 |
| L2 标准库 | Stdlib 各功能仓库 | 基础发行默认安装的通用低层算法及薄封装 | 依赖领域扩展或大型专业运行环境 |
| L3 扩展 | 各领域功能仓库 | 可选专业算法、协议和资源后端 | 修改或复制语言内核 |
| L4 应用 | 用户项目 | 使用已发布能力实现具体任务 | 作为默认基础发行依赖 |
| Tool（旁路） | 独立工具 | 通过公开 CLI 协议处理源码、分析及制品 | 伪装成 Runtime 插件，反向侵入核心 |

仓库增加不改变层数。新增 domain 是**工程治理类型**而非语言层或运行协议。Web 原型产品归入 Web 领域，但它的交互式预览命令属于 Tool 安装机制；同一领域未来仍可有普通 Marqdo Package、Plugin 等运行时组件。

### 2.2 三张关系图

- **治理树**：主仓库→领域聚合仓库→功能仓库，每个组件只有一个父级。以 `.gitmodules` 与 gitlink 表示源码归属及身份。
- **构建/制品依赖 DAG**：由真实 package/Cargo/Tool/Bundle 依赖决定，可以跨治理树；`requires` 表达直接构建或制品依赖。禁止循环；聚合节点不是供程序 import 的包。
- **协同验收关系**：`checks_with` 表达跨组件测试需求，不是安装依赖，允许互相测试；不能拿它规避真正的 `requires` 边。测试调度不得因测试关系互相等待形成死锁。

例如 AI、MCP、Web 服务可以使用同一个 HTTP 实现。HTTP 归 Net 领域，但不需要把 Net 聚合仓库安装到应用中。Web 原型首版不需要 HTTP 运行时扩展。组件关系不能由目录路径推导，也不能把兄弟仓库的源码 `../../...` 当公开依赖。

## 3. 目标治理树和仓库准入

### 3.1 目录规划

```text
marqdo/                         # 生态主仓库（唯一根）
├── cli/                        # 已有 marqdo-cli
├── tools/                      # 分类目录，不额外建聚合仓
│   ├── knowledge/              # 已有 gitlink
│   ├── prove/
│   ├── view/
│   └── lint/
├── stdlib/                     # marqdo-stdlib，领域聚合，产出 Bundle
│   ├── core/                   # marqdo-std-core
│   ├── collections/            # marqdo-std-collections
│   ├── text/                   # marqdo-std-text
│   ├── encoding/               # marqdo-std-encoding
│   ├── data/                   # marqdo-std-data
│   ├── io/                     # marqdo-std-io
│   ├── fs/                     # marqdo-std-fs
│   ├── system/                 # marqdo-std-system
│   └── net/                    # marqdo-std-net
├── extensions/                 # 分类目录
│   ├── web/                    # marqdo-ext-web（领域聚合）
│   │   ├── design/             # marqdo-web-design（首批）
│   │   ├── preview/            # marqdo-web-preview（首批）
│   │   └── components/         # marqdo-web-components（候选）
│   ├── net/                    # marqdo-ext-net（候选聚合）
│   │   ├── http/                # 现有或新建的唯一 HTTP 仓库
│   │   └── websocket/
│   ├── compute/                # marqdo-ext-compute（候选聚合）
│   │   ├── math/
│   │   ├── linalg/
│   │   └── quantum/
│   ├── data/                   # marqdo-ext-data（候选聚合）
│   │   ├── database/
│   │   ├── storage/
│   │   └── cache/
│   ├── security/               # marqdo-ext-security（候选聚合）
│   │   ├── crypto/
│   │   └── auth/
│   ├── ai/                     # 独立组件
│   ├── mcp/
│   ├── browser/
│   ├── document/
│   ├── archive/
│   ├── regex/
│   ├── datetime/
│   └── engineering/
├── tests/conformance/          # 已有独立 conformance gitlink
├── tests/integration/          # 根仓库自有集成测试
├── docs/
├── scripts/
├── components.toml
├── .gitmodules
└── AGENTS.md
```

实际仓库名以自建权威远端盘点为准；已有仓库直接复用，不能因为命名不符创建重复实现。上图不表示所有候选仓库需要立刻建立。首批只激活 Stdlib 聚合（随现有库源码拆分推进）和 Web 聚合（Design、Preview）；其他候选领域在至少两个真实组件、共同测试或发行边界成立时转换。计划中的组件保持 planned，已有仓库可通过 gitlink 固定真实开发提交，激活后才计入完成率。规划中的领域及其后代分别登记在各自父级的 catalog 中。激活聚合时将原直属 planned 记录迁至该聚合自己的 catalog，并在同一根候选清理旧记录；尚未激活的其他领域可继续保留现有直属 planned 记录。工具目录暂不聚合，已有四个 Tool 保持原路径。

### 3.2 聚合仓库的准入及内容

聚合仓库可拥有 `components.toml`、`.gitmodules`、公共领域规范、跨组件集成测试、综合示例、发布装配脚本、ADR、维护指南。禁止在聚合仓库建立生产级 renderer、解析器、数据库驱动或其他子组件实现；禁止把子仓库源码复制进普通目录。允许测试夹具、小型集成脚本，不允许以“脚本”名义迁入业务逻辑。

功能仓库拆分须至少有一种独立价值：独立维护责任、显著不同的依赖/安全/平台边界、独立版本与制品、可重复的独立验收。若两个仓库几乎每个 PR 都必须一起改动且接口尚不稳定，应先收敛合同或合并仓库；不能因为代码行数多而无限拆分。新增三级 gitlink 需要生态 ADR 和真实跨仓维护收益，默认拒绝。

### 3.3 各领域规划与排他边界

| 领域 / 功能 | 应拥有 | 不应拥有 |
| --- | --- | --- |
| Stdlib/Core | Result、基础数值 abs/min/max 等 | 专业计算、外部服务 |
| Collections | List/Map/Table 值操作、选择过滤排序组合 | JSON/CSV/GFM 序列化 |
| Text | Unicode、字节、UTF-8 | HTML DOM、HTTP 请求 |
| Encoding | Base64、Base32、Hex | 密码算法 |
| Data（stdlib） | JSON、CSV、TOML、GFM 数据交换 | SQL、数据库驱动、网页布局 |
| IO | Console/Stream/格式化、官方 print alias | 文件系统策略、业务日志平台 |
| FS | 文件、目录、路径、授权资源封装 | 文档结构修改、工程批量补丁业务 |
| System | 环境、进程、UTC、时钟、随机及基础 CLI/日志封装 | 身份认证、完整时区库 |
| Net（stdlib） | TCP/UDP/TLS 字节流封装 | HTTP、URL、WebSocket 协议 |
| Ext-Net/HTTP | URL、HTTP 客户端/服务端协议、SSE、Cookie、Multipart | Web 页面模型、模型业务协议 |
| Ext-Net/WebSocket | 握手、帧、连接生命周期 | 应用实时协作状态 |
| Ext-Compute/Math | 数值、统计、符号数学 | 内核 math builtin、矩阵领域重复实现 |
| Linalg | 复数、矩阵、张量、分解 | 量子电路业务 |
| Quantum | 量子态、电路、模拟与领域模型 | 复制矩阵内核、通用前端框架 |
| Ext-Data/Database | 参数化查询、事务、驱动 | Web 后台管理页面 |
| Storage | 本地/对象存储接口 | 文件系统原语、Web 表单 |
| Cache | TTL、缓存及失效 | 应用会话授权政策 |
| Ext-Security/Crypto | 成熟密码学、哈希、签名封装 | 自制不安全加密、业务权限模型 |
| Auth（Security） | OIDC、会话、RBAC、多租户应用授权 | 替代 CLI Capability Host |
| AI | 模型访问、流式结果、工具调用数据 | 全量 Agent 调度、HTTP 实现 |
| MCP | MCP 发现、调用及传输 | Agent 框架 |
| Browser | 将来可选的真实浏览器执行宿主及平台桥 | 本期 Web 原型必需依赖、第二套解释器 |
| Document/Engineering | 文档显式编辑 / 工程操作 | 静态分析或编辑器全栈平台 |
| Archive/Regex/Datetime | 专业归档、完整正则、高级时区 | 重复宿主基础原语 |
| Web 领域 | 设计模型、快速预览、交付和组件复用 | 生产级框架、HTTP/DB/认证协议、AI 调度 |

上述是规划边界，不是开发完成承诺。独立 HTTP 等可能已有实现或仓库，激活前核对实际依赖。旧标准库 `net/http.mq.md`、`regex.mq.md`、`compress.mq.md` 的迁移必须有真实替代实现与验收才退出原归属；不得先删造成产品回归，也不得长期维护两个官方实现。无历史 API 兼容承诺不意味着可以静默丢失功能。

## 4. 递归治理清单与 Git 迁移

### 4.1 保留一套 `components.toml` 规则

不新增 registry、workspace lock 或数据库。每个拥有子模块的聚合仓库只登记直接子组件；普通功能仓库不需要另建治理清单。主仓库在检出固定子模块后递归展平为内存图或临时报告。Gitlink 是提交身份唯一权威，`.gitmodules` 是权威远端唯一记录；catalog 不重复 SHA、远端与实际语言包锁。计划依赖不得冒充实际依赖。

在原有字段 `id/path/kind/state/owner/scope/requires/checks_with/release/entry` 基础上，仅新增 `kind="domain"`（聚合角色），不另增 library 等类型，保持原有功能类别。聚合 `requires=[]`，自身没有运行依赖；其组合输入由 active 子树决定，测试和装配依赖由领域集成入口显式消费。`release` 对 domain 采用 `none`（普通领域）或 `base`（Stdlib Bundle 组装）；不能由此将 Web 工具变成默认安装。功能组件仍使用 core/stdlib/extension/tool/test 对应 base/optional/none 的既有规则。`entry` 对 active 仍必填，planned 禁止伪造入口。域内路径为相对本级的单一合法目录名；顶级保留当前直接组件的路径规则，加 `stdlib` 和 `extensions/<domain>`。Web 领域 ID `web`，子组件 `web-design` 等全局唯一。

**根清单示意（待真实仓库接入后才写 active）：**

```toml
[[component]]
id = "web"
path = "extensions/web"
kind = "domain"
state = "active"
owner = "cflmy"
scope = "网页原型设计领域治理与组合验收"
requires = []
checks_with = []
release = "none"
entry = "scripts/integration.sh"
```

**`marqdo-ext-web/components.toml` 示例：**

```toml
[[component]]
id = "web-design"
path = "design"
kind = "extension"
state = "active"
owner = "cflmy"
scope = "设计文档解释、校验、组件图及交付"
requires = ["cli"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"

[[component]]
id = "web-preview"
path = "preview"
kind = "tool"
state = "active"
owner = "cflmy"
scope = "通用原型渲染及 web 命令发行"
requires = ["cli", "web-design"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"

[[component]]
id = "web-components"
path = "components"
kind = "extension"
state = "planned"
owner = "cflmy"
scope = "可组合的设计文档与原型组件模板"
requires = ["web-design"]
checks_with = ["web-preview"]
release = "optional"
```

这是一项需要明确更新旧校验器的决策：独立 Tool 原本只能依赖 core；新的规则允许 Tool 依赖**显式声明的独立领域库制品**，例如 Preview 链接 Design，而不能依赖其他 Tool 的私有实现或主仓库路径。Design 的类别是治理意义的 extension，首版交付 Rust library，不假称它是可由 Runtime import 的 `.mq.md` 包。后续若提供可执行 Marqdo 库，使用现有 Package 合同，不新增一种接入协议。Tool 和 extension 分类依实际公开入口与制品身份决定，不由父目录决定。`requires` 不允许指向 domain（领域测试入口消费组装制品例外在 CI 调度层表达，而非包依赖）。

`stdlib` 聚合自身需要 CLI 进行 Bundle 测试，但构建时只消费指定 CLI 制品；以 active stdlib 子组件 `requires=["cli", ...]` 和显式领域验收输入表达。不能因此让 CLI 编译反向依赖 stdlib。基础发行需要 Bundle，不等于 CLI 语言编译依赖 Bundle。

### 4.2 校验算法（主仓库 G01/G02）

1. 读取指定候选 commit 的根 `components.toml` 和 `.gitmodules`，不要从可能已修改的工作区文件取代它。拒绝重复 TOML 字段、未知类型、非法路径、绝对路径、`..`、大小写或 NFC 冲突、同级路径重叠。
2. 使用 `git ls-tree -rz --full-tree REV` 获取当前层 gitlink 和普通文件。active 必须有 gitlink；已发布的 planned 组件可有 gitlink，且父仓库只记录它的固定提交。聚合层普通文件仅限 docs/examples/scripts/tests/治理配置等白名单，功能层内部不套聚合白名单。
3. 先验证当前层 `.gitmodules` 的直属路径和远端，再检出确切 SHA（逐层进行）。允许的 URL 由维护者批准的自建 HTTPS 远端及明确的迁移映射决定；拒绝 `file://`、任意外部 URL、submodule update command、相对跳转、重复物理仓库及未批准别名。不要在未检查嵌套 `.gitmodules` 前使用携带凭证的盲目递归拉取。
4. 每个 domain 检出后在其固定 commit 重复步骤 1–3，深度最多 2；功能叶子不得再注册未批准 gitlink。检查 `HEAD` 与父 gitlink 一致、可获取、工作树无修改、入口为真实常规文件、无符号链接逃逸。缺失时明确 FAIL/UNVERIFIED，不静默略过。
5. 展平全局 ID、父级、逻辑路径、权威 URL、SHA、类型、state。全局 ID 与远端唯一；解析 `requires` / `checks_with` 至全局 ID；active 不依赖 planned；禁止 stdlib 依赖 extension/tool，禁止 core 构建依赖其他功能，禁止直接依赖 domain，禁止循环 DAG。检查实际 package imports/lock/Cargo/Tool requires 是否与治理声明相符；无法静态确定时列出验证项而非假装完整。
6. 生成临时 `integration-plan.json` 或内存对象用于测试调度（派生、非新权威、非运行协议、不提交作为第二份 catalog）。记录每个递归提交及制品 digest。校验器输出结构化失败位置与责任仓库。

分两阶段检查：无需网络的根树检查、检出后的全树检查。命令 `python3 scripts/check-layout.py [REV] --checkouts` 保留原入口并扩展语义，不再维持平行旧检查器。日常开发可使用 `git submodule status --recursive`；经审查的已知树可用 `git submodule update --init --recursive`；CI 实现须遵循上文远端检查。

### 4.3 迁移步骤（先建立候选，再原子切换）

1. 在自建远端核对现有仓库、可达性、分支保护和权限。保留根旧历史、标签及现有有效 gitlink。建立专门 `architecture/domains` 候选分支，不强推或删除历史。
2. 在主仓库先开发递归 checker 和临时 Git 仓库测试，现有六组件继续验证。该分支最终只保留一套新规则；没有 active domain 时新 checker 同样支持合法的直属功能组件，这不是历史协议兼容。
3. 建立 `marqdo-ext-web`（或复用已存在的 Web 权威仓库）；只有实际 Design/Preview 仓库具备可用入口与候选提交后才注册 active。维护者确认每个子库远端已推送，领域子模块锁定 SHA，根可先固定 planned 组合。域内和根的联合验证通过后再激活。
4. 对已有直属组件移动，先在目标聚合仓库新增同一个远端同一个 SHA 的 gitlink（在领域候选上），验证该聚合仓库，根候选移除旧路径 gitlink 与旧清单记录、加入聚合 gitlink；一次 PR 内切换，不允许集成主分支同时出现两份同一远端。旧源码、历史工单不复制，旧根快照仍可重现。
5. 仅修改本轮相关治理文件和集成脚本。不得借此重构 CLI、Knowledge、Prove、View、Lint 的功能代码。涉及其验收入口时各自发工单，配对候选提交验证。根候选失败时不晋升；回退根 gitlink 组合即可，不重写子仓历史。

示意命令（在干净候选 checkout 执行，SHA/URL 必须由盘点得到，不把占位值提交）：

```bash
# 领域仓库：先建立实际子模块并锁定已推送提交
 git submodule add https://git.cflmy.cn/cflmy/marqdo-web-design.git design
 git -C design checkout --detach "$DESIGN_SHA"
 git submodule add https://git.cflmy.cn/cflmy/marqdo-web-preview.git preview
 git -C preview checkout --detach "$PREVIEW_SHA"
 git add .gitmodules components.toml design preview
 # 通过本领域测试后 commit、push，获得 WEB_SHA

# 根候选：使用真实领域提交
 git submodule add https://git.cflmy.cn/cflmy/marqdo-ext-web.git extensions/web
 git -C extensions/web checkout --detach "$WEB_SHA"
 # 更新根 catalog，先验证嵌套 URL 再检出
 git add .gitmodules components.toml extensions/web
 # 提交候选后运行递归校验及生态 CI
```

示例未实际执行，若远端已有组件需复用其 URL；如路径已存在不得覆盖，应使用单独移动流程。严禁 `git submodule update --remote` 在发布/验收中追踪浮动分支。子模块的 detached HEAD 仅用于复现与集成；开发应在子仓库独立创建分支后推送。

## 5. 标准库聚合：保持一个默认 Bundle

标准库独立拆分仍然遵循“默认安装且通用低层”的定义。`marqdo-stdlib` 是组装仓库，九个功能仓库各自拥有唯一源码与测试。用户仍通过现行 `std:` 导入，安装包仍包含 CLI、Worker、统一 stdlib Bundle。不能为每个标准库新增一种加载协议或分发给用户九份必须安装的运行时清单。

### 5.1 组装与迁移纪律

- 功能仓库以明确的 Bundle 相对路径交付 `.mq.md` 和必要的产品测试。现行 `stdlib.json` 的内容摘要、官方 alias（print 等）及版本规则由 Stdlib 聚合唯一维护；按 CLI 现行 digest 算法生成，不自创版本。官方别名只在一处定义。
- 聚合脚本从固定 gitlink 的允许文件清单构建 staging；拒绝路径碰撞、符号链接、越界、未追踪内容。跨组件引用使用最终统一 `std:` 路径或符合统一 Bundle 的相对路径，不能依赖源码 checkout 布局。先迁移现有模块的完整功能，再补缺口。
- 当前产品文件如 `io.mq.md`、`io/console.mq.md` 在 CLI 打包脚本中被要求存在，必须由新 Bundle 提供，或在一次有意的公共标准库路径切换中同步更新加载、别名、消费者和测试。不能静默删除。
- CLI 编译不需要完整 stdlib；CLI 产品测试、stdlib 独立测试、最终发行装配需要显式固定的真实 Bundle。`MARQDO_STDLIB_DIR` 是现有运行时覆盖入口，不应把开发目录回退当发行行为。CLI 打包器接受明确的外部目录/归档输入，验证摘要与选定 Bundle 身份。具体新构建参数属于构建工具，不是 Runtime 协议。
- 先完成外部输入、完整 stdlib 产品测试与冷安装；在配对候选切换时移除 CLI 内旧权威源码，开发 fixture 只能是明确的小测试输入，不能保留整套重复库。生成文件不得当源码维护。Bundle 元数据、制品摘要、源 gitlink 记录到现有发布证据。

### 5.2 CLI 工单 STD-01（已知必要，不是 Web 新需求）

标题：`Externalize bundled stdlib build/test input without changing std: runtime contract`。责任：CLI 仓库。复现：`tools/release/package.py`、`tools/pack-stdlib.py` 和 `stdlib_product.rs` 当前直接读仓库 `stdlib/`。要求：外部 Bundle 输入明确、无隐式下载、无旧源码 fallback、同一 digest 算法、完整产品测试移交给真实 Bundle、冷安装默认发现仍正常。测试：在无仓库完整 stdlib 源码的干净构建中编译 CLI/Worker；以固定外部 Bundle 打包；篡改摘要/别名拒绝；移动安装目录后 `std:` 与官方 print 可用；产品全量测试通过；重复构建同输入得到确定身份；缺失 Bundle 必须拒绝发行。与 Stdlib 候选联调，最终无双权威。

此工单沿用此前生态迁移要求，若已存在就更新，不重复发工单。Web 首版不等待这一迁移完成，可以使用主仓库当前锁定的 RC.4 基础发行作为输入。

## 6. Web 领域产品定义与功能终点

### 6.1 Why：为什么做原型，而不是新前端框架

传统前端工程擅长复杂视觉、浏览器生态、性能优化和交付；但源码不是理想的需求/组件职责表达。人类及大模型往往要从大量实现反推产品结构。Marqdo 的价值是让设计意图、组件树、接口、状态情景、验收及可定位知识直接成为可读文档，并生成可快速体验的原型。组件可以递归拆分，确认公共接口后将独立职责交给人类或不同 Coding Agent，再由成熟框架完成工程实现。

目标是减少设计理解成本和跨组件协调成本；不是宣称减少所有前端代码，更不承诺任何设计自动生成高质量生产代码。以下边界是硬约束：

| P0 必须实现 | 明确不属于 P0 |
| --- | --- |
| 原生 Markdown 内容、GFM 组件关系表、精确链接、递归布局 | 全部 HTML 元素的通用新 DSL |
| 快速预览、常见桌面/移动布局、基础主题、组件单独预览 | CSS 替代语言、复杂动画/Canvas/WebGPU 编辑器 |
| 有限交互场景与样例数据、显式模拟状态 | 真实登录、支付、数据库写入、网络业务、任意响应式状态管理 |
| 设计检查、组件图、输入/输出事件约定与交付清单 | React/Vue/Svelte 通用编译器、框架私有生成器、双向源码同步 |
| 可供独立任务使用的精确设计切片、依赖和验收 | 内置 LLM、Agent 调度、自动发任务/提交生产代码 |
| 使用真实 CLI Analysis 作为结构事实来源 | 复制 Markdown/语言解析器、把设计链接自动变为程序调用 |

浏览器端首版不需要运行 Marqdo WASM。预览是文档设计模型的有限确定性解释，不是任意 Marqdo 程序执行。允许文档包含真实 `@exec` 纯业务函数供独立 CLI 验证和交付，但预览不调用它们。未来若真实需求需要通用计算互动，可另行设计显式受控执行边界；不得在当前首版悄悄把业务逻辑移入 JS。首版“可交互”具体指布局、导航、有限场景切换；不包括真实业务能力。此取舍确保首版有意义、可完成且不会变成完整前端框架。

### 6.2 仓库职责及装配

```text
marqdo-ext-web/                 # 聚合，不含生产实现
├── design/                    # marqdo-web-design
├── preview/                   # marqdo-web-preview
├── docs/WEB-DESIGN.md          # 唯一 Web 设计文档合同
├── docs/DEVELOPMENT.md
├── examples/                   # 完整组合案例
├── tests/integration/          # 跨组件、浏览器、交付验收
├── scripts/integration.sh
├── components.toml
└── .gitmodules
```

| 仓库 | 产物/接口 | 职责 | 禁止 |
| --- | --- | --- | --- |
| Ext-Web | 固定组合、合同、综合验收 | 定义领域作者面和设计原则；协调变更及发布 | 实现第二个 Design/Preview |
| Design | 独立 Rust library（建议 package `marqdo-web-design`） | 公共 Analysis 读取及校验；文档解释；组件/依赖图；诊断、handoff Markdown | 启动服务、渲染 DOM、读密钥、调用 LLM、复制 CLI parser |
| Preview | 独立 executable + `tool.json` | 消费 Design API，生成安全 HTML 预览及通用有限状态交互；`web` 唯一工具入口 | 自己再解释布局表、任务专用 JS、生产服务器 |
| Components（后续） | 可复用 `.mq.md` 设计模板或真实可选库 | 普通组件样式、设计样例、复用与独立验收 | 扩展 renderer 私有标签以适配单个组件 |

首批仅 Design、Preview 两个功能仓库。若组件库只有少量模板，先放领域 `examples/`（示例不是官方运行实现），或 Preview 的通用测试 fixture；只有独立维护需求和明确合同后才接入 Components。不要先创建 Handoff、Style、Layout、Animation 等一串空仓库。设计交付属于 Design，而不是另一个模型。若跨仓调用的 public Rust API 经原型证明过度频繁变更，则暂停激活两个 active 仓库，先收敛 API；不能在两仓复制解析器来规避不稳定接口。

Design 不依赖 Preview；Preview 依赖 Design 的精确发布 crate 或 Git revision（Cargo.lock 固定）；两个仓库独立 clone/build，不通过 `../design` 的永久路径依赖、不直接依赖 CLI 私有 Rust crate。组合候选允许 CI 通过明确依赖 override 验证，但最终发布必须以锁文件固定真实依赖身份，不能把测试 override 伪装成发布组合。Ext-Web 不提供运行时入口，也不需要另一份 tool.json。Preview 使用现有 Tool 合同注册一个命令：

```json
{
  "schema": "marqdo.cli-tool",
  "schema_version": 1,
  "name": "marqdo-web-preview",
  "version": "0.1.0",
  "command": "web",
  "entrypoint": "bin/marqdo-web-preview",
  "requires": {"api_revision": 1, "analysis": 3}
}
```

这是目标清单示例，版本为建议，不表示已发布。Observation 不用于首版，故不声明。未来若确有必要集成浏览器执行，必须独立评估并使用真实当前协议，不能在预览工具添加第二套解释器。

## 7. Web 作者面合同（首版唯一现行设计约定）

### 7.1 约定的范围及选择方式

采用现行 `.mq.md`、Markdown AST、`@knowledge`、`@exec`、标题、链接和 GFM 表格；不引入 Web 核心关键字、`@component`、`type:web` 语言特殊语义或额外 YAML/JSON 设计源码。以下少量中文标题/表头仅为 Web Design 对**显式选择的知识单元**的领域约定；CLI 不解释其意义。

首版 `web check ENTRY --unit ANCHOR` 明确选择一个 Knowledge H2 作为根设计组件；`--unit` 使用精确 anchor（无 `#`），不扫描整个项目寻找可能的页面。`ENTRY` 为项目内 `.mq.md`，`--root` 是显式项目范围（默认 ENTRY 所属目录；跨兄弟目录案例必须明确 root）。根可以不是首页；同一命令可预览独立子组件。所有组件目标必须是 `.mq.md` 中的 Knowledge H2，根目录之外的目标拒绝。普通文档和其它 Knowledge H2 不会自动成为组件。普通链接仍是普通链接，只有本合同指定单元格才形成设计关系。

H2 直接 H3 子节使用以下保留标题（去掉语义模式后精确匹配，区分大小写和字符，不提供同义词/中英文双份字段别名）。设计文档不需要所有章节：

| H3 | 格式 | 意义 |
| --- | --- | --- |
| `内容` | 普通 Markdown | 实际可展示内容；正文中的链接不形成组件边 |
| `组件` | 一张 GFM 表：`行 \| 槽 \| 组件 \| 宽度` | 子组件布局与精确引用 |
| `输入` | 一张 GFM 表：`名称 \| 类型 \| 说明` | 生产交付输入契约，非运行时类型系统 |
| `事件` | 一张 GFM 表：`名称 \| 数据 \| 说明` | 生产交付输出契约，非浏览器 event listener DSL |
| `场景` | 一张 GFM 表：`状态 \| 内容 \| 初始` | 有限交互的样例内容及初始状态 |
| `交互` | 一张 GFM 表：`当前 \| 事件 \| 下一状态` | 明确有限状态转换，事件列为可见按钮标签 |
| `依赖` | 一张 GFM 表：`组件 \| 原因` | 非视觉的设计依赖与交付前置条件 |
| `验收` | Markdown 列表/段落 | 可观察的验收条件，不自动解释自然语言为测试 |

H2 直接正文为职责、设计意图、视觉说明，默认显示在设计说明/交付而不直接注入页面。非保留 H3（例如 `视觉要求`、`参考资料`）是可读交付知识，不触发自动行为；不要求作者重复写一份设计清单。若意外拼错保留标题，不能模糊映射；可发普通警告，缺少有效布局/内容/场景时硬失败。所有保留节仅在直接 H3（有效 Knowledge）识别，重复、错误层级或 Exec 模式拒绝；内容下的嵌套普通标题和表格只是内容，不成为新声明。表必须由 CLI AST 识别且位于该直接节内，表头完全匹配，不能多表合并、缺列补列或按自然语言猜测。空单元格按每字段规则验证。第一版不支持表格单元格内代码求值。

一个设计组件至少有 `内容`、非空 `组件` 或 `场景` 之一。可以同时有内容及子组件（内容先显示），场景只允许在叶子组件，防止场景系统演变成整个 UI 模板语言。`输入/事件/依赖/验收` 非必填；交付时若缺失，显示“未声明”，不自动补造。

### 7.2 递归布局：表格只划分组件

`组件` 的 `行` 是从 1 开始的连续正整数；同一行的相邻记录形成横排，行之间纵排，不能跳号、倒序。`槽` 为本组件内唯一的简短 ID（`[a-z][a-z0-9_-]*`）。`组件` 必须只含一个 Markdown Link，指向设计组件的精确 anchor；不使用 Link label 推断目标。`宽度` 只支持 `auto`、`1fr` 至 `12fr` 或 `120px` 至 `1200px` 的整数尺寸；拒绝任意 CSS 字符串、计算式、负数、百分比注入。所有行以 CSS Grid 通用实现，fr 单元采用 `minmax(0, Nfr)`；auto 采用 `minmax(0, auto)`；固定 px 只在空间允许时使用，窄屏（默认 768 CSS px）按声明顺序折为单列，必要时由容器实际宽度响应式约束。单行最大 12 个槽。样式布局预览不声称像素级复现生产界面。

子组件内部用同一表格表达下一层布局。组件引用不是 HTML DOM 选择器，禁止父级渲染子级 HTML 字符串后拼接。组件定义以精确文档/Unit identity 为 key；每次出现以根实例+槽路径获得独立实例身份，允许同一组件定义被重复使用而实例状态不串扰。包含环（A→B→A）硬错误；跨组件设计依赖环不等同视觉包含环，但若作为交付前置形成循环，必须报告协调需求，不能自动宣称可并行。设计依赖不执行程序。

组件只拆到有独立职责或便于理解的层级。一个包含标题、段落、图、表格的 Markdown 区块不需要拆成几十个节点。避免将每个按钮作为单独文件。父组件负责关系、子组件负责内容与行为意图。

### 7.3 有限交互场景（唯一首版行为模型）

用有限状态情景确认流程，而不是构建通用状态框架。`场景` 表每一行定义一个 `状态`（ASCII slug，组件内唯一）、一个指向 Knowledge 单元的 Link（内容单元只消费安全普通 Markdown，不递归解释为设计组件）、一个 `初始` 值（`true` 或 `false`，恰好一个 true）。状态内容可以是 Markdown 样例表、文字、成功/失败提示、空状态等。不得借此执行链接中的 `@exec` 或任意脚本。场景内容目标不得包含有效 Exec 子树；若选定目标为组件声明，按错误报告，避免双重解释。`交互` 每行是 `(当前状态, 可见按钮标签, 下一状态)`；三列均纯文本，状态必须存在，事件标签非空且同一当前状态唯一。同一事件可在不同状态产生不同结果。每个状态应从初始状态可达，孤立状态报错；允许合理循环（例如重置），但不得自动触发无限转换。没有场景就不能有交互表；有场景而无交互仅在单状态时允许。

渲染器只按通用规则在当前状态展示对应内容和可用按钮，用户点击按钮改变该实例状态并更新内容；无网络、无定时器自动执行、无隐式业务函数。每个实例具备重置入口，刷新后返回初始状态。按钮标明“模拟”，不会执行真正的创建/提交/保存。输入/事件表记录生产组件接口，不自动从事件名生成业务行为。需要真实录入控件、验证错误、异步竞争、表单提交的设计，先用显式场景展示默认/加载/失败/完成；实际业务测试与实现交由目标框架。不要把模拟“提交成功”误读为后端成功。

### 7.4 视觉表达与 HTML 逃生口纪律

首版提供少量内置主题（light/dark）和通用 Grid 适配，保留 Markdown 原生文本、列表、表、图、链接及代码展示。复杂视觉要求以说明、参考图、设计资源、CSS/SVG 围栏或外部目标框架文件描述；代码围栏默认只显示为代码，不自动执行、不自动嵌入 CSS/SVG/JS。CSS/SVG 仍是专业表达，生产实现可原样使用或调整，不发明完整表格式 CSS。首版暂不自动应用作者任意 CSS（防止注入、外部网络请求、样式越界；不是拒绝生产 CSS）；若未来需要受控设计 CSS 资源另立领域工单，不能扩大 CLI 语义。

Raw HTML 节点以文本/明确占位显示，不渲染为活动 HTML。复杂控件（拖拽电路、图表、富文本、三维、滚动动画）以静态图片、样例数据、可见情景及验收描述表达，标记生产实现尚待开发。不能为每个控件在 renderer 写专用分支，也不能使用 raw_html/set_html 作为默认方案。若普通组件的预览必须修改宿主源码，说明通用原语设计失败，应先缩小表达目标或改进通用原语而非添加业务特例。

## 8. 完整小示例：科研工作台（待真实 CLI fixture 验证）

这个例子专门验证多文件递归组件、共享接口、两份独立开发任务及模拟状态，而不是完整科研业务。以下文件源码只使用现行 Markdown 标记；Web 表格解释是本文定义的新领域合同。它尚未经过本环境实际 CLI `check`，开发者需把每份文件加入真实自动化测试，不得把示例当作已有运行结果。

```text
research-design/
├── index.mq.md
├── components/
│   ├── navigation.mq.md
│   ├── workspace.mq.md
│   ├── statistics.mq.md
│   └── tasks.mq.md
└── shared/
    └── validation.mq.md
```

`index.mq.md`：

```markdown
# 量子科研管理平台

## @knowledge 工作台

面向研究人员确认项目进度和实验任务。左侧导航，右侧工作区。

### 组件

| 行 | 槽 | 组件 | 宽度 |
| --- | --- | --- | --- |
| 1 | navigation | [导航](components/navigation.mq.md#导航) | 220px |
| 1 | workspace | [科研工作区](components/workspace.mq.md#科研工作区) | 1fr |

### 验收

- 桌面并排，窄屏按阅读顺序重排。
- 工作区内可查看统计和任务流程。
- 预览不连接数据库，不声称真实任务已保存。
```

`components/navigation.mq.md`（`内容` 下的普通小标题使用 H4，避免 H2 结束当前设计单元）：

```markdown
# 导航设计

## @knowledge 导航

职责：帮助研究者理解可进入的功能区域。

### 内容

#### Research / QDQC

- 研究项目
- 实验任务
- 量子电路
- 实验记录
```

`components/workspace.mq.md`：

```markdown
# 工作区设计

## @knowledge 科研工作区

职责：在统一工作区中组织项目摘要与实验列表。

### 组件

| 行 | 槽 | 组件 | 宽度 |
| --- | --- | --- | --- |
| 1 | summary | [项目统计](statistics.mq.md#项目统计) | 1fr |
| 2 | tasks | [实验任务](tasks.mq.md#实验任务) | 1fr |
```

`components/statistics.mq.md`：

```markdown
# 统计设计

## @knowledge 项目统计

展示项目摘要，数值来源应由生产端明确给出。

### 输入

| 名称 | 类型 | 说明 |
| --- | --- | --- |
| summary | Map | total、done、pending 三个非负整数 |

### 内容

#### 项目摘要（示例）

| 全部 | 完成 | 进行中 |
| --- | --- | --- |
| 3 | 1 | 2 |

### 验收

- 实际实现中统计从同一份任务数据推导，不维护第二份业务真相。
- 无任务时显示零值；加载及错误时不伪造统计。
```

`components/tasks.mq.md`：

```markdown
# 任务设计

## @knowledge 实验任务

职责：展示实验任务与状态，支持后续真实搜索、新建及更新。
原型使用三个明确情景，不执行实际写入。

### 输入

| 名称 | 类型 | 说明 |
| --- | --- | --- |
| tasks | Table | id、title、done |
| filter | Text | all、pending、done |

### 事件

| 名称 | 数据 | 说明 |
| --- | --- | --- |
| create | title: Text | 请求创建任务 |
| toggle | id: Int | 请求更新状态 |
| search | query: Text | 更新筛选条件 |

### 场景

| 状态 | 内容 | 初始 |
| --- | --- | --- |
| ready | [任务列表](#任务列表样例) | true |
| loading | [正在提交](#提交样例) | false |
| failed | [失败反馈](#失败样例) | false |

### 交互

| 当前 | 事件 | 下一状态 |
| --- | --- | --- |
| ready | 模拟提交 | loading |
| loading | 模拟成功 | ready |
| loading | 模拟失败 | failed |
| failed | 重试 | loading |
| failed | 返回列表 | ready |

### 验收

- 生产实现必须保留失败反馈并防止重复提交。
- 实际列表更新后，统计应随同一业务状态更新。
- 搜索、异步取消、真实持久化均由生产实现另行测试。

## @knowledge 任务列表样例

| id | title | done |
| --- | --- | --- |
| 1 | Bell 态制备 | true |
| 2 | VQE 参数优化 | false |
| 3 | 密度矩阵重建 | false |

## @knowledge 提交样例

正在提交任务……（模拟，不连接服务器）

## @knowledge 失败样例

提交失败，请重试。原型状态没有写入任何真实数据。
```

`shared/validation.mq.md`（可选的实际业务知识，不属于原型交互）：

```markdown
# 共享校验

## @exec valid_title

`params(title: Text)`

标题长度必须大于零。本例只检查长度，不等同于完整业务校验。

`return 长度(title) > 0`
```

此纯函数可由真实 CLI 独立检查/调用（H1 命名空间以实际 Analysis 的 FunctionId 为准）。生产框架可复用其设计或选择相同逻辑实现，验证空字符串、普通文本；Web preview 绝不调用它。不要声称此业务函数由浏览器 Runtime 执行。

**预期操作（目标 API，待开发）：**

```bash
marqdo web check index.mq.md --unit 工作台 --root .
marqdo web preview index.mq.md --unit 工作台 --root . --out ./preview-dashboard
marqdo web preview components/tasks.mq.md --unit 实验任务 --root . --out ./preview-tasks
marqdo web handoff index.mq.md --unit 工作台 --root . --out ./handoff.md
```

预期：工作台在桌面端左右两列，工作区内上下两行；390px 手机纵向顺序正确；独立任务预览可完成 ready→loading→failed→loading→ready，两个同定义不同实例互不串扰；交付列出导航、工作区、统计、任务及其接口/验收和源链接。注意统计是固定样例，不能宣称与任务列表实时联动。真正统计联动在 React/Vue 实现中验证，或将来有证据时增加通用原型能力，而非假实现。

## 9. Design 实现：读取真实 Analysis，不复制语言

### 9.1 数据流

```text
显式 ENTRY + --unit + --root
  -> 静态 marqdo check ENTRY --emit TEMP
  -> 校验 Analysis 3 与 AST/units/anchors/references
  -> 选取 Knowledge H2，解释直接约定节
  -> 对明确组件/场景/依赖链接递归解析
  -> 设计模型（定义、实例、布局、有限状态、交付关系）
  -> 校验 + 诊断
  -> Preview 渲染 / Handoff Markdown
```

首版只接受本地项目内 `.mq.md` 设计文档，不支持远程链接加载、普通 `.md` 作为设计目标、跨包缓存路径推断。普通 Markdown 内容链接可显示；资产在下一节按安全规则处理。执行 `MARQDO_BIN` 指定的真实二进制，独立开发允许显式路径配置，禁止在安装命令中探测多个历史协议。直接 argv（无 shell）、工作目录/超时/输出大小限制、临时目录 owner-only，失败清理，不自动 `lock`/`install`/联网，不启动 plugin/worker/Foreign。`check --emit` 是已有公开能力，不能改用 `inspect` 的私有或重复 AST parser。

### 9.2 链接处理的关键：`not_loaded` 不等于 missing

RC.4 的 Analysis §5 明确只包含已静态加载模块，`check --refs` 仅扩展直接链接 catalog，不把链接目标加到 modules。因此必须由 Design 显式遍历被选择的领域链接，而不是要求作者为设计引用额外写 import。

- 从对应 AST Link span 和 `references` 记录绑定源模块和来源 Unit，优先消费已 resolved 的目标与精确 anchor。领域要求目标片段存在且是本合同规定的单元类型；无片段或目标类型错误拒绝，不用 Link label 代替。
- `not_loaded`：只接受 `candidate_package_instance_id` 属于当前根项目、合法 `candidate_path` 及明确 fragment；使用该次 CLI 准备所采用的 root package module root（无 manifest 的独立 entry 按公开规则确认其根目录，不把 `--root` 假装为 CLI 的 package root）映射到允许的本地文件，确认路径位于 `--root` 且无 symlink/特殊文件。单独对目标 entry 调用 `check --emit`，验证返回 entry、来源路径与源码摘要，在目标 Analysis anchors 中对一次解码后的 fragment 精确定位；不重新实现 slug。为防止同名 fallback 造成误绑定，领域只接受精确 anchor，已 resolved 时也须以目标 anchors 验证。无 manifest 时各 entry root snapshot id 可能不同，不能直接拼接 root id；在本次明确物理项目范围内以根相对路径+结构化 UnitId.path+source_digest 归并，保留每份 Analysis 原始身份与 digest，不伪造同一个 Analysis。
- `missing/ambiguous/outside_scope/invalid/external` 对设计依赖为错误；普通内容链接不触发追踪。若无法从现行公开字段确认物理映射，报告不支持，不能通过搜索文件名或调用私有 loader 解决。带 manifest 的 module root 按公开 package 合同读取，第一版仅支持显式确认的项目根内 root package 设计文件，不支持跨依赖 package 的设计引用。
- 以实际输入 bytes 及摘要进行并发编辑一致性检查；同一文件在同一轮被两个 Analysis 引用时内容不同即失败并重试完整 build；输出前再次校验参与设计的源码/资产摘要，不能用旧模型生成伪新原型。只将真正使用的设计输入列入交付，不能把所有已加载依赖源码打包到浏览器。

### 9.3 数据模型（库 API，非新运行协议）

Design 导出小型 Rust API：`load(config) -> Result<Design, Diagnostics>`、`handoff(&Design) -> Markdown`。命名可按实际实现收敛，但只维护一套结构定义。Preview 通过公开 crate 类型消费，不自行解读表格。以下逻辑字段不要求作者维护 JSON 文件：

| 类型 | 字段与约束 |
| --- | --- |
| SourceRef | 本轮 project-relative path、结构化 Unit path、anchor、source_digest、UTF-8 byte span、原 Analysis identity |
| ComponentDef | 精确 SourceRef、职责原文、可安全渲染的内容 AST 子树、layout rows、输入/事件合同、场景、依赖、验收文本 |
| ChildSlot | 局部 slot、row、width enum、target definition ID、来源 span |
| Instance | 由根与 slot path 唯一标识，指向 ComponentDef；复用定义不复用 mutable state |
| Scenario | 唯一状态 ID、内容单元 SourceRef、初始状态、确定的事件转移 |
| DesignGraph | 定义与实例图、非视觉依赖、直接输入摘要、根身份、诊断 |

内容 AST 可使用经验证的公开 AST 的子集；只投影允许节点到统一渲染数据，不将 raw HTML 当可执行代码，不另造第二种 Markdown 解析方式。不得从 `UnitId` display 字符串切割得到 path；使用结构字段。所有诊断包含稳定代码、来源 path/anchor/span 和说明；失败返回非零，无悄悄跳过子组件。解析和模型保持确定性；相同输入与 CLI 制品应产生相同结构/交付内容。

初始安全上限（工程建议，非已测结果）：最多 100 个独立设计文档、200 个定义、1000 个实例、深度 12、每行 12 槽、每组件 32 场景/128 转移、每源 1 MiB、总输入 16 MiB、单 Analysis 32 MiB、单图片 4 MiB、资产总量 24 MiB、输出 HTML 16 MiB、全部静态调用总时限 120s（单次 20s）；拒绝超限并报告来源，不能部分渲染为成功。若实际 Analysis 因 stdlib 等静态依赖超限，按实测调整统一默认上限，不增加无限模式。缓存只在单次构建中复用，首版不引入持久 SQLite 索引；Knowledge 独立负责知识检索。性能需要时再提出增量设计编译工单。

## 10. Preview 实现、命令、安全与交付

### 10.1 单一工具入口及输出

| 命令 | 语义 |
| --- | --- |
| `marqdo web check ENTRY --unit ANCHOR [--root DIR]` | 静态检查，成功输出摘要及诊断；不执行文档 |
| `marqdo web preview ENTRY --unit ANCHOR [--root DIR] --out DIR [--theme light|dark]` | 生成可用本地浏览器打开的离线 `index.html` 和必要本地资源，不启动服务 |
| `marqdo web handoff ENTRY --unit ANCHOR [--root DIR] --out FILE` | 生成 Markdown 设计交付（源引用、职责、接口、依赖及验收），无生产代码自动生成 |

这里的三条子命令是唯一作者入口；不添加 `web serve`、`web deploy`、`web agent`、`web generate-react`。预览目录默认必须不存在，交付输出文件默认不得覆盖，拒绝写入源文件/项目锁/清单/符号链接目标；在同父目录创建私有临时 staging，校验后原子发布，不删除用户既有目录。构建失败无新成功制品。首版不承诺实时热重载。退出码 0 成功、1 设计/静态校验失败、2 参数/环境/工具协议/IO 失败（CLI 自身返回按文档保留诊断）。stderr 诊断，stdout 摘要；首版不新增公共 JSON 命令结果协议，内部制品数据不是 Runtime 协议。Tool 安装与分发按现有 `marqdo tool install .../tool.json`、`marqdo tool remove web`；冷安装验证不改变 CLI 二进制。

### 10.2 渲染器

从 Design 返回的递归模型构造页面；父级只知道 slot 和布局。通用实现使用固定 HTML shell、安全构造的静态标记或 DOM API `createElement`/`textContent`，固定共享 JS 仅做通用有限状态转换、焦点和重置；不得执行源文本或 JSON 里的函数。推荐单页离线制品，安全序列化数据（防止 `</script>` 和 HTML 特殊字符打断容器），若使用内嵌脚本则由制品生成阶段计算 CSP hash；不从 `file://` fetch 外部 JSON 造成浏览器 CORS 问题。样例数据与业务逻辑不写入 host.js。通用脚本在所有示例内容不同的情况下保持相同 hash（允许发布版本变化）。

只更新发生转换的实例区域，不销毁其他组件；保持焦点或移至明确的可访问状态提示，按钮可键盘操作、状态提示可用 `aria-live`，原生 HTML 表格/标题提供语义。移动端 390px 不得出现页面横向溢出；宽表允许组件内部有明确滚动。静态导航使用生成制品中实际存在的锚点或普通安全链接（指向未打包本地文件的原始链接仅显示为来源提示，不从 file:// 跳出原型范围），外部跳转必须由用户明确触发，不自动加载远端内容。提供清晰的“原型 / 模拟”可见标识及本轮构建摘要，不显示本地绝对路径或秘密。Preview 内容不能被当作生产站点、登录系统或真实 API。

### 10.3 安全边界

| 风险 | 必须控制 |
| --- | --- |
| 原文 HTML/JS 注入 | HTML 原文以文本或占位；无 eval、Function、innerHTML、srcdoc、on* 属性、任意脚本资源 |
| 恶意 Markdown 链接 | allowlist 协议（本地锚点或显式 http/https 链接；禁止 javascript/data/file 等）；绝不自动网络请求 |
| 图片/资源泄漏 | 只允许明确展示的项目内普通 raster（PNG/JPEG/WebP/GIF）文件，经路径/大小/魔数校验复制到 staging；远端图片仅文本占位；SVG/HTML 作为代码或参考，不内联执行 |
| 任意 CSS | 默认仅内置固定 CSS；作者围栏和 CSS 资源只作为设计交付，不自动应用 |
| 资产/源码越界 | 固定 root、一次解码、canonical 检查、拒绝 symlink/路径逃逸及特殊文件；不复制完整 Analysis、环境或 server 源 |
| 内容与 script 数据混合 | 上下文相关转义，含 `</script>`/`<img onerror>`/Unicode/双引号的恶意 fixture 必须通过 |
| 权限误解 | Tool 以当前 OS 用户权限运行，不是 CLI sandbox；不启动文档、插件、进程业务；仅受控调用 CLI 静态 check |
| 离线制品外联 | 默认 CSP `default-src 'none'`，必要固定脚本/样式 hash、受限本地图片，`connect-src 'none'`、禁止表单提交与嵌入；测试实际浏览器行为，不能只检查字符串 |
| 资源耗尽 | 解析、图遍历、资产、输出文件/HTML 大小与静态进程均受限；超限确定失败 |

限制不是永久禁止复杂视觉。生产框架可使用 CSS、SVG、Canvas 和第三方组件；原型工具的安全性和快速表达优先。若某原型需求确实需要高级能力，先发 Web 领域工单评估可重用的表达机制；不能增加核心语法或允许无条件 HTML 注入。

### 10.4 交付与并行开发

Handoff 不发明新的持久设计格式；源文档始终是权威。生成 Markdown 仅为派生快照，可供 Coding Agent 单独读取，也可以被 Knowledge 检索但无需依赖 Knowledge。每份交付至少包含：根及源文件摘要、递归组件关系、每个组件职责/输入/事件、样例与场景、视觉说明/参考、验收、明确未实现的真实功能、直接依赖、精确来源链接。链接使用相对原始项目路径或明确项目相对引用，输出到其它目录时重新计算可打开的链接；不要使用猜测的 anchor。需要单组件任务时，直接 `--unit` 选该组件并附相关依赖，不复制整仓源码。交付摘要随源变化重新生成，不当作第二份手工维护权威。

**任务拆分流程：** 先由人/外部 Agent 确认设计图和共享接口，再按组件职责划分。视觉包含边不是必须先后开发的依赖；例如 Workspace 与 Tasks 可在约定接口后并行实现。非视觉依赖（`依赖`）以及共享数据模型/事件合同必须先冻结或提供 mock。Handoff 报告依赖顺序、循环组与共享合同，但不自动分派任务、不启动 Agent、不声明所有叶子可并行。循环依赖组需要协调拆分或共同实现，不可伪装独立任务。

以 §8 为例：

| 开发任务 | 可用上下文 | 边界 | 验收 |
| --- | --- | --- | --- |
| A：导航 | 导航组件及父级槽约定 | 不改统计/任务状态 | 桌面/手机、键盘、真实导航行为由生产定义 |
| B：统计 | 统计文档及共享任务数据模型 | 不自行请求另一个统计真相 | 同一 tasks 推导 total/done/pending，空/加载/错误状态 |
| C：任务 | 任务组件、事件/输入及服务接口约定 | 不直接改统计、共享布局 | 真实筛选、创建、失败、重复提交、状态更新 |
| D：集成 | 根、工作区及各组件公共接口 | 不替代组件内部实现 | 联动、路由、响应式、实际持久化和安全（生产测试） |

真实任务由 qdagent/其他 Coding Agent 或人类通过独立任务分支执行。Web 工具不内置模型、不自动操作 Git。目标框架可由维护者任选，首版只要求选定一个成熟框架完成真实交付验收，不需要三套生成器。

设计漂移管理：设计文档权威是职责/接口/行为意图，生产源码权威是视觉细节、实现、性能；设计改动发对应组件任务，生产改动若改变设计合同必须同步源设计和测试，不强制全量重新生成。验收通过生产 E2E、接口测试和必要人工视觉审查，不声称工具可以自动证明自然语言要求。禁止把原型测试 PASS 当生产测试 PASS。

## 11. 代码组织及接口防膨胀

建议使用既有 Rust 工程惯例，依赖真实公开 CLI，不需要新 workspace 管理器。Design 和 Preview 各自一个主要 crate；不为每个表格新建 crate、服务或插件。

```text
marqdo-web-design/
├── Cargo.toml / Cargo.lock
├── src/
│   ├── lib.rs           # 公开 API 及结果类型
│   ├── analysis.rs      # CLI 调用、公开协议验证、静态输入快照
│   ├── document.rs      # 确定的章节/表格选择、单元引用
│   ├── model.rs         # 定义、实例、布局、场景、来源
│   ├── graph.rs         # 递归链接、循环、依赖与限制
│   ├── handoff.rs       # Markdown 交付
│   └── diagnostic.rs
├── tests/               # 单元、负例、真实 CLI contract
├── scripts/ci.sh
├── README.md / AGENTS.md
└── docs/

marqdo-web-preview/
├── Cargo.toml / Cargo.lock
├── tool.json
├── src/
│   ├── main.rs          # 3 个子命令、参数、退出
│   ├── render.rs        # 安全结构投影
│   ├── export.rs        # staging、资产、CSP、制品
│   └── cli.rs           # 入口与输入配置（需要时）
├── assets/
│   ├── preview.css      # 固定基础主题、布局
│   └── preview.js       # 固定有限场景运行器
├── tests/               # browser fixtures、XSS、冷安装
├── scripts/ci.sh
├── scripts/package-tool.sh
└── README.md / AGENTS.md
```

`docs/WEB-DESIGN.md`（领域仓库）是领域作者约定权威，Design 是唯一解释实现。Preview 不能另加一套解析规则；本地脚本只包装同一 CLI。测试 fixture 可以覆盖公共合同，但不允许复制生产解释器作为测试 oracle；应使用独立期望结果、真实 Analysis、实际浏览器和交付工程。领域规范变更先在 Ext-Web ADR/Issue 明确语义，再由 Design/Preview 配对候选提交一次切换，旧文档约定留 Git 历史，不提供多版本表头探测。

## 12. CI、跨仓工单与发布工作流（所有领域适用）

### 12.1 分层验收与调度

组件 CI：每个功能仓库独立检出，使用精确上游发行/候选制品，运行格式、静态检查、单元、负例、真实接口、打包。领域 CI：固定本领域子 gitlink 及所有跨领域依赖输入，运行组件入口或复用同 SHA/同制品同环境证据，再执行领域专属组合例。生态 CI：检出固定递归树、验证跨领域闭包、基础冷安装、公共 conformance、跨域组合和发布。三层不是重复运行所有测试三次；同样候选的有效证据可复用，身份不匹配必须重新运行。

**主仓库具体改造：** 修改现有 `scripts/check-layout.py`、`scripts/integration.sh`、`.woodpecker/eco.yml`、`tests/integration`，不另建庞大 CI 控制平台。以现有 Woodpecker 为权威，GitHub 镜像 CI 可做辅助验证但不触发第二套冲突发布。统一选定 CLI/Worker/Stdlib 候选制品由根集成准备阶段提供：输入须绑定 CLI gitlink SHA、目标平台、归档/二进制/Bundle digest、工具链和锁；不再从 View 的 `frozen-beta.sh` 决定生态版本。保留原 root 集成脚本的 conformance、View 安装/浏览器、Knowledge、Prove、Lint 等真实步骤，逐个移至相应责任组件的标准入口或根跨组件测试，不能为了通用调度减少覆盖。原固定六组件列表删除，未知 active 入口不得跳过。

约定：active `entry` 是由该仓库维护的独立验收入口，建议 `scripts/ci.sh` 或 `scripts/integration.sh`；具体语言/命令由组件负责。入口从明确传入的 `MARQDO_BIN`、Bundle/依赖制品位置和隔离的测试输出目录读取，不自行拉取浮动 CLI 或其他组件 main。现有不统一入口按组件工单增加小适配，旧入口可继续作为内部调用，但不能形成两套验收定义。不要在根仓库写针对每个组件的业务实现。运行上下文可以用环境变量和固定制品路径，不引入新网络协议。候选计划依据 `requires` DAG 拓扑调度；并行只在依赖制品准备完成后运行。

候选变更检测：比较已接受根提交与候选递归展开的 `(global_id, parent, remote, SHA, state)`；领域指针变化不能只标记父节点，必须展开差异。变动的叶子加入集合，取其反向传递依赖闭包；加所在祖先领域及所有需要协同验收的 checks_with（固定点、去重、不作为构建先决），公共协议/权限/根集成逻辑变化触发相应全量 conformance 与消费者。删除/移动组件需用新旧图并集计算受影响者。仅文档改动只有明确证明不影响合同才可缩减测试。无法确定影响范围时全量而非跳过。正式基础/生态发布执行所有被声明为发行范围内的 active 组件和相应完整测试；planned 不计入 PASS，不能以测试不存在掩盖缺失。

### 12.2 工单路由与局部修复

发现错误先固定根 SHA、领域 SHA、叶子 SHA、CLI 制品及最小复现，找**第一个违反公开合同的所有者**，不要按错误最终显示的仓库定责。单组件缺陷直接发功能工单；同领域接口冲突由领域协调；跨域接口或公共语义由实际生产者与消费者联动，主仓库只协调集成组合。每个真实缺陷只有一个主要责任工单，可关联消费者阻塞和集成工单；不同问题拆分，重复发现追加证据。任何维护者或 AI 可在权限范围内读其它仓库定位，但修改必须进入对应责任仓独立 PR。禁止在根或聚合脚本加入 workaround、复制协议或绕开权限。

**工单模板（复制到真实责任仓库 Issue）：**

```markdown
## 问题与所有权
责任组件：<全局 ID / 仓库>
类型：bug | missing capability | contract change | migration
影响消费者：<ID / 关联工单>

## 可复现的候选组合
根/领域/组件 SHA：...
CLI、Stdlib、依赖制品 SHA-256 与平台：...
最小 fixture / 命令 / 实际结果：...
预期结果与权威合同链接：...

## 修复范围
必须：...
禁止：兼容旧协议、复制实现、无关重构、扩大权限等
上游阻塞：<工单链接或无>

## 验收
正例 / 负例 / 回归 / 性能安全条件：...
组件 CI：...
领域/消费者组合测试：...
修复提交与制品证据：...
```

典型路由：

| 现象 | 责任归属 |
| --- | --- |
| 组件表头正确但 Design 未识别 | web-design（若 Analysis 的真实 AST 正确） |
| Link 中 `not_loaded` 错误处理 | web-design；若 CLI candidate 违反公开 resolver 合同则 CLI |
| 手机布局溢出、场景实例串扰 | web-preview |
| 特定设计模板不符合合同 | web-components 或作者项目，不修改核心 renderer 迎合坏模板 |
| Web 设计约定出现冲突 | Ext-Web 协调，Design/Preview 各自 PR |
| 真实生产 React 列表渲染错误 | 目标应用仓库，不归 Web 原型 |
| SSE 或网络传输错误 | Ext-Net 的 HTTP/WS 或 CLI Resource 真正责任方，不归 Web Design |
| 固定 SHA 组合构建失败 | 若各组件公开制品正确，由所属聚合/根修复装配，否则责任组件 |

状态流：open → triaged → in-progress / blocked → component-verified → integrated。可用现有 Git 平台标签/关联链接，不设计新工单数据库；阻塞项不能以 mock PASS 关闭。组件 PR 合并后可关闭该组件修复工单，但消费者/领域或生态集成任务保留至真实候选组合通过。具体工作流：功能提交 C2 推送并通过组件 CI；领域 PR 更新 gitlink 至 C2 并通过领域测试，形成 B2；根 PR 更新领域 gitlink 至 B2，执行受影响及公共组合验收，形成 A2。失败停在该层并发工单，不让未验证组合污染受保护集成分支。跨域破坏性合同变更允许配对候选 PR 先在隔离候选组合联调，所有消费者同步迁移再晋升，不让根主分支出现半套协议。

### 12.3 自动化权限与防止失控

首先实现人工触发和 PR/CI 门禁，不开发自动 Agent 管理平台。Webhook 后续可通知上级新候选（含真实 repo/commit/evidence），上级通过 API 验证实际存在、审批与权限，再创建候选 PR；不得自动合入或提升权限。外部不可信 PR、未批准 gitlink 或脚本不得在拥有跨仓库写权限/发版密钥的 runner 中运行。区分只读 checkout/test token、受限 issue/PR token、受保护发布权限。不要在日志输出凭证、Analysis 完整源或生产 API key。对重复失败工单按稳定签名去重（责任 ID、合同、fixture、错误码），限制重试，停止无关重构循环。

### 12.4 发行与证据

组件可独立 SemVer 发布；领域可发布经过验证的组合制品；根生态 release 固定整个 gitlink 树。独立版本不代表维护多套现行协议；预稳定时期破坏性替换必须同候选联调。每份证据记录根及递归组件 SHA、制品摘要、真实命令、退出、环境/工具链、日志、测试范围、未验证项。复用现有 manifest/evidence 模式，不复制成新的权威锁。将 Git tree 作为源码身份权威，Cargo.lock/marqdo.lock/现行 Bundle manifest 作为相应依赖权威。记录不可变归档校验和、可信来源及获取方法；摘要校验不是发布者身份认证。不得只信标签名、浮动 branch 或消费者缓存。基础发行不附带 Web/Lean/AI 等可选大型依赖；用户可独立安装 Web Tool bundle。

现行已验证平台基线为 Linux x86_64/glibc 2.39；其他平台标 UNVERIFIED，除非实际验证。稳定版仍遵守 CLI 专用安全/性能门禁。本规范新增原型安全门禁，不替换 CLI 的 strict 策略。

## 13. 防止架构、仓库和作者面再次无限膨胀

以下为可执行门禁，不只是风格建议。

| 门禁 | 检查与拒绝条件 | 权威 |
| --- | --- | --- |
| 归属唯一 | Git tree/catalog 递归一致、一个全局 ID 和远端只出现一次；不允许同功能两个官方实现 | 根 |
| 聚合纯度 | 根和领域聚合不含生产实现；新增文件须落在治理/集成/示例职责 | 根/领域 |
| 深度 | 超过两级 gitlink 拒绝，除非批准 ADR 先更新单一规则 | 根 |
| 真实依赖 | active 不依赖 planned；构建 DAG 无环；没有父 checkout、跨仓私有源码依赖 | 根/组件 |
| 标准库准入 | 基础发行默认必需、低层、至少三类真实消费者、依赖与授权有界；新功能默认 extension | Stdlib/根 |
| 内核准入 | 新 builtin/HostOp/语法/协议字段须 ADR、现有机制不足的最小复现、三个不相关领域用例；Web 专属需求不进入 CLI | CLI |
| Web 作者面 | 不新增核心语法、重复表头方言、隐式解析普通文档、HTML 注入默认通道 | Ext-Web/Design |
| 通用渲染 | 新普通组件由设计文档复用实现；不得新增任务/页面/业务专用 renderer 分支 | Preview |
| 原型诚实性 | 模拟明确标注；无真实后端、无假执行、无伪造生产测试 | Web |
| 独立产品 | 组件能独立构建，只有实际产品需要才创建新仓库 | 各级 |
| 现行合同唯一 | 不引入旧协议 fallback、版本数组、旧语法探测；配对候选一次切换 | 各级 |
| 默认安装规模 | 基础制品不引入可选 Tool/领域环境；体积、构建时间及依赖增长记录基线 | 根/Stdlib |
| 范围控制 | 每个 PR 关联责任工单、复现与验收；无关重构拆独立任务 | 各级 |

初始规模审查建议：默认发行压缩包体积或固定节点构建 P95 增加超过 10%，必须给出必要性、差异原因与回归证据；不是硬编码产品质量结论，也不能通过拆包隐藏增长。Web 预览首版只允许一套运行器、一套设计合同、三条作者命令。新功能首先问是否能通过已有 Markdown 内容、组件关系、有限情景或目标框架表达；无法复用且只是生产实现需求时，应直接交给目标框架，而不是扩张原型工具。

每次架构变更须回答：谁是唯一责任方？是否必须新增公开边界？如何证明现有机制不足？能否在单仓修复？新功能能否独立测试？删除或替代了什么？如果不能明确回答，不进入开发；不得用重构作为不断寻找新功能的理由。

## 14. 首批开发工单与里程碑

本节可直接拆成实际 Git 工单；编号为本文内计划 ID，不是远程工单编号。查找已有工单并复用，不自动创建重复任务。执行阶段只在责任仓库修改；依赖说明用于配对候选，不要求所有工作串行。

| ID / 责任仓 | 范围、交付物 | 验收依赖 |
| --- | --- | --- |
| ECO-00 / marqdo | 盘点自建远端、六个现有 gitlink、真实制品和 CI；冻结基线，批准本规范替换旧直接子模块策略 | 所有现有组件可核查；缺失明确阻塞 |
| ECO-01 / marqdo | `check-layout.py` 递归校验、domain 类型、全局依赖、真实两级 Git fixtures；更新根规范 | ECO-00；G01–G03 |
| ECO-02 / marqdo | 声明驱动集成计划、统一 CLI 制品准备、递归影响分析、CI 与证据；保留现有六组件门禁 | ECO-01；G04–G05 |
| ECO-03 / 各既有组件 | 必要时为现有 active 组件提供统一 CI 入口，真实 CLI 制品输入和输出证据；不改变其功能 | ECO-02 配对候选 |
| STD-01 / CLI | 外部 Bundle 构建/测试输入，见 §5.2；若已有工单则补充 | 当前完整标准库产品回归 |
| STD-02 / marqdo-stdlib | 领域聚合、路径/摘要/alias 组装器、现有 stdlib 功能迁移台账，九类责任逐步建立 | STD-01 配对联调 |
| STD-03 / 各 std-* | 原库函数与产品测试迁移、重复清理、独立真实 CLI 测试 | STD-02 对应 Bundle 合同 |
| STD-04 / Stdlib+根 | 外部 Bundle 冷安装、无重复源码、最终根 gitlink 及测试；基础发行不变 | STD-01..03 的实际迁移范围通过 |
| WEB-00 / Ext-Web | 固定 §6–§7 设计合同、示例、真实 Analysis 原型验证、明确候选 Rust API | CLI RC.4 制品；无需等待 Stdlib 拆分 |
| WEB-01 / web-design | Analysis 3 消费、精确组件引用、递归布局/场景模型、诊断、源身份 | WEB-00，真实 CLI 夹具 |
| WEB-02 / web-preview | 通用渲染器、离线制品、模拟场景、响应式、XSS/安全、Tool 包 | WEB-01 候选 API |
| WEB-03 / web-design | handoff、组件/依赖图、独立单组件交付、漂移标识 | WEB-01 |
| WEB-04 / Ext-Web | 三个组合设计案例、真实浏览器、一个生产框架交付试验及边界报告 | WEB-01..03 |
| WEB-05 / 根+Ext-Web | 推送精确候选、递归接入、冷安装独立 Tool、生态验收 | ECO-02、WEB-04 |
| WEB-06 / components（候选） | 有至少两个独立真实复用模板及稳定合同才创建；不阻塞首版 | WEB-05 后另行准入 |

### 14.1 治理里程碑

- G-M0：ECO-00 完成，现有根提交不被破坏。
- G-M1：ECO-01 完成，真实本地 Git 仓库（含两级子模块）验证通过，现有六组件仍可识别。
- G-M2：ECO-02..03 完成，实际自建 CI 验证至少一次普通叶子修复→领域更新→根更新；失败候选被拒绝，回退可复现。
- G-M3：第一个真实 domain 接入并通过完整生态验收（可为 Web；Stdlib 路线可并行）。Stdlib 聚合需 STD-01..04 独立完成后才切换基础发行；不能为了凑两级结构先删现有库。治理架构本身不等待所有扩展迁移。

### 14.2 Web 里程碑

- W-M0（技术风险）：在真实已安装 RC.4 CLI 上对 §8 所有 `.mq.md` 调用 `check --emit`；确认表头/Link span/units/anchor/candidate path 与合同一致，尤其验证未导入的跨文档链接。编译独立 Design crate 的最小解析路径。记录是否存在真实公共接口缺口，不凭预想申请 CLI 新 API。
- W-M1（设计编译）：WEB-01 完成，真实 CLI 正/负例、递归组件、跨文档链接、场景及稳定身份全部通过。
- W-M2（交互预览）：WEB-02 完成，离线真实浏览器、动态场景、布局、键盘、XSS、无网络、冷安装；一个通用宿主用于多个设计。
- W-M3（交付）：WEB-03..04 完成，独立组件切片、依赖说明、至少一个实际生产实现；能够比较设计修改前后的增量交付。不得声称生产实现由工具自动生成。
- W-M4（发布）：WEB-05 完成，领域与根同候选证据，发布首个可选 Web Tool。

初期只承诺 W-M0..4。复杂前端、真网络、WASM、动画、生产框架互操作生成、Components 子仓库及多智能体调度全部为后续独立任务，不得成为首版无限延期的理由。

### 14.3 是否需要新增 CLI Web Issue？

**当前不提出新的 Web 专属 CLI Issue。** RC.4 Analysis 已包含源、AST（GFM Table/Link）、标题范围、模式、锚点、引用状态及候选路径。Tool 合同允许独立 `web` 命令。首版只做静态解释和通用原型，不需要 Resource Control、Structured Tasks、Browser Host、Plugin 回调、额外程序执行接口。已知 Stdlib 打包问题为独立 STD-01，不属于 Web 新增需求。

若 W-M0 真实测试发现公开 Analysis 不能明确绑定表格中的 Link 或不能在公开 package 规则下安全解析设计目标，先保留最小 fixture 和实际 Analysis 记录，判断是 Design 使用错误还是生产者合同缺陷。仅后者向 CLI 提交 issue：引用公开规范、固定 SHA、最小复现、其他消费者影响、通用修复及正负例；不得要求 `web.layout` builtin、`@web` 语法、专属 AST 字段或 Web parser。设计领域可自己解释的内容不进入 CLI。没有证据就没有上游新需求。

## 15. 验收矩阵（必须真实执行）

所有以下编号是拟定的验收测试，本文没有执行。测试应在对应责任仓库中自动化；对于需要远端权限或真实生产实现的验收，保留 CI、PR、浏览器证据。`PASS` 必须绑定候选源码及制品身份；未运行就是 `UNVERIFIED`。正例不接受仅 mock CLI；负例可使用专门篡改/恶意测试数据。每项失败都按 §12 路由。

### 15.1 生态与分层治理（E01–E24）

| ID | 真实测试与通过条件 |
| --- | --- |
| E01 | 当前六组件固定树仍通过新 checker；无强制创建 planned 空仓 |
| E02 | 用临时真实 Git 仓库创建根→领域→两个叶子，checkout 精确 SHA，递归校验通过 |
| E03 | 根和领域 catalog/gitlink 不一致、缺 gitlink、额外 gitlink 均拒绝 |
| E04 | 重复全局 ID、重复权威远端、重叠路径、大小写/NFC 冲突拒绝 |
| E05 | 非法 URL、file 协议、相对跳转、update command 拒绝，且未执行远端获取 |
| E06 | 第三级未批准 gitlink、叶子内未注册子模块拒绝 |
| E07 | 缺失提交、dirty checkout、错误 HEAD、符号链接入口拒绝 |
| E08 | planned 有实现或 active 依赖 planned 拒绝 |
| E09 | 跨域正常 DAG 能拓扑排序；循环 requires 拒绝，checks_with 循环不死锁 |
| E10 | Stdlib 依赖 extension、core 反向依赖、requires 指向 domain 拒绝 |
| E11 | 主仓库/领域聚合新增未经允许的生产实现被架构审查阻断 |
| E12 | 从旧一级路径移动真实组件，根新树唯一远端，旧根 SHA 仍可完整 checkout |
| E13 | 叶子独立 clone/build，离开父目录仍可使用真实上游制品 |
| E14 | 统一 CLI 归档绑定根 gitlink、Worker 和 Bundle，篡改或错 SHA 拒绝；无 View 决定 CLI 版本的依赖 |
| E15 | 未知 active 入口缺失/失败必须使候选失败，不静默跳过 |
| E16 | 变更单个叶子，只触发必要消费者、领域祖先和协同验收；无关组件不作为强制重建 |
| E17 | 跨域依赖叶子变更、组件移动/删除、CLI 公共协议变更，反向闭包覆盖正确 |
| E18 | 用实际根→领域→叶子修复 PR 演示逐层晋升，工单关联与证据可访问 |
| E19 | 引入故意失败叶子测试，领域或根候选拒绝，已接受组合不变 |
| E20 | 回退根 gitlink 到旧已接受组合，真实冷安装和关键测试可复现 |
| E21 | 不可信 PR 不能读取发布凭证、执行未验证的嵌套更新或自动合并 |
| E22 | 现有 conformance、View、Knowledge、Prove、Lint 的原集成门禁没有因通用调度被删除 |
| E23 | 基础发行包含默认 Stdlib，未引入 Web/Lean/模型/浏览器等可选运行环境 |
| E24 | 同候选完整证据包含递归 SHA/摘要/工具链/命令/日志，FAIL/UNVERIFIED 不得冒充 PASS |

### 15.2 Stdlib 聚合（S01–S10）

| ID | 真实测试与通过条件 |
| --- | --- |
| S01 | Stdlib 聚合从固定子 gitlink 构建真实 Bundle，同输入摘要一致 |
| S02 | 路径冲突、越界、symlink、未跟踪来源拒绝 |
| S03 | 现有 CLI 算法独立验证 tree/alias digest；篡改失败 |
| S04 | CLI/Worker 在没有完整库源码的独立 checkout 编译 |
| S05 | 外部 Bundle 输入可打包并跑完整真实 stdlib 产品测试 |
| S06 | 官方 print alias 和 `std:` 导入冷安装、移动目录后仍可用 |
| S07 | 默认发行缺失或错误 Bundle 明确失败，不回退旧源 |
| S08 | 迁移台账逐模块对照旧产品功能，已承诺的回归测试全通过 |
| S09 | 活跃 CLI/聚合/功能仓之间无重复权威算法或旧源 fallback |
| S10 | 叶子局部修复经 Stdlib 聚合更新，根更新统一 Bundle，无新加载协议 |

### 15.3 Web Design、Preview 与 Handoff（W01–W36）

| ID | 测试与通过条件 |
| --- | --- |
| W01 | 真正 RC.4 CLI 产生 Analysis 3，真实选中 H2 Knowledge 与表格 AST |
| W02 | 全部 §8 正例通过实际 CLI check；任何语法或 anchor 误差修正示例并记录 |
| W03 | 普通 Markdown/GFM 表不被自动解释；未选中的组件无副作用 |
| W04 | H2 Exec、H3 非目标、缺失 anchor、同名 fallback、错误模式拒绝 |
| W05 | 表头错误、重复章节、错误列数/位置、非 Link 单元格、表达式求值企图拒绝 |
| W06 | 合法未导入的跨文档 not_loaded 递归加载并精确绑定，不要求虚假 import |
| W07 | 缺失、歧义、external、越界、绝对/file 路径、双重编码/逃逸/恶意 symlink 拒绝 |
| W08 | 两 entry 的 root snapshot 不同，按真实输入与来源身份正确关联，不按文件名猜测 |
| W09 | 已加载模块、真实 source digest、UTF-8 span、anchors 和引用状态交叉验证；篡改 Analysis 拒绝 |
| W10 | 并发编辑或陈旧输入导致构建拒绝，不发布不一致模型 |
| W11 | 递归布局三层以上（设计层非 Git 层），视觉包含环拒绝 |
| W12 | 同一组件重复实例，slot 路径不同，状态独立 |
| W13 | 行号、slot 唯一、宽度范围、超出图/输入/时间预算拒绝 |
| W14 | 场景恰有一个 initial、转移目标有效、事件无歧义、状态可达，非法定义拒绝 |
| W15 | 真 Chromium 打开实际 `file://` 制品，不需要开发服务器、网络或 fetch 转接器 |
| W16 | 桌面布局和 390px 手机重排通过，页面无横向溢出（宽表局部滚动允许） |
| W17 | 两种主题、可读 Markdown 标题、列表、表、图片、代码及链接展示正确 |
| W18 | 真实用户点击 ready→loading→failed→retry→ready；重置/刷新规则一致 |
| W19 | 复用两实例、切换一实例不修改另一实例或其焦点 |
| W20 | 键盘导航、可见焦点和状态通知；真实浏览器测试，不只 DOM snapshot |
| W21 | 恶意 HTML、`</script>`、on*、javascript URL、恶意属性/内容不会执行 |
| W22 | 远程图片、危险 SVG/HTML/CSS、代码块无自动执行、无外部网络请求 |
| W23 | 安全本地 raster 资产可用；路径逃逸、伪装文件、超大小拒绝 |
| W24 | 预览制品不包含未选择的私密模块、完整 Analysis、绝对路径、凭证 |
| W25 | 静态 check/preview/handoff 零执行 sentinel；文档中的文件/网络/插件调用不发生 |
| W26 | 缺 CLI、错误协议、超时、输出冲突、失败中断，无虚假成功制品或删除原文件 |
| W27 | 真实 Tool install/dispatch/remove；CLI SHA 不变；篡改 bundle 拒绝 |
| W28 | 同输入输出模型和 Handoff 可复现；源改动后摘要和受影响组件变化 |
| W29 | 单组件预览/交付只包含相关组件和显式设计依赖，源链接可准确打开 |
| W30 | 视觉树与非视觉依赖分离；共享依赖、循环组和并行限制报告正确 |
| W31 | 至少三个不同领域示例共用同一 renderer JS/CSS（主题可选），无业务专用 JS 分支 |
| W32 | 至少一个成熟框架实际交付，生产源码有独立组件与真实状态联动；测试源接口和行为 | 
| W33 | 修改一份叶子设计并重新生成交付，能只修改受影响生产组件/测试；共享合同变化明确标记 |
| W34 | 模拟与真实业务状态始终可区分，未验证生产能力不显示为通过 |
| W35 | Design/Preview 独立 checkout 构建；实际锁定 Design 依赖身份与领域 gitlink 相符 |
| W36 | 同 SHA 领域/根 CI 及完整发布制品证据；三条命令、文档、实现一致，无旧作者面兼容路径 |

### 15.4 三个必做领域案例及实际交付试验

1. **文档型网站**：导航、长文章、表格、图片、两级布局、响应式、独立组件预览。验证静态设计不需要大量手写 HTML。
2. **科研任务台（§8）**：多文件树、接口、有限场景，包含失败与重试；实际生产端用一个成熟框架实现真实动态列表、搜索、创建、统计联动、错误与持久化（可用独立测试后端），生产 E2E 测试。目标框架仅作为实验应用，不成为 Web Tool 运行依赖；可放领域集成 fixture，规模变大后移至独立应用仓。生产实现允许常规 HTML/JS/CSS，不要求所有代码由 Marqdo 生成。源设计和实现之间以组件源链接及测试映射建立可追踪关系。
3. **量子电路编辑器设计**：工具栏、画布区域、参数面板、事件合同、样例电路表、拖拽/撤销/错误场景与参考图。原型显示区域、样例和情景，不实现真正拖拽或计算。生产框架/Quantum 专业能力另立任务。此案例专门验证复杂需求不会迫使 renderer 新增量子专属代码或 HTML 注入。

上述三个案例只要求真正通用的原型能力，不要求每个产品有生产实现。W32 至少完成科研任务台一个生产实现，避免只有结构化文档和静态截图却没有交付验证。若案例需要作者绕开合同，应记录原因、判断该能力属于生产还是原型；不能偷偷写专用宿主分支。

### 15.5 性能、可读性及研究验证

首版工程门槛（拟定，须在固定 Linux x86_64 4 vCPU/8 GiB/SSD 记录实测）：使用已安装 CLI 和本地输入，5–20 个设计文件、50 个组件实例的冷 preview 构建 P95 ≤ 5s；100 文件/1000 实例边界 fixture P95 ≤ 60s（仍受 §9.3 限制，记录峰值 RSS，建议 ≤1 GiB）；30 次场景操作 P95 ≤ 100ms；固定脚本无业务数据特例。预热、样本数（至少 20 次构建）、CLI 启动占比、文件数、资源量及环境单独记录；首版若未达到阈值，分析瓶颈及修订目标需维护者批准，不靠删除真实 CLI 检查提速。上述不是已测性能，也不是科学结论。上限 fixture 必须满足文档总量、图深度等其它限制。

评估目标不仅是生成网页：记录设计源码有效行数、文件数、链接数、需要人工补充的接口、原型生成时间、错误定位与修改影响范围、是否有原型专用业务 JS/HTML 注入。首版三个原型不得需要作者业务 JS、raw HTML 注入或修改 CLI 语法；复杂生产实现可用成熟框架。验收评价可读性时，应让未参与架构设计的开发者仅凭源文档理解页面、独立完成至少两个组件任务，并记录问题、误解和接口修改。没有执行者时此项保持 UNVERIFIED，不能由作者自己宣称人机协作改善。

后续若开展科研实验，设置同一任务/框架/模型/预算的对照：自然语言→框架、普通 Markdown→框架、Marqdo 原型→框架；在控制相同设计输入的前提下额外对比串行和并行。记录 token、真实完成时间、File Changed、接口错误、回归、人工干预、设计一致性、执行成功率及置信区间。不能把增加并行度带来的收益混同于 Marqdo 本身，也不能把模拟预览通过当真实功能通过。研究结论不作为首版架构开发必须先实现的巨大功能。

## 16. 开发结束条件、尚存风险与交付

### 16.1 三个独立结束条件

**分层治理完成**：E01–E24 实际通过；至少一个真实领域组合进入根并成功演示工单、独立修复、递归更新、失败拒绝和回退；根不再固定六组件、依赖 View 获取 CLI 或仅验证一级；原六组件门禁保留；单套治理规则。未完成的候选领域按 planned 管理，不阻止治理完成。

**Stdlib 拆分完成**：S01–S10 实际通过，所有本次迁移约定的现有标准库源码有唯一权威，外部 Bundle 成为正式产品输入，基础安装不退化，根固定 Stdlib 组合；补齐旧版缺失 API 是各独立库后续工单。尚未拆完时明确原 CLI 内库仍是现行权威，不宣布已完成拆分。

**Web 首版完成**：W01–W36 实际通过，真实 CLI（以锁定产品制品为准）/ Analysis 3 接入（Tool `api_revision=1`），Design/Preview 独立构建，三例通用离线原型，一个成熟框架生产交付，同候选冷安装和根集成通过；领域合同唯一。此时可以结束首版，不要求 WASM、全栈服务、真实浏览器业务、所有旧 Web API 或自动生成所有框架。

### 16.2 仍须面对的风险与停止规则

| 风险 | 实施前/实施中验证 | 失败怎么办 |
| --- | --- | --- |
| CLI Analysis 的路径或 table span 事实不满足设计解释 | W-M0 实际 fixture | 先确认公开合同，再决定通用 CLI issue；不造私有 parser |
| 两仓 API 频繁联动 | 独立 CI、变更成本记录 | 先收敛接口，必要时暂缓拆仓；不复制实现 |
| 原型作者面开始变成低效 HTML DSL | 三例、新普通组件零宿主变动 | 缩小原型范围或改善递归关系；生产需求交给框架 |
| 有限场景不足以模拟真实交互 | 科研任务台生产对照 | 明确未实现；必要时新增通用能力单独评审，不强加 WASM |
| AI 不理解接口或组件仍不能并行 | 独立开发试验、共享依赖检查 | 修订职责/合同，不宣称自动并行或强制分派 |
| 设计和生产漂移 | handoff 摘要、源链接、E2E | 关联局部变更工单，不自动重写整个应用 |
| 嵌套 CI 放大维护与凭证复杂度 | 真实递归 Git 测试、CI 权限、局部影响闭包 | 拒绝无价值聚合、按需保留直属叶子，避免建新管理平台 |
| 默认发行膨胀 | base 制品比较 | 将专业依赖留在 optional；有意增长经 ADR/审查 |

本文件依据固定仓库快照、现行 RC.4 公开合同及此前原型实测记录制定。新的 Web 作者合同、模拟状态模型、工具命令、递归 catalog 规则、测试阈值和里程碑是本文提出的实施决策，不是现有源码事实。尚未在此环境构建 Rust 工具、执行真实 CLI 或自建 CI，也未创建远程仓库、工单或修改源码。所有新能力以真实候选测试为准。

### 16.3 文档落库方式

根仓库保存本规范作为本轮实施输入；完成后把治理规则合并进现有 `docs/ARCHITECTURE.md`、`docs/DEVELOPMENT.md` 与 `AGENTS.md`，台账进 `docs/MIGRATION.md`。Ext-Web 按本规范 §6–§11 建立唯一 `docs/WEB-DESIGN.md` 与开发指南，组件 README/AGENTS 链接该权威合同，而非复制整份内容。本文可留作历史 ADR/实施记录，不能形成第二份现行规范。Stdlib 拆分及 CLI 工单以实际责任仓库更新。

最终原则：**生态的复杂性留在独立领域与组件，原型的表达复杂性限制在可读文档，生产的复杂性由成熟框架承担。局部缺陷局部修复，领域问题领域协调，公共问题由真正的契约所有者修复。**
