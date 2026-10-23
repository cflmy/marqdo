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

### 同名裸调用（G02）

`06-reject/ambiguous.mq.md` 同时 `import` 了 `a` 与 `b`（同名 `商品小计`），正文用**裸名**调用。双后端均须拒绝；正确写法是 `a.商品小计` / `b.商品小计` 等显式路径。

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
| S2/G02 BoundCall | **done（作用域子集）** | Bytecode 裸名解析不跨 import scope；`06b` 双后端拒绝 |

## 与后续切片的关系

- S2 后续：sema 路径 BoundCall 记录、Agent eng_reuse 强制 bind
- S3+：局部编辑、来源适用性、整代快照 — 见缺口路线图
