# 变更记录：插件 ABI 共享 crate（`marqdo-plugin-sdk`）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 来源 | [code-volume-audit.md](code-volume-audit.md) §4 第 1 项 |
| 改动 | **新增 `crates/marqdo-plugin-sdk`** · 5 个插件改为依赖它 |
| ABI | **不变**（`include/marqdo_abi.h`、`src/host/plugin.rs` 看到的符号与布局完全一致） |

---

## 1. 问题

5 个原生插件各自复制同一套 ABI 前言（约 140 行/个）：

| 复制的内容 | 说明 |
|-----------|------|
| `type PluginFn` / `HostQueryFn` / `struct MarqdoHostApi` | 布局必须逐字一致，抄错就是 UB |
| `static mut HOST_ALLOC/FREE/QUERY/USERDATA` | 同上 |
| `host_strdup` / `set_out` / `set_err` / `parse_args` / `reply` | 语义必须一致 |
| `register` + `#[no_mangle]` 四导出 | 样板 |

一共 5 份，任何一份漂移都会在运行期表现为"某插件偶发崩"。

## 2. 方案

新增工作区成员 `crates/marqdo-plugin-sdk`，持有唯一一份实现，并提供一个宏生成各 `.so` 必需的导出：

```rust
marqdo_plugin!(marqdo_plugin_sdk::ABI_VERSION, [
    ("llm_ping", "", llm_ping),
    ("llm_chat_completions", "base_url,api_key,...", llm_chat_completions),
]);
```

宏展开为 `marqdo_plugin_abi_version` / `marqdo_plugin_shutdown` / `marqdo_plugin_init`，
其中 init 调 `init_plugin(host, &[…])`（保存回调 + 逐个注册）。

`demo` 是 ABI **v1**，宏接受版本参数：`marqdo_plugin!(1, [...])`，语义不变。

## 3. 安全措施：注册项逐一核对

迁移时把每个插件的 `register(...)` 调用抽取成表。**第一次抽取漏了 11 项**——
正则是 `[a-z_0-9]+`，匹配不到 `mcp_server::agent_mcp_serve`、`sandbox::*` 这类
**路径限定**的函数名。编译能过（表少几行），运行时才会 `unknown function`。

补救：正则允许 `::`，并把新旧两张表逐项比对：

| 插件 | HEAD 注册项 | 迁移后 | 差异 |
|------|------------|--------|------|
| agent | 63 | 63 | **0** |
| quantum | 41 | 41 | 0 |
| linalg | 31 | 31 | 0 |
| llm | 2 | 2 | 0 |
| demo | 2 | 2 | 0 |

比对脚本同时校验 `(名称, 参数串, 函数)` 三元组完全一致，不只比数量。

另外两次由"编译通过但语义已变"暴露的问题：

| 问题 | 现象 | 处理 |
|------|------|------|
| `mcp_client` 被误判为死模块 | 我删掉 327 行后发现 `agent_mcp_client` 正注册它 | 恢复模块（教训：先查注册表，再判死） |
| 量子插件两个"未使用"函数 | 删除时连带吃掉 `density_from_amps`/`flat_to_nested` | 恢复并加 `#[allow(dead_code)]` 注释 |

## 4. 收益

| 插件 | 迁移前 | 迁移后 | 变化 |
|------|--------|--------|------|
| agent | 1,237 | 929 | −308 |
| quantum | 1,045 | 835 | −210 |
| linalg | 688 | 524 | −164 |
| llm | 405 | 278 | −127 |
| demo | 170 | 73 | −97 |
| 新增 SDK | — | 232 | +232 |

净减 **≈ 906 − 232 = 674 行**，且 ABI 前言从 5 份变 1 份。

## 5. 验证

| 项 | 结果 |
|----|------|
| `cargo build --release`（含 5 个插件） | 零警告 |
| `cargo test --release` | **609 passed / 0 failed** |
| 金样实跑（量子） | `p00-ok` / `p11-ok` / `ping-ok` |
| 注册项三元组比对 | 5 个插件全部 0 差异 |

## 6. 边界

- ABI 版本号、`#[no_mangle]` 符号、结构体布局**一律未变**；旧 `.so` 与新宿主仍可互操作。
- 插件私有逻辑（HTTP、矩阵、量子态、agent 工具）一行未动。
- 后续新增插件只需依赖 SDK + 一行宏调用。

## 7. 变更清单

| 文件 | 变更 |
|------|------|
| `crates/marqdo-plugin-sdk/{Cargo.toml,src/lib.rs}` | **新增**：ABI 类型、静态回调、`host_strdup/set_out/set_err/parse_args/reply/host_query_json(_with)/take_host_string/init_plugin`、`marqdo_plugin!` 宏 |
| `Cargo.toml` | workspace 增加 `crates/marqdo-plugin-sdk` |
| `plugins/{agent,quantum,linalg,llm,demo}/Cargo.toml` | 增加 SDK 依赖 |
| `plugins/*/src/lib.rs` | 删除本地 ABI 前言与四导出，改用宏；注册表改为数组 |
| `doc/design/change-plugin-sdk.md` | 本文件 |
