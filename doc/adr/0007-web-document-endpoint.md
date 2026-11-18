# ADR 0007：Web Document / Endpoint / Resource（破坏性 · 代码即文档）

| | |
|---|---|
| Status | **Accepted** |
| Date | 2026-09-28 |
| Related | [ext-web-artifact.md](../design/ext-web-artifact.md) · [ext-web.md](../design/ext-web.md) · [binding.md](../design/binding.md) · ADR [0006](0006-artifact-metadata-binding.md) · [layers.md](../design/layers.md) · [doc/next/007.md](../next/007.md) |

## Context

- [`ext/web/web.mq.md`](../../ext/web/web.mq.md) 已膨胀为小型 Web Framework（Page God Object、`configure` 垃圾抽屉、碎片 Table DSL、隐式路由），与 Marqdo「代码即文档」冲突。
- [doc/next/007.md](../next/007.md) 提出 Document / Endpoint / Resource 三原语与拆分 `ext/web` · `ext/net` · `ext/data` · `ext/security`。
- 现行 [ext-web.md](../design/ext-web.md) **C1** 禁止为 web 改核心；Document/Endpoint 装载识别需要 Language/Runtime 参与（复用 Artifact Metadata `type`，**不**新增 CORE_CONSTRUCTS 标记）。
- Binding（ADR 0006）已提供文件级 Metadata；`type: prompt` 先例在扩展侧消费 metadata。

## Decision

1. **采纳** Document / Endpoint / Resource 为 Web 模型一等公民；正式规范见 [ext-web-artifact.md](../design/ext-web-artifact.md)。
2. **修订 ext-web C1**：允许核心在装载期识别 `type: web|endpoint`（及中文 `类型: 网页|端点`）并暴露 artifact 分类；**装配 / HTTP / DB 语义仍在 ext + plugin**。单元格求值、import 句法、CORE_CONSTRUCTS 计数不变。
3. **文件级 Artifact only**：不在 `##` 函数体嵌套 `---` frontmatter（与 Binding Phase 1 / 解析器一致）。
4. **破坏性重写作者面**：删除 `compose_*` 碎片 DSL、god `# page` 变异堆、`app.configure` 多参数抽屉、作者 `ensure_plugin`；examples 与 `tests/ext/web*` 同步重写。
5. **目录定稿**（007 §三十五 + §四十）：
   - `ext/web/` — page · route · component · dom · client · facade
   - `ext/net/` — http · websocket · url
   - `ext/data/` — db · cache · storage
   - `ext/security/` — auth · oidc · rbac
6. **统一 View 表**：`| type | slot | value | attrs | style |`；Table = Data，禁止每函数私有方言。
7. **declared vs system routes**：用户 route 须声明；`/_part` · `/_form` 等为 system，经 `web.inspect` 可见。
8. **import = capability**：`import web:…` 由 facade 自动加载 native plugin；作者程序不写 `ensure_plugin` / `host_*`。
9. **Metadata v1 锁定（原 007 未决项）**：
   - 发现：`web.serve root=` 扫描 `root/**/*.mq.md`，仅注册 `type`/`类型` ∈ {web,网页,endpoint,端点} 的文件；无 type 的库文件忽略。
   - Data：仅 flat 键 `data_source` · `data_order` · `data_where`（text；where 为 `col=val` 逗号分隔）。嵌套 YAML `data:` 推迟。
   - Result：`{ok: bool, value: any, error: text|None}`。
   - Realtime 模块名：`ext/net/websocket.mq.md`（ZH：`实时.mq.md`）。
   - ZH metadata：`类型`←type，`路由`←route，`方法`←method，`路径`←path，`请求`←request，`响应`←response，`鉴权`←auth，`数据源`←data_source，`排序`←data_order，`条件`←data_where。

## Consequences

### Positive

- `.mq.md` 同时是文档、路由、实现与鉴权声明。
- 与 LLM/Agent Artifact Metadata 统一心智。
- 职责边界清晰，可独立演化 data/security/net。

### Negative / Risks

- 破坏性：全部现有 web 作者程序与 smokes 必须改写。
- Endpoint 热路径仍经 plugin → `CallLibPath` / 文件装载回调；须保持 ABI 稳定演进。
- flat data 键表达力弱于嵌套 YAML；后续另 ADR。

### Neutral

- 不新增 Markdown 核心标记；`${…}` 仍仅 Metadata。
- Go `plugins/web` 可分波删除死 compose ABI；作者面先断。
