# Marqdo Binding · Artifact Metadata（Phase 1）

| | |
|---|---|
| 状态 | **Accepted · Phase 1（Metadata-only）** |
| 日期 | 2026-09-24 |
| ADR | [0006-artifact-metadata-binding.md](../adr/0006-artifact-metadata-binding.md) |
| 讨论稿 | [doc/next/006.md](../next/006.md) |
| 实现 | `src/binding/` · `Module.metadata` · `sys.meta` / `sys.meta_get` · `marqdo run --bind` |

> **边界**：核心负责「声明与解析配置」；扩展负责「解释配置的语义」。  
> **本波**：`${…}` 只出现在文件头 `---` Artifact Metadata 中。正文插值见 §8。

---

## 1. Artifact Metadata

文件开头成对 `---` 之间为 **Artifact Metadata**（声明式元信息环境），不是「随便贴的 YAML」。

```text
.mq.md
├── Metadata   ← Identity / Configuration / Dependencies / Bindings
└── Executable Body
```

- `import bind:…` / `导入` 行：模块依赖（既有语义不变）。
- 其它 `key: value` 行：元信息；值可为字面量或 **Binding Expression**。
- Phase 1 仅支持 **flat** 键（同级 `key: value`）；块标量 `>` / `|` 收成 Text。

例：

```yaml
---
title: demo
type: llm
model: ${env.OPENAI_MODEL ?? "gpt-4o-mini"}
api_key: ${secret.OPENAI_API_KEY!}
base_url: ${env.OPENAI_BASE_URL ?? "https://api.openai.com/v1"}
import sys:lib/sys.mq.md
---
```

加载后内部为已绑定的 `Module.metadata`（确定性快照），执行期不再对同一键反复 `getenv`。

---

## 2. Binding Expression

外形：

```text
${expression}
```

### 2.1 路径

```text
${namespace.name}
```

| Namespace | 意义 | Phase 1 |
|-----------|------|---------|
| `env` | 进程环境变量 | ✅ |
| `arg` | CLI / Runtime 绑定（`marqdo run --bind KEY=VALUE`） | ✅ |
| `sys` | 系统信息：`cwd` · `platform` | ✅ |
| `secret` | 敏感环境变量 → `Value::Secret` | ✅ |
| `config` | 配置 Artifact | 推迟 |
| `file` | 外部文件内容 | 推迟 |

### 2.2 默认与必需

```text
${env.OPENAI_MODEL ?? "gpt-4o-mini"}
${secret.OPENAI_API_KEY!}
```

- `A ?? B`：A 缺失（未设置 / 空）则用 B。
- `A!`：A 必须存在，否则加载失败（`config.required_binding_missing`）。

### 2.3 类型转换

```text
${int(env.MAX_TOKENS ?? "4096")}
${float(env.TEMPERATURE ?? "0.2")}
${bool(env.STREAM ?? "false")}
```

### 2.4 独占 value vs 插值

| 形态 | 语义 |
|------|------|
| `model: ${env.OPENAI_MODEL}` | **Value binding** — 保留解析后的类型（含 Secret） |
| `title: "Hello ${env.USER}"` | **Interpolation** — 结果为 Text |
| `model: gpt-4o-mini` | **Literal** — 字面量；**不会**被环境变量静默覆盖 |

---

## 3. Resolution Policy

对每个 metadata 键：

1. 解析该键的字面量 / Binding / 插值（字面量定稿后不受 env 覆盖）。
2. 若 CLI 提供了同名 `--bind KEY=VALUE`，**覆盖**该键的最终值（Text）。
3. Binding 内部查找顺序（仅当表达式写出时）：`arg` / `env` / `secret` / `sys` 按表达式 namespace；`??` 链从左到右。

推荐作者意图：

```text
CLI --bind
  → Artifact 字面量 / 已写死的 Binding 结果
  → Binding 源（env / secret / …）
  → ?? 默认
```

---

## 4. Secret

`${secret.NAME}` → `Value::Secret`。

- `print` / `str` / Display → `<secret>`（不泄露）。
- `type` → `secret`。
- 与 Text 拼接（如 `bearer + api_key`）时**揭示**明文，供 HTTP Authorization。
- 不得进入普通 episode / 错误信息明文（扩展作者责任；核心 Display 已遮掩）。

---

## 5. Runtime API

| API | 语义 |
|-----|------|
| `sys.meta` / `系统.元信息` | 入口模块已绑定 metadata（map） |
| `sys.meta_get` / `系统.取元信息` | `key=` → 值或 `None` |
| `marqdo run FILE --bind KEY=VALUE` | 可重复；写入 `arg` 并覆盖同名 metadata 键 |
| `sys.env_get` | **保留**逃逸口；新代码优先 Metadata Binding |

---

## 6. 错误

| code | 何时 |
|------|------|
| `config.required_binding_missing` | `${ns.name!}` 且源缺失 |
| `config.unknown_binding_ns` | 未知 namespace |
| `config.bad_binding` | 表达式非法 / cast 失败 |

诊断尽量带 frontmatter 行号。

---

## 7. 与扩展的分工

- **核心**：解析 Metadata、resolve Binding、暴露 `Module.metadata` / `sys.meta*`。
- **`ext/ai/llm`**：构造次序为 **显式实参 → `sys.meta_get`（model / base_url / api_key）→ `sys.env_get` 回退 → 默认**。

---

## 8. Phase 2（不做于本波）

- 正文 / 粗体 / 表 / 调用实参中的 `${…}`
- `config` / `file` namespace、嵌套 YAML map
- 显式 `override:` 糖
- 计入 `CORE_CONSTRUCTS`
