# Marqdo release — reference

Companion to [SKILL.md](SKILL.md). Use during a release; keep SKILL short.

## Detect versions (commands)

```bash
# From repo root
python3 .cursor/skills/marqdo-release/scripts/detect_version.py
# or manually:
git fetch --tags origin 2>/dev/null || true
echo "Cargo: $(sed -n 's/^version = \"\(.*\)\"/\1/p' Cargo.toml | head -1)"
echo "wasm:  $(sed -n 's/^version = \"\(.*\)\"/\1/p' crates/marqdo-wasm/Cargo.toml | head -1)"
echo "tags:  $(git tag -l 'v*' --sort=-v:refname | head -3 | tr '\n' ' ')"
gh release list --limit 3 2>/dev/null || true
```

**SemVer guidance for the ask (do not choose for the user):**

| Kind | When |
|------|------|
| patch `x.y.(z+1)` | Fixes, docs, no public API / MQ surface break |
| minor `x.(y+1).0` | New features (WASM route C, new ext APIs) backward compatible |
| major `(x+1).0.0` | Breaking MQ / CLI / plugin ABI |

## Asset matrix (CI tag `v*`)

Produced by `.github/workflows/release.yml` on **windows-latest**:

| Asset | Role |
|-------|------|
| `marqdo-VER-x86_64-pc-windows-msvc.exe` | CLI (stdlib embedded) |
| `marqdo-VER-x86_64-pc-windows-msvc.zip` | `marqdo.exe` + `lib/` |
| `marqdo-VER-stdlib.zip` | `lib/` only |
| `marqdo-VER-ext.zip` | Official `ext/` L1 |
| `marqdo-VER-native-x86_64-pc-windows-msvc.zip` | Prebuilt Windows plugins (`native/*.dll`) |
| `marqdo-VER-native-x86_64-unknown-linux-gnu.zip` | Prebuilt Linux plugins (`native/lib*.so`) |
| `marqdo-VER-source.zip` | `git archive` of tagged commit |
| `marqdo-VER.vsix` | VS Code / Cursor (from branch `vscode-extension`) |
| `marqdo-VER-public.zip` | Static user docs (`marqdo view output public`) |

Not CI-uploaded (document in notes): Linux/macOS build from source; WASM via `marqdo wasm build` → `dist/wasm/`.

## Doc sync checklist (Phase 2 detail)

### CHANGELOG.md

1. Ensure `## Unreleased` lists everything shipping.
2. Rename to `## vVER — YYYY-MM-DD` (use release day in Asia/Shanghai unless user says otherwise).
3. Add a short **Highlights** subsection (install snippet + 3–6 bullets).
4. Insert empty:

```markdown
## Unreleased

### Added

### Fixed

### Changed
```

### README.md

Update every user-visible version pin, typically:

- Section title `## 现状（vVER）`
- `### 如何使用最新 Marqdo（vVER）`
- `git checkout vVER`
- Release URL `…/releases/tag/vVER`

Keep install / `ext add` / plugin build instructions accurate for this release.

### `.cursor/skills/marqdo/SKILL.md`

- Sync hard rules / feature bullets that this release changes (e.g. WASM, web).
- Do **not** dump full CHANGELOG into the skill.
- If release-only process changed, point to `.cursor/skills/marqdo-release/`.

### `public/` (author Markdown sources)

User tutorials under `public/` (not generated HTML). Update when:

- New CLI (`marqdo wasm …`, `ext add`, `version`)
- New tutorials (browser hello, web client embed)
- Version badges / “requires vVER” lines

Then regenerate HTML for zip/pages:

```bash
./scripts/build-public.sh
# Windows: powershell -File ./scripts/build-public.ps1
```

HTML under `public/**/*.html` is typically gitignored — zip still ships from CI `marqdo view output public`.

### Workflow stub

Edit `.github/workflows/release.yml` `body:` **Highlights** so the first auto body matches this tag (agent still replaces with full notes via `gh release edit`).

### Roadmaps / ADRs

If this release closes a wave: set status Completed in `doc/roadmap/*`; leave ADRs immutable (link from CHANGELOG).

## Extension checklist (Phase 3 detail)

Branch policy: `doc/design/vscode-extension-commit.md`.

```bash
git fetch origin vscode-extension
git checkout vscode-extension
# edit vscode-extension/package.json version → VER
# npm ci && npm run compile
git add vscode-extension/
git commit -m "$(cat <<'EOF'
vscode-extension: bump to VER for release

EOF
)"
git push origin vscode-extension
git checkout main
```

Local VSIX only (fallback):

```bash
git fetch origin vscode-extension
git checkout origin/vscode-extension -- vscode-extension
(cd vscode-extension && npm ci && npm run compile && npx @vscode/vsce package --no-dependencies -o "../dist/marqdo-VER.vsix")
# cleanup ignored tree if desired
gh release upload "vVER" "dist/marqdo-VER.vsix" --clobber
```

## Defaults (mirror of SKILL)

| Setting | Default |
|---------|---------|
| Sync `vscode-extension` to `VER` | yes |
| Push main + annotated tag immediately | yes |
| Release notes | 中英双语 |
| Skip VSIX / docs-only / deploy pages | no |

Phase 0 asks **only** for the next version unless the user wants overrides.

## Notes template (中英双语 — default)

Save as `/tmp/marqdo-vVER-notes.md` then:

```bash
gh release edit "vVER" --notes-file /tmp/marqdo-vVER-notes.md
```

```markdown
## Marqdo vVER

### 亮点 / Highlights

- **中文** … / **EN** …
- …

### 安装 / Install

```bash
git clone https://github.com/cflmy/marqdo.git && cd marqdo
git checkout vVER
cargo build --release
./target/release/marqdo version

cargo build --release -p marqdo_plugin_web -p marqdo_plugin_agent -p marqdo_plugin_quantum -p marqdo_plugin_linalg
marqdo ext add web && marqdo ext add agent && marqdo ext add quantum
# 中文 id：网页 / 智能体 / 量子
```

**浏览器 WASM / Browser WASM**

```bash
marqdo wasm build
# → dist/wasm/marqdo_wasm_bg.wasm + marqdo-bridge.js
# examples: examples/browser-hello/
```

### 破坏性变更 / Breaking changes

- 无 / None
- …

### 下载 / Downloads

| Asset | 内容 / Contents |
|-------|-----------------|
| `marqdo-*-windows-msvc.exe` | CLI（内置 stdlib） |
| `marqdo-*-windows-msvc.zip` | exe + `lib/` |
| `marqdo-*-stdlib.zip` | 仅 `lib/` |
| `marqdo-*-ext.zip` | 官方 `ext/` |
| `marqdo-*-source.zip` | 源码快照 |
| `marqdo-*.vsix` | VS Code / Cursor 扩展 |
| `marqdo-*-public.zip` | 用户文档站 |

扩展源码分支 / Extension branch: **`vscode-extension`** ([policy](https://github.com/cflmy/marqdo/blob/main/doc/design/vscode-extension-commit.md)).

### 链接 / Links

- [CHANGELOG](https://github.com/cflmy/marqdo/blob/main/CHANGELOG.md)
- [README](https://github.com/cflmy/marqdo/blob/main/README.md)
```

## Proxy & auth (network recovery)

Symptoms seen in this project: broken local Clash on `127.0.0.1:7890`, flaky `proxy.cflmy.top` HTTPS (short GET OK, long `git push` / connect timeouts), TLS failures, `gh` needing token, Cursor agent sandbox with only `lo`.

### Diagnose

```bash
env | grep -iE 'proxy|PROXY' || true
curl -sI --max-time 15 https://github.com | head -5
curl -sS --max-time 15 -o /dev/null -w 'proxy=%{http_code}\n' https://proxy.cflmy.top/ || true
gh auth status
ip -br addr | head -5   # agent may lack enp*; use required_permissions: ["all"]
```

### Fix order

1. **Unset bad proxy** if connection refused:

```bash
unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy
# also clear for one git call: git -c http.proxy= -c https.proxy= …
```

2. **Set a working proxy** only if the user provides one (or known working):

```bash
export HTTPS_PROXY=http://HOST:PORT
export HTTP_PROXY=http://HOST:PORT
```

3. **Agent Shell**: retry `git push` / `gh` with `required_permissions: ["all"]` (and `full_network` if needed).

4. **`gh` auth**: if `gh auth status` fails, ask user to `gh auth login` or set `GH_TOKEN` / `GITHUB_TOKEN` for the session — **never commit tokens**. Fallback used in this repo: load password from `git credential fill` (`protocol=https` / `host=github.com`) into `GH_TOKEN` for the session only, then delete the temp env file.

5. **Git remote**: prefer `https://github.com/cflmy/marqdo.git`; SSH needs keys.

6. **HK SSH jump (preferred when GitHub is blocked but HK is reachable)** — see next subsection.

7. **Partial upload**:

```bash
gh release upload "vVER" dist/FILE --clobber
```

8. **Re-run CI** without retagging (if tag already correct):

```bash
gh workflow run Release --ref "vVER"  # only if workflow_dispatch exists; else fix + new patch tag
```

Current Release workflow is **tag-push only**. Red CI after tag → fix on `main`, then either new patch tag or **user-approved** delete+recreate tag (dangerous; prefer patch).

### HK SSH jump (Paramiko CONNECT)

**When to use:** local → GitHub fails (timeout / TLS / Clash flaky / reverse-proxy `proxy.cflmy.top` connect drops), but this host can reach an overseas box that has stable GitHub (default `hk.cflmy.de`). Proven path for v0.3.9 push/tag/`gh`.

**Do not** rely on OpenSSH + `sshpass` here — password auth has hung; use **Paramiko** (`python3-paramiko`).

**Never** commit the HK root password or tokens. Ask the user for the password (or a key), put it in env / a `600` temp file, scrub after.

```bash
# one-shot password file (example — use the secret the user gave this session)
printf '%s' "$HK_PASSWORD_FROM_USER" > /tmp/.hk_ssh_pw && chmod 600 /tmp/.hk_ssh_pw

export HK_SSH_HOST=hk.cflmy.de   # or 83.229.126.119
export HK_SSH_USER=root
export HK_SSH_PASSWORD_FILE=/tmp/.hk_ssh_pw
# optional: HK_JUMP_PORT=18081

SCRIPT=.cursor/skills/marqdo-release/scripts/push-via-hk-jump.py

# push / tag / extension (git gets CONNECT proxy + github.com askpass)
python3 "$SCRIPT" -- git push origin main
python3 "$SCRIPT" -- git push origin vscode-extension
python3 "$SCRIPT" -- git push origin vVER

# gh (sets GH_TOKEN from git credential fill if unset)
python3 "$SCRIPT" -- gh release edit vVER --notes-file /tmp/notes.md
python3 "$SCRIPT" -- gh run list --workflow=Release --limit 3

# leave proxy up for manual commands
python3 "$SCRIPT" --serve   # then: export https_proxy=http://127.0.0.1:18081

rm -f /tmp/.hk_ssh_pw
```

Script behavior: SSH to HK → local `http://127.0.0.1:$HK_JUMP_PORT` HTTP CONNECT via `direct-tcpip` → run `git`/`gh`/`curl` with that proxy. GitHub HTTPS credentials still come from the **local** `git credential` store (askpass), not from the jump host.

Optional path reverse-proxy (`https://proxy.cflmy.top/github.com/…` `insteadOf`) may work for short GETs; treat long `git push` failures as a signal to use the HK jump instead of retrying Clash forever.
## Local Windows fallback

```powershell
powershell -File ./scripts/release-full.ps1 -Tag vVER -Upload
# -SkipVsix / -SkipPublic / -DeployPages as needed
```

Also: `scripts/release-windows.ps1`, `scripts/publish-release.py` if present.

## Commit message style

```text
release: vVER — <highlight>

```

Extension:

```text
vscode-extension: bump to VER for release

```

## Pre-tag test suggestions

Minimum:

```bash
cargo test --test gold structure_hello
./target/release/marqdo version   # after cargo build --release
```

Stronger (time permitting):

```bash
cargo test --test gold
cargo test -p marqdo-wasm -- --nocapture   # if wasm tests exist
# optional Node: node tests/wasm/smoke.mjs after marqdo wasm build
```

## Ask-user prompt (copy)

```text
版本探测结果：
- Cargo.toml: A.B.C
- crates/marqdo-wasm: …
- 最新 git tag: v…
- 最新 GitHub Release: v…
- CHANGELOG Unreleased: 有 / 无（摘要：…）

请指定下一个版本号（例如 0.3.2）。
默认（无需确认）：同步 vscode-extension 同号、立刻 push tag 发版、发布说明中英双语。
若要改默认请一并说明。
```
