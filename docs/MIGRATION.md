# 迁移台账

## 权威盘点

历史主分支最后提交为 `1c55baabd570e18b9da8f9057cef2f848e176690`，v2.9.1 的发布提交为 `ffe06d7afab7acaed87be5ff16edddc37da536a2`。自建 Git 已核验旧标签和主分支。旧实现、文档、示例、测试与工作流以历史提交为迁移资料。[历史主分支末尾](https://git.cflmy.cn/cflmy/marqdo/src/commit/1c55baabd570e18b9da8f9057cef2f848e176690)

| 已有仓库 | 审计时默认分支 | 发行或证据 |
| --- | --- | --- |
| [marqdo-cli](https://git.cflmy.cn/cflmy/marqdo-cli) | `master` | v3.0.0-rc.4 |
| [marqdo-extension-conformance](https://git.cflmy.cn/cflmy/marqdo-extension-conformance) | `main` | 有独立固定 fixture |
| [marqdo-knowledge](https://git.cflmy.cn/cflmy/marqdo-knowledge) | `master` | v0.1.0-beta.3 |
| [marqdo-lint](https://git.cflmy.cn/cflmy/marqdo-lint) | `main` | v0.2.0 |
| [marqdo-prove](https://git.cflmy.cn/cflmy/marqdo-prove) | `master` | v0.1.0-beta.1 |
| [marqdo-view](https://git.cflmy.cn/cflmy/marqdo-view) | `master` | v0.3.0 |

CLI gitlink 采用 RC.4 正式发布源码，以匹配冻结归档和现有 View/Lint 契约。
其文档后续提交曾用于 Knowledge 的独立 CI；生态候选以自身测试判定实际组合。
Conformance 使用 CLI 已固定的真实 fixture 提交。

## 标准库和扩展目录

主仓库的 [标准库目录](../stdlib/README.md) 和 [扩展目录](../extensions/README.md) 提供各组件的开发入口。各组件独立维护源码与验收资料，接入状态以 [组件目录](../components.toml) 为准。

**标准库** · [core](https://git.cflmy.cn/cflmy/marqdo-std-core) · [collections](https://git.cflmy.cn/cflmy/marqdo-std-collections) · [text](https://git.cflmy.cn/cflmy/marqdo-std-text) · [encoding](https://git.cflmy.cn/cflmy/marqdo-std-encoding) · [data](https://git.cflmy.cn/cflmy/marqdo-std-data) · [io](https://git.cflmy.cn/cflmy/marqdo-std-io) · [fs](https://git.cflmy.cn/cflmy/marqdo-std-fs) · [system](https://git.cflmy.cn/cflmy/marqdo-std-system) · [net](https://git.cflmy.cn/cflmy/marqdo-std-net)

**扩展** · [math](https://git.cflmy.cn/cflmy/marqdo-ext-math) · [linalg](https://git.cflmy.cn/cflmy/marqdo-ext-linalg) · [quantum](https://git.cflmy.cn/cflmy/marqdo-ext-quantum) · [http](https://git.cflmy.cn/cflmy/marqdo-ext-http) · [websocket](https://git.cflmy.cn/cflmy/marqdo-ext-websocket) · [ai](https://git.cflmy.cn/cflmy/marqdo-ext-ai) · [mcp](https://git.cflmy.cn/cflmy/marqdo-ext-mcp) · [crypto](https://git.cflmy.cn/cflmy/marqdo-ext-crypto) · [database](https://git.cflmy.cn/cflmy/marqdo-ext-database) · [storage](https://git.cflmy.cn/cflmy/marqdo-ext-storage) · [cache](https://git.cflmy.cn/cflmy/marqdo-ext-cache) · [security](https://git.cflmy.cn/cflmy/marqdo-ext-security) · [web](https://git.cflmy.cn/cflmy/marqdo-ext-web) · [browser](https://git.cflmy.cn/cflmy/marqdo-ext-browser) · [document](https://git.cflmy.cn/cflmy/marqdo-ext-document) · [archive](https://git.cflmy.cn/cflmy/marqdo-ext-archive) · [engineering](https://git.cflmy.cn/cflmy/marqdo-ext-engineering) · [regex](https://git.cflmy.cn/cflmy/marqdo-ext-regex) · [datetime](https://git.cflmy.cn/cflmy/marqdo-ext-datetime)

## 阶段与阻塞

| 原分支 | 独立仓库与迁移状态 |
| --- | --- |
| `gh-pages` | 官网架构、v2.9.1 源内容和部署快照均已迁入 marqdo-site。原分支保留现有公共入口，部署切换以独立托管和路由验收为准。[托管切换工单](https://git.cflmy.cn/cflmy/marqdo-site/issues/1) |
| `vscode-extension` | 扩展目录历史归档到 marqdo-vscode 的 `legacy-v2.8.0`。现行 CLI 适配和新版 VSIX 由扩展仓独立开发和验收。[首版实现工单](https://git.cflmy.cn/cflmy/marqdo-vscode/issues/1) |

| 阶段 | 当前范围 | 后续验收 |
| --- | --- | --- |
| M0 盘点 | 自建 Git、历史标签、已有六组件和新仓位 | 通过真实远端候选持续维护台账 |
| M1 集成骨架 | 六个 gitlink、布局门禁和 RC.4 组合 | 查看候选 Woodpecker 运行及各组件独立 CI |
| M2 标准库切换 | planned，CLI RC.4 提供官方完整 Bundle | [CLI 外部 Bundle](https://git.cflmy.cn/cflmy/marqdo-cli/issues/11) 和 [九组件切换](https://git.cflmy.cn/cflmy/marqdo/issues/2)，包括 HTTP、Regex、Archive 迁出与同步删除旧路径 |
| M3 跨仓闭环 | 工单、人工候选、同 SHA 门禁 | [跨仓库 CI](https://git.cflmy.cn/cflmy/marqdo/issues/3) 跟踪真实缺陷闭环、候选通知鉴权、去重和影响分析 |
| 后续领域 | 空仓和预期职责 | 逐组件开展最小真实功能、独立 CI、制品与接入 |

### 标准库切换顺序

1. CLI 打包器与产品测试接受单一显式外部 Bundle 输入。内核独立编译；制品仍附带一个校验过的标准库。
2. 标准库在各组件维护唯一生产源码和自测。组装拒绝冲突、符号链接或测试污染，调用 CLI 权威打包器。std-io 维护 print/打印 的别名元数据。
3. HTTP、Regex、Archive 的真实替代和授权回归先行。候选同时移除 CLI 中旧领域宿主服务和旧导入。
4. 验证同输入两次组装结果、摘要及路径负例、C1/C2/C3、独立冷安装和真实工具集成后，再切换基础发行。当前状态为 UNVERIFIED。

## 现行 CLI 标准库迁移归属

| CLI 当前路径 | 唯一目标组件 | 新 Bundle 路径 / 处理 |
|---|---|---|
| `clock.mq.md` | std-system | `system/clock.mq.md` |
| `compress.mq.md` | archive 扩展 | 不入最终标准库；先建立等价扩展与替代后端再删除宿主 convenience service |
| `data/csv.mq.md` | std-data | `data/csv.mq.md` |
| `data/json.mq.md` | std-data | `data/json.mq.md` |
| `data/table.mq.md` | std-collections + std-data（函数级切分） | 集合算法 `collections/table.mq.md`；GFM 序列化 `data/gfm.mq.md`；不复制函数 |
| `datetime.mq.md` | std-system | `system/datetime.mq.md`（UTC/基础日历） |
| `demo.mq.md` | std-io 的 examples 或主仓库综合示例（择一） | 不作为公开安装模块；若只是 IO 自测，归 std-io |
| `env.mq.md` | std-system | `system/env.mq.md` |
| `format.mq.md` | std-io | `io/format.mq.md` |
| `fs/file.mq.md` | std-fs | `fs/file.mq.md` |
| `io.mq.md` | std-io | `io/index.mq.md`；官方 print alias 指向唯一权威 `写值`（迁移时同步生成元数据） |
| `io/console.mq.md` | std-io | `io/console.mq.md` |
| `io/stream.mq.md` | std-io | `io/stream.mq.md` |
| `net/http.mq.md` | http 扩展 | 转为普通 Package 的 HTTP 实现，依赖 std-net；保持报文算法及真实 TCP 对照测试 |
| `path.mq.md` | std-fs | `fs/path.mq.md` |
| `process.mq.md` | std-system | `system/process.mq.md` |
| `random.mq.md` | std-system | `system/random.mq.md` |
| `regex.mq.md` | regex 扩展 | 单一正则模块，迁移纯子集及外部引擎；替换后删除核心宿主服务 |
| `text.mq.md` | std-text | 重复 UTF-8 实现合并入 `text/utf8.mq.md`；新增通用 `text/text.mq.md` 不复制 encoder |
| `text/utf8.mq.md` | std-text | `text/utf8.mq.md`（唯一权威 encoder/decoder） |

## v2 功能历史索引

按表中的归属逐项拆分工单，记录来源路径、目标接口、正负例、状态和真实测试提交。

| 旧版 lib 模块 | 目标 | 迁移决策 / 主要缺口 |
|---|---|---|
| archive | archive | ZIP/TAR 等安全归档；现有 gzip 迁移，测试路径穿越 |
| browser | browser | 31 个 H2 所涉 DOM、存储、事件、fetch、WS、下载；不复制旧 WASM runtime |
| bytes | std-text / std-encoding | 字节值、slice、UTF-8 归 Text，base64 归 Encoding，单一 byte 表示 |
| cli | std-system | 应用参数 parse，非 CLI 自身 flag parser |
| csv | std-data | 已有 parse/format；补非法输入与多行测试 |
| datetime | std-system / datetime | UTC 基础已有；时区、DST 和高级日历明确拆分 |
| document | document | sections、tables、links、code blocks、replace；通过受支持静态事实与显式文件编辑 |
| encoding | std-encoding | Base64/Base32/Hex 编解码缺失 |
| foreign | CLI 现有 Foreign / 独立可选会话扩展（待 ADR） | one-shot run/call 已有；不恢复 set_cmd/register 等隐式动态机制；session/replay 需真实设计 |
| fs | std-fs / document / engineering | 40 个 H2 涵盖文件操作；新版14个 H2；二进制、递归、原子、临时、读写组合；文本 patch/工程 sync 归相应扩展 |
| hash | crypto | SHA/HMAC 等，密码学不进入内核 |
| html | web | escape/unescape 作为纯 Web 辅助模块 |
| json | std-data | parse/stringify 已有；get/keys/set 等可复用 collections；严格 Result |
| log | std-system | 结构化、level、输出门面，不引入服务框架 |
| math | std-core / math | abs/min/max/clamp 基础归 Core；数值、超越、公式、CAS、绘图归 Math；随机归 System |
| net | std-net / http / ai / document | TCP/TLS 基础归 Net；HTTP/SSE/cookie/multipart/download 归 HTTP；OpenAI 特定 SSE 归 AI；Markdown 解析归 Document 或公开静态事实 |
| path | std-fs | join/normalize 等已有；relative/with_extension 等补齐 |
| plugin | CLI 现行扩展选择机制 | 不迁移 load/unload 自动加载；使用显式 descriptors、lock、grant |
| re | regex | 查找/替换/分割/捕获和现有纯子集、成熟后端，取消宿主 convenience 重复 |
| result | std-core | ok/error/value_or，统一现行可恢复错误约定 |
| secrets | std-system / crypto | token 生成归 System，密钥/密码学安全操作归 Crypto；不泄漏 secret |
| stats | math | mean/median/stdev；定义样本与总体、空集合、精度 |
| subtask | CLI Structured Tasks（待开发） / std-system | spawn/join/cancel 等先验证通用语义；库只薄封装，不移植旧 scheduler |
| sys | std-system / CLI 公共事实 | env/args/process/clock；旧 stream_publish 和 meta 由现行 Observation/Analysis 或真正应用输出处理；不恢复隐式环境写入 |
| table | std-collections / std-data | 60 个 H2 入口（List/Map/Table 混合）；值操作归 Collections，GFM/数据格式归 Data |
| text | std-text | trim/split/join、contains/replace、大小写、repeat/pad 等缺口 |
| time | std-system | now/UTC parse 已部分有；sleep 仅在受控原语可用后实现 |
| toml | std-data | parse/stringify/try_parse，实际支持标准及错误边界 |
| url | http | URL parse/query/percent 与同协议归属，std-net 不重复 |
| uuid | std-system | v4 使用 OS entropy；检查版本/variant |
| writeback | document / engineering | 显式有界编辑、来源摘要及冲突检查；不恢复隐式自修改/自动 oracle writeback |
| lint/narrative | lint Tool | 静态诊断与显式修复，不进 CLI runtime |

### 领域与插件

| 旧目录 | 目标及处理 |
|---|---|
| `ext/ai` + `plugins/llm` / `plugins/agent` | AI 模型协议、MCP 独立迁移；agent 97 个 H2 涉及 workflow/memory/工具/恢复等，按真实通用能力拆分，不整体复制。qdagent 应用管理其自己的最小工作流、记忆、意图和 SKILL，不作为通用标准库。知识检索、证明分别由对应 Tool；sandbox 用 CLI 的实际保证，不伪造 OS 隔离。 |
| `ext/linalg` + `plugins/linalg` | Linalg；迁移数值算法、复数及矩阵符号知识；重新适配 ABI，减少重复。 |
| `ext/quantum` + `plugins/quantum` | Quantum；gate/state/circuit/simulate/density/viz，移除与 Linalg 重复的实现或建立经测试的后端共享边界。 |
| `ext/net` | HTTP、WebSocket、URL，低层 TCP/TLS 归 Std Net。 |
| `ext/data` | Database/Cache/Storage；form/admin/api/view 的 Web 部分归 Web，数据验证及查询归唯一相应库。旧 Web Go plugin 的 DB、storage、cache 子模块分别迁移。 |
| `ext/security` | Security + Crypto；旧 Go plugin auth/OIDC/RBAC/tenant/password 按归属拆分，不混入 Host capability。 |
| `ext/web` + `plugins/web` | Web、HTTP、WebSocket、Browser、Database、Security 等拆分；保留原生 GFM 组件与布局表达，删除 monolithic Go plugin 调度与重复模块，重新适配公开协议。 |
| `ext/engineering` + `plugins/engineering` | Engineering；Git 操作、文件、archive 分配给已有权威库，避免重复平台操作。 |
| `crates/marqdo-wasm`、旧 browser examples | Browser 组件参考；语言核心可移植性需要 CLI 工单，而非复制旧执行器。 |
| 旧 `src/knowledge`/proof/view/lint/对应历史 CI 与 examples | 独立 Tool，只迁移所需知识、算法和 fixture；当前公共 Analysis/Observation 优先；新旧协议不并存。 |
