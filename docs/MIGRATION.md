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
| [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web) | `main` | Web 领域聚合，planned；唯一设计合同在 `docs/WEB-DESIGN.md` |
| [marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design) | `main` | Web Design 组件，planned；初始开发文档 |
| [marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview) | `main` | Web Preview 组件，planned；初始开发文档 |

CLI gitlink 采用 RC.4 正式发布源码，以匹配冻结归档和现有 View/Lint 契约。
其文档后续提交曾用于 Knowledge 的独立 CI；生态候选以自身测试判定实际组合。
Conformance 使用 CLI 已固定的真实 fixture 提交。

## 标准库和扩展目录

主仓库的 [标准库目录](../stdlib/README.md) 和 [扩展目录](../extensions/README.md) 通过 Git 子模块固定独立仓库的开发提交。Web Design 和 Preview 从 Web 领域聚合仓递归获取。是否参加发行验收以 [组件目录](../components.toml) 与该组件所在层的 catalog 为准。

**标准库（planned，聚合仓库 marqdo-stdlib 尚未创建）** · [core](https://git.cflmy.cn/cflmy/marqdo-std-core) · [collections](https://git.cflmy.cn/cflmy/marqdo-std-collections) · [text](https://git.cflmy.cn/cflmy/marqdo-std-text) · [encoding](https://git.cflmy.cn/cflmy/marqdo-std-encoding) · [data](https://git.cflmy.cn/cflmy/marqdo-std-data) · [io](https://git.cflmy.cn/cflmy/marqdo-std-io) · [fs](https://git.cflmy.cn/cflmy/marqdo-std-fs) · [system](https://git.cflmy.cn/cflmy/marqdo-std-system) · [net](https://git.cflmy.cn/cflmy/marqdo-std-net)

**扩展（planned，独立组件）** · [math](https://git.cflmy.cn/cflmy/marqdo-ext-math) · [linalg](https://git.cflmy.cn/cflmy/marqdo-ext-linalg) · [quantum](https://git.cflmy.cn/cflmy/marqdo-ext-quantum) · [http](https://git.cflmy.cn/cflmy/marqdo-ext-http) · [websocket](https://git.cflmy.cn/cflmy/marqdo-ext-websocket) · [ai](https://git.cflmy.cn/cflmy/marqdo-ext-ai) · [mcp](https://git.cflmy.cn/cflmy/marqdo-ext-mcp) · [crypto](https://git.cflmy.cn/cflmy/marqdo-ext-crypto) · [database](https://git.cflmy.cn/cflmy/marqdo-ext-database) · [storage](https://git.cflmy.cn/cflmy/marqdo-ext-storage) · [cache](https://git.cflmy.cn/cflmy/marqdo-ext-cache) · [security](https://git.cflmy.cn/cflmy/marqdo-ext-security) · [browser](https://git.cflmy.cn/cflmy/marqdo-ext-browser) · [document](https://git.cflmy.cn/cflmy/marqdo-ext-document) · [archive](https://git.cflmy.cn/cflmy/marqdo-ext-archive) · [engineering](https://git.cflmy.cn/cflmy/marqdo-ext-engineering) · [regex](https://git.cflmy.cn/cflmy/marqdo-ext-regex) · [datetime](https://git.cflmy.cn/cflmy/marqdo-ext-datetime)

**Web 领域（planned domain）** · 聚合 [marqdo-ext-web](https://git.cflmy.cn/cflmy/marqdo-ext-web)（唯一合同 `docs/WEB-DESIGN.md`）· 子组件 [marqdo-web-design](https://git.cflmy.cn/cflmy/marqdo-web-design) · [marqdo-web-preview](https://git.cflmy.cn/cflmy/marqdo-web-preview)

## 分层治理与领域聚合（2026-11-26 起）

根 catalog 支持唯一的新增类型 `kind = "domain"`（聚合治理角色），治理树最多两级：**根 → 领域聚合 → 功能组件**。规则要点：

- 每个拥有子模块的聚合仓库只登记直接子组件；全局 ID 与权威远端唯一；功能仓库仍只有一个父级。Gitlink 是提交身份唯一权威，`.gitmodules` 是权威远端唯一记录。
- 聚合仓库可拥有 `components.toml`、`.gitmodules`、领域公共规范、跨组件集成测试、综合示例、发布装配脚本、ADR 与维护指南；**禁止**生产实现与源码复制。
- 已有 `planned` 仓的 gitlink 固定真实开发提交。尚无提交的组件只登记计划，激活以后再填写真实 `entry` 并计入完成率。领域及其后代分别登记在各自父仓库。
- 领域激活门禁：子组件具备真实入口与候选提交、领域 catalog 与 gitlink 一致、领域 CI 通过、根 `check-layout.py --checkouts` 递归校验通过、存在依赖关系的组件以同候选证据验证。
- 主仓库 `scripts/check-layout.py` 已引入 domain 角色与递归校验；统一 CLI 制品准备、声明驱动调度、递归影响分析仍在推进，CI 证据以真实候选为准。

当前状态：**没有任何 active 领域**。Web 领域为 planned（`extensions/web`，聚合仓 marqdo-ext-web，子组件 design/preview 在其 catalog 登记）；标准库聚合 marqdo-stdlib 尚未创建，九个 `std-*` 保持根 catalog 的 planned 直属记录；HTTP/WebSocket 等仍为独立 planned 扩展，等至少两个真实组件形成共同边界后再评估聚合。首批 Web 实施工单为 WEB-00（合同与真实 Analysis 风险探测）、WEB-01（Design）、WEB-02（Preview）、WEB-03（handoff/组件图）、WEB-04（组合案例与框架交付）、WEB-05（根接入与冷安装），WEB-06（components）为候选；工单在对应责任仓库创建。Web 合同语义冲突由 marqdo-ext-web 协调，不回流 CLI 核心。

## 阶段与阻塞

| 原分支 | 独立仓库与迁移状态 |
| --- | --- |
| `gh-pages` | 官网架构、v2.9.1 源内容和部署快照均已迁入 marqdo-site。原分支保留现有公共入口，部署切换以独立托管和路由验收为准。[托管切换工单](https://git.cflmy.cn/cflmy/marqdo-site/issues/1) |
| `vscode-extension` | 扩展目录历史归档到 marqdo-vscode 的 `legacy-v2.8.0`。现行 CLI 适配和新版 VSIX 由扩展仓独立开发和验收。[首版实现工单](https://git.cflmy.cn/cflmy/marqdo-vscode/issues/1) |

| 阶段 | 当前范围 | 后续验收 |
| --- | --- | --- |
| M0 盘点 | 自建 Git、历史标签、已有六组件和新仓位 | 通过真实远端候选持续维护台账 |
| M1 集成骨架 | 六个 active gitlink，另固定 28 个直属和 2 个 Web 子级的 planned 开发提交 | 查看候选 Woodpecker 运行及各组件独立 CI |
| M2 标准库切换 | planned，CLI RC.4 提供官方完整 Bundle | [CLI 外部 Bundle](https://git.cflmy.cn/cflmy/marqdo-cli/issues/11) 和 [九组件切换](https://git.cflmy.cn/cflmy/marqdo/issues/2)，包括 HTTP、Regex、Archive 迁出与同步删除旧路径 |
| M3 跨仓闭环 | 工单、人工候选、同 SHA 门禁 | [跨仓库 CI](https://git.cflmy.cn/cflmy/marqdo/issues/3) 跟踪真实缺陷闭环、候选通知鉴权、去重和影响分析 |
| M4 领域接入 | Web 领域 planned：三仓文档与工单就绪；根 catalog 登记 planned domain | W-M0 真实 CLI 风险探测、WEB-01/02 实现、领域 gitlink、根递归校验与冷安装验收；Stdlib 聚合按 STD-01..04 独立推进 |
| 后续领域 | 空仓和预期职责 | 逐组件开展最小真实功能、独立 CI、制品与接入 |

### 标准库切换顺序

1. CLI 打包器与产品测试接受单一显式外部 Bundle 输入。内核独立编译；制品仍附带一个校验过的标准库。
2. 标准库在各组件维护唯一生产源码和自测。组装拒绝冲突、符号链接或测试污染，调用 CLI 权威打包器。std-io 维护 print/打印 的别名元数据。
3. HTTP、Regex、Archive 的真实替代和授权回归先行。候选同时移除 CLI 中旧领域宿主服务和旧导入。
4. 验证同输入两次组装结果、摘要及路径负例、C1/C2/C3、独立冷安装和真实工具集成后，再切换基础发行。当前状态为 UNVERIFIED。

## 标准库源码归位迁移（marqdo#9，2026-11-26 M0 冻结）

本节是标准库迁出的**唯一现行台账**，取代此前按未来目录规划起草的路径表（如 `system/clock.mq.md`、`io/index.mq.md`）；旧路径方案会改变公开 `std:` 路径，与“迁移不改语言/库语义”冲突，已作废。详细实施与验收见父仓工单 [marqdo#9](https://git.cflmy.cn/cflmy/marqdo/issues/9)，各接收仓子工单在对应仓库 issue #1。

### 基线与身份（M0）

| 对象 | 权威身份 |
| --- | --- |
| 根仓库 | `cflmy/marqdo@484cb10d66db9b92f0527499642ae6abddb3ef29` |
| CLI 标准库源 | `cflmy/marqdo-cli@452e9a32bd9e40dba427d4180691c58bab74449e`（RC.4 发行组合；`stdlib/` 自 `4bbefc9` 起未变） |
| 迁移前 Bundle | `marqdo-stdlib@0.1.0`，tree digest `sha256:5407d3665ba2fe105820d0f1af5efaafa10c277a69eb819c3225719813c8c713`（含 `tests/`） |
| 公开函数台账 | 20 个模块、151 个 H2 函数，由基线 Bundle 上 `marqdo inspect --signature` 生成并记入各接收仓工单 |
| CLI 显式 Bundle 输入 | [marqdo-cli#11](https://git.cflmy.cn/cflmy/marqdo-cli/issues/11) Phase A（PR #12）已提供 `--stdlib-dir`/pin 校验；Phase B（删除仓内源树）以本台账根组合验收为前置 |

### 模块归属与冻结发布路径

接收仓库负责源码、测试和后续缺陷；根只做组合、锁定和验收。`modules/` 镜像 Bundle 根相对路径，只收模块不收 `tests/`；官方 manifest 由根唯一生产者生成。

| 原 CLI 路径（相对 `stdlib/`） | 接收仓 / 工单 | 冻结发布路径（Bundle 根相对） | 公开函数 |
| --- | --- | --- | --- |
| `clock.mq.md` | [std-system#1](https://git.cflmy.cn/cflmy/marqdo-std-system/issues/1) | `clock.mq.md` | 1 |
| `datetime.mq.md` | [std-system#1](https://git.cflmy.cn/cflmy/marqdo-std-system/issues/1) | `datetime.mq.md` | 14 |
| `env.mq.md` | [std-system#1](https://git.cflmy.cn/cflmy/marqdo-std-system/issues/1) | `env.mq.md` | 2 |
| `process.mq.md` | [std-system#1](https://git.cflmy.cn/cflmy/marqdo-std-system/issues/1) | `process.mq.md` | 1 |
| `random.mq.md` | [std-system#1](https://git.cflmy.cn/cflmy/marqdo-std-system/issues/1) | `random.mq.md` | 2 |
| `format.mq.md` | [std-io#1](https://git.cflmy.cn/cflmy/marqdo-std-io/issues/1) | `format.mq.md` | 8 |
| `io.mq.md` | [std-io#1](https://git.cflmy.cn/cflmy/marqdo-std-io/issues/1) | `io.mq.md` | 5 |
| `io/console.mq.md` | [std-io#1](https://git.cflmy.cn/cflmy/marqdo-std-io/issues/1) | `io/console.mq.md` | 4 |
| `io/stream.mq.md` | [std-io#1](https://git.cflmy.cn/cflmy/marqdo-std-io/issues/1) | `io/stream.mq.md` | 3 |
| `data/json.mq.md` | [std-data#1](https://git.cflmy.cn/cflmy/marqdo-std-data/issues/1) | `data/json.mq.md` | 22 |
| `data/csv.mq.md` | [std-data#1](https://git.cflmy.cn/cflmy/marqdo-std-data/issues/1) | `data/csv.mq.md` | 7 |
| `data/table.mq.md` | [std-collections#1](https://git.cflmy.cn/cflmy/marqdo-std-collections/issues/1) / [std-data#1](https://git.cflmy.cn/cflmy/marqdo-std-data/issues/1) | `collections/table.mq.md`（8 个值操作）/ `data/table.mq.md`（4 个序列化） | 12 |
| `fs/file.mq.md` | [std-fs#1](https://git.cflmy.cn/cflmy/marqdo-std-fs/issues/1) | `fs/file.mq.md` | 14 |
| `path.mq.md` | [std-fs#1](https://git.cflmy.cn/cflmy/marqdo-std-fs/issues/1) | `path.mq.md` | 8 |
| `text.mq.md` | [std-text#1](https://git.cflmy.cn/cflmy/marqdo-std-text/issues/1) | `text.mq.md`（`utf8编码` 薄封装） | 1 |
| `text/utf8.mq.md` | [std-text#1](https://git.cflmy.cn/cflmy/marqdo-std-text/issues/1) | `text/utf8.mq.md`（唯一编解码实现） | 10 |
| `net/http.mq.md` | [ext-http#1](https://git.cflmy.cn/cflmy/marqdo-ext-http/issues/1) | 显式 Package 模块，**不进入默认 Bundle**，不再是 `std:` | 11 |
| `regex.mq.md` | [ext-regex#1](https://git.cflmy.cn/cflmy/marqdo-ext-regex/issues/1) | 显式 Package 模块，**不进入默认 Bundle**，不再是 `std:` | 22 |
| `compress.mq.md` | [ext-archive#1](https://git.cflmy.cn/cflmy/marqdo-ext-archive/issues/1) | 显式 Package 模块，**不进入默认 Bundle**，不再是 `std:` | 2 |
| `demo.mq.md` | 不设接收仓；根 Bundle 组合 fixture | 不作为公开安装模块 | 2 |

`std-core`、`std-encoding`、`std-net` 没有与之对应的旧模块：本轮不发空工单、不激活、不要求补齐未来规划；若已实现模块被本期 Bundle 实际依赖，再按真实依赖补充工单。

### 原测试断言归属

每项必须在新归属留下真实执行证据；跨模块断言归根 Bundle 组合测试。

| 原 `stdlib/tests/` 文件 | 接收仓 | 断言载体 |
| --- | --- | --- |
| `clock.mq.md`、`datetime.mq.md`、`env.mq.md`、`process.mq.md`、`random.mq.md` | std-system | 组件 case，覆盖原 Rust 断言 |
| `console.mq.md`、`format.mq.md`、`io.mq.md`、`stream.mq.md` | std-io | 组件 case + resource 端点事件检查 |
| `csv.mq.md`、`json.mq.md` | std-data | 组件 case，覆盖原 Rust 断言 |
| `table.mq.md` | std-collections（值操作）+ std-data（GFM）+ 根组合 | 切分后的组件 case 与根组合用例 |
| `fs.mq.md`、`path.mq.md` | std-fs | 组件 case，覆盖原 Rust 断言 |
| `text.mq.md`、`utf8.mq.md` | std-text | 组件 case，覆盖原 Rust 断言 |
| `http.mq.md` | ext-http | 领域回归（基线仅运行级覆盖，需补真实断言） |
| `regex.mq.md` | ext-regex | 领域回归，覆盖原 Rust 断言 |
| `compress.mq.md` | ext-archive | 领域回归，覆盖原 Rust 断言 |
| `demo.mq.md` | 根 Bundle 组合测试 | 组合运行证据 |

### 决策与取舍

1. **冻结路径**：以 Bundle 根相对路径为公开身份，迁移不改 `std:` 导入路径；被旧台账规划改名（如 `system/clock.mq.md`、`io/index.mq.md`）的条目按本表冻结。
2. **测试不随发行**：`tests/` 迁入组件仓并留在仓库外，默认 Bundle 只含模块；因此 Bundle 内容必然变化，版本与 digest 由 MIG-BUNDLE 唯一生产者显式更新，摘要算法不变。
3. **Table 切分**：值操作以 `std:collections/table` 为唯一权威，GFM 序列化以 `std:data/table` 为唯一权威；不保留旧 `std:data/table` 的值操作兼容转发，已知消费者在同一候选内更新。
4. **Text 去重**：UTF-8 编解码唯一实现在 `text/utf8.mq.md`，`text.mq.md#utf8编码` 为保留公开名称的薄封装。
5. **IO 别名**：`io.mq.md` 保持官方 `print`/`打印` 的别名目标 `写值`，签名与 effects 不变；别名元数据仍由 Bundle 生产者生成。
6. **三个领域模块**：HTTP/Regex/Archive 迁入既有扩展仓并退出默认 Bundle，不保留隐藏的 `std:` fallback；CLI 现有 `regex.*`/`compress.*` 宿主请求是历史例外，本轮不删除也不新增同类请求，彻底消除另开工单。
7. **`demo.mq.md`**：不作为公开库模块，转为根 Bundle 组合 fixture，避免为了放置文件创建无独立责任的新仓库。
8. **staging 不是发行**：迁移期为了隔离测试，允许用基线 Bundle 覆盖组件模块并重算 digest 的 staging 组合；它不是官方发布源，最终只有根唯一生产路径。
9. **CLI 侧最小修改**：Phase A 显式制品输入已落地；Phase B 删除仓内官方源树只在组件替代与根组合验收通过后执行，不同时进行核心重构。
10. **未创建的新仓**：不为缺失模块创建同名替代仓；`marqdo-stdlib` 聚合仓仍未建立，聚合治理按原独立计划推进，不阻塞本次源码归位。

### 迁移状态

| 项 | 状态 | 证据 |
| --- | --- | --- |
| M0 盘点与派单 | 完成 | 本节、9 个接收仓 issue #1、[marqdo#9](https://git.cflmy.cn/cflmy/marqdo/issues/9) 评论 |
| 组件接收（源码+断言） | 候选已推送（6 个 std 仓） | [std-text PR #2](https://git.cflmy.cn/cflmy/marqdo-std-text/pulls/2)、[std-system PR #2](https://git.cflmy.cn/cflmy/marqdo-std-system/pulls/2)、[std-io PR #2](https://git.cflmy.cn/cflmy/marqdo-std-io/pulls/2)、[std-fs PR #2](https://git.cflmy.cn/cflmy/marqdo-std-fs/pulls/2)、[std-data PR #2](https://git.cflmy.cn/cflmy/marqdo-std-data/pulls/2)、[std-collections PR #2](https://git.cflmy.cn/cflmy/marqdo-std-collections/pulls/2) |
| 领域模块（http/regex/archive） | 未开始 | ext-http/ext-regex/ext-archive issue #1 |
| Bundle 唯一生产者 | 脚本与契约测试已落地，待根候选验收 | `scripts/build-stdlib.py`、`scripts/stdlib-bundle.json`、`tests/integration/test_build_stdlib.py` |
| 组合验收入口 | 已落地，待根候选执行 | `scripts/acceptance-stdlib.py`、`tests/integration/composition/` |
| CLI 旧源树删除 | 未开始（等待根组合验收） | [marqdo-cli#11](https://git.cflmy.cn/cflmy/marqdo-cli/issues/11) Phase B |
| 根组合与切换 | 未开始 | 本仓 MIG-RELEASE 候选 |

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
| html | web（合同见 WEB-DESIGN） | HTML escape/unescape 属渲染安全细节，按 WEB-DESIGN 与 Preview 安全合同处理；不恢复旧页面/模板插件模型 |
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
| `ext/web` + `plugins/web` | 拆为 Web 设计原型领域（唯一合同见 marqdo-ext-web `docs/WEB-DESIGN.md`：设计文档解释、递归组件、有限场景、离线预览与 handoff）与 HTTP/WebSocket/Browser/Database/Security 各自独立组件；删除 monolithic Go plugin 调度与旧页面模型，不恢复旧 Web Artifact 兼容层。 |
| `ext/engineering` + `plugins/engineering` | Engineering；Git 操作、文件、archive 分配给已有权威库，避免重复平台操作。 |
| `crates/marqdo-wasm`、旧 browser examples | Browser 组件参考；语言核心可移植性需要 CLI 工单，而非复制旧执行器。 |
| 旧 `src/knowledge`/proof/view/lint/对应历史 CI 与 examples | 独立 Tool，只迁移所需知识、算法和 fixture；当前公共 Analysis/Observation 优先；新旧协议不并存。 |
