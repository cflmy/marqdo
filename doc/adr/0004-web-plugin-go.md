# ADR 0004：网络拓展库原生实现改为 Go

| | |
|---|---|
| Status | **Accepted** |
| Date | 2026-09-10 |
| Related | [ext-web-go-rewrite.md](../design/ext-web-go-rewrite.md) · [ext-abi.md](../design/ext-abi.md) · [0001-implementation-language.md](0001-implementation-language.md) |

## Context

- Marqdo **宿主/解释器**仍为 Rust（ADR 0001 不变）。
- 官方插件契约是稳定 **C ABI**（`include/marqdo_abi.h`），不绑定 Rust ABI。
- `plugins/web`（Rust/axum）已具备 W0–W8 等能力，但迭代成本高；且需验证多语言插件生态。
- 产品目标：网络拓展库支撑完整网页制备，并以暗恋见君（外接 PostgreSQL + Redis）验收；应用层前后端均为 Marqdo 脚本。

## Decision

1. **重写** `plugins/web` 为 **Go** 共享库（`-buildmode=c-shared`），产物名仍为 `libweb.so` / `web.dll` / `libweb.dylib`。
2. **ABI 全量兼容**（函数名、参数、语义）；`ext/web` 作者面默认不动。
3. 施工图：[ext-web-go-rewrite.md](../design/ext-web-go-rewrite.md)。禁止在旧 Rust 插件上继续打补丁演进（ext-web C5）。
4. 其它官方插件（agent / quantum / linalg）**暂不**强制改语言；Go web 作为多语言样板。

## Consequences

- 构建与发版需安装 Go；CI/`marqdo ext add web` 增加 `go build` 回退。
- 根 Cargo workspace 不再包含 `plugins/web` Rust crate（切换完成后）。
- cgo/共享库生命周期与 `shutdown` 需严格测试。
- 金样例 `tests/ext/web-*` 为门禁。
