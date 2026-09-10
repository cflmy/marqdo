# Marqdo `plugins/web` — Go (C ABI v2)

施工图：[doc/design/ext-web-go-rewrite.md](../../doc/design/ext-web-go-rewrite.md) · ADR：[0004](../../doc/adr/0004-web-plugin-go.md)

## Build

```bash
./scripts/build-web-plugin.sh
# → plugins/web/build/libweb.so  (also copied to target/debug|release)
export MARQDO_WEB_PLUGIN=$PWD/plugins/web/build/libweb.so
```

Requires **Go 1.22+** with cgo (`CGO_ENABLED=1`) and a C toolchain.

## Layout

| Path | Role |
|------|------|
| `plugin.go` | `//export` ABI entry + registration |
| `abi/marqdo_abi.h` | C contract (from `include/`) |
| `internal/*` | page / db / http / ws / … |

迁移期 Rust 参考实现：`plugins/web-rust-archive/`（勿再演进）。
