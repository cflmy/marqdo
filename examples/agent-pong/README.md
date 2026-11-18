# agent-pong（Wave B0）

演示 Marqdo 智能体宪法：**代码即文档** — 模型上下文含 **call site** 与本文件 **source**；工具是同文件里的 `## 获取时间`。

```bash
# 仓库根目录
cargo build --release -p marqdo -p marqdo_plugin_agent
export MARQDO_EXT="$PWD/.marqdo-ext"   # 或: marqdo ext add agent
# 可选 live：仓库根 `.env` + `AGENT_LIVE=1 marqdo run examples/agent-pong/index.mq.md`
./target/release/marqdo run examples/agent-pong/index.mq.md
```

期望离线输出含：`call-site-ok` · `source-ok` · `tool-in-source-ok`。  
配置真实 `.env` 时还会跑一次 `step`（CALL 工具）。

见 [agent-framework-2026-09.md](../../doc/research/agent-framework-2026-09.md)。
