# 调研：AI 扩展库沙盒设计（执行隔离 × 工作区暂存 × 写回）

| | |
|---|---|
| 状态 | **调研笔记 · 供 `ext/ai/*` 沙盒设计锁定参考**（**2026-10-16 修订**：纳入「AI 只生成 Marqdo」产品约束） |
| 日期 | 2026-10-16 |
| 范围 | 浏览器 WASM / WASI、OS 进程沙盒、容器 / microVM、CoW 叠加工作区、符号链接暂存方案；对照 Marqdo AI 栈 |
| 相关设计 | **[ext-ai-sandbox.md](../design/ext-ai-sandbox.md)** · **[roadmap/ext-ai-sandbox.md](../roadmap/ext-ai-sandbox.md)** · [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [stdlib-writeback.md](../design/stdlib-writeback.md) · [stdlib-subtask.md](../design/stdlib-subtask.md) · [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) · [adr/0002-browser-marqdo-wasm.md](../adr/0002-browser-marqdo-wasm.md) · [ai-stack.md](../roadmap/ai-stack.md) · [ext-abi.md](../design/ext-abi.md) · [stdlib-modules.md](../design/stdlib-modules.md) |
| 姊妹调研 | [agent-frameworks-and-marqdo.md](agent-frameworks-and-marqdo.md) · [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) · [ai-agent-llm-optimize-2026-10.md](ai-agent-llm-optimize-2026-10.md) · [ext-ai-optimize.md](../design/ext-ai-optimize.md) |

---

## 0. 一句话结论

**在「Agent 只生成 / 只执行 Marqdo（`.mq.md`）」这一产品约束下，Marqdo AI 沙盒应以 Wasm 为主沙盒——且必须是「内核 + `lib/*` + `ext/*`」可运行的完整运行时沙盒，而不是只跑光秃解释器。**

锁定方向：

1. **执行隔离 = Marqdo Wasm 运行时（与 ADR 0002 同构）** — deny-by-default 能力导入；**默认不授予** shell / 外联 / `dlopen` 原生插件。AI 产出面收窄到 Markdown 程序后，不必为通用 coding Agent 的「任意二进制」付沙盒税。  
2. **库完备性是验收线，不是加分项** — 沙盒内须能 `import lib/…` 与受支持的 `ext/…`；缺能力时 **显式失败**，禁止静默空成功（对齐 [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §5）。  
3. **工作区突变 = CoW / whiteout / 显式 commit**（应用层 VFS，必要时 overlay）— 「修改短暂不生效 → 确认写回」保留；**禁止以符号链接森林作为隔离边界**。  
4. **OS 沙盒 / microVM** — 降为 **逃逸舱 / 托管 Profile**：仅当显式启用 foreign、`subtask` 真子进程、或未来多租户托管时使用；**不是** Marqdo-only Agent 的默认主路径。

初稿曾按「通用 coding Agent 还需 shell」把 Wasm 与 bwrap 并列；在 Marqdo-only 约束下，主结论改为 **Wasm-first + 库完备 + CoW 写回**。

---

## 1. 调研方法与边界

### 1.1 做了什么

| 动作 | 说明 |
|------|------|
| 厂商/产品文档 | Anthropic Claude Code sandbox / `@anthropic-ai/sandbox-runtime`；VS Code Copilot agent sandboxing；Docker AI Sandboxes；Turso AgentFS CoW overlay；OpenSandbox isolation sessions；WASI / jco preview2-shim；VS Code for the Web WASM webshell |
| 工程对比文 | Northflank / Cosmonic 2026 agent sandbox 指南；gVisor vs Firecracker vs Docker 对比；ContainAI security comparison |
| 学术与标准 | USENIX SEC'22 *Provably-Safe Multilingual Software Sandboxing using WebAssembly*；KTH 论文 *Intra-process Fault Isolation Using WebAssembly*（SFI 通信开销）；ACM TOSEM *Research on WebAssembly Runtimes: A Survey*（2025）；WASI Design Principles（capability / 无 ambient authority） |
| 安全案例 | brig 项目挂载 symlink 逃逸（CVSS 8.2）；Anthropic sandbox-runtime 对 bwrap bind 跟随 symlink 的修复；Docker Sandbox「direct mount vs clone」威胁说明 |
| 对照本仓 | `ext/ai/{llm,agent}`、`lib/writeback`、`lib/subtask`、ADR 0002 浏览器 WASM、`RunOptions.allow_fs_write` 默认关 |

### 1.2 不做的

- 不把「发版用的容器镜像」与「Agent 工具沙盒」混为一谈。  
- 不宣称某一云厂商托管沙盒（E2B / Modal / Daytona）为 Marqdo 默认依赖。  
- 不修改 `doc/next/*`（手写真相源）；本文件只进 `doc/research/`。

### 1.3 产品约束（修订要点）：AI 只生成 Marqdo

这是相对通用 coding Agent（Claude Code / Codex）的 **结构性好消息**：

| 通用 Agent | Marqdo AI Agent（本调研采纳） |
|------------|------------------------------|
| 模型可产出 shell / Python / 任意仓库补丁 | 模型产出面收束为 **`.mq.md`（文档即程序）** |
| 沙盒必须兼容「整台开发机能力」 | 沙盒只需跑 **同构 Marqdo 运行时** |
| Wasm 往往只能罩住工具子集 | Wasm **可以**成为主执行边界 |
| 标准库 = 语言运行时 + OS 工具链 | 标准库 / 扩展 = **`lib/*.mq.md` + `ext/**`（+ 受控宿主能力）** |

硬规则建议（后续 design 应写死）：

1. Agent 工具表 **不提供** 通用 `bash` / 任意语言 `exec`（除非显式 opt-in 逃逸舱，且另套 OS 沙盒）。  
2. Agent 改仓库的权威动作是 **写 / 修订 `.mq.md`（含写回块）**，经 CoW 会话再 `commit`。  
3. 运行 Agent 生成的程序 = 在沙盒里 `run` 该 `.mq.md`，且其 `import` 的 **lib 与 ext 必须可用或显式报不可用**。

### 1.4 威胁模型（在 Marqdo-only 下收窄）

| 威胁 | 典型路径 | Marqdo-only 下的权重 |
|------|----------|----------------------|
| T1 提示注入 | 诱导读密钥、外泄、破坏 `.mq.md` | 仍高 — 靠能力拒绝 + CoW |
| T2 工具侧信道 | `subtask`/`foreign` 子进程 | **默认关闭**；opt-in 才升权 |
| T3 工作区即时破坏 | 直接写穿源树 | 靠 CoW；commit 闸门 |
| T4 挂载逃逸 | symlink / bind 打穿 | 仍相关 — **勿用 symlink 作 W 层** |
| T5 网络外泄 | `lib/net` / llm 出站 | 能力开关 + 域名策略 |
| T6 Wasm 宿主桥过宽 | `host_fs_*` / 插件桥过宽 | **主威胁** — 桥 = 权限表 |
| T7 库残缺假安全 | 沙盒能跑 `# main` 但 `import lib/fs` 怪崩或静默空 | **产品验收威胁** |

**成功标准：** 失败域止于会话 upper + 已授予能力；真实工作区默认只读直至 commit；**代表性 `lib/*` 与 `ext/*` 金样在沙盒内可跑或得到明确 `unavailable in sandbox` 诊断**。

---

## 2. 先拆三个「沙盒放哪里」（Marqdo-only 版）

业界「Harness / Workspace / Execution」三分法仍成立，但 **Execution 的内容变了**：

```text
┌──────────────────────────────────────────────────────────┐
│ Pattern H — Harness（编排环）                              │
│   ext/ai/agent · parent Plan-and-Move ·（可选）MCP client  │
├──────────────────────────────────────────────────────────┤
│ Pattern W — Workspace mutation（.mq.md 真相如何变）         │
│   CoW upper / whiteout / diff / commit → writeback        │
├──────────────────────────────────────────────────────────┤
│ Pattern X — 执行 Agent 产出的 Marqdo 程序                   │
│   Wasm Marqdo runtime + embedded/packaged lib + sandboxed  │
│   ext 能力面（非「任意 shell」）                            │
└──────────────────────────────────────────────────────────┘
```

| Pattern | Marqdo 落点 | 沙盒要解决什么 |
|---------|-------------|----------------|
| H | `ext/ai/agent` | 编排是否持有过宽宿主权限；模型 I/O 与执行隔离 |
| W | `lib/writeback` · 工作簿 | 编辑是否立刻污染仓库；人审/父裁决后落盘 |
| X | **Wasm 内 `run` `.mq.md`** | 计算隔离；**lib/ext 能力是否完备且受控** |

用户提案在 Marqdo-only 下：**Wasm 作 X 主底层是正确方向**；仍须改掉 symlink，并补上 **库完备性** 这一环。

---

## 3. 方案谱系（横向对比）

> **读法（修订后）：** 下表仍保留通用 Agent 对照。对 **Marqdo-only** 产品，请以 §0 / §5 / §6 的 Wasm-first 结论为准；本节 A/B 升权，F/G/H 降为逃逸舱或托管。

评分：● 强 / ◐ 中 / ○ 弱 / — 不适用。维度按「隔离强度」等客观属性；**产品适配**见 §5。

| 方案 | 隔离强度 | 启动成本 | 任意二进制 | 工作区暂存语义 | 与 Marqdo 契合 | 主要风险 |
|------|----------|----------|------------|----------------|----------------|----------|
| **A. 浏览器 WASM + 虚拟 FS** | ●（计算）◐（经桥的 FS） | ● | ○ | ◐（需自建 overlay） | ●（已有 ADR 0002） | 宿主桥过宽；无原生 toolchain |
| **B. WASI / Component Model（非浏览器亦可）** | ● | ● | ○（仅 WASI 目标） | ●（preopens + 自建 VFS） | ● | 工具链不全则需降级 |
| **C. 符号链接映射工作区 + 延迟写回** | ○ | ● | —（本身不是执行沙盒） | ◐（似是而非） | ○ | **symlink 逃逸是已知高危类** |
| **D. 应用层 CoW VFS（upper/lower/whiteout）** | ◐（逻辑） | ● | — | ● | ●● | 必须配合 X 层 OS 隔离 |
| **E. OS overlayfs / FUSE / AgentFS 式 CoW** | ●（配合命名空间） | ◐ | ●（在沙盒内） | ●● | ● | Linux/macOS 实现分叉；运维 |
| **F. bubblewrap / Seatbelt（Anthropic SRT 类）** | ● | ● | ● | ○（默认可写 cwd） | ● | 需另加 CoW 才有「暂不生效」 |
| **G. Docker / 硬化容器 / gVisor** | ◐→● | ◐ | ● | ◐（volume 策略） | ◐ | 共享内核或兼容性；重 |
| **H. microVM（Firecracker / Kata）** | ●● | ◐ | ● | ◐ | ○（本地 CLI 过重） | KVM/运维；适合多租户托管 |
| **I. 纯语言沙盒（RestrictedPython 等）** | ○ | ● | ○ | — | ○ | 对 Agent 工具链几乎无用 |

### 3.1 A — 浏览器 WASM + 虚拟 FS

**机制：** Wasm 线性内存 + 校验 + 仅经 import 碰宿主；FS 多为内存树 / OPFS / 经 JS bridge 的映射。Marqdo 已锁定「解释器进 WASM」（ADR 0002），默认 `allow_fs_write=false`。

**文献支撑：** Wasm 作为 SFI 窄腰（Bosamiya et al., USENIX SEC'22）；进程内沙盒跨边界通信可比 IPC 快数量级（KTH Wasm-SFI 论文）；WASI 明确 **capability / 无 ambient authority**。

**优点：** 与「代码即文档」浏览器运行同构；deny-by-default；适合跑 `.mq.md`、公式库、受控插件。  
**缺点：** 不能原生跑 `cargo test` / `npm` / 任意 shell；安全上限取决于 **bridge 授予了什么**；浏览器场景下「符号链接用户工作区」通常要落到 File System Access API 或服务端中继，**不是** POSIX symlink 语义。

**判定：** 在 **Marqdo-only** 约束下，作为 **Pattern X 主沙盒** 与浏览器内 Marqdo 的正解；完备性取决于 **lib 嵌入 + ext 能力桥**（§4.1），不取决于 shell。通用「任意二进制 Agent」场景下仍不够——但那不是本产品默认。

### 3.2 B — WASI / Component Model（CLI 宿主）

**机制：** preopens 显式挂载；`WASIShim({ sandbox: { preopens, enableNetwork: false } })`（Bytecode Alliance jco）；Component imports = 权限表。

**优点：** 与 Wasm 同族，可在 Node/Wasmtime 跑，不绑浏览器；能力模型清晰。  
**缺点：** 同 A，任意 Linux 工具链需「宿主机再包一层」或改编译目标。

**判定：** Marqdo **官方扩展 / 插件中可 Wasm 化的部分**应走这条；与 ADR 0002 一条能力谱。

### 3.3 C — 符号链接映射工作区（用户设想的核心）

**设想：** 沙盒根下用 symlink 指向用户文件；写改暂存别处；确认后写回；删除 = 删 symlink。

**表面吸引力：** 零拷贝读、实现「像」有暂存。

**为何行业不把它当隔离原语：**

1. **Bind/mount 跟随 symlink 是经典逃逸。** Anthropic `sandbox-runtime` 明确修复：`allowWrite` 若为指向界外的 symlink，bwrap `--bind` 会把真实目标变成可写（PR #138 类修复）。  
2. **项目挂载未解析 symlink 可致高危。** Endor Labs 披露 brig「project mount symlink traversal」（CVSS 8.2）：Agent 在可写项目内把子目录换成指向 `/etc` 的 symlink，下一轮宿主机按字符串挂载即打穿。  
3. **「删除 = 删 symlink」语义错误。** 删链接只去掉别名，**不等于**对 lower 层文件的 whiteout；若链接指向真实文件且曾被打开写穿，删除链接也救不回。  
4. **写穿风险。** 若某路径被解析成真实 inode 且可写，暂存承诺立刻破产。  
5. **跨平台。** Windows junction/reparse、浏览器虚拟 FS、容器 bind，行为不一致。

**判定：** **否决作为 Marqdo 默认工作区隔离方案。** 允许在 **只读、已解析、拒绝可写路径上的 symlink 组件** 的前提下作实现细节，但不得当作安全边界叙事。

### 3.4 D — 应用层 CoW VFS（推荐作 W 层默认）

**机制：**

```text
lower  = 真实工作区（只读打开 / 只读能力）
upper  = 会话 delta（内存 map / 目录树 / SQLite blob）
whiteout = 逻辑删除集合
view(path) = upper ?? (!whiteout && lower)
commit() = 把 upper/whiteout 应用到 lower（可经人审）
discard() = 丢弃会话
diff() = 可审变更集
```

与 Turso AgentFS、OpenSandbox `workspace.mode: overlay` 的 **语义同构**，但不强制 FUSE。

**与 Marqdo 对齐点：**

| VFS 操作 | Marqdo 真相 |
|----------|-------------|
| `diff()` | 父 Agent / 人审材料；可进 view |
| `commit()` 文本结果 | `writeback.record` / 工作簿补丁 |
| `discard()` | 失败回合不污染仓库 |
| 只读 lower | 防止 T3 即时破坏 |

**优点：** 纯用户态、可测、可嵌入 `plugins/agent`；不依赖 root/FUSE；金样可锁。  
**缺点：** 对 **沙盒内原生工具**（它们走真实 POSIX）不可见，除非再挂 E 或把工具 I/O 也改走 VFS API。

**判定：** **Pattern W 的默认实现。**

### 3.5 E — OS CoW overlay（AgentFS / overlayfs / OpenSandbox）

**机制（AgentFS 公开设计）：** lower=宿主只读；upper=SQLite delta；删除=whiteout；Linux 用 FUSE+user/mount ns；macOS 用 NFS+`sandbox-exec`。OpenSandbox 默认 `overlay`，会话结束可丢 upper。

**优点：** 对 **任意 shell 工具**透明；「改了但宿主机 `git status` 仍干净」体验业界已验证。  
**缺点：** 平台依赖；FUSE 权限；与应用层 D 双实现成本。

**判定：** **当 Pattern X 需要真实 shell 且要求暂存对工具透明时**启用（Linux 优先）；作为 D 的「native 加速后端」，不是替代 Wasm。

### 3.6 F — OS 进程沙盒（bwrap / Seatbelt / VS Code Copilot）

**机制：** Anthropic：文件系统隔离 + **网络隔离（缺一不可）**；Linux bubblewrap + 代理；macOS Seatbelt。VS Code Copilot：同类依赖；路径分 readwrite / readonly / denied；明确「不是 VM，削弱授权即削弱边界」。

**优点：** 本地编码 Agent 的事实标准；覆盖子进程；启动轻。  
**缺点：** 默认可写 cwd → **没有**「暂不生效」；必须与 D/E 组合。Symlink 配置路径需解析与拒绝。

**判定：** **Pattern X（shell/外联/MCP）本地默认。**

### 3.7 G / H — 容器与 microVM

**共识：** 裸 Docker 共享内核，不适合作唯一边界应对不可信代码；gVisor 加强 syscall 拦截；Firecracker/Kata 适合多租户与高威胁（~125ms 级启动）。Docker AI Sandboxes 区分 **direct mount（即时写穿）** vs **clone（只读源 + 私有副本）**——后者才接近「确认后写回」。

**判定：** Marqdo **本地 CLI 默认不强制 microVM**；若未来 `ext` 提供托管执行 / 多租户，再把 H 定为服务端 Profile。G 可作为打包/CI Profile。

### 3.8 I — 语言级沙盒

对「Agent 会跑 shell」场景基本无效；仅作极窄解释器内嵌参考。

---

## 4. 专门评估：用户提案（修订）

> 浏览器 WASM 作沙盒底层 + 符号链接用户工作区 + 修改短暂不生效 + 确认后写回 + 删除变删 symlink。  
> **补充约束：** AI 只生成 Marqdo；沙盒须能用标准库与扩展库。

| 子命题 | 评价 | 说明 |
|--------|------|------|
| Wasm 作主执行隔离 | **在 Marqdo-only 下正确** | 与 ADR 0002 同构；不必为 shell 全集付税 |
| 沙盒须含 lib + ext | **正确且关键** | 「能跑 `# main`」≠ 好沙盒；见 §4.1 |
| 「短暂不生效」 | **正确** | CoW / staging（方案 D，必要时 E） |
| 「确认后写回」 | **正确且极契合 Marqdo** | 与 writeback、父裁决、workbook 同一哲学 |
| 「symlink 映射工作区」 | **仍错误** | 与是否只用 Marqdo 无关；换 CoW/whiteout |
| 「删除 = 删 symlink」 | **仍错误** | 应为 whiteout；commit 时再对 lower 删除 |
| 「性能最好且安全性最高」 | **Wasm 主路径下接近成立** | 前提是桥最小权限 + CoW；symlink 仍拉低安全 |

**修订后的正确表述：**

> **以 Wasm Marqdo 运行时为主沙盒，打包/嵌入 `lib/*`，经能力桥提供受支持的 `ext/*`；工作区用 lower 只读 + upper/whiteout；人审/父裁决后 commit。符号链接不作隔离原语。shell/外联/原生 dlopen 默认关闭，仅作显式逃逸舱。**

### 4.1 库完备性：什么叫「标准库和扩展库也能用」

沙盒若只能跑内置算术、却不能 `import lib/json` 或跑 `ext/ai/llm` 的离线金样，对 Agent 开发几乎无用——模型生成的 `.mq.md` **本来就会依赖 lib/ext**。

三类库进沙盒的路径不同：

| 类别 | 今日形态 | 沙盒策略 |
|------|----------|----------|
| **纯 `.mq.md` 库**（多数 `lib/*`、部分 `ext` 门面） | 源码 import | **嵌入或会话导入图**（已有 `embedded_lib` / import map）— 必须默认可用 |
| **需宿主能力的 lib**（`fs` / `net` / `writeback` / `subtask` / `datetime.now`…） | `host_*` | **经能力表桥接**到会话 VFS / 受控 clock / 受控 fetch；默认最小集 |
| **原生 ABI 扩展**（`plugins/web`、`plugins/agent`、quantum 等 `.so`） | `plugin.load` + dlopen | 浏览器/Wasm **无 dlopen**（ADR 0002 N2）。沙盒内应：**纯 mq 门面 + Wasm 插件**、或 **宿主代执行且权限计入会话**、或 **显式 `unavailable`** — 禁止假装成功 |

与 [browser-marqdo-wasm.md](../design/browser-marqdo-wasm.md) §5 对齐并扩展为 **Agent 沙盒能力矩阵（草案）**：

| 能力 | 沙盒默认 | 备注 |
|------|----------|------|
| builtins / 表 / 控制流 / 对象 | ✓ | 内核 |
| `lib/math` `json` `table` `text` `collection`… | ✓ | embedded |
| `lib/fs` | ✓（仅会话 VFS） | 写进 upper；禁越出 root |
| `lib/writeback` | ✓（写会话内 `.mq.md`） | commit 后才进真实树 |
| `lib/net` / llm HTTP | 默认关 / 域名白名单 | 与 T5 |
| `lib/subtask` `fn=` | ✓（同 Wasm 会话） | |
| `lib/subtask` `path=`/`code=`/`lang=` | **默认拒** | 逃逸舱 + OS 沙盒 |
| `lib/plugin` + 原生 `.so` | **默认拒** | 改 Wasm 插件或宿主代执行 |
| `ext/ai/llm` · `ext/ai/agent`（mq 门面） | ✓（能力允许时） | 网络/密钥另授 |
| `ext/web` 服务端 listen/DB | **默认拒或只读 stub** | 完整 axum 不进沙盒；可测「装配/渲染」子集 |
| `ext/quantum` / `ext/linalg` | 目标 ✓ | 优先纯逻辑 / Wasm 插件，避免强依赖宿主机 .so |

**验收金样建议（S 阶段）：**  
一套「沙盒烟囱」：`import` 常用 lib → 写会话文件 → writeback → `diff` → `commit`；再各挑一个 **纯逻辑 ext** 与一个 **需网络的 ext**（后者验证拒绝/白名单）。缺能力必须诊断含 `unavailable in sandbox`（或同等措辞）。

---

## 5. 综合评分（面向 Marqdo-only 产品）

加权（修订）：执行隔离契合 Marqdo-only 30%、**lib/ext 完备** 25%、与 writeback 真相 20%、实现成本 15%、跨端（CLI↔浏览器）10%。  
（「任意 shell 工具完备」不再作为主权重。）

| 候选架构 | 分（相对） | 备注 |
|----------|------------|------|
| **A/B Wasm-first + D CoW + 库完备能力表** | **100（推荐）** | 主路径 |
| 同上 + 逃逸舱 F（opt-in shell） | 95 | 兼容少数 foreign 需求 |
| D + E + F（通用 coding 向） | 75 | 过重；非 Marqdo-only 默认 |
| 仅 F（可写 cwd） | 55 | 缺 CoW；且鼓励非 mq 工具 |
| 仅 A 但无 CoW、无 lib 嵌入 | 40 | 「半沙盒」 |
| 仅 C（symlink） | 20 | 否决 |
| 仅 H（人人 microVM） | 50 | 托管可选；本地默认过重 |

**绝对「最硬」单点**仍可以是 microVM（托管 Profile）。  
**Marqdo「最合适」：** **Wasm 完整运行时沙盒（内核+lib+受控 ext）+ CoW 工作区 + commit 闸门**。

---

## 6. Marqdo 应采用的沙盒设计（推荐锁定草案 · 修订）

### 6.1 产品形态

| Profile | 场景 | 执行（X） | 工作区（W） |
|---------|------|-----------|------------|
| **`mq-sandbox`（默认）** | Agent 生成并试跑 `.mq.md` | **Wasm Marqdo** + embedded `lib/*` + 能力桥 `ext` | 应用层 CoW；commit→真实树 / writeback |
| **`browser`** | 页内同构运行 | 同左（ADR 0002） | 内存/OPFS；无 POSIX symlink |
| **`escape`（显式）** | foreign / 真子进程 | OS 沙盒包一层 | 仍建议 CoW 可见 |
| **`hosted`（可选）** | 多租户 | Wasm 或 microVM | 会话隔离；凭证外置 |

### 6.2 推荐架构图

```text
                 ┌──────────────────────────────────────┐
                 │  ext/ai/agent  (Harness)             │
                 │  只产出 / 编排 .mq.md · 人审闸门      │
                 └──────────────────┬───────────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                                           ▼
     ┌────────────────────┐                    ┌────────────────────┐
     │ Workspace CoW (W)  │                    │ Marqdo Wasm (X)    │
     │ lower = RO 工作区  │◄── fs/writeback ───│ 同构解释器/VM      │
     │ upper / whiteout   │    能力导入         │ + embedded lib/*   │
     │ diff·commit·discard│                    │ + ext 能力面       │
     └─────────┬──────────┘                    │ 默认: 无 shell/so  │
               │                               └────────────────────┘
               ▼
        真实工作区 / <!-- marqdo-out -->
        （代码即文档真相源）

     （可选）escape: subtask path|code|lang → OS sandbox
```

### 6.3 API 面建议（设计级，非立即实现）

落点倾向：宿主能力进 `plugins/agent` / Wasm HostContext；作者面经 `ext/ai` 或 `lib`（**禁止** `ext/**` 直调 `host_*`）。

| 概念 | 语义 |
|------|------|
| `sandbox.open(root, mode=cow)` | 打开会话；lower 只读 |
| `sandbox.read/write/list/remove` | 经 view；remove → whiteout |
| `sandbox.diff` / `stat` | 供父 Agent / view |
| `sandbox.commit` / `discard` | 闸门；可要求 HITL |
| `sandbox.run(source\|path, caps=…)` | **Wasm 跑 Marqdo**；caps 声明 fs/net/llm/… |
| `sandbox.caps` / `inspect` | 列出已授能力与 lib/ext 可用性（对齐 `web.inspect` 哲学） |
| `sandbox.exec(...)` | **仅 escape Profile**；默认不存在于作者工具表 |

与现有衔接：

- `lib/*`：优先 embedded；FS/时间/网络走会话 caps。  
- `ext/*`：mq 门面可加载；原生插件默认不可用并诊断；长期可 Wasm 化的（linalg/quantum 逻辑）优先进 Wasm。  
- `lib/writeback`：写入会话内文件；`commit` 时进入真实树。  
- `lib/subtask`：默认仅 `fn=`；`path`/`code`/`lang` 要 escape。

### 6.4 硬禁（写入后续 design 时应保留）

1. **不以 symlink 森林作为隔离或暂存原语。**  
2. **不以「能跑空 `# main`」宣称沙盒完成** — 必须过 lib/ext 完备金样或显式 unavailable。  
3. **默认工具表不提供通用 shell / 任意语言 exec。**  
4. **不默认可写 bind 真实工作区。**  
5. **宿主桥 = 权限表**：未授能力不得 ambient 成功。  
6. **不静默降级**：缺 Wasm/能力时不得假装已隔离。  
7. **`ext/**` 不调 `host_*`**（既有宪法）。

### 6.5 分阶段落地（建议，非 `doc/next`）

| 阶段 | 交付 | 验收线索 |
|------|------|----------|
| S0 | 威胁模型 + 本调研（含 Marqdo-only） | 本文 |
| S1 | Wasm `run` + embedded 核心 `lib/*` + 会话 VFS 读 | 常用 lib 金样在沙盒绿 |
| S2 | CoW upper/whiteout + diff/commit/discard | 未 commit 时宿主机 `git status` 干净 |
| S3 | writeback / 工作簿在会话内可写；commit 对齐 | agent step 金样 |
| S4 | caps：net/llm 白名单；`subtask` 非 fn 默认拒 | 诊断字符串稳定 |
| S5 | 代表性 ext（纯逻辑 + 需网）沙盒金样 | 完备性验收 |
| S6 | （可选）escape OS 沙盒；hosted microVM | 文档标明非默认 |

---

## 7. 外部资料索引（抽样）

| 类型 | 引用 |
|------|------|
| 产品 | [Anthropic: Claude Code sandboxing](https://www.anthropic.com/engineering/claude-code-sandboxing) · [sandbox-runtime](https://github.com/anthropic-experimental/sandbox-runtime) · [VS Code agent sandboxing](https://code.visualstudio.com/docs/agents/run/agent-sandboxing) · [Docker AI Sandboxes security](https://docs.docker.com/ai/sandboxes/security/) · [Turso AgentFS overlay](https://turso.tech/blog/agentfs-overlay) · [OpenSandbox isolation sessions](https://open-sandbox.ai/guides/isolation-sessions) |
| 标准 | [WASI Design Principles](https://github.com/WebAssembly/WASI/blob/main/docs/DesignPrinciples.md) · [jco preview2-shim sandbox](https://github.com/bytecodealliance/jco/blob/main/packages/preview2-shim/README.md) |
| 论文 | Bosamiya et al., *Provably-Safe Multilingual Software Sandboxing using WebAssembly*, USENIX Security 2022 · *Intra-process Fault Isolation Using WebAssembly* (KTH) · *Research on WebAssembly Runtimes: A Survey*, ACM TOSEM 2025 |
| 综述 | [Cosmonic: Choosing an AI agent sandbox (2026)](https://cosmonic.com/ai-sandbox/choosing-an-agent-sandbox/) · [Northflank: MicroVMs, gVisor & isolation](https://northflank.com/blog/how-to-sandbox-ai-agents) |
| 安全案例 | [brig symlink traversal / Endor Labs](https://www.endorlabs.com/learn/project-mount-symlink-traversal-in-brig-how-a-sandbox-reaches-host-directories-it-was-never-given) · Anthropic sandbox-runtime symlink bind 修复 |

---

## 8. 结语

- **产品好消息：** Agent 只生成 Marqdo → **Wasm 主沙盒设计成立**，不必默认扛通用 shell Agent 的复杂度。  
- **好沙盒的定义：** 不只跑内核，还要 **`lib/*` 可用、`ext/*` 在能力表内可用或显式不可用**；再加 **CoW + commit** 保护仓库真相。  
- **仍要改掉的：** 符号链接当暂存/隔离；默认开放 foreign/dlopen；「空 main 能跑」就算完工。  
- **降级为非默认的：** bwrap/microVM — 留给 escape / hosted，而不是 Marqdo-only 主路径。

不急着实现。下一步若继续收敛，再开 `doc/design/ext-ai-sandbox.md` 把 **能力矩阵 + 库完备金样清单** 锁死即可；**仍不要改 `doc/next/*`**。
