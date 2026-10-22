# Marqdo `plugins/web` — Go (C ABI v2)

施工图：[doc/design/ext-web-go-rewrite.md](../../doc/design/ext-web-go-rewrite.md) · ADR：[0004](../../doc/adr/0004-web-plugin-go.md)

**Default native web plugin (W-G13).** `plugin.native_path name=web` resolves `plugins/web/build/libweb.so` (and copies under `target/{debug,release}/`).

## Build

```bash
./scripts/build-web-plugin.sh
# → plugins/web/build/libweb.so
# → also copied to target/debug|release/libweb.so
```

Requires **Go 1.22+** with cgo (`CGO_ENABLED=1`) and a C toolchain.

Override path: `export MARQDO_WEB_PLUGIN=$PWD/plugins/web/build/libweb.so`

## Layout

| Path | Role |
|------|------|
| `plugin.go` / `abi_*.go` | `//export` ABI entry + registration |
| `abi/marqdo_abi_cgo.h` | C contract for cgo |
| `internal/*` | page / db / http / ws / … |

The legacy Rust implementation was archived during v1.6 and then removed from the
working tree (it remains in git history before commit `<this commit>`); see
`CHANGELOG.md` “默认 libweb 为 Go”。
