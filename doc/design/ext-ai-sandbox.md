# Official AI sandbox (`ext/ai/sandbox`)

| | |
|---|---|
| Status | **Accepted · implementing** |
| Date | 2026-10-16 |
| Related | [research/ai-agent-sandbox.md](../research/ai-agent-sandbox.md) · [roadmap/ext-ai-sandbox.md](../roadmap/ext-ai-sandbox.md) · [ext-agent.md](ext-agent.md) · [ext-abi.md](ext-abi.md) · [browser-marqdo-wasm.md](browser-marqdo-wasm.md) · [ai-stack.md](../roadmap/ai-stack.md) · [stdlib-writeback.md](stdlib-writeback.md) |

---

## 0. One-liner

**Agent-only Marqdo trial runtime:** capability-isomorphic to browser Wasm defaults, CoW workspace until explicit `commit`, author surface under `ext/ai` (not stdlib).

---

## 1. Goals / non-goals

### Goals

| # | Goal |
|---|------|
| G1 | Open a CoW session over a workspace root; edits stay in upper until `commit` |
| G2 | `run` executes `.mq.md` (or in-memory source) under deny-by-default caps with core `lib/*` usable |
| G3 | `diff` / `discard` / `inspect` for review gates |
| G4 | Stable `unavailable in sandbox` for plugin.load, exec foreign/subtask path|code|lang, net (default), `ext/web` server APIs |
| G5 | Author API in `ext/ai/sandbox` via `plugins/agent` — **no** public `lib/sandbox` |

### Non-goals (this wave)

| # | Non-goal |
|---|----------|
| N1 | Embed wasmtime in CLI |
| N2 | True Wasm ABI plugins (linalg/quantum) |
| N3 | bwrap / microVM |
| N4 | Full `ext/web` listen/DB/WS inside sandbox |
| N5 | Symlink-forest workspace isolation |
| N6 | Default host-proxy of all native ABI plugins |
| N7 | Wire `agent.step`/`plan` through sandbox automatically (later) |

---

## 2. Layering

```text
ext/ai/sandbox.mq.md  →  plugin.load agent  →  sandbox_* ABI
                                              ↓ host_query
                                    HostContext + CowSession
                                    (fs / writeback see upper)
```

| Layer | May call `host_*` | Role |
|-------|-------------------|------|
| `ext/ai/sandbox` | **No** | Author facade |
| `plugins/agent` | via `host_query` only | Register `sandbox_*`; forward to host |
| `src/host` | — | CoW session store; fs/writeback intercept; nested `run` |

---

## 3. Author API

EN: `ext/ai/sandbox.mq.md` · ZH: `ext/ai/沙盒.mq.md` (separate files, no mixed names in one module).

| Method | Params | Result |
|--------|--------|--------|
| `open` | optional `root=` (default entry dir / cwd) | map `{ id, root }` |
| `write` | `id=`, `path=`, `text=` | none |
| `read` | `id=`, `path=` | text |
| `remove` | `id=`, `path=` | none (whiteout) |
| `diff` | `id=` | list of `{ path, kind: write\|remove }` |
| `commit` | `id=` | `{ ok, applied }` |
| `discard` | `id=` | `{ ok }` |
| `run` | `id=`, `path=` **or** `source=` | `{ ok, value, stdout?, error? }` |
| `inspect` | `id=` | caps, root, change count, notes |

Default `run` caps: `fs_write=true` (CoW only), `exec=false`, `net=false`, `plugin=false`.

---

## 4. CoW semantics

- **lower** = real workspace root (read-through).
- **upper** = session map path → bytes.
- **whiteout** = logical deletes; reads miss; commit unlinks lower.
- No symlink-as-isolation; resolve paths under root; reject `..` escape.
- Until `commit`, host tree unchanged (verifiable with `git status` / file mtime).

---

## 5. Hard bans

1. No public `lib/sandbox`.
2. No symlink forest as the isolation story.
3. No silent success when capability missing — diagnose `unavailable in sandbox`.
4. `ext/**` must not call `host_*`.
5. Do not treat empty `# main` as sandbox completeness.

---

## 6. ABI / host_query

Plugin exports (registered on agent plugin): `sandbox_open`, `sandbox_write`, `sandbox_read`, `sandbox_remove`, `sandbox_diff`, `sandbox_commit`, `sandbox_discard`, `sandbox_run`, `sandbox_inspect`.

Implementation: each forwards to allowlisted `host_query` names of the same stem (e.g. `sandbox_open`), which mutate the process-local CoW store and (for `run`) invoke nested capture with a sandboxed `HostContext`.
