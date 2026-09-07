# 中层标准库（Mid Stdlib）

| | |
|---|---|
| 状态 | **M1–M6 已落地** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib.md](stdlib.md) · [stdlib-modules.md](stdlib-modules.md) · [ext-abi.md](ext-abi.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 目标 | 在接下来数个发版周期内，把「通用编程」能力沉到 **L0.5 宿主 + L1 `.mq.md`**，减少「凡是稍有能力就必须 ABI 插件」的路径依赖 |

---

## 1. 为什么现在做

现状分层（已落地）：

```text
L0   内置          print / len / split / …
L0.5 宿主原语      src/host/*（供 L1 包装，作者原则上不直调）
L1   官方库        lib/*.mq.md（嵌入二进制 / 可覆盖）
ext  官方扩展      .mq.md + 可选 cdylib（web / agent / quantum / linalg）
```

痛点：

1. **通用语言能力缺口**：`lib/text` 几乎只是 `trim/split/join`；无正则、无编解码、无哈希、无路径代数、无 CSV……日常脚本要么外联 Python，要么新开插件。
2. **插件路径过重**：ABI 插件适合**领域热路径**（量子核、公式矩阵、HTTP 服务+SQLite），不适合「再来一个 base64」。每加一个插件都要：crate、Release 资产、`ext add`、CI、WASM 缺口。
3. **扩展作者被逼进插件**：`ext/**` 禁止 `host_*`。若 L1 没有正则 / 哈希 / 路径，扩展只能 `plugin.load`，导致「库」与「原生模块」边界模糊。
4. **对照成熟语言**：Go 的 `strings` / `regexp` / `encoding/*` / `crypto` / `path/filepath`、Python 的「难写对才进 stdlib」标准——都把**通用原语**留在语言自带层，把**领域产品**留给第三方。Marqdo 的「通用」应对齐前者；`ext/` 对齐后者。

中层标准库 = **在现有 L1 之上、在官方 ext 之下**，专门补「写通用程序必备、又不该做成插件」的那一档。

---

## 2. 分层命名（本波约定）

| 名称 | 位置 | 用户导入 | 实现形态 |
|------|------|----------|----------|
| **核心 L1** | 已有 `lib/{fs,sys,time,json,net,math,…}` | 是 | 薄包装 → 已有 `host_*` |
| **中层 L1（Mid）** | 新增 / 加厚 `lib/{re,encoding,hash,path,…}` | 是（仍属 `lib/` / `std/`） | **优先**薄 `host_*` + L1；能纯 `.mq.md` 则纯库 |
| **官方 ext** | `ext/{web,agent,quantum,linalg,ai}` | 显式 `ext/` | `.mq.md`；**仅**领域热路径允许 ABI 插件 |
| **catalog / 用户库** | 发版外收录 | 各自约定 | 应能主要靠 Mid + 核心 L1 写成，**默认无插件** |

> **不**新开第三棵目录树。中层仍落在 `lib/`，用文档与路线图分期交付，避免 `lib/` vs `lib-mid/` 分裂。

---

## 3. 准入 / 不准入（宪法）

### 3.1 进入中层的条件（须同时满足多数）

| # | 条件 | 说明 |
|---|------|------|
| 1 | **跨领域高频** | 脚本、站点、智能体、教学示例都会用到 |
| 2 | **难写对或不可移植** | 正则、Unicode、密码学、路径跨平台——纯 Marqdo 成本过高 |
| 3 | **解锁「无插件写库」** | 有了它，更多 `ext/` / catalog 可用纯 `.mq.md` |
| 4 | **宿主面积极小** | 一个或少数 `host_*` + 稳定 crate；不把巨型框架链进主二进制 |
| 5 | **API 可长期冻住** | 宁可窄接口，也不提前做成第二个 Requests |

参考（外部）：

- Python：[stdlib 准入](https://devguide.python.org/developer-workflow/stdlib/) 与 [Language Summit：stdlib 何用](https://blog.python.org/2023/05/the-python-language-summit-2023-what-is.html)——偏「启动器 / 新手脚本 / 难写对的协议」，而非「万物入 std」。
- Go：日常几乎离不开的 `strings`、`regexp`、`encoding/*`、`crypto`、`path/filepath`、`slices`——对标我们的 Mid 候选，而非 `database/sql` 驱动全家桶。
- 反面：PEP 594「死电池」——过窄协议、过时格式、与生态脱节的模块不要进 Mid。

### 3.2 明确不准入（继续走 ext + 可选插件）

| 领域 | 原因 |
|------|------|
| HTTP **服务端** / ASGI / 路由框架 | 产品面；已有 `ext/web` |
| SQLite / ORM / 管理后台 | 同上 |
| LLM / 智能体编排 / OKF | `ext/ai` · `ext/agent` |
| 量子线路 / 密度矩阵核 | `ext/quantum` |
| 大学线性代数 CAS / 分解可视化 | `ext/linalg`（公式文档面） |
| 完整图像 / 音视频 / ML | 体积与依赖爆炸 |
| 任意第三方云 SDK | 不属于语言 |

### 3.3 插件仍保留的边界

ABI 插件 **不废除**。规则收紧为：

```text
若能力可用「少量 host_* + lib/*.mq.md」表达 → 必须走中层，禁止新开官方插件。
若能力是领域数值核 / 长生命周期服务 / 可选重依赖 → 才允许官方插件。
```

`lib/plugin` 继续服务：本地实验、第三方原生、官方领域包。

---

## 4. 实现分工（怎么长）

```text
用户 / catalog / 官方 ext（纯 mq 优先）
        │  import re:lib/re.mq.md
        ▼
中层 L1（.mq.md，中英分文件，除 json 类共用外）
        │  **> host_re_find …**
        ▼
L0.5 宿主（Rust：regex / sha2 / …）
        │
        ▼
OS / 算法库（精选 crate，默认 features 最小化）
```

| 规则 | 说明 |
|------|------|
| `ext/**` 仍禁止 `host_*` | 中层补齐后，ext 经 `lib/re` 等调用 |
| 每个新 `host_*` | 必须有 L1 包装 + 金样 + 中英对称（见 [stdlib-i18n.md](stdlib-i18n.md)） |
| 能纯 mq 的 | 例如在 `table` 上的 `sort_by` 若性能可接受，可先纯库；热路径再下沉 host |
| WASM | 新 host 须标注是否进浏览器构建；不能 WASM 的在文档标 `host-only` |
| 依赖 | 新增 crate 写进 [dependencies.md](dependencies.md) 动机；禁止「顺手」拉大型框架 |

---

## 5. 模块候选总表（按优先级波次）

完整排期与验收见 [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md)。摘要：

| 波次 | 模块（英 / 中） | 典型能力 | 主要形态 |
|------|-----------------|----------|----------|
| **M1** | `re` / `正则` · `encoding` / `编码` | match / find / replace；base64 / hex | **done** |
| **M2** | `path` / `路径` · `fs` 加厚 | join / normalize；copy / move / temp | **done** |
| **M3** | `hash` / `哈希` · `secrets` / `机密` | sha256 / hmac；token | **done** |
| **M4** | `csv` / `逗号表` · `text` 加厚 | parse / stringify；contains / replace | **done** |
| **M5** | `table` 加厚 · `cli` / `命令行` | sort / zip；flags | **done** |
| **M6** | `log` / `日志` · `uuid` / `标识` | level 日志；v4 | 薄 host 或纯 mq |
| **可选** | `yaml` / `Toml` | 仅当 public/catalog 强需求 | 慎入 |

**刻意延后：** 完整 `datetime` 时区库、连接池、异步 runtime、模板引擎、完整 `collections` 类型系统——等中层前几波用起来再议。

---

## 6. 与现有库的关系

| 现有 | 中层策略 |
|------|----------|
| `lib/text` | **加厚**，不另起 `strings` 英文名（保持 Marqdo 习惯）；中文 `文本` |
| `lib/table` | Mid 集合算法优先挂这里或薄 `listx`；避免与 GFM 表语义打架 |
| `lib/net` | HTTP 客户端保留；编解码类能力与 `encoding` 边界写清（url 组件归谁） |
| `lib/math` | 保持**高中**；哈希 / 安全随机不进 math |
| `lib/json` | 仍只 parse/stringify/quote；CSV/YAML 另库 |
| `lib/plugin` | 不变；文档写明「通用能力请先查 Mid」 |
| `ext/*` | 逐步**改写依赖**：能改用 `lib/re` 的路由/校验，去掉不必要的原生 |

---

## 7. 文档与工程节奏

每个 Mid 波次固定流水线（与现有发版习惯对齐）：

1. 本设计不变的前提下，在 roadmap 勾选该波次工作项。  
2. `doc/design/stdlib-<module>.md`（可短）→ `host_*` → `lib/` 中英 → `tests/lib/*` 金样 → `public/features` 短文。  
3. 更新 [stdlib-modules.md](stdlib-modules.md) 模块表。  
4. **不**为 Mid 单开 SemVer 故事；随正常产品发版收进 CHANGELOG「标准库」小节。  
5. 发版技能 / Release 资产：仅当新增 **必须** 随 CLI 分发的东西时改 release 脚本——Mid 默认已嵌入 stdlib，**无** native zip。

---

## 8. 成功标准（数个版本后）

1. 用 Mid + 核心 L1 **不加载任何插件**，能写：日志采集脚本、CSV 清洗、带正则的文本管线、带哈希校验的下载器、简单 CLI 工具。  
2. 新的官方 **非领域** 能力默认进 `lib/`，评审时问：「能否不做成插件？」  
3. `ext/web` / `ext/agent` 等仍可有插件，但依赖面写进文档：哪些能力已改走 Mid。  
4. 主二进制体积与依赖可解释；WASM 可用性有清单。

---

## 9. 一句话

**中层标准库把「通用原语」沉回语言自带层，让插件专属于真正的领域引擎——这样 Marqdo 才像一门通用语言，而不是「Markdown + 一堆 .so」。**
