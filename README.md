# Marqdo

**Marqdo 是一门以 Markdown 为源文档的编程语言。** 用 `.mq.md` 可以把业务规则、公式、解释和可执行步骤写在同一份文档中。读者用 Markdown 阅读器查看正文，开发者用 Marqdo 检查、运行和验证其中的程序。这个仓库维护生态组件的集成、验收和发行入口。

生态治理采用有限分层：主仓库固定领域聚合仓库，领域聚合仓库固定本领域功能仓库，最多两级 Gitlink。主仓库和聚合仓库只做治理、组合与验收，不实现子组件业务；Web 领域把设计文档直接作为原型与工程交付的源材料，详见 [架构边界](docs/ARCHITECTURE.md)。

## 为什么开发 Marqdo

规则通常写在说明文档里，实现放在脚本里，结果再写进报告。三份内容分开维护后，读者很难确认某个结果用了哪条规则，也很难重新验证。Marqdo 让解释和计算随同一份源文档修改、评审和运行。它适合写带示例的算法说明、可重跑的分析流程，以及需要交代验证过程的知识文档。

## 主要特性

- **Markdown 原生**。文档保留 CommonMark/GFM 的结构，并支持公式。在 `.mq.md` 中，以 `@exec` 标明可执行区，以 `@knowledge` 标明说明与知识区。正文可以直接阅读。
- **先检查，再运行**。`marqdo check` 静态检查语法与调用，`--refs` 检查文档引用。Analysis 提供结构化分析结果，供编辑器和独立工具使用。
- **可观察的运行**。Rust 解释器显式运行文档代码，Observation 记录执行事件与结果。文件、网络、进程等外部操作受运行模式与能力授权约束。
- **可组合的生态**。基础发行包含标准库。程序可以通过锁文件固定包与领域扩展的来源和摘要。阅读、知识索引、证明和静态诊断由独立工具提供。

## 快速开始

从 [CLI 发行页](https://git.cflmy.cn/cflmy/marqdo-cli/releases) 下载适合平台的发行包。把解包目录加入 `PATH`，保持 CLI、Worker 和旁置 `stdlib/` 的目录结构。将下面的内容保存为 `hello.mq.md`。

````markdown
# 问候示例

这一节说明程序的用途。下方函数由 CLI 执行。

## @exec main

`params()`

`打印("你好，Marqdo")`
````

```sh
marqdo check hello.mq.md
marqdo run hello.mq.md
```

```text
你好，Marqdo
```

[CLI 使用指南](https://git.cflmy.cn/cflmy/marqdo-cli) · [语言规范](https://git.cflmy.cn/cflmy/marqdo-cli/src/branch/master/spec/mapping.md)

## 仓库与职责

### 已接入的核心和工具

| 仓库 | 职责 |
| --- | --- |
| [marqdo](https://git.cflmy.cn/cflmy/marqdo) | 生态主仓库，维护仓库索引、固定提交组合、跨仓库验收与发行 |
| [marqdo-cli](https://git.cflmy.cn/cflmy/marqdo-cli) | 语言规范、Rust 解释器、CLI、运行权限和公开协议 |
| [marqdo-extension-conformance](https://git.cflmy.cn/cflmy/marqdo-extension-conformance) | 用冻结的真实 CLI 独立验收包、插件、Provider、Foreign 和 Tool |
| [marqdo-knowledge](https://git.cflmy.cn/cflmy/marqdo-knowledge) | 使用公开 Analysis 编译、索引和检索知识文档 |
| [marqdo-prove](https://git.cflmy.cn/cflmy/marqdo-prove) | 处理证明依赖、Lean 证书与独立重放 |
| [marqdo-view](https://git.cflmy.cn/cflmy/marqdo-view) | 阅读和预览文档、显示执行观察、静态导出 |
| [marqdo-lint](https://git.cflmy.cn/cflmy/marqdo-lint) | 静态规则、诊断定位与显式修复 |

### 独立产品

| 仓库 | 职责 |
| --- | --- |
| [marqdo-app-harness](https://git.cflmy.cn/cflmy/marqdo-app-harness) | 官方单用户应用：生态安装、文稿程序化执行、私人知识库与 `/mqdh` 工作台（DSH 下游） |
| [marqdo-site](https://git.cflmy.cn/cflmy/marqdo-site) | 官网、入门文档、静态站点构建与托管 |
| [marqdo-vscode](https://git.cflmy.cn/cflmy/marqdo-vscode) | VS Code 文件关联、编辑体验、CLI 集成与 VSIX 发布 |

### 标准库

标准库组件负责低层通用能力，验收后共同组成基础发行的标准库 Bundle。`stdlib/` 的子模块固定各独立仓库的开发提交，状态为 `planned`。 [标准库目录](stdlib/README.md)

| 仓库 | 职责 |
| --- | --- |
| [marqdo-std-core](https://git.cflmy.cn/cflmy/marqdo-std-core) | Result 与基础数值 |
| [marqdo-std-collections](https://git.cflmy.cn/cflmy/marqdo-std-collections) | List、Map 和 Table 值操作 |
| [marqdo-std-text](https://git.cflmy.cn/cflmy/marqdo-std-text) | Unicode 文本、字节值与 UTF-8 |
| [marqdo-std-encoding](https://git.cflmy.cn/cflmy/marqdo-std-encoding) | Base64、Base32 和 Hex |
| [marqdo-std-data](https://git.cflmy.cn/cflmy/marqdo-std-data) | JSON、CSV、TOML 与 GFM 数据转换 |
| [marqdo-std-io](https://git.cflmy.cn/cflmy/marqdo-std-io) | Console、Stream、格式化和官方 print 别名 |
| [marqdo-std-fs](https://git.cflmy.cn/cflmy/marqdo-std-fs) | 文件、目录与路径 |
| [marqdo-std-system](https://git.cflmy.cn/cflmy/marqdo-std-system) | 时钟、UTC、环境、进程与随机 |
| [marqdo-std-net](https://git.cflmy.cn/cflmy/marqdo-std-net) | TCP、UDP、TLS 和网络字节流 |

### 领域扩展

领域扩展负责按需安装的算法、协议和业务能力。`extensions/` 的子模块链接各独立仓库，功能从 `planned` 状态开始开发。 [领域扩展目录](extensions/README.md)

| 仓库 | 职责 |
| --- | --- |
| [marqdo-ext-math](https://git.cflmy.cn/cflmy/marqdo-ext-math) | 数值算法、统计、符号数学及绘图 |
| [marqdo-ext-linalg](https://git.cflmy.cn/cflmy/marqdo-ext-linalg) | 复数、矩阵、分解及数值线性代数 |
| [marqdo-ext-quantum](https://git.cflmy.cn/cflmy/marqdo-ext-quantum) | 量子电路、态向量、采样与模拟 |
| [marqdo-ext-http](https://git.cflmy.cn/cflmy/marqdo-ext-http) | HTTP、URL、SSE 与报文生命周期 |
| [marqdo-ext-websocket](https://git.cflmy.cn/cflmy/marqdo-ext-websocket) | WebSocket 握手、帧与连接语义 |
| [marqdo-ext-ai](https://git.cflmy.cn/cflmy/marqdo-ext-ai) | 模型协议、流式输出与工具调用数据 |
| [marqdo-ext-mcp](https://git.cflmy.cn/cflmy/marqdo-ext-mcp) | MCP 协议、传输、发现和调用 |
| [marqdo-ext-crypto](https://git.cflmy.cn/cflmy/marqdo-ext-crypto) | 成熟密码学实现的封装 |
| [marqdo-ext-database](https://git.cflmy.cn/cflmy/marqdo-ext-database) | SQL 参数绑定、事务和驱动连接 |
| [marqdo-ext-storage](https://git.cflmy.cn/cflmy/marqdo-ext-storage) | 本地和对象存储接口 |
| [marqdo-ext-cache](https://git.cflmy.cn/cflmy/marqdo-ext-cache) | TTL、缓存和失效 |
| [marqdo-ext-security](https://git.cflmy.cn/cflmy/marqdo-ext-security) | 应用身份、会话及权限模型 |
| [marqdo-ext-browser](https://git.cflmy.cn/cflmy/marqdo-ext-browser) | 浏览器宿主、WASM 适配、DOM 和事件 |
| [marqdo-ext-document](https://git.cflmy.cn/cflmy/marqdo-ext-document) | Markdown 结构与显式文档编辑 |
| [marqdo-ext-archive](https://git.cflmy.cn/cflmy/marqdo-ext-archive) | 压缩、安全归档和解包 |
| [marqdo-ext-engineering](https://git.cflmy.cn/cflmy/marqdo-ext-engineering) | 工程操作、文件补丁与产物 |
| [marqdo-ext-regex](https://git.cflmy.cn/cflmy/marqdo-ext-regex) | 正则匹配、捕获、扫描和替换 |
| [marqdo-ext-datetime](https://git.cflmy.cn/cflmy/marqdo-ext-datetime) | IANA 时区、DST 和高级日历 |

### Web 领域聚合

Web 领域以设计文档为源材料，面向可交互的原型预览与工程交付。HTTP、数据库、认证和浏览器运行时各自独立维护。`extensions/web` 链接 active 的 Web 领域仓；Design 和 Preview 已在该领域 catalog 激活并登记真实入口，`web` Tool 通过冷安装验证。 [WEB-DESIGN](https://git.cflmy.cn/cflmy/marqdo-ext-web/src/branch/main/docs/WEB-DESIGN.md)

| 仓库 | 职责 |
| --- | --- |
| [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web) | Web 领域聚合：唯一设计合同、示例、组合验收与交付协调，不含生产实现 |
| [marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design) | 设计文档解释、校验、组件/依赖图、诊断与 handoff（独立 Rust library） |
| [marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview) | 通用离线预览渲染、有限状态交互与 `web` Tool 发行 |

## 参与开发

在负责的独立仓库提交实现和测试。主仓库用 Git 子模块按“根 → 领域 → 功能”最多两级固定提交，组件的发行状态由 catalog 独立记录。需要获取当前组合时，运行：

```sh
git clone https://git.cflmy.cn/cflmy/marqdo.git
cd marqdo
git submodule update --init --recursive
python3 scripts/check-layout.py --checkouts
python3 -m unittest discover -s tests/integration
```

`check-layout.py` 校验每层 catalog、gitlink、权威远端、全局 ID 和依赖图。`--checkouts` 同时核对递归检出的实际提交。`planned` 子模块提供固定开发快照，只有 `active` 组件进入发行验收。

[各仓库任务](docs/REPOSITORY-TASKS.md) · [组件目录](components.toml) · [架构边界](docs/ARCHITECTURE.md) · [开发流程](docs/DEVELOPMENT.md) · [迁移台账](docs/MIGRATION.md) · [架构决策记录](docs/adr/README.md) · [AGENTS](AGENTS.md)

旧版实现与文档可在历史标签查阅。 [v2.9.1 历史树](https://git.cflmy.cn/cflmy/marqdo/src/tag/v2.9.1)
