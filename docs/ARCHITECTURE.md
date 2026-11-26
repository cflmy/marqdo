# 架构与所有权

自建 Git `git.cflmy.cn/cflmy` 是代码、工单和合并的权威，`ci.cflmy.cn` 是 CI 权威，GitHub 仅作镜像或分发渠道。主仓库维护生态组件关系、精确组合、跨组件验收、发行与责任治理；领域聚合仓库维护本领域的合同与组合验收；功能仓库拥有唯一实现、测试和缺陷修复。

## 三种关系，不得混用

| 关系 | 表达载体 | 用途 | 约束 |
| --- | --- | --- | --- |
| 治理树 | `.gitmodules` + Git tree gitlink + 每层 `components.toml` | 源码归属、父级与提交身份 | 每个组件只有一个父级；全局 ID 与权威远端唯一；Gitlink 是提交身份唯一权威，`.gitmodules` 是权威远端唯一记录 |
| 构建/制品依赖 DAG | `requires` 与真实 package/Cargo/Tool/Bundle 依赖 | 编译、打包、发行装配顺序 | 可以跨治理树；必须有向无环；聚合节点不是可被程序 import 的包 |
| 协同验收关系 | `checks_with` | 跨组件测试与消费者回归 | 不是安装依赖；允许互相测试；调度固定点必须去重且不得互相等待形成死锁 |

组件关系不能由目录物理位置推导，也不能把兄弟仓库的源码路径当作公开依赖。`requires` 只表达真实直接构建或制品依赖，禁止把计划假想写成实际依赖。

## 治理树与分层

治理树最多两级 Gitlink：**根仓库 → 领域聚合仓库 → 功能仓库**（主仓库→领域→功能，深度 2）。新增三级 gitlink 需要生态 ADR 和真实跨仓维护收益，默认拒绝。目录层级不等于运行时依赖层级。

- **领域（`kind = "domain"`）**：工程治理角色，不是语言层也不是运行协议。领域仓库只登记直属子组件，可拥有 `components.toml`、`.gitmodules`、领域公共规范、跨组件集成测试、综合示例、发布装配脚本、ADR 与维护指南；禁止在其中建立 renderer、解析器、数据库驱动等生产实现，也禁止把子仓库源码复制进普通目录。领域 `requires = []`，`release` 为 `none`（普通领域）或 `base`（Stdlib Bundle 聚合）。
- **功能组件**：`kind` 仍为 `core/stdlib/extension/tool/test`，在责任仓库独立实现、测试和发版；`release` 与 `kind` 的对应关系不变（base/optional/none）。
- **唯一归属**：一个功能只有一个权威实现，一个组件只有一个治理归属，一个公共边界只有一套现行合同。已有仓库直接复用，不能因为命名不符创建重复实现。
- **`planned` 纪律**：已有仓库可以通过 gitlink 固定已发布的开发提交。尚无提交的仓库登记在 [迁移台账](MIGRATION.md)。组件通过联合验收后才晋升为 `active` 并登记实际 `entry`。将直属组件迁入领域时，在同一根候选转移清单和 gitlink，保证一个组件只有一个父级。
- **聚合准入**：功能拆分须至少有一种独立价值（独立维护责任、显著不同依赖/安全/平台边界、独立版本与制品、可重复独立验收）。若两个仓库几乎每个 PR 都必须一起改动且接口未稳定，应先收敛合同或合并，不能无限拆分。
- **工具目录**（`tools/`）暂不聚合，现有四个 Tool 保持原路径。

**当前状态**（以 [components.toml](../components.toml) 与 [迁移台账](MIGRATION.md) 为准）：根仓库有六个直属 active 组件（`cli`、`conformance`、`view`、`knowledge`、`prove`、`lint`）。九个标准库和十八个独立扩展是根层的 planned 子模块。`extensions/web` 是 planned 领域子模块，Design 和 Preview 由 Web 领域仓挂载。`marqdo-stdlib` 聚合和 `web-components` 仍在计划中。

## 语言与产品层（不因仓库变多而改变）

| 层 | 权威所有者 | 允许 | 禁止 |
| --- | --- | --- | --- |
| L0 语言核心 | CLI | Markdown 映射、AST、IR、Value、Memory、求值 | Web/AI/数据库专属关键字 |
| L1 平台能力 | CLI Capability Host | 权限、预算、通用资源/扩展机制 | 网页布局和业务协议 |
| L2 标准库 | Stdlib 各功能仓库 | 基础发行默认安装的通用低层算法及薄封装 | 依赖领域扩展或大型专业运行环境 |
| L3 扩展 | 各领域功能仓库 | 可选专业算法、协议和资源后端 | 修改或复制语言内核 |
| L4 应用 | 用户项目 | 使用已发布能力实现具体任务 | 作为默认基础发行依赖 |
| Tool（旁路） | 独立工具 | 通过公开 CLI 协议处理源码、分析及制品 | 伪装成 Runtime 插件，反向侵入核心 |

## 权威记录与递归校验

每个拥有子模块的聚合仓库只登记直接子组件；普通功能仓库不需要另建治理清单。配置与身份的唯一来源：

| 记录 | 权威内容 |
| --- | --- |
| `components.toml`（每层） | 组件 ID、父级路径、kind、state、owner、scope、`requires`、`checks_with`、release、active 入口 |
| `.gitmodules`（每层） | 直属子组件权威远端 |
| Git tree gitlink | 精确提交身份 |
| Cargo.lock / `marqdo.lock` / 现行 Bundle manifest | 相应依赖与制品身份 |

主仓库 `scripts/check-layout.py` 校验 catalog、gitlink、远端与文件白名单。`--checkouts` 递归读取每个已挂载领域的固定提交，逐层验证 gitlink、远端、工作树和活跃入口，并检查全局 ID、依赖边界和无环性。CI 用 `--domain-paths` 在检出子组件前校验领域远端。 [开发流程](DEVELOPMENT.md)

## 各领域排他边界

| 领域 / 功能 | 应拥有 | 不应拥有 |
| --- | --- | --- |
| Stdlib/Core、Collections、Text、Encoding、Data、IO、FS、System、Net | 默认安装的通用低层能力及薄封装 | 专业计算、外部服务、HTTP 协议 |
| Ext-Net（HTTP/WebSocket） | URL、HTTP 客户端/服务端协议、SSE、Cookie、Multipart、WebSocket 帧与生命周期 | Web 页面模型、应用实时协作状态 |
| Web 领域 | 设计模型、快速预览、交付清单与组件复用 | 生产级前端框架、HTTP/DB/认证协议、AI 调度 |
| Browser | 将来可选的真实浏览器执行宿主与平台桥 | 本期 Web 原型必需依赖、第二套解释器 |
| AI / MCP | 模型访问、流式结果、工具调用数据；MCP 发现/调用/传输 | 全量 Agent 调度、HTTP 实现、Agent 框架 |
| Database / Storage / Cache / Auth | 参数化查询与事务、存储接口、TTL 缓存、OIDC/会话/RBAC | Web 后台页面、文件系统原语、替代 Capability Host |
| Crypto / Datetime / Regex / Archive | 成熟密码学封装、IANA 时区、完整正则、安全归档 | 自制不安全加密、重复宿主基础原语 |
| Document / Engineering | 文档显式编辑、工程操作与产物 | 静态分析或编辑器全栈平台 |
| Quantum / Linalg / Math | 量子态/电路/模拟、矩阵/张量/分解、数值与符号数学 | 复制矩阵内核、通用前端框架 |

上表是规划边界，不是开发完成承诺。旧标准库中 `net/http.mq.md`、`regex.mq.md`、`compress.mq.md` 的迁移必须有真实替代实现与验收才能退出原归属，不得先删造成回归，也不得长期维护两个官方实现。

## Web 领域

Web 领域的产品定义是**文档原生、可交互的前端设计原型及工程交付**，不是新的通用前端框架。GFM 表描述组件关系，普通 Markdown 描述内容与意图，链接表达精确的组件引用与设计依赖，有限场景表达交互原型；生产级前端由成熟框架完成。HTTP、数据库、认证、浏览器运行时仍是独立通用能力，不是原型工具的强制依赖。

- **唯一作者合同**：[marqdo-ext-web 的 WEB-DESIGN](https://git.cflmy.cn/cflmy/marqdo-ext-web/src/branch/main/docs/WEB-DESIGN.md)。主仓库、组件仓库和作者文档都引用它，不并列维护第二份现行合同；合同变更在该领域走 ADR 并与 Design/Preview 同候选验证。
- **领域聚合** `marqdo-ext-web`（planned）：治理、合同、示例、组合验收；无运行时入口，不实现 renderer/parser。
- **Web Design** `marqdo-web-design`（planned）：独立 Rust library，唯一解释作者合同、消费真实 CLI Analysis、产出设计模型/诊断/handoff；不启动服务、不渲染 DOM、不复制 CLI parser。
- **Web Preview** `marqdo-web-preview`（planned）：独立 executable + `tool.json`，注册唯一 `web` 命令；消费 Design 公开 API，生成离线安全预览与有限状态交互；不自行解释表格、不写任务专用 JS、不启动生产服务器。
- **组件模板** `web-components`：候选，只有至少两个独立真实复用模板及稳定合同后才准入，不阻塞首版。
- **边界**：首版不迁移 WASM、不重新实现语言、不要求旧 plugin 回调，不新增 CLI Web 关键字或 `@component` 语义；复杂视觉、真实登录/支付/数据库/网络和任意状态管理交给生产实现。Web 领域已通过 gitlink 固定开发提交，状态为 `planned`。 [开发流程](DEVELOPMENT.md)

## 新增能力与公共边界

先复用职责匹配的组件。需要独立维护、依赖、安全或发行边界时，为新仓库发起评审；加入标准库须证明基础发行默认需要、至少三个领域真实复用、依赖闭包受控，并说明错误、授权和预算语义。

需要新增 builtin、HostOp、值类型、语法或公共协议字段时，在 CLI 提交 ADR，写明现有机制为何不足、最小复现、三个领域用例、替换及正负例。Web 专属需求不得进入 CLI 核心；设计领域能自己解释的内容留在 Web 合同。公开接口破坏性更新按同一候选组合切换产生者和消费者，保留单套现行协议；历史行为通过固定版本的历史发行复现。

## 架构门禁

| 门禁 | 检查与拒绝条件 | 权威 |
| --- | --- | --- |
| 归属唯一 | Git tree/catalog 递归一致，一个全局 ID 与远端只出现一次，不允许同功能两个官方实现 | 根 |
| 聚合纯度 | 根与领域聚合不含生产实现；新增文件须落在治理/集成/示例职责 | 根/领域 |
| 深度 | 超过两级 Gitlink 拒绝，除非批准 ADR 先更新单一规则 | 根 |
| 真实依赖 | active 不依赖 planned；requires DAG 无环；无父 checkout 或跨仓私有源码依赖 | 根/组件 |
| 标准库准入 | 基础发行默认必需、低层、至少三类真实消费者、依赖与授权有界；新功能默认 extension | Stdlib/根 |
| 内核准入 | 新 builtin/HostOp/语法/协议字段须 ADR、最小复现、三个不相关领域用例；Web 专属需求不进入 CLI | CLI |
| Web 作者面 | 不新增核心语法、重复表头方言、隐式解析普通文档、HTML 注入默认通道 | Ext-Web/Design |
| 通用渲染 | 新普通组件由设计文档复用实现；不得新增任务/页面/业务专用 renderer 分支 | Preview |
| 原型诚实性 | 模拟明确标注；无真实后端、无假执行、无伪造生产测试 | Web |
| 独立产品 | 组件能独立构建；只有实际产品需要才创建新仓库 | 各级 |
| 现行合同唯一 | 不引入旧协议 fallback、版本数组、旧语法探测；配对候选一次切换 | 各级 |
| 默认安装规模 | 基础制品不引入可选 Tool/领域环境；体积、构建时间及依赖增长记录基线 | 根/Stdlib |
| 范围控制 | 每个 PR 关联责任工单、复现与验收；无关重构拆独立任务 | 各级 |

每次架构变更须回答：谁是唯一责任方？是否必须新增公开边界？如何证明现有机制不足？能否在单仓修复？新功能能否独立测试？删除或替代了什么？回答不明确则不进入开发。

跨组件测试以真实 CLI/Worker、独立消费者和现行权限配置验证。Native 后端与外部工具按 CLI 安全边界执行，权限、隔离和预算声明以可验证的实际行为为准。
