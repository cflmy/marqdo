# `marqdo prove`：SKILL 编排的 Lean 编译与顾问式证明

| | |
|---|---|
| 状态 | **计划（Proposed）· 未实现**；决策已定（§10），实施见 [roadmap/prove.md](../roadmap/prove.md) |
| 日期 | 2026-11-13 |
| 层归属 | **Language**：定理表角色 + `check --proof`（§7，[ADR 0010](../adr/0010-skill-proof-boundary.md)）· **Runtime**：`prove` 管线（§4–§6、§8）· **Document**：`verification/skill-proof` 证据 |
| 相关 | [core-surface.md](core-surface.md)（复用 `table` 构造，不新增标记）· [layers.md](layers.md)（准入）· [markdown-mapping-v0.3.md](markdown-mapping-v0.3.md) · [ADR 0010](../adr/0010-skill-proof-boundary.md) · [verification/replay-policy](../../verification/replay-policy/README.md)（Lean 伴随判定的先例） |
| 目标版本 | CLI **2.8.0**（wasm / ext 不随动：`prove` 不进 wasm） |

---

## 0. 一句话与三条契约

`prove` 把一份 `.mq.md` 的**控制平面**（参数 → 分支 → 外部操作 → 返回值）降级成 Lean 4 定义，把作者声明的**定理**变成 Lean 定理，并尽最大努力自动关闭它们；同时输出一份可供人复核的证据（证书、假设、被拒清单）。它**不**是发布门禁，也**不**声称"过了 Lean 就绝对正确"。

三条契约，按优先级排列：

1. **诚实**：`proved` 只表示"Lean 内核接受了这条形式化陈述"。它不表示陈述写对了意图、不表示外部 API 会守约、更不表示整份 SKILL 安全。反过来，`unproved` 只表示"这次没有关闭它"——可能是定理不成立，也可能只是策略不够。
2. **单调**：离线判定（没有 Lean 时）**只能**给出 Lean 也能给出的结论。若两者分歧，以 Lean 为准并报内部错误，绝不放绿。`checked` 永远弱于 `proved`。
3. **不执行**：`prove` 从头到尾不运行被证程序，不产生任何效应；它只读源码、写缓存与 `--emit` 目录。

> 一句话给论文：*给编排文档配一张可复核的证明收据*，而不是给整个语言发一张正确性证书。

---

## 1. 定位：降低审核量，不取代审核

### 1.1 为什么是顾问而不是门禁

Agent 改写 SKILL 后，人要看的是"这一步会不会绕开审核""失败分支会不会吞掉错误"。这些恰好是控制平面命题，Lean 能帮我们把"逐行读代码"压缩成"读几条定理 + 看一眼反例"。但：

- 真实 SKILL 里有字符串、JSON、外部服务；把这些都形式化，成本远高于收益；
- 形式化陈述与业务意图之间永远隔着"作者写得对不对"；
- 大多数团队没有 Lean，强制门禁只会让人绕过工具。

所以 `prove` 的默认行为是**报告**，不是**拦截**。规范与手册**推荐**在发布 SKILL 前跑一次，但不强求；要不要把某个状态当失败，由调用方用 `--require-lean` / `--report-only` 自己决定。

### 1.2 三种"绿"和两种"红"

| 状态 | 含义 | 由谁产生 | 能不能当"已证明" |
|---|---|---|---|
| `proved` | 全部定理被 Lean 内核接受 | Lean | **能**（且仅限陈述本身） |
| `checked` | 全部定理在离线片段（§5）内判定为真，已生成对应 Lean 脚本但未运行 | 离线判定 | **不能**，只能说"离线判定通过" |
| `unverified` | 有定理超出离线片段，且没有跑 Lean | 离线判定 | 不能 |
| `unproved` | 至少一条定理未被关闭；若能给出反例则附见证 | Lean / 离线 | 不能；这是红灯 |
| `rejected` | 有函数不在可证子集（`while`/递归/动态派发），已跳过 | 降级器 | 不能；这是"未覆盖"，不是"错误" |

`rejected` 与 `unproved` 的区别很重要：前者是**工具够不着**，后者是**命题没过**。把它们混成一个红，会让"文件里有个循环"看起来像"业务逻辑错了"。

此外还有 `error`（编码/类型/效应/IO/超时）：它不是判定结果，而是**工具或程序本身出错**，退出 1。五种判定（proved/checked/unverified/unproved/rejected）与 `error` 在 JSON 里共用 `status` 字段，但语义分组不同，报告会分开统计。

### 1.3 诚实性词汇表（写进 README 与 `--json`）

- `proved` ≠ 正确：它只对**写下来的陈述**负责，且假设 `Env` 的返回值任意、`假设` 表成立。
- `unproved` ≠ 错误：策略失败、超时、片段外都可能；只有带 `counterexample` 字段时才是"确实不成立"。
- `rejected` ≠ 不安全：只表示这部分没被形式化覆盖。
- 证书不替代代码评审：它是**缩小**评审范围的工具，`audit` 会明确列出"哪些外部操作完全未知"。

### 1.4 非目标

- 不形式化解释器、字符串库、JSON、浮点、`formula`。
- 不证明外部服务与网络；这些由 `假设` 表显式承担。
- 不在 Agent 每次运行时跑 Lean；运行期只读证书。
- 不自动安装 Lean；只探测并给出可执行提示。
- 不追求"零人工"：目标是让人从"读 200 行"变成"读 5 条定理 + 1 张假设表"。

---

## 2. 证明范围

### 2.1 义务分级

| 级别 | 义务 | 作者要写什么 | 谁保证 | 离线可做 |
|---|---|---|---|---|
| **T0 类型** | 生成的 Lean 通过内核；字段、元数、分支同型 | 无 | Lean | 部分（Rust 类型检查） |
| **T1 结构** | 终止（无 `while`/递归 ⇒ 结构递归）、分支出口完整、返回类型符合 `返回` 契约、效应不越权（给了 `--effects` 时） | 无 | 降级器 + Lean | 是 |
| **T2 声明** | 定理表里的业务命题 | 定理表 | Lean / 离线片段 | 是（F1–F5） |
| **T3 契约** | 外部操作前置/后置假设下的组合推理 | `假设` 表 | Lean | 部分 |
| **T4 一致性** | 生成语义与解释器在纯片段上同值 | 测试语料 | 差分测试 | 是 |

### 2.2 三条硬边界（拒绝，不降级）

| 构造 | 处理 | 诊断码 |
|---|---|---|
| `while`（`-` + 条件行） | 跳过该函数 | `proof.while` |
| 直接/间接递归（调用图有环） | 跳过该环 | `proof.recursion` |
| 动态派发（插件调用、变量函数名、`code` 值调用） | 跳过该调用点 | `proof.dynamic_call` |

被跳过的函数不参与定理，其余函数照常出证。整体状态按固定优先级合成（差者胜）：`error` > `unproved` > `rejected` > `unverified` > `checked` > `proved`。`--allow-opaque <FN>` 可把某个被跳过的函数**整体**降级为不透明操作，让调用它的编排仍可证——此时它的行为完全未知，`audit` 必须列出来。

---

## 3. 命令面

### 3.1 命令与别名

```bash
marqdo prove [PATH]                # 降级 → 生成 → 离线判定 →（有 Lean 时）Lean 判定
marqdo prove emit  [PATH] -o DIR   # 只生成 Lean，不判定（离线/论文/AI 取用）
marqdo prove audit [PATH]          # 只报降级结论：内联了什么、不透明了什么、拒了什么、假设了什么
marqdo prove clean                 # 清 .marqdo/prove 缓存
```

`prove` 带 `visible_alias = "lean"`；文档与 CI 统一写 `prove`。它与既有 `verify` 不重叠：`verify` 查知识一致性，`prove` 查控制平面命题。

### 3.2 参数

| 参数 | 默认 | 含义 |
|---|---|---|
| `PATH` | `index.mq.md` | 单个 `.mq.md`；目录需 `--all` |
| `--all` | off | 目录递归（跳过 `target`/`dist`/`node_modules`/`.git`/`.marqdo`）；`--max-files <N>` 默认 200 |
| `--entry <FN>` | `main` | 入口与报告分组；不限制文件内全量证明 |
| `--only <FN>` | — | 只证这些函数（可重复），用于增量 |
| `--emit <DIR>` | `.marqdo/prove/<key>/Generated` | 保留生成的 Lean |
| `--json` | off | stdout 一个 JSON 对象（§3.5）；诊断走 stderr |
| `--lean <PATH>` / `--lake <PATH>` | 探测 | 显式工具链 |
| `--library <DIR>` | 探测 | `MarqdoProof` 库目录 |
| `--timeout <SEC>` | 120 | 单次 Lean 调用上限 |
| `--heartbeats <N>` | 400000 | `maxHeartbeats` |
| `--no-cache` | off | 绕过证书与 olean 缓存 |
| `--no-offline` | off | 跳过离线判定，只用 Lean（CI 复现/调试用） |
| `--require-lean` | off | 严格模式：`checked`/`unverified`/`rejected` 一律返回 2 |
| `--report-only` | off | 除工具错误外恒返回 0 |
| `--allow-opaque <FN>` | — | 把被跳过的函数降级为不透明操作 |
| `--effects <LIST>` | — | 与 `marqdo call --effects` 同格式；给出时检查实际调用是否越权 |
| `--strict-types` | on | 类型无法推断即报错；`--no-strict-types` 落 `Val`（证明力下降，报告标注） |
| `--tactic <T>` | `auto` | 定理表未写 `策略` 时的默认策略 |

### 3.3 状态与退出码

| 状态 | 退出码 | `--require-lean` | `--report-only` |
|---|---|---|---|
| `proved` | 0 | 0 | 0 |
| `checked` | 0 | **2** | 0 |
| `unverified` | 0 | **2** | 0 |
| `rejected` | 0 | **2** | 0 |
| `unproved` | 1 | 1 | 0 |
| `error` / `timeout` | 1 | 1 | 1 |

设计理由：

- 缺工具链、有循环、定理在片段外，都**不是**程序的错，默认不返回非零——否则没装 Lean 的机器上 `prove` 等于不可用。
- 默认情况下只有两种非零来源：**你写下的定理未被关闭**（1），或**工具出错**（1）。不写定理就不会有第一类失败。
- 想要严格覆盖的团队用 `--require-lean`，把"未判定"升级成 2；CI 里加这一个参数即可。
- 全局开关 `--report-only` 让 Agent 在探索期只看报告。

### 3.4 文本输出

```text
$ marqdo prove skills/marqdo-release/index.mq.md
审批        2/2 定理   proved      Lean 0.8s
plan        1/1 定理   checked     离线判定 12ms（附 Lean 脚本，未运行）
预检         —         rejected    离线 3ms   proof.while L88
prove: proved 1 · checked 1 · rejected 1
  说明: checked 不代表 Lean 已验证；安装 Lean 或传 --lean 可交由 Lean 判定。
  提示: `marqdo prove audit …` 查看降级细节与假设清单。
```

失败时：

```text
error[proof.claim_failed]: 定理「不该自动放行」未被关闭
  --> examples/approval.mq.md:31:3
   = 目标: (run env p).1.ok = false
   = 上下文: h : p.amount > 10000
   = 反例: 金额 = 10001 时走「人工审核」分支，结果 ok = true
   = 说明: 可能是命题不成立，也可能是策略不够；补 `策略` 或改判据后重跑。
```

### 3.5 JSON（面向智能体的稳定契约）

一个对象，`schema` 版本化；永不交互、stdout 只有结果、诊断在 stderr、路径为相对工作目录：

```json
{
  "schema": 1,
  "status": "checked",
  "file": "skills/example/index.mq.md",
  "source_sha256": "…",
  "artifact_id": "mq://…",
  "functions": [{
    "name": "审批",
    "status": "checked",
    "tier": "offline",
    "claims": [
      {"id": "大额必经人工", "status": "checked", "decided_by": "trace", "ms": 3},
      {"id": "退款必须留痕", "status": "unverified", "reason": "out_of_fragment"}
    ],
    "obligations": [{"id": "terminates", "status": "passed"},
                    {"id": "effects_subset", "status": "skipped", "reason": "no --effects"}]
  }],
  "lean": null,
  "assumptions": [],
  "diagnostics": [],
  "emitted": [".marqdo/prove/ab12…/Generated/Approval.lean"],
  "duration_ms": 15
}
```

`lean` 为 `null` 表示没跑；跑了则是 `{"toolchain": "...", "library_sha256": "...", "module_sha256": "...", "cache": "hit|miss"}`。`tier` ∈ `{lean, offline, none}`，标明该函数的结论来自哪一层。`status` 的取值集合固定（§1.2），Agent 可按它分支，不需要解析自然语言。

### 3.6 面向智能体的使用契约

`prove` 的主要调用者不是人，是 Agent 的循环。因此：

- **一次调用一个结论**：不做交互式提问，不读 TTY，不依赖环境变量之外的隐藏状态。
- **只解析 JSON**：`--json` 的键与状态词是稳定契约（`schema` 升级才变）；stdout 不混入程序输出或进度条。
- **失败即数据**：每条诊断都有 `code` / `file` / `line` / `column` / `message`，能定位的还带 `goal`、`context`、`counterexample`、`hint`。Agent 不需要从自然语言里猜。
- **可增量**：`--only <FN>` 只证受影响函数；证书命中让第二轮降到毫秒级。
- **先审计再证明**：Agent 应先 `prove audit --json` 看"有多少内联、多少不透明、多少被跳过"，再决定是否值得为该文件写定理。
- **推荐循环**：写 SKILL → `check` → `prove --json` → 读 `claims[].status` → 改代码/定理/策略 → 重跑 → 把证书附进交付说明。

### 3.7 选择规则

- **文件**：显式路径；目录必须 `--all`（否则 `proof.need_all`）。
- **函数**：文件内全部 `##` 函数默认参与；`# main` 只作分组。
- **导入**：默认内联可判定的纯函数（§4.5），其余成为不透明操作；不递归证明被导入文件。
- **`--all` 聚合**：逐文件出状态，整体取最差；`--json` 输出 `files: [...]`。

---

## 4. 管线

### 4.1 六段

```text
.mq.md
 ① 装载       load_module（只读，不执行）
 ② 降级       AST → Plan IR：子集验收、拒绝码、span
 ③ 定型       contract::Ty 驱动；未定 ⇒ 报错或 Val
 ④ 生成       IR → Lean（确定性：稳定排序、无时间戳、相对路径）
 ⑤ 离线判定   符号执行 + 片段判定（§5），产出 checked / 反例 / unverified
 ⑥ Lean 判定  有工具链时运行；以它为准，与 ⑤ 分歧即 error
```

⑤ 永远在 ⑥ 之前：离线先给快速反馈，Lean 随后确认。`--no-offline` 可只跑 ⑥。

### 4.2 Plan IR

`src/prove/ir.rs`，`serde` 序列化为 `plan.json`（版本化，缓存键的一部分，论文工具可直接消费）：

```jsonc
{
  "schema": 1,
  "module": "skills/example/index.mq.md",
  "source_sha256": "…",
  "functions": [{
    "name": "审批",
    "params": [{"name": "金额", "ty": {"kind": "int"}}],
    "result_ty": {"kind": "map", "fields": [["ok","bool"],["value","any"],["error","any"]]},
    "ops": [{"id": "人工审核", "effect": "network.read", "params": [["金额","int"]]}],
    "steps": [
      {"kind": "if", "cond": {"op": "gt", "args": [{"var": "金额"}, {"int": 10000}]},
       "then": [{"kind": "bind", "var": "审核", "expr": {"call": "人工审核", "args": [{"var": "金额"}]}},
                {"kind": "return", "expr": {"ctor": "result.ok", "args": [{"var": "审核"}]}}],
       "else": [{"kind": "return", "expr": {"ctor": "result.ok", "args": [{"text": "auto"}]}}],
       "span": {"line": 21, "col": 1}}],
    "claims": [{"id": "大额必经人工", "premise": null, "conclusion": "…", "tactic": null}],
    "effects": ["network.read"],
    "assumptions": []
  }]
}
```

表达式分两层：`PureExpr`（字面量、变量、算术/比较/布尔、字符串拼接与插值、列表/映射字面量、`access`、`len`、类型转换）可翻译成 Lean 项；其余一律 `Call`（不透明操作），或按 §4.3 拒绝。**没有"猜"的回退。**

### 4.3 接受 / 拒绝矩阵

| Marqdo | 处理 | 备注 |
|---|---|---|
| `**x = 纯表达式**` | `bind` | 类型由右值推断 |
| `**x = [标签](调用)**` | `bind` + `Op` + `trace` | 效应来自 sema |
| `**x**`（粗体无赋值） | `return` | 纯表达式或已内联构造器 |
| `*语句*` / `> 调用` | 语句形式，值丢弃 | 丢弃的调用仍进 trace |
| `1. 条件 / 2. *` | `if/else` | 条件必须是纯布尔表达式 |
| `- [项](遍历 列表 为 项)` | `foreach`（P3） | 只接受有限列表；此前 `proof.foreach_pending` |
| `-` + 条件 | **跳过** `proof.while` | |
| 递归 | **跳过** `proof.recursion` | 调用图环检测 |
| 方法调用 `` `recv`.方法 `` | 不透明操作 | 接收者不展开 |
| `code` 值 / 插件调用 | **跳过** `proof.dynamic_call` | 不在纯片段 |
| GFM 表绑定 | `list`/`map` 字面量 | 列长不一致已在解析期拒绝 |
| `$$` 公式 | 不透明值（`Formula`） | 不翻译 TeX，不做推理 |

### 4.4 类型推断

起点：`参数` 表 + `返回` 表（`contract::Ty`）。合一规则：

- 字面量给最具体类型；`+ - * /` 要求两侧 `int`；`and/or/not` 要求 `bool`。
- 分支合流：两臂同名变量类型必须相容，否则 `proof.type_mismatch`（带两处行号）。
- 映射字段：返回表达式是映射字面量时得到 `structure`；是内联的 `result.ok/error`（lib/result 真实 API）时沿用 Result 形状；否则落 `Val`，此时 `结果.ok` 报 `proof.result_untyped`（提示：返回映射字面量或包成 Result）。
- `--no-strict-types` 时未知类型落 `Val`；定理里只能做等式/成员判断，报告标注"弱类型"。

### 4.5 内联与不透明化

1. 建调用图（本文件 + 导入文件）；有环即跳过该环。
2. **内联**：被调函数（含 `lib/result` 一类纯库）满足 ①契约 `Effects: pure` ②语句数 ≤ 24 ③不含被跳过构造 ④非递归 → 展开函数体。这样 `result.ok(...)` 在 Lean 里就是 Val 映射构造（`mqResultOk`），`结果.ok` 变成可判定字段。
3. **不透明化**：其余调用生成 `Op` 构造器 + 效应，由 `Env` 参数处理；`--allow-opaque` 可强制。
4. `audit` 逐条列出：内联 / 不透明 / 跳过，以及每个不透明操作的效应与参数类型。

---

## 5. 离线判定（没有 Lean 也能给结论）

### 5.1 为什么需要

- 大多数开发机与 CI 没有 Lean；没有离线路径，`prove` 就只是"生成器"。
- Agent 迭代需要毫秒级反馈，Lean 一次数百毫秒到数十秒。
- 反例（"金额 = 10001 时轨迹里没有人工审核"）比 tactic 失败更有用。

但离线**绝不能**比 Lean 强。做法不是"相信我们的判定器"，而是把它关进一个**极小的片段**，并让每条离线结论都附带 Lean 脚本。

### 5.2 片段 F1–F5（保守，超出即 `unverified`）

| 片段 | 能判定什么 | 为什么安全 |
|---|---|---|
| **F1 路径枚举** | 控制流是有限树（无循环/递归）⇒ 返回路径有限 | 由 §2.2 的拒绝保证 |
| **F2 轨迹** | 每条路径的 `Op` 序列由分支决定；`包含/长度/下标` 逐路径求值 | 轨迹是语法产物，不依赖 `env` 返回值 |
| **F3 字面结果** | 返回表达式只由字面量、参数、内联构造器组成时按结构求值 | 与 Lean 的 `mqResultOk` 形状一一对应 |
| **F4 命题** | `and/or/not`、`=/≠` 作用于 F2/F3 的布尔结果；外部返回值上的**布尔判定**（如 `access(回执,"ok") == False`）视为自由原子，可分叉 | 自由原子只做二分、不假设原子间关系；两条分支都关闭才 `checked`，因此证明是对任意 `env` 成立的 |
| **F5 单变量区间** | 原子 `x ⋈ c`（`c` 为整数字面量，`⋈ ∈ {<,≤,>,≥,=,≠}`）；用路径条件 + 前提的上下界判定蕴含 | 单变量区间推理是线性整数算术的子集，`omega` 必然也能证 |

明确**不**离线判定（直接 `unverified`，不猜）：多变量线性组合、非线性算术、字符串比较、外部返回值的非布尔字段（`.value` 之类）、任何需要归纳的命题。

### 5.3 判定过程

1. 符号执行函数体：状态 `Var → SymVal`（`Const` / `Param` / `OpResult` / `Unknown`），路径条件为原子列表，轨迹为 `Op` 列表。
2. 每个 `if` 分叉两支（条件 / 其否定）；每个 `Op` 调用产生一个 `OpResult` 符号（值未知，但**出现**已知）。
3. 每个 `return` 收集 `(路径条件, 结果 SymVal, 轨迹)`。
4. 对每条路径检查 `前提 ∧ 路径条件 ⊢ 结论`：把 `结果/轨迹` 代入后按 F2–F5 求值或判定。
5. 全部路径关闭 ⇒ 该定理 `checked`；任一原子超出片段 ⇒ `unverified`；某条路径可满足且结论为假 ⇒ `unproved` + 反例。

### 5.4 证书与"单调性"

每条 `checked` 定理同时产出：

- **情形证书**：`(路径 id, 路径条件, 规范化目标, 关闭理由 ∈ {literal, hypothesis, interval, atom})`，写进 `certificate.json`；
- **Lean 脚本**：`unfold run` 后按同一棵 `if` 树 `split_ifs`，每个叶子 `simp_all <;> omega`。离线情形树与 Lean 的 `if` 树来自同一份 IR，叶子一一对应；某个叶子离线关闭而 Lean 关不掉，就是 `proof.divergence`。

因此"离线判定 ⊂ Lean 判定"不是口头承诺，而是三件事：

1. **构造上**：F5 的推理是 `omega` 的子集，F1–F4 是 `simp`/`decide` 的子集；
2. **运行期**：只要有 Lean，就在 ⑥ 复核；`checked` 但 Lean 未关闭 ⇒ `error[proof.divergence]`，退出 1，永不落绿；
3. **测试上**：语料库强制 `offline_checked ⊆ lean_proved`（roadmap 测试矩阵 A1）。

**不夸大**：我们不说"离线判定器已被证明正确"。我们说它片段极小、保守、每条结论都带 Lean 脚本，且有运行期复核与 CI 语料强制。

### 5.5 反例

当某条路径可满足、且结论在该路径上求值为假时，从区间端点取一组参数值作为见证：

```text
反例: 金额 = 10001
路径: 金额 > 10000 → 人工审核
结论: 结果.ok = False
实际: 结果.ok = True（人工审核的返回值被包进 result.ok）
```

反例只承认**确定性否定**：结论必须是字面 `false`，路径必须由区间证明可满足，且路径条件里**不含自由布尔原子**（否则可能凑出真实语义下不存在的组合）。任何一条不满足就退回 `unproved`（无见证）或 `unverified`，不编故事。

---

## 6. Lean 编码

### 6.1 产物布局

```text
verification/skill-proof/          # 入仓：语义库 + 论文样例（人工评审）
  lean-toolchain                   # 与 replay-policy 同钉（v4.35.0）
  lakefile.lean
  MarqdoProof.lean
  MarqdoProof/{Basic,Result,Tactic}.lean
  examples/{approval,false-claim}.lean
  README.md                        # TCB 与"证了什么"的边界声明
  LIBRARY-SHA256                   # 由构建脚本写入

.marqdo/prove/                     # 生成物（.marqdo/ 已 gitignore）
  pkg/                             # 一次性 Lake 包（库副本 + Generated/）
  <key>/plan.json module.lean certificate.json lean.log
  index.json                       # key → 指纹/状态/耗时
```

### 6.2 无公理：操作是参数，假设是前提

```lean
-- 库提供（MarqdoProof/Basic.lean，节选）
inductive Val | none | int : Int → Val | str : String → Val | bool : Bool → Val
              | list : List Val → Val | map : List (String × Val) → Val

structure Err where code : String; message : String; operation : String
structure Result where ok : Bool; value : Val := .none; error : Option Err := none

-- lib/result 的真实构造器（ok/error/value_or）映射：
def Result.success (v : Val) : Result := { ok := true, value := v }
def Result.failure (e : Err) : Result := { ok := false, error := some e }
```

生成文件里**不出现 `axiom`，也不出现 `opaque`**：外部操作是 `Env := Op → Val` 参数，后置条件是定理的假设 `h_post`。于是 `#print axioms` 为空，定理陈述本身就把"信任了什么"写清楚。

### 6.3 直接风格降级（无 monad、无 `do`）

`StateM`/`do` 在证明里要额外展开 monad 定律，早返回的语义也容易踩坑。改用**续延式直接风格**：`compile_stmts(stmts, k)` 在 Rust 里把后续语句当续延，早返回就是"把 `k` 换成函数出口"。产出的 Lean 是纯 `let`/`if`/`pair`，`simp`/`split`/`omega` 直接可用。

输入（`verification/skill-proof/examples/approval.mq.md` 节选）：

```markdown
## 审批
+ `金额`

| 参数 | 类型 | 说明 |
|---|---|---|
| 金额 | int | 申请金额（元） |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result |

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 大额必经人工 | 包含(轨迹, "人工审核") | 金额 > 10000 | auto |

1. `金额` > 10000
  **审核 = [提交人工审核](人工审核(金额=`金额`))**
  **结果 = [保留人工审核结论](result.ok(value=`审核`))**
2. *
  **结果 = [自动通过](result.ok(value="auto"))**

**结果**
```

输出：

```lean
import MarqdoProof
open MarqdoProof

namespace Generated.Approval

structure Params where
  金额 : Int
  deriving Repr, DecidableEq

inductive Op where
  | 人工审核 (金额 : Int)
  deriving Repr, BEq

def Op.name : Op → String
  | .人工审核 _ => "人工审核"

abbrev Env := Op → Val

def run (env : Env) (p : Params) : Result × List Op :=
  if p.金额 > 10000 then
    let v := env (.人工审核 p.金额)
    (Result.success v, [.人工审核 p.金额])
  else
    (Result.success (.str "auto"), [])

theorem 大额必经人工 (env : Env) (p : Params) (h : p.金额 > 10000) :
    (run env p).2.any (fun o => o.name == "人工审核") = true := by
  unfold run
  split_ifs <;> simp_all <;> omega

end Generated.Approval
```

要点：每步带 `-- mq: line:col` 注释；中文标识符直接可用，含点/空格的用 `«…»`；命名冲突在生成前消解。

### 6.4 义务与定理

| 项 | 形状 | 来源 |
|---|---|---|
| 类型检查（T0） | 文件通过内核：字段、元数、分支同型 | Lean，编译结果即证据 |
| 终止（T1） | 无 `while`/递归 ⇒ Lean 接受为 `def` | 同上；需要 `partial` 的一律已跳过 |
| `effects_subset` | 实际 `Op` 集合 ⊆ `--effects` 授权 | Rust 判定，写证书 `obligations` |
| 作者定理（T2） | `∀ p, pre p → post p (run env p)` | 定理表 |
| 假设（T3） | `h_* : ∀ …, Post …` 作为定理前提 | 假设表 |

违反 `effects_subset` 不是 `rejected` 而是 `error[proof.effects_exceeded]`：前者是"够不着"，后者是"够得着且越权"，必须区分。未给 `--effects` 时该义务记为 `skipped`，不假装通过。

### 6.5 策略

```lean
macro "mq_auto" : tactic =>
  `(tactic| first
    | decide
    | omega
    | simp_all
    | (split <;> simp_all)
    | (split_ifs <;> simp_all <;> omega)
    | aesop)
```

定理表 `策略` 列可写 `auto` / `omega` / `simp [run]` / 任意 tactic 片段；写 `none` 表示只生成不证（论文留人工证明钩子），状态记 `unverified` 而不是失败。

### 6.6 源码映射

每个语句前放 `-- mq: <line>:<col>`；`module.lean` 与 `plan.json` 同缓存。Lean 报 `Generated/X.lean:41:7: error: …` 时，驱动把 41 映射回 `.mq.md` 行，诊断指回源码，`lean_line` 放在 JSON 细节里。映射表进证书。

---

## 7. 作者面：定理表与语言习惯

### 7.1 语法与位置

表格构造**已有**（不新增核心标记），这里只给它一个新的**语义角色**：

```markdown
| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 大额必经人工 | 包含(轨迹, "人工审核") | 金额 > 10000 | auto |
```

规则（与 `参数/返回/字段` 契约表同构）：

1. 表头别名：`定理/证明/claim/theorem`；列名 `结论/conclusion`、`前提/premise`（可缺省）、`策略/tactic`（可缺省）。
2. 位置：函数体**首条可执行语句之前**，与契约表同一槽位；提取后从 body 移除，**不参与运行**。出现在别处仍是普通表，`check --proof` 报 `proof.claim_misplaced`。
3. 一行一条；`定理` 列是函数内唯一 id，重复即 `proof.claim_duplicate`。
4. 结论/前提是**受限表达式**：参数、`结果`、`轨迹`、`包含/contains`、`长度/length`、`下标/index`，以及算术/比较/布尔。其他名字 → `proof.claim_unknown_symbol`（带最近建议）。
5. 一张表如果既像契约表（有 `类型` 列）又像定理表（有 `结论` 列）→ `proof.claim_ambiguous`，不猜。

### 7.2 语义

一条定理展开为：

```lean
theorem <id> (env : Env) (p : Params) (h : <前提>[p]) :
    <结论>[p, (run env p)] := by …
```

- 前提在**入口状态**求值（参数），结论可引用参数、`结果`、`轨迹`。
- 没有前提就是全称命题；`结果.ok = True` 天然表示"所有成功终止都满足"。
- `轨迹` 有序；`包含/长度/下标` 三个谓词够表达"存在/顺序/计数"，不再长大。
- 定理是**声明**，不是代码：没有求值顺序、没有副作用、不能调用任意函数。

### 7.3 语言习惯审查（是否引入了多余语义）

这是这次设计最需要交代清楚的一节。逐条回答"会不会破坏语言习惯"。

#### 7.3.1 实际新增了什么

| 新增项 | 性质 | 是否影响运行 |
|---|---|---|
| `Function.claims` 元数据 | 声明（与 `Function.contract` 同类） | **否**，提取后从 body 移除 |
| 定理作用域的两个保留名（`结果/轨迹`）与三个谓词 | 只在定理内可见的绑定 | **否**，不在普通表达式作用域 |
| 假设表作用域的 `返回` | 只在假设表内可见的绑定 | **否** |
| `check --proof` | 静态校验开关，默认关 | 否 |

除此之外没有新语法、没有新标记、没有新求值规则、没有新类型系统。

#### 7.3.2 没有新增什么（逐项对照）

| 风险项 | 结论 | 依据 |
|---|---|---|
| 新核心标记？ | 否 | 用的是既有 GFM 表（`core-surface.md` 的 `table`）；`CORE_CONSTRUCTS` 计数不变 |
| 新求值顺序/作用域？ | 否 | 定理表与契约表一样，在解析期提取，不进 AST body |
| 新调用约定/形参？ | 否 | 不要求调用者传证明参数，不给函数加隐藏形参 |
| 新类型系统（refinement/依赖类型）？ | 否 | 只在表里写命题；不做类型层扩展 |
| 影响叙述比例/可读性？ | 否 | 表是 code 构造；GitHub 渲染正常；`labels`/`narrative_ratio` 行为不变 |
| 影响 `check` 既有诊断？ | 否 | 新表头不命中 `参数/返回/字段`，`parse_contract_table` 不认领；新增校验默认关 |

#### 7.3.3 为什么不是别的写法

| 备选 | 为什么不用 |
|---|---|
| frontmatter `proof:` 块 | 丢函数作用域与行级锚点；一个函数一条定理时锚点落在文件头，诊断无法指回具体行；多函数文件无法分派 |
| `# 证明` 标题 | `#` 是对象、`##` 是函数，标题即单元——新增"标题=命题"会破坏"标题即可调用单元"这条既有习惯，且会进命名空间 |
| 围栏块（` ```proof ` 语言标签） | 代码围栏必须跟在空赋值后（`x =` + fence），否则解析器直接报错；为了写定理先造一个死变量，比表更别扭，且该值会被当作可执行值 |
| 注释里写定理（`<!-- -->`） | 注释不参与语义，也拿不到行锚点；工具无法可靠区分"说明"与"定理" |
| 单独 `.proof.md` 旁挂文件 | 真理被拆成两份，正是这个项目一直在避免的事（"代码与文档同一份"） |
| 类型即契约（refinement） | 语义增量太大，且现有契约表已经承担类型声明；再来一套会形成"两份作者真相" |

函数级表格是最符合本项目习惯的：**声明与函数同址、可定位、可校验、不可执行**——这正是 `参数/返回/字段` 表已经建立的模式。

#### 7.3.4 命名冲突与遮蔽

- 定理作用域里 `结果`/`轨迹`、假设作用域里 `返回` 是保留名。若函数有形参或局部变量同名，`check --proof` 报 `proof.name_shadow`（错误），要求改名——**不**做隐式优先，避免"同名不同义"。
- 谓词名 `包含/长度/下标` 若与用户函数同名，同样报 `proof.name_shadow`，不做静默覆盖。

#### 7.3.5 仍然存在的风险与护栏

| 风险 | 说明 | 护栏 |
|---|---|---|
| "定理像代码"的错觉 | 作者可能以为 `结果.ok = False` 会在运行时求值 | 文档措辞统一为"声明"；`view` 渲染成定理卡片；`check` 提示不可运行位置 |
| 定理表长出第二门语言 | 有人想写量词、自定义谓词、调用函数 | 谓词集合封闭（3 个）；超出即 `proof.claim_unsupported`；ADR 改集合需重新评审 |
| 定理与代码漂移 | 改了分支但没改定理 | 定理锚定行号；`check --proof` 校验引用的变量仍存在；`prove` 每次重证 |
| 定理变成装饰 | 写了 `策略 none` 或永远 `unverified` | `audit` 统计"声明/已判定/未判定"；证书只记录实际判定结果 |
| 表被误当数据表 | 普通文档表恰好叫 `定理` | 只有表头命中且列形正确才认领；两列的"定理/说明"表仍是普通表 |

结论：**语义增量被压到三条声明级绑定，且不进入运行时**。它没有引入新的编程习惯，只是把"契约表"这一既有习惯扩展到"命题"。

### 7.4 假设表（T3）

外部操作默认完全未知。要让"只要银行返回失败，我们就不会记账"这类推理成立，把假设写成表：

```markdown
| 假设 | 前置 | 后置 |
|---|---|---|
| 银行.转账 | 金额 > 0 | 返回.ok = True |
```

- 表头 `假设/assumption`；`操作` 列是调用显示名（`库.成员`）。
- 前置在实参上求值；后置在 `返回`（操作返回值）上求值；支持 `=/≠/包含/长度` 与 `返回.ok`（映射字段读取的语法糖）。
- 生成 Lean 时不是公理，而是**定理前提**：`h_转账 : ∀ a, a > 0 → (env (.转账 a)).field? "ok" = some (.bool true)`。
- 无法定型的后置 → `proof.assumption_untyped`（提示改成等式或补返回契约）。
- `audit` 与证书 `assumptions` 一一对应。

### 7.5 与 `check` / `view` / `catalog`

- `check`：新增 `--proof`（**默认关**，保持既有性能），校验表形、名字、片段；不跑 Lean。
- `view`：定理表渲染成"已声明定理"卡片；有证书时附状态徽标（`proved`/`checked`/`unverified`）。不触发证明。
- `catalog`/`knowledge`：`Artifact` 增可选 `proof:` 摘要（证书哈希 + 状态 + 定理数），供 EKM 与 `reuse` 过滤；不改变现有 schema 必填项。

---

## 8. 工程落点

### 8.1 新增 / 修改文件

| 文件 | 变更 |
|---|---|
| `src/prove/mod.rs` | API、状态机、报告聚合 |
| `src/prove/ir.rs` | Plan IR + 规范化 JSON + 指纹 |
| `src/prove/lower.rs` | AST → IR、子集验收、拒绝码、内联 |
| `src/prove/types.rs` | 类型推断与合一 |
| `src/prove/emit.rs` | IR → Lean（确定性、命名消解、span 注释、脚本生成） |
| `src/prove/offline.rs` | 符号执行、F1–F5 判定、反例 |
| `src/prove/driver.rs` | 工具链探测、Lake 包、`lean` 调用、超时、诊断映射 |
| `src/prove/cache.rs` | 证书、`index.json`、失效判定 |
| `src/prove/report.rs` | 文本 / `--json` 渲染 |
| `src/ast.rs` | `Function.claims: Vec<Claim>` |
| `src/claim.rs`（或并入 `contract.rs`） | 定理表提取与校验 |
| `src/parse/mod.rs` | 定理表提取入口（首条可执行语句前） |
| `src/check.rs` | `--proof` |
| `src/main.rs` | `Commands::Prove`、`ProveAction`、json_errors |
| `src/lib.rs` | `pub mod prove;` |
| `verification/skill-proof/**` | Lean 库、样例、`LIBRARY-SHA256`、README |
| `tests/prove.rs` + `tests/prove/**` + `tests/prove_lean.rs` | 集成测试、夹具、Lean 语料（独立目标） |
| `.github/workflows/proof.yml` | elan + `lake build` + 语料门禁 |

不新增 crate 依赖（`serde_json`、`sha2`、`anyhow` 已在）；Lean 侧零依赖（只用 Std）。

### 8.2 API

```rust
pub struct ProveOptions {
    pub entry: Option<String>,
    pub only: Vec<String>,
    pub allow_opaque: Vec<String>,
    pub emit_dir: Option<PathBuf>,
    pub effects: Option<String>,
    pub strict_types: bool,
    pub offline: bool,
    pub timeout: Duration,
    pub heartbeat: u64,
    pub require_lean: bool,
    pub report_only: bool,
    pub no_cache: bool,
    pub lean: Option<PathBuf>,
    pub library: Option<PathBuf>,
}

pub enum Verdict { Proved, Checked, Unverified, Unproved, Rejected, Error }

pub fn prove_path(path: &Path, opts: &ProveOptions) -> Result<Report>;
pub fn emit_path(path: &Path, out: &Path) -> Result<EmitReport>;   // 不判定
pub fn audit_path(path: &Path) -> Result<AuditReport>;             // 不判定
```

`main.rs` 只做参数搬运与退出码映射（与 `Commands::Verify` 同风格）。

### 8.3 缓存与证书

缓存键：`sha256(源码 ⊕ plan.json ⊕ schema ⊕ 库 sha256 ⊕ lean --version ⊕ marqdo 版本)`。证书：

```jsonc
{
  "schema": 1,
  "key": "…",
  "status": "proved",
  "tier": "lean",
  "source": {"path": "skills/…", "sha256": "…", "artifact_id": "mq://…"},
  "lean": {"toolchain": "…", "library_sha256": "…", "module_sha256": "…", "axioms": []},
  "claims": [{"id": "…", "status": "proved", "decided_by": "omega"}],
  "offline": {"claims": [{"id": "…", "status": "checked", "cases": 3}]},
  "obligations": [{"id": "effects_subset", "status": "passed"}],
  "assumptions": ["银行.转账: 前置 金额 > 0；后置 返回.ok = True"],
  "span_map": {"41": "21:1"},
  "created": "2026-11-13T09:00:00Z",
  "marqdo": "2.8.0"
}
```

- 命中即跳过判定（报告标 `cache: hit`）。
- 源文件、库、工具链任一变化即失效；暂不做部分重证。
- 证书是**复核收据**，不是发布许可：推荐附在发布说明里，不强制。
- `prove clean` 删 `.marqdo/prove`（`--emit` 目录不动）。

### 8.4 工具链与库发现

1. `--lean`/`--lake`；2. `MARQDO_PROOF_LEAN`/`MARQDO_PROOF_LAKE`；3. `PATH` 上的 `lake`，再 `lean`；4. 都没有 → 只跑离线，`lean: null`。

显式给出（flag 或环境变量）但路径不存在/不可执行 → `error[proof.lean_missing]`，**不**回退到离线：避免"我以为在跑 Lean，其实没有"。

库发现：`--library` → `MARQDO_PROOF_LIB` → 仓库内 `verification/skill-proof` → 安装根 `share/marqdo/proof/` → 相邻 `marqdo-proof/`。首次使用按 `LIBRARY-SHA256` 复制进 `.marqdo/prove/pkg` 并 `lake build` 一次（缓存）；库哈希进证书。

### 8.5 环境变量

| 变量 | 作用 |
|---|---|
| `MARQDO_PROOF_LEAN` / `MARQDO_PROOF_LAKE` | 工具链路径 |
| `MARQDO_PROOF_LIB` | 语义库目录 |
| `MARQDO_PROOF_TIMEOUT` | 默认超时（秒） |
| `MARQDO_PROOF_CACHE` | 缓存根（默认 `.marqdo/prove`） |
| `MARQDO_PROOF_NO_CACHE` | 等价 `--no-cache` |
| `MARQDO_PROOF_REQUIRE_LEAN` | 等价 `--require-lean`（CI/测试用） |

---

## 9. 测试与证据（摘要）

完整矩阵见 [roadmap/prove.md](../roadmap/prove.md) §6。四类必须守住的证据：

1. **单调性**（用户硬要求）：语料库里每条离线 `checked` 的定理，Lean 都必须证出来；任何分歧 → `error[proof.divergence]`。
2. **不执行**：带 `fs.write`/`exec` 的夹具在 `prove` 后不产生任何副作用。
3. **确定性**：同输入两次运行，`module.lean` / `plan.json` / 证书键全等。
4. **诚实性文案**：`checked` 与 `proved` 在文本、JSON、证书三处措辞一致，不允许出现"已验证"字样描述离线结果。

---

## 10. 已定事项

| # | 事项 | 结论 | 备注 |
|---|---|---|---|
| 1 | 命令名 | `marqdo prove` | 别名 `lean` |
| 2 | 是否强制 | **不强制**；规范与手册推荐 | 无 `--require-lean` 时缺工具链不失败 |
| 3 | 离线能否给结论 | 能，但只到 `checked` | 严格弱于 Lean（§5） |
| 4 | 定理写法 | 函数级 `定理` 表 | [ADR 0010](../adr/0010-skill-proof-boundary.md) |
| 5 | `check --proof` | 加，默认关 | |
| 6 | 首版 `foreach` | 不做，P3 再加 | 之前 `proof.foreach_pending` |
| 7 | 状态词 | `proved` / `checked` / `unverified` / `unproved` / `rejected` | 只有 `proved` 能说"已证明" |

遗留（不阻塞开工，P2 前定）：`假设` 表是否也认领 `前置/后置` 之外的列（如 `来源`）；`--all` 的聚合粒度（按文件还是按函数）；证书是否需要签名。

---

## 11. 文档与发布清单

- [ ] 本文件转 `Implemented`（分阶段勾）
- [ ] `doc/roadmap/prove.md`：里程碑、周计划、测试矩阵
- [ ] `doc/adr/0010-skill-proof-boundary.md`：定理表角色 + 子集边界 + 顾问式定位
- [ ] `doc/design/skill-proof-theory.md`：语义映射、定理形态、TCB 表
- [ ] `verification/skill-proof/README.md`：构建、身份哈希、与 replay-policy 的关系
- [ ] `README.md`：CLI 三行示例 + 状态表 + "推荐但不强制"
- [ ] `doc/tutorial/13-命令行工具.md`：先 `emit`，再 `prove`
- [ ] `skills/marqdo-dev/index.mq.md`：新增 `prove` 任务（推荐在发布前跑）
- [ ] `skills/marqdo/reference.md`：定理表语法速查
- [ ] `CHANGELOG.md`：2.8.0；「核心表面 Δ：无（表格新增语义角色：定理）」
- [ ] `.github/workflows/proof.yml`；`.gitignore` 已含 `.marqdo/`
- [ ] 发版手册：推荐附证书哈希，不设为硬门禁

---

## 12. 实现状态与偏差记录（P0/P1 落地时修订）

| 状态 | 内容 |
|---|---|
| **P0 已落地** | `src/prove/`：`ir.rs`（Plan IR + 稳定 JSON）、`lower.rs`（子集验收/内联/不透明化/效应标签）、`types.rs`（字面量推断 + 分支合流 + `proof.type_mismatch`）、`emit.rs`（确定性 Lean 生成）、`mod.rs`（Report/Verdict/ProveOptions + `prove/emit/audit/clean`）。测试：`tests/prove.rs` 12 例 + 模块单元测试，全部不依赖 Lean。 |
| **P1（Rust 侧）已落地** | 定理表提取（`ast::Function.claims` + `contract::parse_claim_table`，首条可执行语句之前，提取后移出 body）；`check --proof`（默认关）：`proof.claim_malformed` / `claim_ambiguous` / `claim_misplaced` / `claim_duplicate` / `claim_unknown_symbol` / `claim_unsupported` / `name_shadow`；定理脚手架生成（受限文法 → Lean `theorem` 陈述 + §7.2 策略脚本）。Lean 判定（lake/driver/cache/CI）按 roadmap P1 第 2 周继续。 |

### 与本文前面章节的偏差（以本表为准）

1. **`result.ok/error/value_or`，不是 `result.success/failure`**：设计起草时假设的库名与 `lib/result.mq.md` 真实 API 不一致，实现以真实库为准（§4.5/§6.2 示例已同步修正）。Lean 侧库提供 `Result.success/failure` 作为映射目标。
2. **P0/P1 的 `run` 返回 `Val × List Op`**：`result.ok(...)` 内联为 Val 字面映射构造（`mqResultOk`），而非单独的 `Result` 结构。P2 起按返回契约提升为类型化 `structure Params`/`Result`（§6.2 的形态），届时定理表可直接做字段级推理。
3. **Op 构造器带元数**：`inductive Op` 的每个构造器按首现实参个数声明 `参数1..参数N : Val`（§6.2 示例省略了这一点）。
4. **`--allow-opaque` 的效应标签恒为 `opaque`**：即便函数声明了契约效应也不沿用——行为完全未知时引用其声明只会误导 `audit` 读者（§2.2）。
