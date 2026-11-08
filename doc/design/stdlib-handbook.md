# 标准库：代码、文档、SKILL 与 tools 同源

标准库通常是单文件：直接阅读并调用 [fs.mq.md](../../lib/fs.mq.md)、[文件.mq.md](../../lib/文件.mq.md)、[table.mq.md](../../lib/table.mq.md) 或其他任务所需源码。
31 个模块族对应 61 份公开标准库单文件；源码本身承担手册与 SKILL，不设强制总入口或模块 index。
多文件拓展库的 index 用于组织多文件，标准库沿用其可读性和调用规范。

## 源码就是手册

每个函数标题下立即放参数表（名称、类型、默认、含义），随后是返回契约、效应、叙述形参、
用途和边界、实际调用例子及短实现。无参数函数直接放返回表。旧 `+` 形参已移除。
契约表位于首条执行语句之前；默认值和参数顺序保持兼容。

叙述比例逐函数超过 50%；既检查比例，也检查真实契约、调用和错误语义。
不透明宿主指令使用遮蔽调用，例如 `` *[读取 UTF-8 文本](> host_read_text path=`path`)* ``：
方括号说明目的，圆括号执行真实代码；赋值语句同样采用描述标签。
标签不替代参数与权限，也不能用空标签或重复模板代替说明。调用与返回自然写入正文；
行内表达式仍计作代码，绑定数据表不冒充叙述。125 份库源码的统一验收见
[本轮记录](../roadmap/library-authoring-refactor.md)，公共标准库当前为 650 个函数。
键值行记录可用 table.from_pairs / 表.键值转记录生成具名记录，保留空值和重复键更新语义。

## 接口事实来自编译器

现有 call、catalog、MLSP 与编译器读取同一份标准库源码，不额外包装一层目录函数或示例函数。
`examples/stdlib_catalog.rs` 使用实际装载、签名和效应结果检查源码中的效应说明；
--json 输出接口事实，--write 仅刷新效应说明，不生成第二套手册。
人工维护用途、参数含义、失败处理和工程实践。不透明值保留渐进式 any，并说明来源和限制。

```sh
cargo run --example stdlib_catalog -- --json
cargo run --example stdlib_catalog -- --check
marqdo check lib/文件.mq.md --narrative-ratio
cargo test --test stdlib_tasks
```

测试直接读取 61 份公开源码中的实际命令，在 Tree 与 Bytecode 执行 126 次。
验证每个函数的参数/返回契约、标题下表格、无旧形参声明以及逐函数叙述比例。

## 能力与兼容性

| 任务 | 新的公共入口 | 关键契约 |
|---|---|---|
| 文件整理 | fs.find/read_lines/write_lines/read_json/write_json/read_csv/write_csv | 明确根目录、格式和资源上限 |
| 文件修改 | atomic_write/write_changed/write_checked | 单文件原子替换；摘要检查不是内核 CAS，竞争写入仍需协调 |
| 批量文件 | copy_tree/sync_tree/rename_many | 先预检冲突，覆盖需选择；不承诺多文件事务；目录复制面向普通文件，不复制权限、链接与空目录 |
| 临时文件 | make_temp/temp_dir | 在授权根目录创建，由调用方明确清理；真实工程演示清理 |
| 二进制 | bytes 与 fs.read_bytes/write_bytes/read_chunk/append_bytes/file_hash | 普通 bytes 记录，Base64 跨 CLI/JSON/插件传递，字节偏移从零起 |
| 集合 | range/enumerate/pluck/project/where/find/any/all | 不增加回调字符串求值或新的查询方言 |
| 表数据 | group_by/aggregate/summarize/join/rename/unique_by/sort_keys | 行记录、分组键和字段含义明确；批量汇总避免作者循环反复复制列表 |
| 数据形态 | from_columns/to_columns/cast/try_cast/validate | 不猜测列式表与普通对象；列长、字段及类型错误有明确结果 |
| 集合运算 | union/intersection/difference | 返回保序列表，沿用运行时值相等规则 |
| 网络 | net.get/request/download/header_values | 超时、响应上限、重复响应头、字节响应；失败下载保留原文件 |
| 配置与日期 | toml.parse/stringify、datetime.parse_local/add_calendar | 标准 TOML；内嵌 IANA 规则；夏令时缺口和重叠明确失败或选择 |
| 子任务 | wait_timeout/cancel_all/run_many | 文件任务并发上限、超时终止回收、父权限与根目录继承 |
| 文档维护 | document.sections/section/tables/links/code_blocks/replace_section/replace_table | 与渲染共用 Markdown 解析器，不执行文档，保留周围源码 |
| 归档 | archive.pack/unpack | 可选 ZIP，预检路径、链接、重复项和展开上限，目标必须不存在 |

旧 remove 保持递归删除行为，公开文档已纠正；新代码用 remove_file/remove_empty_dir。
access 与 put 的整数路径从一开始；旧 at/slice/insert/remove_at 等接口从零开始，不能混用。
更新返回新集合，原值保留；单次追加仍有复制成本。规模任务使用批量变换，去重使用分桶并保留原相等语义。
混合排序采用明确类型顺序，修复数字与文本混排造成非传递比较的问题。
大下标在 32/64 位目标上保持一致：取元越界为空，更新越界为诊断，巨大分块大小不会截断成零或导致死循环。

## 权限与错误

宿主身份声明效应，编译器沿解析后的依赖传播，不依据 fs、net 等导入别名猜测。
函数、BoundCall 和步骤共享最终效应，外部标准库和中英文别名也使用同一逻辑。

```sh
marqdo call lib/fs.mq.md read_text 'path="notes.md"' --effects fs.read --fs-root /absolute/project --json
marqdo call lib/fs.mq.md atomic_write 'path="notes.md"' 'text="updated"' --effects fs.write --fs-root /absolute/project --json
```

默认 none 只准纯函数；fs.write 包含读取授权，其余能力仍需明确选择。
plugin 保留加载提示所需的既有能力，不增加文件写入、网络或进程执行授权。
根目录检查覆盖新文件的最近已存在父路径与符号链接越界。
会话保存规范化根目录与可持久化结果；重建会话不重复文件、网络或进程写操作。
不可重放写操作缺少快照时明确拒绝，不能为了重建句柄偷偷重做。

可恢复操作统一返回 `{ok,value,error}`，错误含 code/message/operation，校验错误可附 details。
代码来自明确错误标签与 IO 错误种类，不扫描任意路径文本猜类别。Result 不授权操作，也不捕获任意语言错误。
CLI 外层 ok 表示调用成功；函数值的 Result.ok 表示操作协议成功；HTTP status、子进程 code 仍需按领域判断。
语法、契约与不可恢复运行错误继续使用原有结构化 Diagnostic，保留库来源和调用点。

## 平台与资源边界

| 能力 | 原生 | WebAssembly 浏览器目标 | CoW 试运行 |
|---|---|---|---|
| 集合、文本、JSON/CSV/TOML、字节数据、文档结构、静态时区计算 | 支持 | 纯计算可构建；具体浏览器熵/时钟依赖桥接 | 支持 |
| 原生文件与分块 IO | 支持 | 未接入浏览器文件系统时明确不可用 | 文件内容经覆盖层；已有目录枚举/元数据为宿主快照，试运行变更用 sandbox.diff/inspect |
| 文件复制、移动、追加与文本补丁 | 支持 | 无原生 FS | 经覆盖层；复制最多 64 MiB，不绕过试运行写入真实文件 |
| 目录状态、目录树批处理、归档、流式下载 | 支持 | 原生专属能力不可用 | 未建模的目录/原生任务明确拒绝 |
| HTTP、进程、动态插件 | 按构建特性支持 | 原生接口不可用；浏览器使用 browser 效果与桥接 | 依 profile/caps，不以库包装绕过 |
| 子任务取消 | 进程可终止并回收 | 不提供原生子进程 | 函数线程仅逻辑取消；有界批处理只接收文件任务 |

新增有界读取默认 8 MiB、上限 64 MiB；旧 read_text 及其格式组合接口保留整文件读取语义，大文件使用 read_chunk/file_hash；原生文件摘要和下载使用 64 KiB 缓冲区。
查找最多十万条目；序列和连接最多一百万结果；文件批处理最多一万任务、六十四并发。
HTTP 时限与字节上限由调用方明确设置；不自动重试可能修改远端状态的请求。
TOML 日期转换为 ISO 文本；写出不保留注释，None 和非有限数没有支持的表示。
IANA 数据随锁定依赖交付，不声称总是最新政策数据。ZIP 不提供完整备份语义。

## 真实工程与验收

[标准库工程](../../examples/stdlib-handbook/index.mq.md)和[中文入口](../../examples/stdlib-handbook/索引.mq.md)同时完成：

1. 扫描 Markdown，保留逐文件读取结果；坏 UTF-8 不阻止其他记录。
2. 读取 CSV，转换字段，筛选并批量分组汇总，输出 CSV。
3. 精确替换章节，以读取文本的摘要检查来源，保留其他叙述并原子写入。

`tests/stdlib_tasks.rs` 验证公开文档和双后端，而非只测私有宿主调用。
附加测试覆盖 CoW 不落盘、写入快照不重放、超时后继续等待、网络参数预检、重复响应头、
失败下载保留目标、ZIP 越界与链接、夏令时、TOML、混合排序和十万项去重。
旧诊断测试从编译器 AST 定位库指令，继续严格检查独立调用点；不再因添加文档而依赖固定行号。

## 构建验证

原生默认特性、精简 wasm-core 特性、真实 wasm32 交叉编译均已验证。
浏览器 ABI 与 `tests/wasm/stdlib.mjs` 在当前 wasm 产物上由 Node 执行，覆盖集合、字节、
文档、TOML、IANA/DST、32 位大下标与嵌套路径诊断。Node 验收不是浏览器 DOM 布局验收。

发行版 rustc 与官方 wasm 标准库的构建元数据不能混用；本次使用同版本匹配的官方组件，
校验后在临时目录构建并执行，不依赖修改系统工具链。复现时使用已安装对应 wasm32 标准库的工具链：

```sh
cargo check --no-default-features --features wasm-core
cargo build -p marqdo-wasm --target wasm32-unknown-unknown
node tests/wasm/smoke.mjs target/wasm32-unknown-unknown/debug/marqdo_wasm.wasm
node tests/wasm/stdlib.mjs target/wasm32-unknown-unknown/debug/marqdo_wasm.wasm
```
单文件结构修正后，全量 970 项测试通过，0 失败，2 个原有忽略项；其中包括 17 项标准库工程测试。
完整命令、分轮计数和交叉目标结果见[实施验收记录](../roadmap/stdlib-executable-handbook.md)。
