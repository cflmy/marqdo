# 判定库 `MarqdoProof.lean` 的形式化说明

> 本文是 [design/prove-cli.md](../design/prove-cli.md) §4–§5 的配套理论摘要：
> 生成文件依赖哪些库事实、离线判定凭什么与 Lean 单调一致、边界在哪里。
> 库本体：[skill-proof/MarqdoProof.lean](skill-proof/MarqdoProof.lean)
> （由 `marqdo prove` 内嵌携带，`scripts/check-proof-library.sh` 校验一致性）。

## 1. 语义模型

生成文件把每个函数降级为纯 Lean 定义：

```lean
def «模块.函数» (env : Env) («轨迹» : List Op) («参数» : Val)… : Val × List Op
```

- `Val`：8 形状和（none/int/num/bool/text/list/map + formula/code 边界），
  与解释器 `value::Value` 一一对应（T4 差分一致性测试锁定）。
- `env : Env`：不透明操作的答案函数。**调用者的定理以 env 为全称量词**——
  证出即"对任意环境实现成立"，这正是 assume-guarantee 的保守形态。
- 不透明操作 = `Op.«调用_显示名»` 构造器 + 追加轨迹；操作名用链接显示文本，
  与定理表里 `包含(轨迹, "显示名")` 的写法一致。
- 库函数面：`mqAdd/mqSub/mqMul/mqDiv`（int 面，除零→0，已记录偏差）、
  `mqGt/mqGe/mqLt/mqLe/mqEq`（decide 编码）、`mqAccess`（映射字段读取，
  方程友好）、`mqResultOk/mqResultError/mqValueOr`（lib/result 结构）。

## 2. 定理形状与判定

每条定理表行生成：

```lean
theorem «模块.函数.定理名» (env : Env) («参数» : Val)… :
    (前提 → True) →
    ((«结果», «轨迹») = «模块.函数» env [] «参数»…) →
    mqTruth «结论的 Lean 编码» := by
  intro … h_run
  unfold «模块.函数» at h_run
  try split at h_run        -- 按最大 if 嵌套深度重复
  all_goals (simp_all [域引理]; try omega)
```

- `decided_by = "lean"`：Lean 内核接受 ⇒ `proved`；`sorry` ⇒ `unverified`；
  报错 ⇒ `unproved`（错误指回定理表行）。
- 策略 `none`：定理生成但主体 `sorry`，状态恒 `unverified`——绝不冒充。

## 3. 离线判定（F1–F5）与单调性论证

离线判定器对同一 IR 做符号执行，路径记录 (路径条件, 结果, 轨迹, 变量)。
它与 Lean 的对应关系是**逐构造镜像**：

| 离线闭包理由 | Lean 侧对应 | 单调性依据 |
|---|---|---|
| F1 路径枚举穷尽 | `split at h_run` 产生的同一组目标 | 同一 IR 树，分支集合相同 |
| F2 轨迹形状（包含/长度） | `(轨迹.map Op.name).contains` / `List.length` simp | 字面轨迹的列表运算两侧同构 |
| F3 构造器结构求值 | `mqResultOk/mqResultError/mqValueOr` + `mqAccess` simp 方程 | 同一套库定义；结构字面归约是计算的 |
| F4 自由原子 | split 后假设里的同一原子（`hypothesis` 关闭） | 只承认路径条件中字面出现的原子；路径反例含自由原子时放弃见证（保守） |
| F5 区间字面重述 | split 后假设直接就是结论（`hypothesis`） | 字面匹配，无推理步 |
| F5 区间违反（反例） | 无对应（Lean 同样证不出） | 反例只产生 `unproved`，不产生绿灯；见证 = 区间端点 |
| 不可行路径跳过 | split 后假设矛盾（互斥比较引理 `.elim` + omega） | 前提 ∩ 路径条件为空 ⇒ 该分支假设集 Lean 侧同样矛盾 |

**单调性护栏**：有 Lean 时逐定理复核。离线 `checked` 而 Lean 关不掉，或
离线反例被 Lean 证出 ⇒ `error[proof.divergence]`，永不落绿。护栏使
"离线比 Lean 乐观"成为运行时可见错误而非静默错误；40 夹具语料库
（`tests/prove_corpus.rs`）在 CI 强制 `offline_checked ⊆ lean_proved`。

**已知保守边界**（离线能、Lean 脚本不能，因此离线也必须不做）：

- 区间**蕴含**（`金额 > 10000 ⊢ 金额 > 9999`）：数学上真，但当前脚本没有
  Val 比较的传递引理，simp+omega 关不掉 ⇒ 离线同样 `unverified`
  （`f5_interval_closure_scope` 单测锁定）。库补传递引理后两侧同时放开。
- 多变量线性、非线性、文本相等：超出 F5 单变量区间 ⇒ `unverified`；
  Lean 侧若能证则升级 `proved`（允许，这是"弱判定被强判定确认"）。

## 4. 证书与缓存

证书记录：claim id、结论、策略、状态、`decided_by`（lean/trace/none）、
证据（Lean 报错或离线见证）、库与模块 sha256、Lean 版本、CLI 版本。
缓存键 = 上述全部的 sha256 ⇒ 任何一侧变化自动失效，命中即跳过判定。
证书是**证据陈述**（"这次判定是这样"），不是正确性保证——与设计 §0 契约 1 一致。
