# agent-sandbox-okf（O1）

演示 **opt-in 沙盒试跑 + 表风格**（少 json 胶水）：`agent.trial` 写入先落 upper，`commit=False` 时宿主不变，成功后再 `commit=True`。

```bash
cargo build --release -p marqdo -p marqdo_plugin_agent
export MARQDO_EXT="$PWD/ext"
./target/release/marqdo run examples/agent-sandbox-okf/index.mq.md
```

期望：`dry-ok` · `host-clean` · `wet-ok` · `proof-visible` · `profile-default`。

见 [ext-ai-optimize.md](../../doc/roadmap/ext-ai-optimize.md) · 金样 [agent-sandbox-optin.mq.md](../../tests/ext/agent-sandbox-optin.mq.md)。
