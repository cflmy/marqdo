# agent-okf-flywheel（Wave B0）

演示 **文档即知识库（OKF）**：固化工作簿 promote 后，相同 goal 的 `plan` 走 `cache=hit`（`llm_free` 路径），无需再问模型。

```bash
cargo build --release -p marqdo -p marqdo_plugin_agent
export MARQDO_EXT="$PWD/.marqdo-ext"
./target/release/marqdo run examples/agent-okf-flywheel/index.mq.md
```

期望：`promoted` · `hit` · `flywheel-ok`。

见 [okf.md](../../doc/design/okf.md) · [agent-framework-2026-09.md](../../doc/research/agent-framework-2026-09.md)。
