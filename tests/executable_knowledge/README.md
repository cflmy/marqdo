# 可执行知识 — S0 作者基线

| | |
|---|---|
| 状态 | **S0 基线语料与无网络门禁**；不是 S1+ 能力声明 |
| 规范 | [文档原生指导](../../doc/design/document-native-development-guide.md) §3 / §7 S0 · [缺口路线](../../doc/roadmap/executable-knowledge-gaps-2026-10-23.md) |
| 门禁 | `cargo test --test executable_knowledge --locked`（无网络） |

## 六类任务

| ID | 目录 | 状态 | 期望 | 作者手工维护项（基线计数） |
|---|---|---|---|---|
| 01 | `baseline/01-inline/` | implemented | Tree/Bytecode → `60` | 0（无注册/无锁/无知识 ID） |
| 02 | `baseline/02-extract/` | implemented | → `60` | 标题名 + 叙述形参 `` `数量` `` `` `单价` ``（2） |
| 03 | `baseline/03-import/` | implemented | → `60` | frontmatter `import` 一行（1） |
| 04 | `baseline/04-access-loop/` | implemented | 取元+遍历输出 | `import json` + `access`/`遍历` 写法（与现网一致） |
| 05 | `baseline/05-policy-draft/` | **design** | 可试跑 → `113`；**不**宣称税率事实已证实 | 叙述中的来源意图（尚未有强制 schema） |
| 06a | `baseline/06-reject/missing-import.mq.md` | expected_failure | 双后端均非 0 | 0（正确拒绝） |
| 06b | `baseline/06-reject/ambiguous.mq.md` | expected_failure | **双后端均拒绝**裸名同名（G02） | — |
| 10 | `baseline/10-boundcall/` | implemented | BoundCall 路径 SymbolId + DynamicCall(`call_fn`)；双后端 `hi`/`60` | import 一行 |
| 11 | `baseline/11-narrative-ref/` | implemented | `narrate`/`navigation` 步骤；围栏不计；run 不拉 URL | 0 |
| 12 | `baseline/12-tax-source/` | implemented | `sources:` → evidence=sourced；过期 → expired；可试跑 `113` | sources + valid_until |
| 13 | `baseline/13-bookstore/` | implemented | 书店含税可运行子集 + narrate；输出 `113` | import + sources |

### 同名裸调用（G02）

`06-reject/ambiguous.mq.md` 同时 `import` 了 `a` 与 `b`（同名 `商品小计`），正文用**裸名**调用。双后端均须拒绝；正确写法是 `a.商品小计` / `b.商品小计` 等显式路径。

`10-boundcall`：显式 `pricing.商品小计` 写入 `bound_calls.json`（`binding: BoundCall` + SymbolId）；`call_fn` 写入 `dynamic_calls.json`（不得假装已绑定）。

## 不计为成功的项

- `status: design` 样例通过运行 ≠ 来源/适用性/ready 已实现（G05/G07）
- 任何 `#[ignore]` / skip **不得**充当核心能力证据
- 目标示例 [`examples/executable-knowledge/`](../../examples/executable-knowledge/) 仍为设计态，不在本门禁

## 切片进度（本目录门禁）

| 切片 | 状态 | 证据 |
|---|---|---|
| S0 | **done** | `s0_*` 基线正负例 |
| S1/G01 | **done** | `graph/` 权威 + `ekc/graph/` 检索分离 |
| S1/G03 | **done** | 步骤 kind ≠ 全 call；`steps.json`；view `#stmt-` 导航 |
| S1 同名 | **done** | `08-same-name` resolve→A + 运行 `60` |
| S2/G06 | **done** | 包装继承 `fs.write`；未知不标 pure |
| S2/G07 子集 | **done** | MLSP bind 含 `symbol_id`；execute 按源行绑定；pure 拒非纯 |
| S2/G02 BoundCall | **done** | 共享 `resolve_for_exec`；Tree/Bytecode 静态路径绑定；`bound_calls.json` / `dynamic_calls.json`；`10-boundcall` + `06b` |
| S2/G07 eng_reuse | **done（子集）** | `reuse_json` / `agent_eng_reuse`：`discovery_only` + `bind_required` + `bind_targets`；执行仍须 MLSP bind |
| S1/G03 NarrativeRef | **done（子集）** | 注释行 `[]()` → narrate/navigation；`11-narrative-ref` |
| S4/G05 来源 | **done（子集）** | frontmatter `sources`/`valid_until`；`12-tax-source`；business bind |
| S3 patch / extract | **done** | `artifact.patch`；`artifact.extract_pure` |
| S4 书店 | **done** | `13-bookstore` + `examples/executable-knowledge` 发票链 |
| S5/G04·G08 | **done** | 整代 `CURRENT`；锁 classes+closure；impact |
| S6 CI / 语料 / 冒烟 | **done** | CI；100 单元；CLI smoke；扫描上限 |
| F0/M02 证据·闭包·句柄 | **done** | evidence 含来源摘要；锚点诊断；business handle 改/删来源失效；SCC closure；`m02_*` |
| F0/M03 验证裁决 | **done（子集）** | `artifact.verify` 结构化 may_bind；policy 枚举；`knowledge adjudicate` |
| F0/M05 语法 Skill | **done** | CORE SYNTAX 置顶；`migration-classic.md`；`skill_core_card` |
| View V0–V1 | **done** | 正文优先；no-exec 不执行；制品摘要；`KnowledgePresentation`；frontmatter title；说明锚点 |
| View V2 | **done（子集）** | `page_token`/`semantic_generation`；SSE 过期包拒绝；帧选择保留；`local_deps`；sourced≠verified |
| View V3 | **done（子集）** | print/响应式 CSS；100 单元确定性；1000 单元 compile smoke；无 JS 结构块仍在静态 HTML |
| F1/M04 bind 闭环 | **done** | `artifact.reuse_execute` 实际复用；`agent_eng_bind_execute` + `publish_gate`（candidate_only / product gate）；`tests/ext/agent-eng-bind-gate.mq.md` |
| F2/M06–M07 | **done（子集）** | 统一 `EXCLUDED_DIRS`；`publish_manifest.json`；`function_sources`；story 失效/修复 |
| F3/M01 提取 | **done（子集）** | `document.extract` text/Markdown；PDF/DOCX/OCR 显式失败；超限截断 partial；上传 `max_bytes` |

## 与后续切片的关系

- 远程凭据/多租户授权、平台打包产物冒烟、作者/AI 对照实验指标 — 按发布声明另验；不得用本地门禁覆盖为「已安全支持远程」
