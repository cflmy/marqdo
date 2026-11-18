# 可执行知识语法：仓库级测试与兼容性规程

本文是 `Marqdo_Executable_Knowledge_Development.md` 的验证规程。它约束稳定符号 URI、零参数知识路径、关联函数/实例方法区分及其 CLI 解析进入标准库、扩展库和发布流程后的测试方式。

## 覆盖目标

新写法只在需要值的位置把零参数路径解释为调用：

```marqdo
**pi = math.pi**
**year = facts.Person.birth_year**
```

旧的 `**pi = > math.pi**` 和 `> math.pi` 继续兼容。对象子函数不读取 `self` / `自` 时是 `associated_function`；读取接收者时是 `instance_method`，不能由 `Type.method` 直接执行。

每项语法变更必须同时证明：新写法正确、旧写法未回归、Tree/Bytecode 一致、CLI 静态解析和运行时规则一致，并且标准库与扩展库仍能加载、运行。

## 自动化测试矩阵

| 层级 | 覆盖内容 | 命令 | 通过条件 |
|---|---|---|---|
| 解析 | 零参路径降级 Call；带参路径不能静默执行 | `cargo test --locked --lib parse::expr` | AST、参数数与诊断稳定 |
| 语义 | ProjectRoot、ModuleId、SymbolId、关联/实例分类、歧义 | `cargo test --locked --lib sema::` | 扫描顺序不影响结果；分类不改变 SymbolId |
| Tree | 标准库裸路径、关联函数、旧写法 | `cargo test --locked --test gold structure_new_path_stdlib structure_ns_associated_function` | stdout 金样一致 |
| Bytecode | 同一正例的后端一致性 | `cargo test --locked --test gold bytecode_new_path_stdlib bytecode_ns_associated_function` | stdout、退出码一致 |
| 负例 | 接收者依赖路径被拒绝 | `cargo test --locked --test gold error_ns_instance_method knowledge_resolve_rejects_instance_path` | 不构造对象、不执行方法 |
| CLI | `inspect` / `resolve` JSON | `cargo test --locked --test gold knowledge_resolve_new_path` | 返回 URI、source、callable；resolve 不执行代码 |
| 标准库 | 英文/中文零参知识 | `cargo test --locked --test gold lib_log_uuid_smoke lib_log_uuid_zh_smoke` | `uuid.v4` 等裸路径可用 |
| 扩展库 | 改写后的扩展调用和插件加载 | `cargo test --locked --test gold ext_quantum_gates_smoke` | `math.pi` 新写法输出不变 |

## 每次语法改动的执行顺序

```bash
source scripts/dev-env.sh
cargo check --locked
cargo test --locked --lib
cargo test --locked --test gold -- --test-threads=1
cargo test --locked
cargo build --locked
```

发布前再做可观察的 CLI/双后端冒烟：

```bash
target/debug/marqdo knowledge tests/structure/ns --json resolve \
  helpers.Box.birth_year --from tests/structure/ns/associated.mq.md
target/debug/marqdo run tests/structure/new-path-stdlib.mq.md --backend tree
target/debug/marqdo run tests/structure/new-path-stdlib.mq.md --backend bytecode
```

完整 gold 会构建部分原生/Go 扩展，必须单线程执行。网络、真实 LLM 与 ignored 实验不属于离线语法门禁，不能以其失败掩盖核心回归。

## 标准库与扩展库改写准则

仅改写可机械证明等价的值表达式：

| 原写法 | 新写法 | 前提 |
|---|---|---|
| `**x = > math.pi**` | `**x = math.pi**` | 目标零参数、值位置 |
| `**id = > uuid.v4**` | `**id = uuid.v4**` | 目标零参数、值位置 |
| `**x = > Type.fact**` | `**x = Type.fact**` | 目标为关联函数 |

禁止自动改写副作用语句、带必填参数函数、实例方法、文本/围栏/URL/导航链接，以及不能同时通过 Tree/Bytecode 的变更。本轮用 `math.pi`（量子扩展夹具）、`数学.圆周率`（中文标准库夹具）和 `uuid.v4`（标准库夹具）作为真实金样。

## CLI 合约

```text
marqdo knowledge PATH --json inspect SYMBOL
marqdo knowledge PATH --json resolve REFERENCE --from CALLER.mq.md
```

`resolve` 只绑定、不执行。它依据 `--from` 的 frontmatter import 解析第一个路径段，不会在项目中按名字猜测目标。关联函数返回 `resolution: "symbol"`；实例方法返回 `resolution: "instance_only"` 并以退出码 1 拒绝直接路径调用。

## 发布门槛

1. `git diff --check` 无错误；
2. Rust 核心、gold、完整 `cargo test --locked` 通过；
3. 每个新正例有 Tree、Bytecode 和 CLI（适用时）覆盖；
4. 每个语义拒绝路径有稳定负例；
5. 标准库/扩展库改写保留原 stdout 金样；
6. 不提交 `.marqdo/`、插件构建物或本地凭据。

失败报告必须记录命令、后端、夹具、stdout/stderr、插件/网络依赖及与旧写法的差异。
