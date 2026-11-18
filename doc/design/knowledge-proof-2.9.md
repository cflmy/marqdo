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
