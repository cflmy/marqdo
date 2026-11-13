# `marqdo prove` 实施路线（P0–P3）

| | |
|---|---|
| 状态 | **计划（Proposed）· 未开工** |
| 日期 | 2026-11-13 |
| 设计 | [design/prove-cli.md](../design/prove-cli.md)（命令面、离线片段、定理表、语言习惯审查） |
| 决策 | [ADR 0010](../adr/0010-skill-proof-boundary.md)（定理表角色、子集边界、顾问式定位） |
| 目标版本 | CLI 2.8.0 |
| 总工期 | 6.5 人周（单人；P0+P1 共 3 周可演示） |

---

## 0. 完成定义

| 阶段 | 完成定义（DoD） |
|---|---|
| P0 | `prove emit` / `prove audit` 可跑；确定性生成；12 个夹具的降级结论稳定；`cargo test --test prove` 在**没有 Lean** 的机器上全绿 |
| P1 | 定理表落地；有 Lean 时 `proved`，无 Lean 时至少 `unverified`；证书与源码映射可用；CI 跑通「假定理必须红」 |
| P2 | 离线判定 `checked` 可用；语料库强制 `offline_checked ⊆ lean_proved`；反例可读；`--require-lean` 严格模式 |
| P3 | 跨函数组合、`--all`、差分一致性；文档/教程/技能手册齐；2.8.0 发布 |

关键指标（P3 收口时复核）：

- 语料 **≥ 40** 个夹具，覆盖 8 类（§6）。
- 控制平面类定理的**离线覆盖率 ≥ 60%**（其余 `unverified`，不算失败）。
- 冷启动（50 函数文件）**< 30s**；证书命中 **< 50ms**。
- **单调性分歧 0 例**（语料 + CI 每次运行）。

---

## 1. 为什么先 Lean、后离线

离线判定器需要一把"标尺"：没有 Lean 作为判据，无法知道它会不会"过度自信"。所以顺序是：

1. P0 先把降级与生成做确定（无判定，不会撒谎）；
2. P1 接上 Lean，得到**权威判据**与失败证据格式；
3. P2 才写离线判定，并用 P1 的 Lean 结果做语料护栏（`offline ⊆ lean`）。

如果反过来，P2 的判定器会在没有标尺的情况下生长，最后很难证明它保守。

---

## 2. P0 · 骨架与确定性生成（1 周）

### 任务

| 天 | 任务 | 产物 |
|---|---|---|
| D1 | `Commands::Prove` + `ProveAction::{Emit,Audit,Clean}`；`--json` 外壳；`src/prove/mod.rs` 状态机骨架 | `marqdo prove --help` 正确；未知参数报 clap 错误 |
| D2 | `ir.rs` + `lower.rs`：参数、赋值、返回、调用→`Op`、`if/else`；`while`/递归/动态调用的拒绝码 | 12 个夹具降级到 IR，`plan.json` 可读 |
| D3 | `types.rs`：从契约表起步的推断与合一；`proof.type_mismatch` 等错误码 | 类型错误夹具给出两处行号 |
| D4 | `emit.rs`：确定性 Lean 生成（稳定排序、无时间戳、相对路径）、中文名、`-- mq:` 注释 | 两次运行字节一致 |
| D5 | `report.rs` + `audit`：内联/不透明/跳过三类清单；快照测试；`tests/prove.rs` | `cargo test --test prove` 无 Lean 全绿 |

### 验收

```bash
marqdo prove emit tests/prove/fixtures/approval.mq.md -o /tmp/lean
marqdo prove audit tests/prove/fixtures/approval.mq.md --json | jq '.functions[0].ops, .functions[0].rejected'
# 两次 emit 的 sha256 相同
```

### 明确不做

判定、证书、Lean 调用、定理表。P0 的 `prove` 只有 `emit`/`audit` 两个子命令可用，默认命令直接转发到 `emit` 并提示"判定将在 P1 提供"。

---

## 3. P1 · 定理表 + Lean 判定（2 周）

### 第 1 周：语言面与工具链

| 任务 | 产物 |
|---|---|
| ADR 0010 落定；`Function.claims`；定理表提取（首条可执行语句前，提取后移出 body） | 解析后 `claims` 非空，body 里没有该表 |
| `check --proof`（默认关）：表形、名字、片段、遮蔽 | `check` 报 `proof.claim_malformed` / `proof.name_shadow` |
| `verification/skill-proof/`：`lean-toolchain`、`lakefile`、`Basic/Result/Tactic`、`LIBRARY-SHA256` 脚本 | `lake build` 通过 |
| `driver.rs`：工具链探测、Lake 包、`lake env lean`、超时、stderr 解析 | 手工跑通 approval 样例 |
| `cache.rs`：证书、`index.json`、失效 | 二次运行毫秒返回 |

### 第 2 周：定理语义与失败证据

| 任务 | 产物 |
|---|---|
| 结论/前提求值：参数、`结果`、`轨迹`、三个谓词 | 定理编译通过 |
| 脚本生成：`intro` → `unfold run` → `split_ifs <;> simp_all <;> omega` | 正例 `proved` |
| 失败解析：`unsolved goals` → 目标 + 上下文 → 映射回 `.mq.md` 行 | 假定理给出源码行与残余目标 |
| `.github/workflows/proof.yml`：elan 钉版、`lake build`、跑 Lean 用例 | CI 绿；假定理用例必须红 |
| 文本/JSON/退出码对齐设计文档 | 状态表逐项手测 |

### 验收

```bash
marqdo prove tests/prove/fixtures/approval.mq.md --json | jq .status      # "proved"
marqdo prove tests/prove/fixtures/false-claim.mq.md; echo $?               # 1
PATH=/usr/bin:/bin target/debug/marqdo prove tests/prove/fixtures/approval.mq.md --json | jq .status
# P1 阶段没有离线判定，因此是 "unverified"；P2 起升级为 "checked"
```

---

## 4. P2 · 离线判定与单调性（2 周）

### 第 1 周：符号执行与 F1–F4

- `offline.rs`：符号状态（`Const/Param/OpResult/Unknown`）、路径条件、轨迹。
- 分支分叉与返回路径收集；`if` 条件求值到符号原子。
- F2 轨迹求值（`包含/长度/下标`）、F3 字面结果、F4 命题组合。
- 结论：`checked` / `unverified` 两态；证书写入情形列表。

### 第 2 周：F5 区间、反例、护栏

- F5：单变量 `x ⋈ c` 的区间传播与蕴含判定（保守；不可判定即 `unverified`）。
- 反例：路径可满足 + 结论字面假 → 端点见证值。
- 分歧护栏：`checked` 但 Lean 未关闭 → `error[proof.divergence]`（含测试专用注入开关，见 §6 A3）。
- 假设表（T3）：前置/后置 → 定理前提；`proof.assumption_untyped`。
- 单调性脚本：对语料逐条比对离线与 Lean 的定理状态。

### 验收

```bash
# 无 Lean 也能给出离线结论
PATH=/usr/bin:/bin target/debug/marqdo prove tests/prove/fixtures/approval.mq.md --json | jq .status  # "checked"
# 单调性：语料中不存在 checked 而 Lean unproved 的定理
cargo test --test prove_lean   # 需工具链
```

---

## 5. P3 · 组合与产品化（1.5 周）

| 任务 | 产物 |
|---|---|
| 跨函数内联 + assume-guarantee（调用者用被调者证书里的后置条件） | 多函数编排证明 |
| `foreach`（有限列表，fold 编码） | Q1 夹具 |
| `--all` 目录模式、聚合报告、`--max-files` | 仓库级报告 |
| 差分一致性（T4）：`Conformance.lean` + 解释器期望值 | CI 比对 |
| `view` 徽标、`catalog` `proof:` 字段、MLSP `proof` 方法（可选） | 读侧可见 |
| 文档：README、教程、两个 skill 手册、CHANGELOG、2.8.0 | 发布 |
| 发版手册：推荐附证书（不强制） | 流程更新 |

### 验收

```bash
marqdo prove --all skills/ --json | jq '.status, (.files|length)'
lake build   # verification/skill-proof/examples 全部通过
```

---

## 6. 测试矩阵（针对性）

夹具目录 `tests/prove/fixtures/`，集成测试 `tests/prove.rs`；Lean 相关用 `tests/prove_lean.rs`（独立测试目标，不进默认 `cargo test`）。

### A · 单调性与诚实性（最高优先级）

| ID | 夹具 / 操作 | 期望 | 守住什么 |
|---|---|---|---|
| A1 | `approval.mq.md`：`包含(轨迹,"人工审核")` | 离线 `checked`；Lean `proved` | 离线结论被 Lean 确认 |
| A2 | `false-claim.mq.md`：大额时断言 `结果.ok = False` | `unproved`，退出 1；P2 起附反例 `金额=10001` | 假定理不被放绿 |
| A3 | 同一夹具 + 测试注入 `--fault-inject offline-claim`（仅 debug 构建存在） | `error[proof.divergence]`，退出 1，JSON `status=="error"` | 分歧必须显式失败，不许静默以离线为准 |
| A4 | 多变量线性命题（`金额 + 手续费 > 10000`） | 离线 `unverified`；跑 Lean 后 `proved` 或 `unproved` | 片段保守，不越界断言 |
| A5 | 语料全量：`checked` 集合 ⊆ Lean `proved` 集合 | 集合包含成立，0 反例 | 长期单调性回归 |

### B · 语义一致性（T4）

| ID | 内容 | 期望 |
|---|---|---|
| B1 | 纯函数语料（算术、比较、布尔、字符串拼接、`access`、`len`） | 解释器输出 == Lean `example : f args = expected := by decide` |
| B2 | 除零、取模零 | 与运行时错误语义一致（生成 `mq_div`/`mq_mod` 返回 Result.failure） |
| B3 | 中文标识符 `计算器.加法`、含空格名 `多 步` | 生成 `«…»`，Lean 接受 |
| B4 | 映射/表字面量：字段顺序、`ok` 字段读取 | Lean 结构与 Marqdo 取值一致 |
| B5 | 负数、大整数（`i64` 边界） | 无溢出漂移；超界按契约拒绝 |

### C · 不执行（安全）

| ID | 夹具 | 期望 |
|---|---|---|
| C1 | 函数里调 `fs.write`、`fs.make_dirs` | `prove` 后目标路径不存在 |
| C2 | 函数里调 `print` | stdout **只有** `prove` 的报告；没有程序输出 |
| C3 | `--emit` 到只读目录 | 清晰 IO 错误，退出 1，不 panic |

### D · 子集与拒绝

| ID | 夹具 | 期望 |
|---|---|---|
| D1 | `while` 循环 | `proof.while`，行号准确；整体 `rejected`，退出 0 |
| D2 | 自递归 + 互递归 | `proof.recursion`，列出环上函数名 |
| D3 | 插件/动态调用 | `proof.dynamic_call` |
| D4 | 未定型变量（strict） | `proof.type_mismatch`，两处行号 |
| D5 | 同上 + `--no-strict-types` | 落 `Val`，报告标"弱类型"，不报错 |
| D6 | `--allow-opaque 预检` | 该函数列为不透明操作，调用者仍可证 |

### E · 定理表

| ID | 夹具 | 期望 |
|---|---|---|
| E1 | 未知名字 `审批人.ok` | `proof.claim_unknown_symbol` + 最近建议 |
| E2 | 重复 id | `proof.claim_duplicate` |
| E3 | 参数名 `结果` 遮蔽保留名 | `proof.name_shadow` |
| E4 | `策略 none` | 定理生成、不证；状态 `unverified`，不是失败 |
| E5 | 非法策略片段 `策略 nonsense` | `proof.claim_failed` 带 Lean 原始错误，不 panic |
| E6 | 缺 `结论` 列 / 表在语句之后 | `proof.claim_malformed` / `proof.claim_misplaced`（`check --proof`） |
| E7 | 两列「定理 / 说明」普通表 | 不认领，`check --proof` 不报 |
| E8 | `假设` 表：`返回.ok = True` 用于分支判断 | Lean `proved`；证书 `assumptions` 有该条 |

### F · CLI / JSON / 退出码

| ID | 操作 | 期望 |
|---|---|---|
| F1 | 各状态夹具 | 退出码 == 设计表（0/0/0/0/1/1） |
| F2 | `--report-only` on A2 | 退出 0，状态仍 `unproved` |
| F3 | `--require-lean` 且 `PATH` 无 lean | 退出 2，`status=="unverified"` |
| F4 | `--json` | 键集合与类型快照；`schema==1`；无绝对路径 |
| F5 | 目录不带 `--all` | `proof.need_all`，退出 1 |
| F6 | `prove clean` | `.marqdo/prove` 消失；`--emit` 目录保留 |

### G · 缓存与确定性

| ID | 操作 | 期望 |
|---|---|---|
| G1 | 连续两次 `prove emit` | `module.lean`、`plan.json` 字节相同 |
| G2 | 假 `lean` 脚本写日志；连续两次 `prove` | 日志只有 1 行（第二次命中缓存） |
| G3 | 改源码一个字符 | 证书键变化，重新判定 |
| G4 | 改库文件（模拟 `LIBRARY-SHA256` 变化） | 缓存失效 |
| G5 | 手工损坏 `certificate.json` | `proof.cache_corrupt`；重算或退出 1，不读坏证书 |

### H · 编码边角

| ID | 内容 | 期望 |
|---|---|---|
| H1 | 空 else、两臂都早返回 | 生成退化正确，Lean 通过 |
| H2 | 不可达分支（`1. False`） | `audit` 标 warning；不阻塞 |
| H3 | 字符串含引号/换行/中文 | 转义正确，Lean 字面量等值 |
| H4 | 0 参数、0 语句函数 | 生成 `def run := ...` 合法 |
| H5 | 参数默认值 | 与契约默认一致；Lean 用显式值代入 |

---

### 关键夹具草稿（P0 落盘到 `tests/prove/fixtures/`）

正例（轨迹命题，A1）：

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

反例（A2）：同构文件，只把定理换成

```markdown
| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 不该自动放行 | 结果.ok = False | 金额 > 10000 | auto |
```

期望 `unproved`、退出 1；P2 起附反例 `金额 = 10001`。

不执行（C1）：

```markdown
## 落盘
+ `path`

**written = [真的写一个文件](fs.write_text(path=`path`, text="证明工具不该执行我"))**

**written**
```

跑 `prove` 后指定路径必须不存在；`prove` 只做降级与生成。

拒绝（D1）：

```markdown
## 倒计时
+ `n`

- `n` > 0
  **n = n - 1**

**n**
```

期望 `proof.while`，整体 `rejected`、退出 0（若没有别的未关闭定理）。

遮蔽（E3）：

```markdown
## 遮蔽
+ `结果`

| 定理 | 结论 |
|---|---|
| 读结果 | 结果.ok = True |
```

参数名与保留名冲突 → `proof.name_shadow`，要求改名，不做隐式优先。

---

## 7. 验收证据（可直接粘进 PR）

```bash
# 1) 离线可用（把 PATH 收窄到没有 lean）
PATH=/usr/bin:/bin target/debug/marqdo prove tests/prove/fixtures/approval.mq.md --json | jq -r .status
# checked

# 2) 有 Lean 时升级为证明
target/debug/marqdo prove tests/prove/fixtures/approval.mq.md --json | jq -r .status
# proved

# 3) 假定理必须红，且给出行号
target/debug/marqdo prove tests/prove/fixtures/false-claim.mq.md; echo "exit=$?"
# error[proof.claim_failed] … --> false-claim.mq.md:31:3
# exit=1

# 4) 单调性（语料）
cargo test --test prove_lean -- --nocapture
# offline_checked=37 lean_proved=37 divergences=0

# 5) 不执行
rm -f /tmp/mq-prove-should-not-exist
target/debug/marqdo prove tests/prove/fixtures/side-effect.mq.md
test ! -e /tmp/mq-prove-should-not-exist && echo "no side effect"

# 6) 确定性
a=$(target/debug/marqdo prove emit tests/prove/fixtures/approval.mq.md -o /tmp/a && sha256sum /tmp/a/Generated/Approval.lean)
b=$(target/debug/marqdo prove emit tests/prove/fixtures/approval.mq.md -o /tmp/b && sha256sum /tmp/b/Generated/Approval.lean)
[ "${a%% *}" = "${b%% *}" ] && echo deterministic
```

---

## 8. CI 设计

```yaml
# .github/workflows/proof.yml（示意；action 版本以仓库既有 workflow 为准）
jobs:
  corpus:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install elan (toolchain pinned by verification/skill-proof/lean-toolchain)
        run: |
          curl -sSf https://elan.lean-lang.org/elan-init.sh | sh -s -- -y --default-toolchain none
          echo "$HOME/.elan/bin" >> "$GITHUB_PATH"
      - name: Build proof library (cache .lake)
        working-directory: verification/skill-proof
        run: lake build
      - run: cargo build --locked
      - run: cargo test --test prove --locked              # 不需要 Lean 的部分
      - run: cargo test --test prove_lean --locked
        env: { MARQDO_PROOF_REQUIRE_LEAN: "1" }
      - run: bash scripts/check-proof-library.sh           # LIBRARY-SHA256 与 BUILD-IDENTITY
```

原则：

- `cargo test` **不依赖** Lean；Lean 用例在独立目标里，缺失工具链时打印 `SKIP: lean toolchain not found` 并在 `MARQDO_PROOF_REQUIRE_LEAN=1` 下转为失败。
- 不使用 `#[ignore]` 作为成功证据（沿用 `executable-knowledge.yml` 的纪律）。
- Lean 构建缓存 `.lake`，避免每次拉依赖。

---

## 9. 风险与回滚

| 风险 | 触发 | 应对 |
|---|---|---|
| Lean 工具链在 CI 不可用 | action 变更/网络 | 钉版本 + 缓存；`proof.yml` 失败只红该 job，不阻塞其它 |
| 离线判定"过度自信" | 片段实现 bug | 语料 A5 强制包含关系；运行期 A3 分歧即 error |
| 生成不确定 | HashMap 遍历顺序 | 全量 `BTreeMap`；两次 emit 字节比对进 CI |
| 定理表被误用成数据表 | 表头重名 | 只有表头+列形同时命中才认领；`check --proof` 可关 |
| 破坏既有 `check` 性能 | 提取逻辑 | `--proof` 默认关；提取只做一次线性扫描 |
| 范围蔓延（想把整个语言编译过去） | 需求诱惑 | ADR 0010 写死子集；新增构造需重新评审 |

回滚：`prove` 是**纯增量**（新命令 + 新可选表角色）；删除命令与 `Function.claims` 字段即可回到现状，已有 `.mq.md` 不受影响（定理表在无 `prove` 的旧版本里只是普通表格）。

---

## 10. 不做清单（本版本）

- `while`、递归、动态派发的形式化（拒绝而非近似）。
- 字符串/JSON/浮点/公式库的语义建模。
- 自动修复不成立的定理。
- 证明缓存的服务端共享。
- IDE 内联证明状态（先走 `view` 徽标）。
- 强制门禁：任何"必须 proved 才能发布"的默认行为。
