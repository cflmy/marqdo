---
markup: occlusion
title: 以 Marqdo 本身开发和验证 Marqdo
import:
  files: ./files.mq.md
  browser_tools: ./browser.mq.md
  fs: lib/fs.mq.md
  repository: ./repository.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
  sys: lib/sys.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
---

本目录是开发智能体的可执行手册。先阅读 [编程规范](../../doc/library-development.mq.md)，再按具体任务调用本目录的函数。单文件标准库本身就是手册；只有多文件库或工程才需要 index.mq.md 组织入口。叙述应解释任务、数据、失败和限制，不能靠重复句子凑比例。

日常读写通过 files.mq.md 的 read、excerpt、write、patch、search、inventory 和 apply 完成。批量写入前可携带 expected_sha256，防止覆盖已经变化的源码。大段文本使用 CLI 的 --args-json 文件或标准输入传送；它保留换行、引号、中文和以美元符号开头的字面数据。不要为了搬运源码再生成 Python 或 shell 程序。

原生库能完成的计算和文件操作直接使用 lib。缺少可复用能力时新增拓展库；需要现有操作系统工具时使用 engineering.run 的参数数组。编排、条件、重试和结果判断留在 mq.md 中。process.exec 与 plugin 是真实的外部能力授权，fs-root 不能限制外部进程自己的权限。

调用示例：marqdo call skills/marqdo-dev/index.mq.md prepare_engineering --effects process.exec --fs-root . --json。读取文件可调用 files.mq.md read 并传入 path；修订现有文件优先使用 patch 或带哈希的 apply。每个真实函数标题下都应先给出参数和返回值表，然后是能够独立指导调用者的叙述。

## prepare_engineering

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：构建工程插件，失败时保留 Cargo 输出 |

第一次使用工程拓展前执行此任务。这里通过已有 sys.exec 引导 Cargo 构建本仓库的原生插件，之后开发和发布流程都可以使用同一份工程拓展。构建需要本机 Rust 工具链和已经可用的依赖缓存；不会安装 Python，也不修改版本或发布远端资源。

调用需要 process.exec。子进程退出不为零时返回失败，调用者应查看真实 stderr 并修复编译问题；成功只表示本机插件已构建，不意味着其他平台包已验证。依赖尚未缓存时应按仓库工具链配置获取依赖，再运行同一任务。

`args` =

| argument |
|---|
| "build" |
| "-p" |
| "marqdo" |
| "-p" |
| "marqdo_plugin_engineering" |
| "--offline" |

We **built = [构建已维护的工程边界插件](sys.exec(cmd="cargo", args=`args`, capture=True))**。
1. access(`built`, "code") != 0
  返回*[保留构建错误](result.error(code="dev.plugin_build", message=str(`built`), operation="prepare_engineering"))*。
构建完成后返回*[真实 Cargo 证据](result.ok(value=`built`))*。

## check

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | 需要完整检查依赖的真实文件路径 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：编译检查和逐函数叙述比例检查的进程证据 |

使用 `path` 指定本次修改的入口或单文件库。从仓库根目录调用，并授权 plugin。检查器加载真实依赖，验证契约、参数与调用，同时要求函数的叙述比例超过一半。故意损坏的测试夹具和明确标记禁止修改的人工测试不应当被这项作者任务批量改写。

检查通过不能证明页面视觉效果、网络服务或发布凭据正确。根据变更运行实际用例：纯函数比较返回值，文件操作使用临时目录，外部发布操作使用隔离服务夹具。失败时先修复实际问题，保留诊断位置，不通过删除业务能力或补空话来消除告警。

`argv` =

| argument |
|---|
| "target/debug/marqdo" |
| "check" |
| `path` |
| "--narrative-ratio" |

We **checked = [检查真实文件及其可读性](engineering.run(argv=`argv`))**。
检查结束后返回*[按实际退出状态保留通过或失败证据](engineering.check_exit(outcome=`checked`))*。

## audit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| write_report | bool | False | 保存同一份真实报告到忽略目录 .marqdo/audit |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：实际发现的维护源码与逐文件问题清单 |

使用 `write_report`=False，在仓库根目录授权 plugin 和 fs.read，即可发现维护中的源码并逐个审查。文件枚举、读取和报告写入都由固有能力完成；分析使用真实编译器，不会运行被审查程序或其导入。需要保留证据时传 True 并增加 fs.write。

测试夹具、人工保护样例和历史兼容文件保留既有语义。占位演示的 warning 留在报告中供阅读，不伪造实现以消除提示。失败 Result 仍携带完整清单，因此可结合 --require-ok 阻止后续自动化；比例与契约通过后还应执行实际用例，不能把结构审查当成运行成功。

审查完成后返回*[实际仓库源码的完整观察结果](repository.audit(write_report=`write_report`))*。


## test

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| targets | list | required | 明确选择的 Rust 集成测试目标名称 |
| filter | any | None | 可选测试名称过滤文本；不是 Cargo 命令选项 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：真实 Cargo 退出状态、标准输出与错误输出 |

使用 `targets` 和可选的 `filter`=None，在仓库根目录授权 plugin。目标清单转成逐个 --test 参数，Cargo 在离线模式下执行；这里不会安装依赖、调用 Python 或生成 shell 程序。需要先完成工具链与插件引导，测试使用的本地服务或其他依赖仍按各目标的实际要求准备。

传入的是测试目标名称和可选名称过滤器，不能用空目标清单或把命令选项伪装成过滤文本。构建错误、断言失败、超时与启动失败都返回失败结果；失败仍携带真实执行证据，便于定位哪个用例没有通过。成功只说明所选测试通过，不覆盖未运行的目标，更不能替代发布服务的实际状态验证。名称过滤只缩小用例集合；检查输出中的实际运行数量，避免过滤器没有命中却被当作完成验收。

这个入口把重复的测试命令收进源码手册。修改相关模块后运行对应目标，已通过后只有新改动、失败或未解决的疑问才扩大或重复检查。外部进程保留操作系统权限，--fs-root 不是 Cargo 子进程的沙箱。

1. len(`targets`) == 0
  Return *[要求明确的验收范围](result.error(code="dev.test_targets", message="select at least one integration test target", operation="test"))*.
1. `filter` != None
  1. [过滤器不能冒充 Cargo 选项](text.starts_with(text=`filter`, prefix="-"))
    Return *[拒绝选项形式的过滤文本](result.error(code="dev.test_filter", message="filter is a test name, not an option", operation="test"))*.

`argv` =

| argument |
|---|
| "cargo" |
| "test" |
| "--offline" |

- [将每个目标作为独立参数交给测试工具](for targets as target)
  1. [测试目标不能冒充命令选项](text.starts_with(text=`target`, prefix="-"))
    Return *[保留错误目标的诊断](result.error(code="dev.test_targets", message=`target`, operation="test"))*.
  We **argv = [增加明确的集成测试选择标记](table.append(list=`argv`, item="--test"))** and **argv = [增加原样传入的目标名称](table.append(list=`argv`, item=`target`))**.
1. `filter` != None
  We **argv = [追加可选的用例名称过滤器](table.append(list=`argv`, item=`filter`))**.
We **checked = [执行实际离线构建与验收](engineering.run(argv=`argv`, timeout_ms=300000, max_output=2097152))**.
验收结束后返回*[按真实退出状态判断结果并保留输出](engineering.check_exit(outcome=`checked`))*。

## build_browser

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| out | text | ".marqdo/browser-handbook-assets" | Workspace output directory for current WASM and canonical bridge |
| compiler | any | None | Optional absolute rustc path from toolchain.rust_wasm_linux |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with native artifact hashes and the actual bounded build report path |

Use `out`=".marqdo/browser-handbook-assets" and `compiler`=None from the checkout root. This task calls Marqdo's own wasm build operation through the explicit engineering process boundary, then hashes its real outputs with native filesystem tools. If the system compiler lacks a compatible WASM target, first call [the workspace toolchain task](toolchain.mq.md) and pass its returned compiler path. That override applies only to this child process, with no global environment or system installation change.

Grant plugin, process.exec and fs.write. Native directory validation runs before the external build and keeps selected output under the granted root. Compiler logs are bounded and saved natively to .marqdo/build/browser.json; the CLI returns compact evidence so an oversized log does not hide the error. A failed Result remains failed for --require-ok and points to that report. Success includes the hashes of both generated files, not an assumption based on Cargo's exit code. Browser interaction and real HTTP mounting still need their own tests after this build.

We **[Validate and create the selected native output directory](fs.make_dirs(path=`out`))** and **[Create the native build-report directory](fs.make_dirs(path=".marqdo/build"))** and **environment = None**.
1. `compiler` != None
  We **environment = [Select this compiler for one child process](table.put(in=None, at="RUSTC", value=`compiler`))**.

`argv` =

| argument |
|---|
| "target/debug/marqdo" |
| wasm |
| build |
| "-o" |
| `out` |

We **execution = [Build the current official browser artifact](engineering.run(argv=`argv`, env=`environment`, timeout_ms=300000, max_output=65536))** and **checked = [Retain actual build success or failure](engineering.check_exit(outcome=`execution`))**.
We **[Save bounded compiler evidence through native IO](fs.write_json(path=".marqdo/build/browser.json", value=`checked`))**.
1. not access(`checked`, "ok")
  We **failed = [Keep this build incomplete](result.error(code="dev.browser_build", message="inspect .marqdo/build/browser.json", operation="build_browser"))**.
  Return *failed*.
We **wasm = out + "/marqdo_wasm.wasm"** and **bridge = out + "/marqdo-bridge.js"** and **wasm_sha256 = [Hash the actual new WASM](fs.file_hash(path=`wasm`))** and **bridge_sha256 = [Hash the actual canonical bridge](fs.file_hash(path=`bridge`))**.

`receipt` =

| out | wasm_sha256 | bridge_sha256 | report |
|---|---|---|---|
| `out` | `wasm_sha256` | `bridge_sha256` | .marqdo/build/browser.json |

Return *[Expose real browser build evidence](result.ok(value=`receipt`))*.


## prepare_browser

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | "1.65.0" | Pinned Playwright driver version |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result of workspace-local browser driver preparation |

Use `version`="1.65.0" when preparing actual browser tests. The browser handbook keeps dependency installation and browser caches inside the workspace, using native directory operations and explicit existing npm/Playwright tools. Grant plugin, process.exec and fs.write. This prepares a driver only; call build_browser for current runtime artifacts and test_browser for actual behavior. No Python runner or global npm package is required.

Return *[Prepare the pinned external browser capability](browser_tools.prepare(version=`version`))*.


## test_browser

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| out | text | ".marqdo/browser-handbook-assets" | Current native browser build output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result from actual browser-effects, SPA, collaboration and bilingual network/complete-site browser tests |

Use `out`=".marqdo/browser-handbook-assets" after prepare_browser and build_browser. The browser handbook stages the named runtime files with native IO, then supervises real callable servers and drives current WASM through the official bridge. The test result includes actual failures and is saved under .marqdo/browser-tests. Grant plugin, process.exec and fs.write; use --require-ok to stop a later task when a browser case fails. Constructed effects and source audit results remain separate evidence.

Return *[Verify the actual callable sites in a real browser](browser_tools.verify(out=`out`))*.


## prepare_web

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：当前 Go Web 插件构建的真实退出状态和日志 |

先完成 prepare_engineering，再从仓库根调用并授权 plugin。既有构建脚本负责调用 Go/cgo 编译本仓库的原生 Web 实现，并把编译产物放到插件解析器使用的构建目录。需要本机 Go 与 C 编译工具链；这里不安装工具、不更改全局环境，也不选择新的发布版本。Windows 构建仍须具备脚本要求的 cgo 工具链，Linux 的通过不能替代 Windows 验证。

这个入口把常用插件准备动作保存在可执行手册中。工程边界为子进程传递本仓库的 Git 信任配置，避免 Go 的 VCS 检查因不同文件所有者失败；该配置不写入全局 Git 设置，也不关闭 VCS 来源记录。只有编译和编译产物处理外联维护中的构建工具；日常源码读写仍使用原生文件任务。缺少工具、编译错误或非零退出会保留实际日志并返回失败，不能继续把旧插件当成本轮实现。成功只证明当前平台的插件已构建，随后还需运行对应 HTTP 与浏览器用例。

`argv` =

| argument |
|---|
| bash |
| scripts/build-web-plugin.sh |

构建结束后返回 *[在保留来源记录的受控进程边界编译当前插件](engineering.checked(argv=`argv`, timeout_ms=300000, max_output=65536))*。
