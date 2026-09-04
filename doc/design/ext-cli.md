# Official extension installer (`marqdo ext`)

| | |
|---|---|
| Status | **Accepted (v0.3.5 prep)** — prebuilt native download |
| Date | 2026-09-04 |
| Related | [ext-llm.md](ext-llm.md) · [ext-agent.md](ext-agent.md) · [ext-abi.md](ext-abi.md) · [08-ext-deploy-coupling.md](ext-web-customization/08-ext-deploy-coupling.md) |

## Commands

```text
marqdo ext list
marqdo ext add llm
marqdo ext add agent
marqdo ext add web
marqdo ext remove llm
```

## Install root

1. `MARQDO_EXT` if set  
2. Else `~/.marqdo/ext`

Imports use paths like `import llm:ext/ai/llm.mq.md` (see [`load.rs`](../../src/load.rs)).

## Goal: no local Rust required

End users who install Marqdo from **GitHub Releases** should run:

```bash
marqdo ext add web
```

and get both L1 `.mq.md` **and** the matching native plugin (`.dll` / `.so`) **without** `cargo build`.

### Resolution order (`add` with native crate)

1. Local artifact (`MARQDO_*_PLUGIN`, `target/{debug,release}`, beside `marqdo`, `MARQDO_EXT_SOURCE/native`, …)
2. **Download** `marqdo-{VER}-native-{target}.zip` from the GitHub Release for this CLI version → cache under `~/.marqdo/cache/`
3. If Rust/`cargo` is available → `cargo build -p marqdo_plugin_*` (developers)
4. Else fail with a clear message pointing at the Release assets

L1 sources:

1. `MARQDO_EXT_SOURCE` / repo `./ext` / beside binary  
2. Else download `marqdo-{VER}-ext.zip` into `~/.marqdo/cache/ext-src-v{VER}/`

### Env knobs

| Variable | Meaning |
|----------|---------|
| `MARQDO_EXT_VERSION` | Override SemVer used for Release asset names (default: CLI `CARGO_PKG_VERSION`) |
| `MARQDO_EXT_NO_DOWNLOAD=1` | Never hit the network (CI / air-gap); local/cargo only |
| `MARQDO_WEB_PLUGIN` / `MARQDO_AGENT_PLUGIN` / `MARQDO_QUANTUM_PLUGIN` | Explicit native path |

Windows Release **zip** also ships `ext/` + `ext/native/*.dll` next to `marqdo.exe` so a portable unzip works offline after one download.

## Source for `add` (developers)

1. `MARQDO_EXT_SOURCE` — directory containing `ai/llm.mq.md` …  
2. Else repo `./ext` near cwd / binary  
3. Else Release `marqdo-*-ext.zip` (see above)

## Layout (`ext/`)

```text
ext/
  ai/          # llm + agent L1
  web/
  quantum/
```

Installed tree (`~/.marqdo/ext` or `MARQDO_EXT`):

```text
web/web.mq.md
web/网页.mq.md
native/libweb.so   # or web.dll / libweb.dylib
web.plugin         # absolute path hint
```

## Catalog

| Id | Installs under `MARQDO_EXT` |
|----|-----------------------------|
| `llm` | `ai/llm.mq.md`, `ai/大模型.mq.md` |
| `agent` | `ai/agent.mq.md`, `ai/智能体.mq.md`, **native `agent`** |
| `web` | `web/web.mq.md`, `web/网页.mq.md`, **native `web`** |
| `quantum` | `quantum/quantum.mq.md`, `quantum/量子.mq.md`, **native `quantum`** |

## Release assets (native)

| Asset | Platform |
|-------|----------|
| `marqdo-VER-native-x86_64-pc-windows-msvc.zip` | Windows x64 (`native/*.dll`) |
| `marqdo-VER-native-x86_64-unknown-linux-gnu.zip` | Linux x64 (`native/lib*.so`) |

macOS: use local `cargo build` until CI adds Darwin natives (same zip naming pattern reserved).

## Tests

`tests/gold.rs`: `ext_cli_add_list_remove_llm`, `ext_cli_add_agent`, `ext_cli_add_web` (local build path; set `MARQDO_EXT_NO_DOWNLOAD=1` in sandboxes if needed).

Unit: `ext_fetch` URL / triple tests.

## Non-goals

- Third-party registry  
- Merging domain helpers into `lib/`  
- Guaranteeing plugin **source** ABI stability (author `.mq.md` API is the contract)
