# Marqdo 的 OKF 实现

| | |
|---|---|
| 状态 | **Accepted（规范）；实现分层见 §10** |
| 日期 | 2026-08-07 |
| 调研 | [okf-and-marqdo.md](../research/okf-and-marqdo.md) |
| 命令现状 | [catalog-cli.md](catalog-cli.md)（v0 已落地） |
| 生成式清单方向 | [generated-yaml-manifest.md](generated-yaml-manifest.md) |
| 智能体消费面 | [ext-agent.md](ext-agent.md) · [ext-agent-plan.md](ext-agent-plan.md) · 任务知识包见本文 §7 |
| 外部规范 | [OKF SPEC v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) |

本文是 **Marqdo 语言侧 OKF 实现的详细设计**：界定我们借鉴什么、已经实现什么、下一阶段必须补什么。  
**不把 Google OKF 整本照搬进语言语义**；Marqdo 的可执行真相永远是 `.mq.md` 源。

---

## 1. 定位

### 1.1 OKF 是什么（对我们）

[Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog) 用 **目录中的 Markdown + YAML frontmatter** 表示可交换知识：概念一页一文、路径即 ID、链接成图、最小强制面（几乎只强制 `type`），并在 v0.2 加入信任信号。

Marqdo 对齐的判断是：

> 人类可读的 Markdown 可以成为人与 Agent 的一等交换面；**程序本身也应能投影成同一形态的知识包**。

### 1.2 Marqdo 多走的一步

| | OKF（通用知识） | Marqdo |
|--|-----------------|--------|
| 正文 | 自由散文 | **可执行语法**（`.mq.md`） |
| 成功标准 | 互通、可策展、可信任 | 能跑 + 能读 + 可被 Agent 索引 |
| 清单角色 | 知识本身 | **从源派生的投影**（非第二套权威配置） |

一句话：

**OKF 形、Marqdo 义 — 形态对齐 OKF；词汇与生命周期服务模块、依赖、任务技能与信任，而不是 BigQuery 业务概念表。**

### 1.3 三层文档（勿混）

| 层 | 谁写 | 是什么 |
|----|------|--------|
| **源** | 人 / 编码 Agent | `.mq.md`：唯一语义权威，打开即读、即可跑 |
| **工程目录包** | `marqdo catalog` | 模块依赖/导出总览（`Marqdo Module`） |
| **任务知识包** | `plan` 晋升等 | 任务↔技能↔可执行资源（`Marqdo Task` / `Marqdo Agent Skill`） |

`marqdo view output` 出 HTML 站点，与 OKF 包**分工**：人读程序站 vs Agent/工具读知识包。见 [view.md](view.md)、[catalog-cli.md](catalog-cli.md) §4。

---

## 2. 设计原则（锁定）

1. **源为唯一权威** — 依赖、导出、可执行行为以 `.mq.md` 为准；知识包是提取或晋升结果。  
2. **生成、可复现** — 同一源树 + 同一工具版本 → 同一工程目录包（时间戳字段可规范化忽略）。  
3. **勿手改生成物** — 文件头 / `generated` 标明 GENERATED；手改不被承认为权威。  
4. **最小强制面** — 互通只要求：合法 frontmatter、`type` 字段、可解析的 `resource` / 链接；未知 `type` **必须优雅降级**（仍可读正文与已知字段）。  
5. **格式而非平台** — 包可 `cat` / `git clone`；不绑定某一云或某一模型。  
6. **清单可删可再生** — 无 catalog 时解释器仍能直接从源工作；catalog **不是**语言语义的一部分。  
7. **可执行资源显式** — 凡声称「可跑」的概念必须有 `resource:` 指向 `.mq.md`（或等价路径），禁止只靠散文描述冒充程序。

---

## 3. 概念模型

### 3.1 Bundle（知识包）

一个 OKF 风格 bundle 是目录树：

```text
<bundle-root>/
  index.md              # 可选：人/Agent 总览
  log.md                # 可选：变更史（策展用）
  catalog.yaml          # 可选：机器向总清单
  concepts/…            # 或 modules/…：概念页（frontmatter + Markdown）
  resources/…           # 可选：被引用的可执行 .mq.md（任务包常用）
```

**Concept ID** = 相对 bundle 根、去掉扩展名的路径（OKF 惯例）。  
例：`concepts/tasks/a1b2c3d4e5f6.md` → ID `concepts/tasks/a1b2c3d4e5f6`。

**联系** = frontmatter 中的路径字段 + 正文中的 Markdown 链接，形成可遍历图。

### 3.2 Marqdo 使用的两类 bundle

| Bundle | 默认根 | 生产者 | 消费者 |
|--------|--------|--------|--------|
| **工程目录包** | `.marqdo/`（`-o` 可改） | `marqdo catalog` / `sync` | 人、CI、未来编译加速、通用 Agent |
| **任务知识包** | `.marqdo/agent-kb/` | `ext/ai` 的 `plan` 晋升等 | 父智能体快路径、策展、复用 |

二者词汇共享（`type` / `generated` / `verified` / `status`），目录职责分离：  
**不要**把探索用的 `agent-runs/` 时间戳文件当成稳定概念 ID。

---

## 4. Frontmatter 词汇表

### 4.1 所有概念页的公共字段

| 字段 | 必填 | 含义 |
|------|------|------|
| `type` | **是** | 概念类型字符串（见 §5） |
| `title` | 推荐 | 短标题 |
| `description` | 可选 | 一句话说明 |
| `tags` | 可选 | 字符串列表 |
| `resource` | 视类型 | 指向权威源或可执行 `.mq.md` 的相对路径 |
| `status` | 推荐 | `draft` \| `stable` \| `deprecated` |
| `generated` | 生成物必有 | `{ by, at? }` — 谁生成 |
| `verified` | 可选 | `{ by, at? }` — 谁核验（非 LLM 口述） |
| `sources` | 可选 | 出处列表（OKF v0.2） |
| `stale_after` | 可选 | 绝对日期，过期后消费者应降级信任 |
| 未知字段 | — | **保留、不拒绝** |

信任档位（推导，不必单独存字段）：

| 条件 | 档位 |
|------|------|
| 无 `verified` | 未验证（机器生成草稿） |
| 有 `verified.by` 为确定性工具（如 `marqdo-agent/spawn`） | 机器确认 |
| 有人工审核约定（未来） | 人工审核 |

### 4.2 工程模块页额外字段（v0 已用）

| 字段 | 含义 |
|------|------|
| `depends` | 导入列表（从源 frontmatter `> path` 提取） |
| `exports` | 顶层 `#` 导出名列表 |

### 4.3 任务 / 技能页额外字段（目标词汇）

| 字段 | 所在 type | 含义 |
|------|-----------|------|
| `skill` | `Marqdo Task` | 指向 Skill 概念页的相对路径 |
| `goal` 或正文中的规范化 goal | `Marqdo Task` | 匹配材料（规范化规则见 §7.3） |
| `llm_free` | `Marqdo Agent Skill` | 资源是否仍含 `agent.step` / LLM 调用（机器启发） |
| `quality` | `Marqdo Agent Skill` | 整数启发分（长度、llm_free 等），**非**模型打分 |

---

## 5. `type` 注册表（Marqdo 义）

未知 type **不得**导致解析失败；工具可跳过专用逻辑，仍展示 title/正文/链接。

| `type` | 状态 | 含义 | 主要生产者 |
|--------|------|------|------------|
| `Marqdo Catalog` | **v0 已实现** | 包总览（`catalog.yaml` / `index.md`） | `marqdo catalog` |
| `Marqdo Module` | **v0 已实现** | 一个 `.mq.md` 模块的投影 | `marqdo catalog` |
| `Marqdo Task` | **规范已定；实现待做** | 一项可匹配的任务意图 | `plan` 晋升 |
| `Marqdo Agent Skill` | **规范已定；实现待做** | 指向可执行工作簿的技能 | `plan` 晋升 |
| 其他 | 开放 | 第三方/未来扩展 | — |

**不**引入 Google OKF 的 `Metric` / BigQuery 表类型作为 Marqdo 核心词汇；需要时可由用户包自行使用，运行时降级。

---

## 6. 工程目录包（v0 — 已实现）

### 6.1 CLI

```bash
marqdo catalog [PATH] -o OUT_DIR
marqdo sync [PATH] -o OUT_DIR          # 别名
```

| 参数 | 默认 | 说明 |
|------|------|------|
| `PATH` | `.` | 工程根或单文件；目录则递归 `*.mq.md` |
| `-o` | `.marqdo` | 输出根；写入前创建，**覆盖**同名生成文件 |

实现：[`src/catalog.rs`](../../src/catalog.rs)。金样：`tests/gold.rs` → `catalog_writes_yaml`、`catalog_includes_agent_kb_concepts`。

### 6.2 生成物布局

```text
OUT_DIR/
  catalog.yaml
  index.md
  modules/
    <path-with-__>.md
  concepts/                    # O3：从 **/.marqdo/agent-kb/concepts/ 复制
    tasks/…
    skills/…
```

路径拍扁：`structure/import/main.mq.md` → `modules/structure__import__main.md`。  
`agent-kb` 下的可执行 `.mq.md` **不**进 Module 表（避免与概念/资源重复）；Task/Skill 概念页进 `concepts/` 与 `catalog.yaml` 的 `concepts:`。

### 6.3 `catalog.yaml`（现行字段）

```yaml
# GENERATED by marqdo — do not edit by hand
type: Marqdo Catalog
title: marqdo-project
generated:
  by: marqdo/<crate-version>
modules:
  - id: structure/hello          # resource 去掉 .mq.md
    resource: structure/hello.mq.md
    title: …                     # 源 frontmatter title，否则用路径
    imports: []                  # 或 ["lib/x.mq.md as x", …]
    exports:
      - name: Hello World
        kind: fn
    # 可选（源 frontmatter 有则透传）：
    # verified: { by: … }
    # sources: […]
concepts: []                     # 或 Task/Skill 条目（O3）
  # - id: concepts/tasks/<sig>
  #   type: Marqdo Task
  #   title: …
  #   status: …
  #   page: concepts/tasks/<sig>.md
  #   resource: …
  #   skill: …
```

提取规则：

- **imports**：源文件 YAML frontmatter 中以 `>` 开头的导入行（经 `parse_import_spec`）。  
- **exports**：解析成功则用 AST 顶层函数名；否则回退扫描 `# ` 标题行。  
- **verified / sources**：源 frontmatter 有则写入 module/concept 条目（O3）。  
- **concepts**：扫描 `**/.marqdo/agent-kb/concepts/**/*.md`，复制到 `OUT_DIR/concepts/`。  
- **不**提取正文 `>` 动态调用图、不写锁哈希、不写跨仓 URL。

### 6.4 模块概念页（现行）

```yaml
---
type: Marqdo Module
title: …
resource: path/to/file.mq.md
depends: […]
exports: […]
generated:
  by: marqdo/<version>
# verified / sources：源有则透传
---
```

正文含「勿手改」、**可点击依赖链接**（解析到同包其它 Module 页）、导出表。`index.md` 含 Modules 表与 **Agent knowledge** 节（链到 `concepts/`）。

### 6.5 仍相对完整 OKF 愿景的缺口

| 缺口 | 说明 |
|------|------|
| 无通用 Concept ID 遍历 API | catalog 为生成物；无独立查询命令 |
| 无 `stale_after` / Attested 再现 | 信任信号仅可选 `verified`/`sources` |
| 无 `log.md` | 无策展历史（O4） |
| 运行时不读 catalog | 解释器不依赖生成物（有意为之） |

O2/O3 已补：Task/Skill 知识包、catalog 扫入 agent-kb、模块页可点依赖、可选信任字段。

---

## 7. 任务知识包（规范 — 实现路线 O2）

服务「父智能体优化过的工作簿，同任务应直接执行」：把子 Marqdo / 技能登记为 OKF 概念，用 ID 与链接匹配，而不是私有 `index.json`。

### 7.1 布局（锁定）

```text
.marqdo/
  agent-runs/workbook-<slug>-<ts>.mq.md   # 仅当显式 workbook_dir= 时（可 gc）
  agent-kb/                          # 任务知识包 bundle（默认首次 plan 就写这里）
    index.md
    log.md                           # 可选
    catalog.yaml                     # 可选；可由 promote 或 catalog 刷新
    concepts/
      tasks/<slug>.md                # type: Marqdo Task；sig 在 frontmatter
      skills/<slug>.md               # type: Marqdo Agent Skill
    resources/
      <slug>.mq.md                   # 默认可执行工作簿（探索与晋升同一路径）
```

默认根可通过 `plan` 参数 `kb_dir` / `知识库目录` 覆盖。`.gitignore` 已忽略 `**/.marqdo/`。

路径用可读 **slug**（由 goal 派生：拉丁小写、CJK 保留、其它变 `-`，最长约 48 字符）。**`sig` 仅写在 frontmatter**，用于精确匹配。同 slug 异 sig 时用 `slug-<sig前4位>`。旧包若仍以 `<sig>` 为文件名，lookup 会回退识别。

### 7.2 概念页模板

**Task**（匹配入口）：

```yaml
---
type: Marqdo Task
title: <短摘要>
description: <规范化 goal>
sig: <12hex>
status: stable
tags: [agent-task]
skill: ../skills/<slug>.md
generated:
  by: marqdo-agent/plan
verified:
  by: marqdo-agent/spawn
  at: <ISO8601>
---

# Task

See [skill](../skills/<slug>.md).
```

**Skill**（可执行绑定）：

```yaml
---
type: Marqdo Agent Skill
title: <短名>
sig: <12hex>
resource: ../../resources/<slug>.mq.md
status: stable | candidate
llm_free: true | false
quality: <int>
hits: <int>
generated:
  by: marqdo-agent/plan
  at: <ISO8601>
verified:
  by: marqdo-agent/spawn
  at: <ISO8601>
---

# Skill

Prefer spawning `resource` over re-planning.
```

`status`：`llm_free` 的 resource 晋升为 **stable**（零父 LLM 快路径）；否则为 **candidate**（可复用，但按 `improve_every` 偶发再优化，每轮最多 1 次改进）。失败运行不晋升；质量更差不覆盖。

### 7.3 任务签名 → Concept 路径

材料：

1. `goal`：Unicode NFC → trim → 连续空白折叠为单空格。  
2. 若调用方工具表非空：工具名排序后追加进材料。  
3. **不**把父 `standing` 计入签名（易变导致永不命中）。  

输出：稳定短哈希 `sig`（`agent_goal_sig`）+ 可读 `slug`（`agent_goal_slug`）。路径：

- `concepts/tasks/<slug>.md`
- `concepts/skills/<slug>.md`
- `resources/<slug>.mq.md`

匹配顺序：

1. **精确**：slug 试探 + frontmatter `sig` → 扫目录 `sig:` → 旧 hex 文件名。  
2. **别名（Accepted）**：精确 miss 后，扫 Task FM `aliases:`（YAML 列表或 `[a, b]`）；`normalize_goal(query)` 与某条 alias **精确相等**则命中同一 Skill/resource。lookup 返回 `match: exact|alias`；`plan` 对 alias 写 `cache=soft-hit`。  
3. **不做**默认 embedding / 编辑距离主路径。父裁决 soft_match 仍见 [okf-near-match.md](../roadmap/okf-near-match.md)。

`agent_kb_promote` 可选 `aliases=`（字符串或列表）写入 Task；再晋升时合并既有 aliases。

### 7.4 与 `## plan` 的契约

| 参数 | 默认 | 含义 |
|------|------|------|
| `reuse` / `复用` | `True` | 允许知识包快路径 |
| `optimize` / `优化` | `False` | 跳过知识包，强制再规划 |
| `force` / `强制` | `False` | 忽略知识包 |
| `promote` / `晋升` | `True` | DONE 后固化并写入/更新概念与 resource |
| `kb_dir` / `知识库目录` | `.marqdo/agent-kb` | bundle 根 |
| `workbook_dir` / `工作簿目录` | `None` | 默认直接写 `kb_dir/resources/<slug>.mq.md`；显式传入则用该目录下的临时 `workbook-<slug>-<ts>.mq.md` |
| `improve_every` / `改进周期` | `3` | candidate 每 N 次命中触发 ≤1 轮改进 |
| `explore_n` / `探索次数` | `3` | 同任务已有文件数 `< N` 且非 `llm_free` 时，强制新建 `explore/<slug>/<k>.mq.md` 尝试不同路径（第 2 次默认 dual 骨架） |

调用示例（路径可直接写在参数里，无需 `json.parse`）：

```markdown
> `助手`.plan goal=… max_rounds=3
> `助手`.plan goal=… workbook_dir=".marqdo/agent-runs"
> `助手`.plan goal=… explore_n=3
```

快路径：

1. lookup 命中且 resource **llm_free**（含现场检测已固化源）→ spawn → `cache=hit`，**父 LLM 不调用**。  
2. 否则若任务相关文件数 `< explore_n` → 写入新的 explore 变体，父 LLM 被提示尝试不同路径；成功则固化并晋升。  
3. 否则走 candidate 命中 / `improve_every` 改进逻辑。  

`DECISION: DONE` 时先 `agent_workbook_solidify`（把仍含 `worker.step` 的簿固化为 `# main` **返回**答案，而非仅 `print`），再晋升并写 `verified`。  
显式 `workbook=` 续跑优先于知识包查找。

质量启发（覆盖 skill 时，不调模型）：`llm_free` 优先；源更短优先；同质取较新。更差 `quality` 不覆盖现 skill。

### 7.5 自我成长如何落在 OKF 上

| 能力 | 机制 |
|------|------|
| 优化 | 规划循环固化代码 → 更优 `resources/<slug>.mq.md` |
| 迭代 | 同 Concept slug/`sig` 覆盖 resource + 刷新 `generated`/`verified` |
| 成长 | 命中快路径；`candidate`/`hits`/`improve_every` 驱动再优化；`log.md` 可审计 |
| 分发 | 整包 `agent-kb/` 可复制（格式即契约） |

---

## 8. 运行时与工具职责

| 组件 | 读 OKF？ | 写 OKF？ | 备注 |
|------|----------|----------|------|
| 解释器 / 字节码 VM | 否（语义） | 否 | 只跑 `.mq.md` |
| `marqdo catalog` | — | 工程目录包 | v0 |
| `plugins/agent`（`agent_kb_*`）+ `ext/ai` `plan` | 任务包 | 晋升时写任务包 | O2；**不**进核心 `src/host` |
| `lib/fs` | 通用文件 | 通用文件 | L1 包装 |
| `marqdo view` | 否 | 否 | HTML 另一轨 |
| 未来 `marqdo kb query` | 是 | 否 | 可选：按 type/tag/链接查询 |

agent-kb 运行时实现归属 **agent 插件**（`agent_goal_sig` / `agent_kb_lookup` / `agent_kb_promote` / `agent_kb_record_hit` / `agent_workbook_solidify` / `agent_kb_task_files`），由 `ext/ai/agent` 调用；禁止把这些原语加回 `HostFn`。

解释器**不得**因缺少 catalog 而拒绝运行源文件。

---

## 9. 安全与边界

| 风险 | 对策 |
|------|------|
| 把生成物当配置手改 | 文档 + GENERATED 头；CI 可选校验 |
| 错误技能被复用 | 快路径必 spawn+inspect；失败标 `draft` 并重规划 |
| 签名过宽 | tools 指纹；文档强调 exact goal |
| 密钥进入知识包 | 继承 env；禁止把 key 写入 resource/概念正文 |
| 包膨胀 | runs 与 kb 分离；仅 DONE 晋升 |
| 与语言语义耦合 | catalog/kb 失败不影响 `marqdo run` 源文件 |

---

## 10. 实现路线

| 阶段 | 内容 | 状态 |
|------|------|------|
| **O0** | 调研 + 生成式清单方向 + `.mq.md` 后缀 | **done** |
| **O1** | `marqdo catalog` / `sync`：Module + Catalog YAML | **done**（v0） |
| **O2** | 本文锁定的任务知识包：`Marqdo Task` / `Agent Skill`；`plan` reuse/promote | **done**（host kb_* + plan 快路径） |
| **O3** | catalog 增强：`verified`/`sources` 可选；模块页可点击依赖；扫入 `agent-kb` 概念供人浏览 | **done** |
| **O4** | `log.md` 策展、可选 `kb query`、Attested-style「再现跑通」收据（非 BigQuery） | 远期 |
| **O5** | 相近任务软命中：Task `aliases` 第二趟已落地；规范句 / 父裁决 / 向量仍见 [okf-near-match.md](../roadmap/okf-near-match.md) | **aliases done** |

O2 设计落盘后，智能体侧实现文档可薄化为指向本文 §7，避免两套真理。

---

## 11. 非目标

- 在 Marqdo 内核实现完整 OKF 验证器产品或云托管知识库。  
- 用 OKF 清单替代模块命名空间 / 导入语义（[module-namespace.md](module-namespace.md)）。  
- 强制用户提交 `.marqdo/` 进 git。  
- **默认**语义相似检索 / 向量库（可选软命中见 O5 / [okf-near-match.md](../roadmap/okf-near-match.md)；不进内核）。  
- 跨仓技能市场（可后续在概念图上扩展）。  
- 照搬 OKF `Attested Computation` 的 BigQuery executor 模型。

---

## 12. 与既有文档的关系

| 文档 | 关系 |
|------|------|
| [okf-and-marqdo.md](../research/okf-and-marqdo.md) | 调研；产业背景与借鉴边界 |
| [generated-yaml-manifest.md](generated-yaml-manifest.md) | 早期「生成式清单」方向；**词汇与原则并入本文，命令细节见 catalog-cli** |
| [catalog-cli.md](catalog-cli.md) | O1 CLI 与 v0 落盘的操作说明 |
| [ext-agent-plan.md](ext-agent-plan.md) | 多步执行；缓存/复用应引用本文 §7 |
| [ext-agent.md](ext-agent.md) | 框架总览；自写回 ≠ 跨调用 OKF 知识包 |

---

## 13. 决议摘要

1. Marqdo 的 OKF 实现 = **形态对齐 + Marqdo 类型词汇 + 生成/晋升管道**，不是语言换皮。  
2. **O1–O3 已落地**（工程目录包、任务知识包、catalog 扫入 agent-kb + 人读链接图）；**O4** 策展/查询为远期。  
3. 可执行真相是 `.mq.md`；概念页负责**寻址、信任与联系**——人与 Agent 都是闭环里的一环。  
4. 父智能体复用优化结果，应走 **Task → Skill → resource → spawn**，而不是私有 JSON 缓存主路径。  
5. 未知 `type` 降级；无清单亦可运行源码。

---

## 14. 一句话

**Marqdo 用 OKF 风格的知识包索引程序与任务技能：源负责跑，概念负责找与信，catalog 与 agent-kb 分管工程目录与自我成长。**
