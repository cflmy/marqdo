# v2.8.0 发布说明 · Release Notes（prove）

> 仓库权限说明：本文件由 2.8.0 收口时的自动化会话写入 `doc/roadmap/`
> （`CHANGELOG.md`、`README.md`、`doc/releases/`、`skills/marqdo-dev/` 为
> root 属主、运行账号不可写）。内容已定稿，合入即可；§4 为待合入原文。

## 1. 一句话

`marqdo prove` 正式发布：知识文件的定理表 → Lean 4 证明 → 自动判定。
没有 Lean 工具链也能跑（离线符号执行，诚实降级）；有 Lean 时全链路证明，
假定理给出路径见证反例，离线与 Lean 分歧时显式报错永不落绿。

`marqdo prove` ships: theorem tables in knowledge files compile to Lean 4
and are judged automatically. It runs without a Lean toolchain (offline
symbolic execution, honest degradation); with Lean, fake theorems get
counterexamples and any offline/Lean divergence fails loudly.

## 2. 亮点（Highlights）

| | |
|---|---|
| 定理表 | `| 定理 | 结论 | 前提 | 策略 |`，结论文法：`结果.ok`、`包含(轨迹,…)`、`长度(轨迹)`、参数比较，且/或/非组合 |
| 六段管线 | 装载 → 降级 → 定型 → 生成（Lean 4.35）→ 判定（离线先行 + Lean 复核）→ 证书 |
| 离线判定 | F1–F5 片段符号执行：路径枚举、轨迹形状、构造器结构、自由原子二分、区间字面重述与反例见证（毫秒级） |
| 单调性护栏 | 离线 `checked` 但 Lean 证不出（或反例被证出）→ `error[proof.divergence]` 退 1——绝不让离线判定比 Lean 更乐观 |
| 诚实状态 | `proved > checked > unverified > rejected > unproved > error`；`checked ≠ proved`；无 Lean 时状态如实降级（咨询性退 0），`--require-lean` CI 严格模式退 2 |
| 反例 | 假定理给出区间端点见证（如 `金额 = 10001`）并指回定理表行号 |
| 证书缓存 | 复合键（源码⊕计划⊕库⊕Lean 版本⊕CLI 版本），`.marqdo/prove/`，命中跳过判定；`--no-cache` 绕过 |
| 目录模式 | `prove --all 目录`（`--max-files` 默认 200）：逐文件出证，整体取最差，聚合 JSON |
| 差分一致性 | T4 语料：解释器求值 == Lean 语义（算术/中文标识符/分支/拼接/i64 边界/结构字段） |
| 工具链 | `--lean` 显式指定 → `MARQDO_PROOF_LEAN` → PATH；内嵌判定库按内容哈希缓存构建，无需 lake/elan |

## 3. 使用（Usage）

```bash
marqdo prove 审批.mq.md --json        # 判定 + 单个 JSON 报告
marqdo prove emit 审批.mq.md          # 只生成 Lean 模块 + plan.json
marqdo prove audit 审批.mq.md         # 只审查可证子集（不写盘）
marqdo prove --all skills/ --json     # 目录聚合（整体取最差）
marqdo prove clean                    # 清空 .marqdo/prove
```

测试：`cargo test --test prove`（无 Lean 全绿）·
`MARQDO_PROOF_LEAN=<lean> cargo test --test prove_lean --test prove_corpus --test conformance`（有 Lean 全链路 + 40 夹具单调性 + T4 一致性）。

## 4. 待合入原文（需 root 属主文件权限）

### 4.1 `CHANGELOG.md` 追加

```markdown
## v2.8.0 — 2026-11-17

`marqdo prove` 正式发布：定理表 → Lean 4 证明 → 自动判定。离线符号执行
（F1–F5）无工具链可用，`checked` 永远弱于 `proved`；单调性护栏让判定器
缺陷显式失败；假定理给出反例见证；证书缓存与目录模式面向智能体批量出证。

`marqdo prove` ships: theorem tables compile to Lean 4 and are judged
automatically. Offline symbolic execution works without a toolchain,
monotonicity guardrails fail loudly on judger defects, fake theorems get
counterexample witnesses, and certificate caching plus directory mode
support agent-scale proving.

详见 [双语发布说明](../releases/v2.8.0.md)。
```

### 4.2 `README.md` 修改

版本行（第 19–21 行）替换为：

```markdown
当前 CLI **2.8.0** 新增 `marqdo prove`：知识文件的定理表编译为 Lean 4 并自动
判定真假——没有 Lean 工具链也能离线符号执行（诚实降级，绝不冒充证明），
有 Lean 时全链路证明并给出反例见证。参见[发布说明](../releases/v2.8.0.md)。

CLI **2.8.0** adds `marqdo prove`: theorem tables in knowledge files compile
to Lean 4 and are judged automatically — offline symbolic execution without
a toolchain (honest degradation, never impersonating a proof), full proofs
with counterexample witnesses when Lean is available.
```

「愿景与期许」清单追加第 7 条：

```markdown
7. **知识可证明**：知识文件内的定理表 → Lean 4 定理 → 自动判定
   （`marqdo prove`）；离线符号执行与 Lean 单调一致，分歧显式报错。
```

### 4.3 `skills/marqdo-dev/index.mq.md` 追加 `prove` 任务

```markdown
## prove

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | "." | 单个 `.mq.md` 或目录（目录需要 all=True） |
| all | bool | False | 目录模式：递归收集 .mq.md 逐文件出证，整体取最差状态 |
| max_files | num | 200 | 目录收集上限 |
| require_lean | bool | False | CI 严格模式：未判定状态退 2 |
| json | bool | False | 单个 JSON 对象输出（智能体可读契约） |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：真实 `marqdo prove` 退出状态、标准输出与错误输出 |

证明知识文件的定理表：先离线符号执行（毫秒级 checked，不是 proved），
有 Lean 工具链时由 Lean 复核升级 proved；假定理给出路径见证反例；
离线与 Lean 分歧触发 proof.divergence 错误并退 1，永不落绿。
不带 Lean 时状态如实停留在 unverified/checked（咨询性退 0），
require_lean=True 才把未判定当作阻塞。

通过只说明所选文件判定完成，不能替代 test 任务的运行时验收；
两者结合才是完整验收。

1. len(text.trim(`path`)) == 0
  Return *[要求明确的证明范围](result.error(code="dev.prove_path", message="path is required", operation="prove"))*.
1. `all`
  We **argv = ["prove", "--all", `path`, "--max-files", text.num_text(`max_files`)]**.
1. not `all`
  We **argv = ["prove", `path`]**.
1. `require_lean`
  We **argv = [追加严格模式](table.append(list=`argv`, item="--require-lean"))**.
1. `json`
  We **argv = [追加机器可读输出](table.append(list=`argv`, item="--json"))**.
We **checked = [执行真实证明管线](engineering.run(argv=`argv`, timeout_ms=600000, max_output=4194304))**.
验收结束后返回*[按真实退出状态判断结果并保留输出](engineering.check_exit(outcome=`checked`))*。
```

## 5. 已知限制（Known limitations）

- `foreach` 维持 `proof.foreach_pending`（可证子集无循环；fold 编码 → 2.9）。
- 跨模块调用 = 不透明操作；assume-guarantee 后置条件复用 → 2.9（依赖假设表 T3）。
- 除零/模零：解释器为运行时错误，P1 判定库模型返回 `.int 0`（已记录偏差）。
- 分支条件暂不支持字段访问（解析器文法限制；prove 侧已就绪，放开即生效）。
