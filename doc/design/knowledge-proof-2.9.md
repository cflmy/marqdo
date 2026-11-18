# Marqdo 知识锚点与证明制品（2.8 → 2.9）

> 实现状态：PR #2 第一阶段。保持解释器和原有 `[]()` 求值语义不变，复用 2.8.0 `prove` 与既有 SemanticProject。以下明确区分**已经写入分支的功能**和**尚未完成的跨模块证明**。

## 核心语义

- AST 内部步骤仍有原来的 `#step-N` 序号，供旧查询、调试与执行日志使用。
- 在**同一个函数内名称唯一**的标记步骤，获得独立的 `anchor_id = SymbolId#anchor-sha256(normalized_label)`；在前面插入其他步骤不会改变地址。
- 同函数重复的标记名称**不产生公共锚点**；必须重命名，不得静默采用首个。
- `definition_digest` 来自该语句源码位置的内容，与稳定身份分开；语句内容更改时身份保留、定义指纹改变。
- 两种文档引用不执行目标：同函数的 `[阅读依据](#@演算)` 和跨模块绝对地址 `[阅读依据](mq://...#anchor-<sha256>)`。后者应从编译生成的 anchors.json 拿完整 ID，不应凭空手写。
- 2.8.0 定理表每条唯一声明生成稳定 `ClaimId = SymbolId#claim-sha256(定理名称)`；正文 `[阅读定理](#@定理名称)` 可以定位**声明**，但不会读取或伪造证明状态；同一名称同时匹配知识步骤和定理声明时标记 `ambiguous_anchor`，不猜测。\n- 普通 `[说明](https://...)` 仍为导航，原来表达式位置的 `[描述](payload)` 仍依照原 Marqdo 执行规则求值。
- **链接不意味着执行权**：解析到知识锚点只生成引用边，不自动调用函数、加载插件或赋予权限。

## 制品

`marqdo knowledge` 所用的 SemanticProject 原子生成目录下新增：

| 路径 | 内容 |
|---|---|
| `steps.json` / `refs.json` | 增量字段 `anchor_id`、`definition_digest`、`link_target`、`target_anchor` |
| `claims.json` | 定理声明索引，初始状态固定为 `declared_unverified`，不等同于证明通过 |\n| `anchors.json` | 稳定公共步骤索引，附所有者、标记文本、源码、效应、定义指纹 |
| `graph/edges.json` | `defines_anchor`、`references_anchor`、知识步骤到已解析调用目标的 `calls` |
| `proof-index.json` | 可用或过期的证明收据索引；它不是可执行授权，也不自动证明步骤 |

`marqdo prove` 在原本的证明目录附带 `knowledge-links.json`：
每个定理通过文件 + 函数全名唯一绑定到 `SymbolId`，稳定 `claim_anchor_id` 附带前提、结论、来源、函数的 interface/implementation/closure digest 与可用时的 Lean 证书摘要。
无法安全绑定时输出 `unresolved`，不匹配别的同名函数。
`related_anchor_ids` 仅是 **defined_in_same_function_not_individually_proved** 导航关系，绝不意味着 Lean 已分别证明这些内部锚点的性质。

## 证据有效性

1. 离线 `checked` 不能当作 `proved`。
2. 只有 `ClaimReport` 显示真实 Lean 判定，且匹配当前报告的原始 `certificate.json`、命题、文件、Lean 工具链与证明库，才可输出 `proved` 关联。
3. 在知识编译时复核文件源码 SHA、函数接口、实现指纹、传递依赖 closure 指纹及证书字节摘要；失败 => `stale` / `unverified`，`usable=false`。
4. 同一个声明出现多份可见报告时 => `ambiguous`，不能随意取旧证书或新证书。
5. 当前证明文件是**本地缓存收据**，不是经过签名的远程授权凭证。对抗性环境必须在可信 CI 中再次用 Lean 验证，不能仅相信 JSON 内的 `status`。

## 下一阶段：跨模块 assume–guarantee

这是比单纯增加更多 Lean tactic 更重要的方向，但 **本 PR 第一阶段尚未实现**：

- 将函数可复用契约分成 `requires / ensures / effects`，全部有已解析 SymbolId 作为身份；
- 在调用方生成 “调用前提满足” 的证明义务，并以被调者的已验证后置条件作为假设，**明确记录假设链**；
- 若被调者实现改变、但同一契约被重新证明支持，允许复用不受影响的调用方证明；
- 若契约失效、闭包改变或假设不能满足，所有受影响证明回退 `stale`，不能保持 `proved`；
- 对外部 ABI 插件调用，必须给出明确 effect 契约、来源和可信主体，不能把不透明调用宣告为纯函数；
- 保留 Lean 与解释器对 i64 溢出、除零/模零行为的差异护栏，在证明模型不等价时不给出无限定的成功保证。

## 验收指标

1. 在任意步骤前插入不相关语句，公共锚点地址不漂移；旧内部编号可以变化。
2. 只修改锚点表达式，定义 digest 必须变化，锚点地址保持；同名重复拒绝公开。
3. `#@标签` 与完整 `mq://...` 必须精确解析；不存在锚点报告 unresolved，且不能变成 BoundCall。
4. `prove` 的函数证明只能关联函数知识，不能将所有带标记步骤标成 proved。
5. 更改源码、契约、传递依赖或证书后，知识索引拒绝使用过期证明。
6. 与 2.8.0 原有 CLI / SemanticProject / Lean 测试保持兼容。



## M4：模块化证明内核与当前信任边界

本工作在 `src/knowledge/modular.rs` 增加证明义务图，输出于
`proof-obligations.json`；可通过命令：

```sh
marqdo knowledge proof-deps --json --strict .
```

`--strict` 在缺少来源校验、证明收据或契约时非零退出。这个
`audit_passes` **不是**一个 Lean 证明的调用方定理，也不能发布
`proved`。它只说明当前本地可见的证明收据与契约引用是否完整。

T3 声明采用既有 GFM 表格（不增加解释器语法）：

| 假设 | 前置 | 后置 |
|---|---|---|
| rates.lookup | 金额 > 0 | 返回 = 2 |

函数开头的此表现在被 parser 提取进 `Function.assumptions`，映射到
`FnPlan.assumptions`，由 `SemanticProject` 生成 `assumptions.json`；
不参与运行时执行、也不赋予事实可信度。重复、未找到、前置条件待验证
均应 fail closed。

`doc/verification/skill-proof/MarqdoModularProof.lean` 定义参数化的
`Contract`、`Implements` 和 `compose_sound`：

- 被调函数真正满足其前后置条件；
- 调用方/前一个函数真正满足其前后置条件；
- 显式桥接引理证明前一个函数的后置条件足以满足后一个函数的前置条件；

Lean 才能得出组合契约成立。文件含一个缺少桥接前提时组合错误的反例，
以防未来实现将“被调者 proved”直接等价于“调用者 proved”。

### 尚未完成的关键项（不得标记 done）

1. **参数绑定**：从真实 Marqdo `BoundCall` 的表达式实参生成
   调用点前置条件的 Lean 目标。不能直接复用未绑定实参的 `前置` 文本。
2. **模块化执行语义**：在 Lean 中准确表示 import / 调用 / 返回 / 效应，
   并与原解释器的数值和异常行为建立等价性。
3. **证明组合与导出**：仅在被调证明证书经过受信核验、桥接义务
   由 Lean 消除且调用方自己的 Lean 定理成立时，输出新的模块化证明证书。
4. **可靠的跨实现复用**：不能因为后置条件文字未改变就绕过重新核验；
   必须要有新实现支持同一契约的证明，且证明/模块 ABI 版本匹配。
5. **安全与正确性测试**：成功与失败调用、条件前置、不透明调用、效应、
   多版本证书、循环依赖、证书篡改、解释器/Lean 差分一致性与崩溃恢复。

所以目前 **已经实现的是可寻址知识编译、证明绑定和保守的模块依赖审计**；
**没有实现完整跨模块自动推理并生成 `proved` 的系统**，不应声称后者已完成。


## M5：跨模块纯叶函数的受限源码透明证明

`src/prove/lower.rs` 新增第一条**真实跨文件执行语义**通路。
`Target::Import` 仅对以下情形将导入函数源码透明降级入调用方 Proof IR：

- 被调函数体只有一个 `Return`；表达式由字面量、形参、无副作用一元/二元表达式和遮蔽表达式构成，不得依赖全局状态或其他函数；
- 调用实参全部显式传入、位置匹配且与参数数目一致，不能通过默认参数或任意命名重排来隐式猜测绑定；
- 展开深度有严格上界；不支持分支、循环、递归、动态分派、ABI 效应或未知调用；
- 当导入表达式包含算术操作时，必须通过实际实参的静态整数求值确保 `i64` 不溢出、不除零。包含变量或非整数的未知算术保持不透明，绝不因 Lean 的无界 `Int` 语义而自动升级。

这条通路设计的真实 Lean 正例分别是无参数的跨文件纯返回，以及带实际参数绑定的加一计算；`tests/knowledge_proof.rs` 包含导入文件产生副作用和 `i64` 溢出的反例。

**该特性是源码透明的跨模块证明（有限展开），不是抽象契约复用。**
只有独立证明调用点前置条件、将已证被调契约接入调用方而无需展开实现，并由 Lean 校验接口桥接，才算真正完成通用 assume–guarantee 模块化证明。


## M6：真实 Lean 的封闭调用前置条件（2026-11-18）

模块化校验现在有两种不同的操作：

```sh
# 纯分析：不会冒充 Lean 已检查
marqdo knowledge proof-deps --json .

# 要求真实 Lean：先重放被调函数证明，再检查可闭合的调用点前提
marqdo knowledge proof-deps --verify --json .

# 审计严格退出码；不是“整个程序的模块化定理已被证明”
marqdo knowledge proof-deps --verify --strict --json .
```

工作过程：

1. `BoundCall` 保留精确的 caller/target 知识身份，以及**按解释器执行约定归一化后的实参**（名称、顺序、保守静态整数值、源表达式摘要）。
2. `knowledge/vc.rs` 用被调方函数签名，将受限前提（整数比较/加减乘/布尔连接）绑定到**真实实参**。拒绝除法、溢出、动态变量、不匹配的命名/顺序参数及无法解释的谓词。
3. 先校验原 `certificate.json` 的复合缓存键、`plan.json`、嵌入证明库哈希、生成 Lean 文件 SHA、源码和语义闭包。
4. `--verify` 额外使用现有 Lean 工具链**重新检查被调方生成的 Lean 定理**。仅当重放后的确具有精确匹配的 proved 定理时，才承认该条 callee 收据。
5. 从验证过的收据与闭合前提生成 `closed-goals.lean`，对每个目标使用 `theorem vc_i : ... := by decide`；调用 Lean 4 核验。Rust 计算的布尔结果只能用于提前拒绝，不能独立充当证据。
6. `proof-obligations.json` 与实时 JSON 审计报告分别记录 `precondition_pending` / `precondition_refuted` / `lean_kernel_verified` 等状态。无工具链、超时、证书篡改、Lean 不接受、源变更全部拒绝进入成功状态。

两个严格区分的边界：

- `lean_kernel_verified` 表示**当前闭合调用点的条件命题已被核验**，而非调用方的完整结论定理。
- `proof_completion` 仍为 `not_established_by_dependency_analysis`。目前缺少通用符号实参、分支路径条件、任意效应 ABI 的正式组合证明与可重放的跨模块结论证明证书。**不得将依赖图审计通过升级为完整模块化 proved。**

新增 `tests/prove_lean.rs` 场景验证被调定理与调用前置条件**双重重放**，且实参不满足前置条件时必然退出非零；新增单元测试覆盖实际工程导入、整数溢出和未绑定实参。此处描述的是提交的实现及其预期测试，最终验收仍以最新提交 CI 结果为准。


## M7：路径敏感符号证明和独立组合证书（限定子集）

### 路径条件

`SemanticProject.BoundCall` 保存调用身份、归一化实参、直接变量、
进入该调用点的分支条件（包括 else 否定条件）以及无法证明时的
`control_blocker`。副作用条件、循环、迭代和未知运算不作为已证前提。

`src/knowledge/symbolic.rs` 对纯整数比较和布尔条件产生带
`(v0 : Int)` 等量词绑定与路径假设的 Lean 定理；调用 Lean 核验后仅为
`symbolic_path_lean_model_proved`，不等于运行时契约通过。数学 Int 与
Marqdo 动态值、有界 i64 语义对应仍需独立证明。

### 独立 Lean 组合证明（首个真正闭环）

```sh
marqdo knowledge compose --json --strict .
marqdo knowledge replay .marqdo/modular/composed/{proof-id}/composition-certificate.json --json .
```

`compose` 仅接受：不同文件间的唯一静态调用；纯整数函数；
调用方直接返回或一次局部赋值再返回被调结果；被调方只有一条整数常量返回；
参数均有明确 int 契约、无默认参数；实参仅为调用方参数或安全整数常量；
双方具有相同的无条件结果定理；无分支、循环、递归、异常、ABI 效应。

分别生成 `CalleeModule.lean` 和 `CallerModule.lean`：

- 被调方源码生成定义，其契约由 Lean 证明。
- 调用方 `import CalleeModule`，只使用被调方证明的契约，不展开其函数体。
- 两模块独立生成 .olean 并通过真实 Lean 4 核验。
- `composition-certificate.json` 绑定符号、源码闭包、Lean 版本、Lean 源文件
  和编译模块哈希，证明作用域限定为
  `typed_int_pure_forwarder_constant_postcondition`。
- `replay` 必须从**当前项目**重新编译出相同的 Lean 源，匹配依赖闭包，
  然后独立重新执行两个 Lean 模块并校验产物哈希；源码/证书有变即失效。

此证书状态是 `lean_kernel_proved_model`，**不是** Marqdo 运行时全部语义已证。

### 异常、循环、递归与 ABI 的保守边界

`MarqdoModularProof.lean` 用 `Exit.ok / Exit.error / Exit.diverged`
显式区分成功、异常、不终止；`total_sequence_sound` 证明包含异常和
发散结果的总正确性契约只有在前置条件桥接被证明后才能组合。
异常结果和发散结果不能等同于正常返回。

这是经过 Lean 检查的**抽象组合规则**，并非已证明所有插件、循环与递归
实现满足这一模型。遇到 ABI 不透明效应、无法验证的异常处理、缺失循环
不变式/终止性或递归下降变元时，编译器保守拒绝组合。

### 未达到完整通用系统的部分

动态类型语义保持、复杂算术的安全条件、任意路径的 WP、
循环不变式与终止性、递归归纳证明、插件状态转换可信契约、
并发效应，以及一般性参数化后置条件的源码无关组合，目前仍不完整。
不能将已完成的限定子集扩大宣传为完整的通用模块化证明系统。

### 2026-11-18 类型安全修正

目前类型表源文本中的 `int` 被契约解析器归一化为 `number`，它也允许 Num 和 Formula。因此**不能**把 `number` 的证明自动解释为任意 Marqdo 运行时输入均为 i64。组合证书作用域改为 `number_contract_int_model_pure_forwarder_constant_result`，且根报告固定标注 `full_marqdo_runtime_semantics_proved: false`。这仅证明整数抽象模型上的独立模块组合。完整运行时证明必须新增可执行类型收窄及类型保持定理。

另新增 `MarqdoEffects.lean`：把状态、插件异常、未决结果和耗尽 fuel 的循环统一建模，证明异常与未决结果不能充当成功返回；该抽象 ABI 契约仍须具体插件的实现见证，不能由表格声明自动授予。
