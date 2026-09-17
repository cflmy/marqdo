# Extension CDN (Cloudflare R2 → https://ext.marqdo.com)

| | |
|---|---|
| Status | **Accepted** — download prefer CDN; upload via `scripts/upload-ext-r2.py` |
| Date | 2026-09-17 |
| Related | [ext-cli.md](ext-cli.md) · [marqdo-release](../../.cursor/skills/marqdo-release/SKILL.md) |

## Why

CLI (语法解释器) and official **extension packs** (`ext/` L1 + native plugins) update on different cadences. Users behind unstable GitHub links should get packs from a stable object store first.

## Public URL layout

Bucket: `marqdo` · Custom domain: **https://ext.marqdo.com**

| Key | Meaning |
|-----|---------|
| `v{VER}/marqdo-{VER}-ext.zip` | L1 sources |
| `v{VER}/marqdo-{VER}-native-{triple}.zip` | Prebuilt plugins |
| `v{VER}/VERSION` | Pack SemVer echo |
| `latest/VERSION` | Recommended pack SemVer (plain text) |
| `latest/marqdo-…zip` | Same assets mirrored for convenience |

`{VER}` is the **extension pack** version (`ext/VERSION`), which **may differ** from CLI `Cargo.toml` version.

## Client download order (`marqdo ext add`)

1. `MARQDO_EXT_DOWNLOAD_BASE` (optional override)
2. **CDN** `https://ext.marqdo.com/v{VER}/…` (override host with `MARQDO_EXT_CDN`)
3. GitHub Releases
4. `https://proxy.cflmy.top/github.com/…`

Pack SemVer resolution (`release_version()`):

1. `MARQDO_EXT_VERSION`
2. CDN `GET …/latest/VERSION` (short timeout)
3. Embedded `ext/VERSION` at build time
4. CLI `CARGO_PKG_VERSION`

## Separate ext release (no CLI bump)

1. Bump `ext/VERSION` (e.g. `1.0.3`) — leave root `Cargo.toml` alone if CLI unchanged.
2. Build native zips (CI Linux/Windows jobs or local `scripts/build-web-plugin.sh` + cargo plugins).
3. Package `marqdo-{VER}-ext.zip` and `marqdo-{VER}-native-*.zip`.
4. Upload **both**:
   - R2: `python3 scripts/upload-ext-r2.py --version VER --from-dir dist/`
   - GitHub: attach to a GitHub Release (tag `ext-vVER` **or** reuse latest CLI release assets)
5. Confirm `https://ext.marqdo.com/latest/VERSION` and a sample zip URL return 200.

## Full product release (CLI + ext)

Keep `ext/VERSION` equal to (or compatible with) CLI SemVer unless documenting a deliberate skew. CI still publishes GitHub assets; **after** CI green, run `upload-ext-r2.py` so CDN is populated (or add a CI step with R2 secrets).

## Credentials

| Where | What |
|-------|------|
| Local | `~/.marqdo/r2.env` (`chmod 600`) — see `.env.r2.example` |
| CI (optional) | GitHub Actions secrets: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_ENDPOINT`, `R2_BUCKET` |

**Never** commit Account API tokens or S3 secret keys. If a token was pasted into chat, **rotate** it in the Cloudflare dashboard.

## Object ACL

Custom domain public reads require the bucket/objects to be publicly readable (R2 public bucket / r2.dev / custom domain). Uploads use S3 API credentials; downloads by `ext add` are anonymous HTTPS GETs.
