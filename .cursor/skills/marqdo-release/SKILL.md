---
name: marqdo-release
description: >-
  Run a Marqdo product release: detect latest version and ask the user for the
  next tag (or confirm mode), update CHANGELOG/README/skills/public docs, bump
  Cargo and/or ext/VERSION, sync VS Code extension when needed, publish GitHub
  Release assets, upload extension packs to Cloudflare R2 (ext.marqdo.com) and
  GitHub, ship Ubuntu CLI via PPA (scripts/ppa-ship.sh), and print manual
  commands when sudo/GPG/network steps need the user. Use when the user asks to
  release, cut a version, publish vX.Y.Z, ext-only pack, PPA, 发版, 发布新版本,
  or tag marqdo.
---

# Marqdo release

Canonical release playbook for **cflmy/marqdo**. Read this skill **before** tagging or uploading assets. Details: [reference.md](reference.md).

## Hard rules

1. **Never invent the next version.** Detect current versions → **stop and ask the user** for the next SemVer (`X.Y.Z`) and **release mode** if unclear. Do not proceed until they confirm (unless they already stated both in the same message).
2. **Never force-push** `main` / tags. **Never** `--no-verify` unless the user explicitly orders it.
3. **Never commit `vscode-extension/` on `main`** (gitignored). Extension source lives only on branch **`vscode-extension`**. See `doc/design/vscode-extension-commit.md`.
4. **Do not release from a dirty tree** (except intentional release commits you create in this flow).
5. **Tag format:** CLI/full → `vX.Y.Z` matching root `Cargo.toml`. Ext-only → `ext-vX.Y.Z` matching `ext/VERSION` (Cargo may stay unchanged).
6. Prefer **tag push → GitHub Actions** (`.github/workflows/release.yml`) for Windows/Linux install assets. Local packaging is fallback / verification.
7. After network errors: apply [reference.md § Proxy](reference.md). **First try** `https://proxy.cflmy.top/github.com/cflmy/marqdo.git`. Retry; do not silently skip uploads. If needed: **HK jump** (`scripts/push-via-hk-jump.py`).
8. **Do not ship a full GitHub Release without the Linux extension/native zip** when the mode includes ext packs. Missing Linux packages = incomplete for that mode.
9. **Extension packs always publish to both** Cloudflare R2 (`https://ext.marqdo.com` via `scripts/upload-ext-r2.py`) **and** GitHub (Release assets / `ext-v*` tag). Credentials only in `~/.marqdo/r2.env` / CI — **never** commit. See [ext-cdn.md](../../../doc/design/ext-cdn.md).
10. **CLI Ubuntu package = PPA** (`ppa:cflmy/marqdo`) via `./scripts/ppa-ship.sh` / `ppa-build-source.sh`. GitHub zips remain for Windows and non-apt Linux.
11. **When a step needs the user** (sudo password, GPG passphrase, Launchpad UI, interactive `dput`): **stop, state what failed, and give the exact commands** to run in their terminal. Do not pretend the step succeeded.

## Release modes

Ask (or take from user message) which mode:

| Mode | Bumps | Git tag | GitHub assets | R2 CDN | Ubuntu PPA |
|------|-------|---------|---------------|--------|------------|
| **full** (default) | `Cargo.toml` + `ext/VERSION` (same SemVer) | `vX.Y.Z` | CLI + ext + native + VSIX + … | yes | yes |
| **cli** | `Cargo.toml` only | `vX.Y.Z` | CLI (+ optional attach prior ext) | no (unless user asks) | yes |
| **ext** | `ext/VERSION` only | `ext-vX.Y.Z` (or attach to latest CLI release) | ext.zip + native-*.zip | **required** | no |

Defaults after version+mode confirmed — **do not re-ask**:

| Setting | Default |
|---------|---------|
| Sync / bump `vscode-extension` to same `VER` | **yes** for full/cli; **no** for ext-only |
| Push `main`, create annotated tag, push tag (trigger CI) | **yes** for full/cli |
| Ext packs → R2 **and** GitHub | **yes** whenever ext ships |
| Ubuntu PPA upload after CLI tag | **yes** for full/cli |
| Release notes language | **中英双语** |
| Deploy gh-pages after release | **no** (only if user asks) |

Phase 0 ask example:

> 当前最新 CLI **vA.B.C**，扩展包 **E.F.G**。请指定下一个版本号与模式（`full` / `cli` / `ext`，如 `1.0.3 full`）。

## Phase 0 — Detect versions (mandatory stop)

```bash
python3 .cursor/skills/marqdo-release/scripts/detect_version.py
```

Report briefly, then **ask for SemVer + mode** (unless already given). Normalize `VER` / `TAG`.

## Phase 1 — Preflight

```
Release progress:
- [ ] Phase 0: version + mode confirmed
- [ ] Phase 1: preflight green
- [ ] Phase 2: docs + version bumps committed
- [ ] Phase 3: extension branch (full/cli)
- [ ] Phase 4: main pushed; tag created & pushed
- [ ] Phase 5: CI / GitHub assets / notes
- [ ] Phase 6: R2 upload (if ext ships)
- [ ] Phase 7: Ubuntu PPA (full/cli)
- [ ] Phase 8: post-release checks
```

Checks: clean tree; `main` ff-only; smoke tests; CHANGELOG Unreleased; no secrets.

## Phase 2 — Docs & version bumps (main)

| File | full | cli | ext |
|------|------|-----|-----|
| `Cargo.toml` + `crates/marqdo-wasm/Cargo.toml` | bump | bump | — |
| `ext/VERSION` | bump | — | bump |
| `CHANGELOG.md` | `## vVER` | `## vVER` | `## Ext pack vVER` |
| `README.md` / `public/**` install + download table | update | update | update ext notes |
| `.cursor/skills/marqdo/SKILL.md` | if surface changed | same | rare |
| `.github/workflows/release.yml` Highlights stub | yes | yes | — |

```bash
./scripts/build-public.sh   # when public/ sources changed
```

Commit + push `main` (proxy origin). On push failure → reference § Proxy; if still blocked, **print** for the user:

```bash
git push https://proxy.cflmy.top/github.com/cflmy/marqdo.git HEAD:main
# or: python3 scripts/push-via-hk-jump.py
```

## Phase 3 — VS Code / Cursor extension (full/cli only)

Default: bump `vscode-extension` package.json to `VER`, compile, push branch `vscode-extension`, return to `main`.

## Phase 4 — Tag & trigger CI (full/cli)

```bash
git tag -a "vVER" -m "vVER"
git push origin "vVER"
```

Ext-only:

```bash
git tag -a "ext-vVER" -m "ext-vVER"
git push origin "ext-vVER"
```

(or build zips locally / via workflow and `gh release upload`).

If tag push needs the user:

```bash
git push https://proxy.cflmy.top/github.com/cflmy/marqdo.git vVER
```

## Phase 5 — GitHub Release notes & assets

1. Wait for Actions (full/cli): both `windows` and `linux` green when ext natives required.
2. Rewrite release body 中英双语 — [reference.md § Notes](reference.md).
3. Confirm Linux `marqdo-VER-native-x86_64-unknown-linux-gnu.zip` when mode ships ext.

## Phase 6 — Extension CDN (R2) — required when ext ships

```bash
# credentials: ~/.marqdo/r2.env (never commit)
python3 scripts/upload-ext-r2.py --version VER --from-dir dist/
curl -fsS https://ext.marqdo.com/latest/VERSION
```

Also ensure the **same** zips are on GitHub (`vVER` or `ext-vVER`). If R2 credentials missing, **tell the user**:

```bash
# put keys in ~/.marqdo/r2.env then:
python3 scripts/upload-ext-r2.py --version VER --from-dir dist/
```

## Phase 7 — Ubuntu PPA (full/cli)

Linux CLI for apt users. After `Cargo.toml` is at `VER` and `debian/changelog` has a matching `VER-1ppaN` stanza:

```bash
./scripts/ppa-ship.sh noble
```

Packaging notes / noble Cargo.lock v4: [ubuntu-ppa.md](../../../doc/design/ubuntu-ppa.md).

**If the agent cannot sudo / sign / dput**, print exactly:

```bash
cd ~/work/marqdo
./scripts/ppa-ship.sh noble
# or build then upload:
./scripts/ppa-build-source.sh noble
dput ppa:cflmy/marqdo ../build-area/marqdo_*~noble_source.changes
```

Watch: https://launchpad.net/~cflmy/+archive/ubuntu/marqdo/+builds  
On Launchpad failure: read the build log, fix `debian/` or bundle script, bump `1ppaN`, re-upload. Tell the user the new `dput` command.

## Phase 8 — Post-release

1. `gh release view TAG` — assets + notes.
2. CDN `https://ext.marqdo.com/vVER/` (if ext shipped).
3. PPA build green (if cli shipped).
4. Tell user URLs: GitHub release, CDN, PPA install one-liner.
5. Optional gh-pages only if asked.

## Extension-only release (mode=ext)

1. User confirms **ext** SemVer (may differ from CLI).
2. Bump `ext/VERSION`; CHANGELOG → `## Ext pack vVER`.
3. Build/package `marqdo-VER-ext.zip` + `marqdo-VER-native-*.zip` (CI or local).
4. Upload to **R2 and GitHub** (required both).
5. Do **not** retag CLI / do **not** PPA unless user also asked for cli.

## Failure matrix (short)

| Failure | Action |
|---------|--------|
| `gh` / git TLS or Clash `7890` refused | Unset bad proxy; see [reference § Proxy](reference.md) |
| GitHub blocked | proxy.cflmy.top, then HK jump; **print commands** if agent cannot push |
| Tag exists | Stop; ask user |
| CI red | Fix on main; retag only if user allows |
| R2 upload fails | Print `upload-ext-r2.py` command; do not skip silently |
| PPA / sudo / GPG | Print `./scripts/ppa-ship.sh noble` (or `dput …`); do not skip silently |
| Launchpad `Cargo.lock` v4 / old cargo | Ensure bundled `third_party/rust` via current `ppa-build-source.sh`; bump `1ppaN` |
| Linux native zip missing (full/ext) | Incomplete — fix linux job or upload |

## Do not

- Ship without user-confirmed version (and mode when ambiguous).
- Ship ext packs to only one of R2 / GitHub.
- Call a full/cli release done without attempting PPA **or** giving the user the PPA commands.
- Commit `target/`, `dist/`, `*.wasm`, `third_party/rust/`, or `public/**/*.html` unless policy changes.
- Merge extension tree into `main`.
