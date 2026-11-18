# Ext pack v2.8.2 发布说明 · Release Notes（AI native flow）

## 1. 一句话

独立拓展包 **2.8.2**（CLI 仍为 **2.8.0**）：在 2.8.1 的 AI 持久化工作流之上，修复 Windows
`extension-release` 打包根路径（`root does not exist`），使双平台 ZIP 可发布。
`ext-v2.8.1` 标签保留但未形成完整 Release 资产。

Independent extension pack **2.8.2** (CLI remains **2.8.0**): AI durable workflows from
2.8.1, plus a Windows `extension-release` checkout-root fix (`root does not exist`) so
both-platform ZIPs can publish. Tag `ext-v2.8.1` remains but did not complete assets.

## 2. 亮点（Highlights）

| | |
|---|---|
| 持久步骤机 | `plugins/agent` `workflow`：`open/next/commit/cancel/retry_*`，CheckpointV2，逐步提交边界 |
| 作者面 | `ext/ai/workflow` · `flow` · `context` · `recovery`；`tick` 单步执行，等待输入不阻塞解释器线程 |
| 选择解析 | 精确 / 编号 / 标签 / 短中英句 / JSON 信封；歧义与未知路径失败关闭 |
| 作用域上下文 | Markdown 感知的 file/class/function 摘录；歧义拒绝；不把整库 import 塞进提示 |
| 安全并行恢复 | 保留成功兄弟结果；仅重试已分类为只读的失败子任务；写/未知效应不自动派发 |
| 可观测性 | `inspect`/`list` 只读进度；测试与调用方可传入 host-relative `resume_dir` |

设计契约：[native-agent-flow-v3.md](../design/native-agent-flow-v3.md)。

## 3. 安装（Install）

```bash
marqdo ext add agent   # CDN / Releases → 2.8.2
# 或：MARQDO_EXT_VERSION=2.8.2 marqdo ext add agent
marqdo ext add llm
```

产物：`marqdo-2.8.2-ext.zip` 与双平台 native ZIP（含 agent / llm 等既有原生库）。

## 4. 验证（Validation）

```bash
cargo test -p marqdo_plugin_agent --locked
cargo run --locked -- check ext/ai/flow.mq.md
cargo run --locked -- check ext/ai/context.mq.md
cargo run --locked -- check ext/ai/recovery.mq.md
cargo run --locked -- check ext/ai/workflow.mq.md
cargo run --locked -- run tests/ext/ai-native-flow.mq.md
cargo run --locked -- run tests/ext/ai-native-workflow.mq.md
```

CI：`.github/workflows/native-ai-flow.yml` 与 `extension-release.yml`（`ext-v2.8.2`）。

## 5. 已知限制（Known limitations）

- 输入闸门保存的是**步骤边界**，不是任意字节码解释器帧；回复后须显式进入下一 Marqdo 函数。
- 作用域选择依赖 Markdown 标题约定，不是编译器已解析的 AST 符号表。
- 调用方提供的 principal/session/source revision 须由已认证宿主证明，不能来自模型文本。
- 并行复用 `subtask.run_many`；跨进程子任务租约与已在跑外部进程的取消传播仍待专项注入验证。
- 实时大模型与目标 1B 本地模型评测仍开放；写效应自动重试仍被拒绝，直至 EffectLedger 对账完成。

**核心表面 Δ**：无增删——19 个核心构造不变；本包为 Extension / Agent Runtime。
