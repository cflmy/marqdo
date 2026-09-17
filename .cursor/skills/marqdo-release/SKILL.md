---
name: marqdo-release
description: >-
  Run a full Marqdo product release: detect latest version and ask the user for
  the next tag, update CHANGELOG/README/skills/public docs, bump Cargo versions,
  sync VS Code extension on branch vscode-extension when needed, build or
  trigger install assets (Windows CLI + **required Linux native/CLI zips**,
  stdlib, ext, public zip, VSIX), push via origin or
  https://proxy.cflmy.top/github.com/cflmy/marqdo.git, publish GitHub Release
  with detailed notes, and recover from network failures via that proxy or HK
  SSH jump (scripts/push-via-hk-jump.py). Use when the user asks to release,
  cut a version, publish vX.Y.Z, ship a GitHub release, 发版, 发布新版本, or tag
  marqdo.
---

# Marqdo release

Canonical release playbook for **cflmy/marqdo**. Read this skill **before** tagging or uploading assets. Details: [reference.md](reference.md).

## Hard rules

1. **Never invent the next version.** Detect current versions → **stop and ask the user** for the next SemVer (`X.Y.Z`). Do not proceed until they confirm.
2. **Never force-push** `main` / tags. **Never** `--no-verify` unless the user explicitly orders it.
3. **Never commit `vscode-extension/` on `main`** (gitignored). Extension source lives only on branch **`vscode-extension`**. See `doc/design/vscode-extension-commit.md`.
4. **Do not release from a dirty tree** (except intentional release commits you create in this flow).
5. **Tag format** is always `vX.Y.Z` matching root `Cargo.toml` `version = "X.Y.Z"`.
6. Prefer **tag push → GitHub Actions** (`.github/workflows/release.yml`) for **both** Windows and **Linux** install assets. Local packaging is fallback / verification.
7. After network errors: apply [reference.md § Proxy](reference.md). **First try** `https://proxy.cflmy.top/github.com/cflmy/marqdo.git` (this clone’s usual `origin`). Retry; do not silently skip uploads. If the reverse-proxy still drops long pushes, use **HK jump** (`hk.cflmy.de`) via `scripts/push-via-hk-jump.py` — see reference § **Proxy** / **HK SSH jump**.
8. **Do not ship a GitHub Release without the Linux extension/native zip.** Job `linux` must attach `marqdo-VER-native-x86_64-unknown-linux-gnu.zip` (and the Linux CLI bundle). Missing Linux packages = release incomplete.
9. **After publishing GitHub assets that include ext/native zips, upload the same packs to Cloudflare R2** (`https://ext.marqdo.com`) via `scripts/upload-ext-r2.py` so `marqdo ext add` prefers CDN. Credentials live only in `~/.marqdo/r2.env` / CI secrets — **never** commit. See [ext-cdn.md](../../../doc/design/ext-cdn.md).
10. **CLI and extension packs may version independently** (`ext/VERSION` / CDN `latest/VERSION` vs `Cargo.toml`). Ext-only releases use tag `ext-vX.Y.Z` (or attach assets without bumping CLI).

## Defaults (after version is confirmed)

Unless the user overrides, apply these automatically — **do not re-ask**:

| Setting | Default |
|---------|---------|
| Sync / bump `vscode-extension` to same `VER` | **yes** |
| Push `main`, create annotated tag, push tag immediately (trigger CI) | **yes** |
| Release notes language | **中英双语** (Chinese first, then English section, or paired ZH/EN under each heading) |
| Skip VSIX | **no** |
| Docs-only release | **no** (full asset pipeline via CI) |
| Deploy gh-pages after release | **no** (only if user asks) |

Phase 0 asks **only** for the next version (and optional overrides). Example ask:

> 当前最新是 **vA.B.C**。请指定下一个版本号（如 `0.3.2`）。其余按默认：同步扩展、立刻打 tag 发布、中英双语说明。

## Phase 0 — Detect versions (mandatory stop)

Run in repo root (parallel OK):

```bash
python3 .cursor/skills/marqdo-release/scripts/detect_version.py
# or: rg / git tag / gh release list — see reference.md
```

Report briefly, then **ask only for the next SemVer** (defaults above). Wait for confirmation. Normalize to `VER=X.Y.Z` and `TAG=vX.Y.Z`.

## Phase 1 — Preflight

Copy and track:

```
Release progress:
- [ ] Phase 0: version confirmed by user (TAG=…)
- [ ] Phase 1: preflight green
- [ ] Phase 2: docs + version bumps committed
- [ ] Phase 3: extension branch (if needed)
- [ ] Phase 4: main pushed; tag created & pushed
- [ ] Phase 5: CI / assets / release notes verified
- [ ] Phase 6: post-release checks
```

Checks:

1. `git status` clean or only release WIP you control.
2. `git checkout main && git pull --ff-only origin main`.
3. Quick gate: `cargo test --test gold structure_hello` (and any release-critical tests the user named). Full gold suite if time allows.
4. `CHANGELOG.md` has meaningful **Unreleased** notes; if empty, draft from `git log vLAST..HEAD --oneline` and confirm with user.
5. Confirm no secrets in the commit (`*.env`, credentials).

## Phase 2 — Docs & version bumps (main)

Bump **in lockstep**:

| File | Change |
|------|--------|
| `Cargo.toml` | `version = "VER"` |
| `crates/marqdo-wasm/Cargo.toml` | same `VER` |
| `CHANGELOG.md` | Move `## Unreleased` → `## vVER — YYYY-MM-DD`; leave fresh empty `## Unreleased` |
| `README.md` | 「现状 / 如何使用最新」version strings, checkout tag, release URL |
| `.cursor/skills/marqdo/SKILL.md` | Version-sensitive bullets if any (WASM/web status); keep rules accurate |
| `public/**` | User-facing tutorials if features shipped (welcome, features/*, extensions). Prefer small factual updates over rewrites |
| `doc/roadmap/*.md` | Mark shipped waves done if this release closes them |
| `.github/workflows/release.yml` | Update **Highlights** body template to this release (CI still uses hardcoded blurb—keep in sync) |

Public HTML: regenerate before packaging / pages deploy:

```bash
./scripts/build-public.sh
# outputs under public/ (gitignored HTML); used by release zip / gh-pages
```

Commit on `main` (HEREDOC message), e.g.:

```text
release: vVER — <one-line highlight>

EOF
```

Push `main` (this repo’s `origin` is usually the reverse proxy):

```bash
# origin typically: https://proxy.cflmy.top/github.com/cflmy/marqdo.git
git push origin main
# one-shot if origin still points at github.com:
git push https://proxy.cflmy.top/github.com/cflmy/marqdo.git HEAD:main
```

If that times out: [reference § HK SSH jump](reference.md).

## Phase 3 — VS Code / Cursor extension

**Default: always** bump extension to `VER` on branch `vscode-extension` (skip only if user said so).

1. `git fetch origin vscode-extension`
2. Work on `vscode-extension` branch (not main):
   - Bump `vscode-extension/package.json` `version` to `VER`.
   - Update `engines` / CLI version hints in extension README if present.
   - `npm ci && npm run compile`
3. Commit & `git push origin vscode-extension`.
4. Return to `main`. **Do not** add `vscode-extension/` to main.

## Phase 4 — Tag & trigger CI

```bash
git tag -a "TAG" -m "TAG"
git push origin "TAG"
```

This runs `.github/workflows/release.yml`:

- **windows** — CLI exe/zip, stdlib zip, ext zip, source zip, public zip, VSIX, Windows native zip
- **linux** (`needs: windows`) — **required** Linux native plugins zip + Linux CLI bundle (`ext/` + `native/*.so`)

If tag push fails: [reference.md § Proxy](reference.md) (`proxy.cflmy.top` first), retry once; then HK jump.

## Phase 5 — Release notes & assets

1. Wait for Actions success: `gh run list --workflow=Release --limit 3` / open the run URL. **Both** `windows` and `linux` jobs must be green.
2. Ensure GitHub Release for `TAG` exists with assets (workflow uses `softprops/action-gh-release`).
3. Confirm Linux assets exist: `marqdo-VER-native-x86_64-unknown-linux-gnu.zip` and `marqdo-VER-x86_64-unknown-linux-gnu.zip`. If missing, do **not** call the release done — re-run / upload.
4. **Rewrite release body** (workflow body is a stub) with **中英双语** — see [reference.md § Notes template](reference.md). `gh release edit TAG --notes-file …`.
4. Include: Highlights, Breaking, Install (`ext add`, plugins build), Downloads table, WASM (`marqdo wasm build`), links to CHANGELOG/README, extension branch note.
5. Optional Linux local extras (not required if CI green): `marqdo wasm build` smoke; document in notes that wasm is built from source via CLI.

Local Windows fallback: `scripts/release-full.ps1 -Tag TAG -Upload` (see script header).

## Phase 6 — Post-release

1. `gh release view TAG` — confirm assets + notes.
2. **Upload extension packs to R2 CDN** (required when shipping ext/native zips):
   ```bash
   # credentials: ~/.marqdo/r2.env (never commit) — see doc/design/ext-cdn.md
   python3 -m pip install --user boto3
   python3 scripts/upload-ext-r2.py --version VER --from-dir dist/
   # verify: curl -fsS https://ext.marqdo.com/latest/VERSION
   ```
   Prefer downloading CI artifacts into `dist/` if local build skipped Linux/Windows natives.
3. `git status` clean; `main` synced.
4. Tell user: release URL `https://github.com/cflmy/marqdo/releases/tag/TAG` **and** CDN `https://ext.marqdo.com/vVER/`.
5. Optional: `./scripts/deploy-public.ps1` / pages workflow if user wants user-site refresh.
6. Open fresh `## Unreleased` already done in Phase 2.

## Extension-only release (no CLI bump)

When only `ext/` / native plugins change:

1. Ask user for **ext pack** SemVer (may differ from `Cargo.toml`).
2. Bump `ext/VERSION`; update CHANGELOG Unreleased → `## Ext pack vVER`.
3. Build/package `marqdo-VER-ext.zip` + `marqdo-VER-native-*.zip` (CI or local).
4. Upload to **R2** (`upload-ext-r2.py`) **and** GitHub (tag `ext-vVER` or attach to latest CLI release).
5. Do **not** retag CLI `vX.Y.Z` unless Cargo also bumps.
6. Detail: [ext-cdn.md](../../../doc/design/ext-cdn.md).

## Failure matrix (short)

| Failure | Action |
|---------|--------|
| `gh` / git TLS or Clash `7890` refused | Unset bad proxy; `required_permissions: ["all"]`; see [reference § Proxy](reference.md) |
| GitHub blocked | **`origin` via proxy.cflmy.top** (see below), then HK jump |
| Tag exists | Stop; ask user to bump or delete tag (no force on shared tags without explicit order) |
| CI red | Fix on main, move tag only if user explicitly allows delete+re-push tag |
| VSIX missing | Fetch `vscode-extension`, build locally, `gh release upload TAG dist/*.vsix` |
| Linux native/CLI zip missing | Incomplete release — fix `linux` job or `gh release upload`; do not skip |

## Do not

- Ship without user-confirmed version.
- Hardcode release highlights only in Actions YAML and forget CHANGELOG.
- Commit `target/`, `dist/`, `*.wasm`, or `public/**/*.html` unless project policy changes.
- Merge extension tree into `main`.
