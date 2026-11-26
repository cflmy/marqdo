# Marqdo 各仓库任务

按仓库领取任务。标为 **大改** 的条目可涉及模块拆分、实现重写或发布切换。责任仓库应先限定本轮交付范围，涉及公共接口时与实际消费者一起验收。

生态治理采用“根 → 领域聚合 → 功能组件”最多两级 Gitlink：功能仓库实现与修复、领域聚合仓库维护合同与组合验收、主仓库固定 gitlink、治理与发行。`planned` 组件可以挂载真实开发提交，实际入口随验收登记。当前没有 active 领域；Web 领域为 planned（聚合仓 `marqdo-ext-web`，子组件 `marqdo-web-design`/`marqdo-web-preview` 在其 catalog 登记）。先推进 CLI 外部 Bundle、九个标准库，以及 HTTP、Regex、Archive 的迁出；这些改动在同一候选组合中验证后切换基础发行。官网和 VS Code 可独立推进。[迁移台账](MIGRATION.md) · [开发流程](DEVELOPMENT.md)

## 集成和语言

| 仓库 | 要完成的任务 |
| --- | --- |
| `marqdo` | 固定已验收的组件组合与领域 gitlink，完成跨仓库工单闭环、CI 和生态发行验证。推进递归治理校验（domain 角色与全局依赖 DAG）、声明驱动集成计划、统一 CLI 制品输入与递归影响分析；保留现有 conformance/View/Knowledge/Prove/Lint 真实门禁。 |
| `marqdo-cli` | **大改**。拆分内核构建和基础发行装配，接受显式外部标准库 Bundle。完成领域能力迁出后，移除旧标准库入口和被替代的专用宿主实现。 |
| `marqdo-extension-conformance` | 更新真实 CLI 的公共契约测试，覆盖 Bundle 切换、五类扩展机制及其权限边界。 |

## 领域聚合（planned）

领域聚合仓库只做治理、唯一合同、示例、跨组件验收与交付协调，不含生产实现。激活前保持 planned，子组件在其自身 `components.toml` 登记，根 catalog 不重复登记。

| 仓库 | 要完成的任务 |
| --- | --- |
| `marqdo-ext-web` | Web 领域聚合：维护唯一作者合同 `docs/WEB-DESIGN.md`、§8 组合示例与领域 CI；验证真实 Analysis 风险（WEB-00），协调设计合同变更与 Design/Preview 配对候选；在子组件通过 WEB-01..04 后以领域 gitlink 接入根（WEB-05）。 |
| `marqdo-web-design` | WEB-01：消费真实 CLI Analysis 3，解释精确组件引用、递归布局、有限场景与依赖，输出稳定诊断、组件图和源身份；WEB-03：handoff、单组件切片与漂移标识。独立 Rust library，禁止复制 parser/启动服务。 |
| `marqdo-web-preview` | WEB-02：通用离线预览渲染器与有限状态交互、响应式、安全负例、Tool 打包；作为唯一 `web` 命令入口消费 Design 公开 API，禁止任务专用 JS、第二套解析或生产服务器。 |

## 工具和产品

| 仓库 | 要完成的任务 |
| --- | --- |
| `marqdo-knowledge` | 基于公开 Analysis 完善可复现的知识编译、索引和检索，验证独立发行。 |
| `marqdo-prove` | 完善证明依赖、Lean 证书生成和独立重放，清楚展示各项证明的核验状态。 |
| `marqdo-view` | 维护文档阅读、显式运行、Observation 和静态导出，验证与现行 CLI 及官网构建的组合。 |
| `marqdo-lint` | 扩展静态规则和诊断定位，为可修复规则提供显式修复。 |
| `marqdo-site` | **大改**。用现行 CLI 和 View 重建官网与入门内容，验证新站和旧链接后切换托管。 |
| `marqdo-vscode` | **大改**。按现行 CLI 重做诊断和运行适配，先完成高亮、静态诊断、显式运行和可安装的 VSIX。 |

## 标准库

每个仓库交付可独立测试的模块，共同组成默认 Bundle。聚合仓库 `marqdo-stdlib` 尚未创建，九个 `std-*` 当前保持根 catalog 的 planned 直属记录；CLI 内置标准库仍是现行发行权威，迁移完成前不得声称拆分完成。

| 仓库 | 要完成的任务 |
| --- | --- |
| `marqdo-std-core` | 实现共用 Result 和基础数值操作。 |
| `marqdo-std-collections` | 实现 List、Map 和 Table 的通用集合操作，从数据格式处理中拆出表格算法。 |
| `marqdo-std-text` | 统一 Unicode 文本、字节值切片和 UTF-8 编解码。 |
| `marqdo-std-encoding` | 提供 Base64、Base32 和 Hex，并定义非法输入的行为。 |
| `marqdo-std-data` | 迁入 JSON、CSV 和表格格式转换，补齐 TOML 与 GFM。 |
| `marqdo-std-io` | 迁入 Console、Stream 和格式化，负责官方 `print` 与 `打印` 别名。 |
| `marqdo-std-fs` | 迁入文件、路径和目录操作，核验读写边界。 |
| `marqdo-std-system` | 迁入时钟、UTC、环境、进程和随机能力。 |
| `marqdo-std-net` | 提供有界 TCP、UDP、TLS 和字节流，供上层协议复用。 |

## 扩展

每个仓库先完成一个可安装、可验证的用例。历史实现可用于迁移取证，公开接口按现行契约验收。

| 仓库 | 要完成的任务 |
| --- | --- |
| `marqdo-ext-math` | 建立数值算法、统计、符号数学和绘图能力，先交付可验证的数值用例。 |
| `marqdo-ext-linalg` | 迁移复数、矩阵和分解，向量子组件提供可复用的线性代数接口。 |
| `marqdo-ext-quantum` | 迁移量子电路、态向量、采样与模拟，验证代表性电路结果。 |
| `marqdo-ext-http` | **大改**。把现有 HTTP 迁为基于 std-net 的独立包，覆盖 URL、消息编解码、请求生命周期和 SSE。 |
| `marqdo-ext-websocket` | 交付基于 HTTP 与网络流的 WebSocket 握手、帧和连接生命周期。 |
| `marqdo-ext-ai` | **大改**。从旧 AI 实现拆出模型协议、流式输出和工具调用数据。 |
| `marqdo-ext-mcp` | 交付 MCP 传输、能力发现和受控工具调用。 |
| `marqdo-ext-crypto` | 封装成熟密码学实现，先交付一种带标准测试向量的能力。 |
| `marqdo-ext-database` | 交付一个支持参数绑定和事务的数据库驱动。 |
| `marqdo-ext-storage` | 定义本地与对象存储接口，先验证一套读写和错误处理流程。 |
| `marqdo-ext-cache` | 交付带 TTL 和失效行为的缓存。 |
| `marqdo-ext-security` | 交付应用身份、会话与授权，复用加密扩展。 |
| `marqdo-ext-web` | 已改为 Web 领域聚合仓库，任务见「领域聚合」小节；不再作为 HTTP 页面/路由渲染实现。 |
| `marqdo-ext-browser` | **大改**。按现行运行时边界适配浏览器宿主、WASM、DOM 和事件。 |
| `marqdo-ext-document` | 交付 Markdown 结构查询与显式编辑接口。 |
| `marqdo-ext-archive` | 实现归档与安全解包，为 CLI 移除旧压缩入口提供可验证的替代。 |
| `marqdo-ext-engineering` | 拆分工程操作、文件补丁和产物处理，先交付一个独立工程用例。 |
| `marqdo-ext-regex` | 提供匹配、捕获、扫描与替换，为 CLI 移除旧正则入口提供可验证的替代。 |
| `marqdo-ext-datetime` | 交付 IANA 时区、夏令时和高级日历能力，与标准库 UTC 接口对接。 |
