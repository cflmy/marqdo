---
name: marqdo-release
description: >-
  Run a full Marqdo product release: detect latest version and ask the user for
  the next tag, update CHANGELOG/README/skills/public docs, bump Cargo versions,
  sync VS Code extension on branch vscode-extension when needed, build or
  trigger install assets (CLI, stdlib, ext, public zip, VSIX, wasm notes), push
  tag, publish GitHub Release with detailed notes, and recover from network
  failures via proxy. Use when the user asks to release, cut a version, publish
  vX.Y.Z, ship a GitHub release, 发版, 发布新版本, or tag marqdo.
---

# Marqdo release

Canonical release playbook for **cflmy/marqdo**. Read this skill **before** tagging or uploading assets. Details: [reference.md](reference.md).

## Hard rules

1. **Never invent the next version.** Detect current versions → **stop and ask the user** for the next SemVer (`X.Y.Z`). Do not proceed until they confirm.
2. **Never force-push** `main` / tags. **Never** `--no-verify` unless the user explicitly orders it.
3. **Never commit `vscode-extension/` on `main`** (gitignored). Extension source lives only on branch **`vscode-extension`**. See `doc/design/vscode-extension-commit.md`.
4. **Do not release from a dirty tree** (except intentional release commits you create in this flow).
5. **Tag format** is always `vX.Y.Z` matching root `Cargo.toml` `version = "X.Y.Z"`.
6. Prefer **tag push → GitHub Actions** (`.github/workflows/release.yml`) for Windows CLI + VSIX + zips. Local packaging is fallback / verification.
7. After network errors: apply [reference.md § Proxy](reference.md); retry; do not silently skip uploads.

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

Push `main`: `git push origin main` (with proxy/`all` perms if needed).

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

This runs `.github/workflows/release.yml` (Windows): CLI exe/zip, stdlib zip, ext zip, source zip, public zip, VSIX.

If tag push fails: [reference.md § Proxy](reference.md), retry once; then report.

## Phase 5 — Release notes & assets

1. Wait for Actions success: `gh run list --workflow=Release --limit 3` / open the run URL.
2. Ensure GitHub Release for `TAG` exists with assets (workflow uses `softprops/action-gh-release`).
3. **Rewrite release body** (workflow body is a stub) with **中英双语** — see [reference.md § Notes template](reference.md). `gh release edit TAG --notes-file …`.
4. Include: Highlights, Breaking, Install (`ext add`, plugins build), Downloads table, WASM (`marqdo wasm build`), links to CHANGELOG/README, extension branch note.
5. Optional Linux local extras (not required if CI green): `marqdo wasm build` smoke; document in notes that wasm is built from source via CLI.

Local Windows fallback: `scripts/release-full.ps1 -Tag TAG -Upload` (see script header).

## Phase 6 — Post-release

1. `gh release view TAG` — confirm assets + notes.
2. `git status` clean; `main` synced.
3. Tell user: release URL `https://github.com/cflmy/marqdo/releases/tag/TAG`.
4. Optional: `./scripts/deploy-public.ps1` / pages workflow if user wants user-site refresh.
5. Open fresh `## Unreleased` already done in Phase 2.

## Failure matrix (short)

| Failure | Action |
|---------|--------|
| `gh` / git TLS or proxy `7890` refused | Unset bad proxy; use working `HTTP(S)_PROXY` or `required_permissions: ["all"]`; see reference |
| Tag exists | Stop; ask user to bump or delete tag (no force on shared tags without explicit order) |
| CI red | Fix on main, move tag only if user explicitly allows delete+re-push tag |
| VSIX missing | Fetch `vscode-extension`, build locally, `gh release upload TAG dist/*.vsix` |
| Partial assets | Re-run workflow or upload missing files only |

## Do not

- Ship without user-confirmed version.
- Hardcode release highlights only in Actions YAML and forget CHANGELOG.
- Commit `target/`, `dist/`, `*.wasm`, or `public/**/*.html` unless project policy changes.
- Merge extension tree into `main`.
