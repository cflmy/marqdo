# ADR 0006：Artifact Metadata Binding（`${…}` 进核心 · Metadata-only）

| | |
|---|---|
| Status | **Accepted** |
| Date | 2026-09-24 |
| Related | [binding.md](../design/binding.md) · [core-surface.md](../design/core-surface.md) · [layers.md](../design/layers.md) · [doc/next/006.md](../next/006.md) |

## Context

- Executable Document 需要声明「依赖哪些外部运行时资源」（模型名、API 密钥、base URL），而不是在 `ext/*` 里各自 `sys.env_get`。
- [doc/next/006.md](../next/006.md) 提出 **Marqdo Binding**（`${namespace.name}`）与 Artifact Metadata 统一解析。
- [layers.md](../design/layers.md)：Language 层变更须 ADR；核心构造计数只许持平或下降；默认进 ext。
- 正文插值会与 `$$…$$` 公式面冲突，且会新增内联构造压力。

## Decision

1. 采纳 **Artifact Metadata Binding** 为 Language 层能力；正式规范见 [binding.md](../design/binding.md)。
2. **`${…}` 仅**在文件头 `---` Metadata 中生效；正文 / 粗体 / 调用 / 表 / 叙述 **永不**写 Binding Expression（否决 [006 §17](../next/006.md)）。
3. **不**把 Binding 记入 [core-surface.md](../design/core-surface.md) 的 19 构造守卫表——与 `import` 同属 §4「其它语言面」。
4. 核心只做声明与解析；扩展（如 `ext/ai/llm`）消费已绑定的 `Module.metadata` / `sys.meta*` / 入口提升变量。
5. 第一版 namespace：`env` · `arg` · `sys` · `secret`。`config` / `file` 推迟。
6. 加载期一次 resolve → Bound Artifact；`Value::Secret` 遮掩展示；`sys.env_get` 保留为逃逸口。
7. **Phase 2**：入口执行前把 metadata 键提升为普通变量（形参/赋值可覆盖）；正文用 `` `model` ``，不再 `${env…}`。

## Consequences

### Positive

- 配置成为文档的一部分；LLM / agent / 其它 ext 共用同一套 Binding，避免各写一套 getenv。
- 缺失 `!` 在加载期失败，而不是运行到 HTTP 401。
- 不膨胀 CORE_CONSTRUCTS；与公式 `$$` 无冲突。
- 正文保持普通变量语义，不引入第二套环境变量语法。

### Negative / Risks

- Secret 在 `Text + Secret` 拼接时会揭示明文（供 Authorization）；须约束日志面。
- metadata 键与业务变量同名时依赖「形参/赋值覆盖」规则，作者需注意命名。

### Neutral

- `#` / `##` / `**` / `*` 等核心标记不变；frontmatter `import` 句法不变。
