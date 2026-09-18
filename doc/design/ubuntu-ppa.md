# Ubuntu PPA packaging (Marqdo)

| | |
|---|---|
| Status | **Active** — `ppa:cflmy/marqdo`; noble builds need bundled Rust (Cargo.lock v4) |
| Maintainer | cflmy \<pingan@cflmy.cn\> |
| GPG | `505943294D04C803` (fingerprint `52A1…C803`) |
| PPA | `ppa:cflmy/marqdo` |

## Goal

Users install with:

```bash
sudo add-apt-repository ppa:cflmy/marqdo
sudo apt update
sudo apt install marqdo
```

CLI and extension packs stay independent: the `.deb` ships the **interpreter**; `marqdo ext add …` still pulls from **https://ext.marqdo.com** (then GitHub / proxy).

## Why bundled Rust?

Ubuntu **noble** ships cargo/rustc **1.75**, which cannot parse **Cargo.lock version 4** (`lock file version 4 requires -Znext-lockfile-bump`). Launchpad builders also have **no network**, so `ppa-build-source.sh` vendors crates **and** installs a modern toolchain into `third_party/rust` inside the `.orig.tar.xz`. `debian/rules` prefers that toolchain over distro cargo.

## One-time Launchpad setup

1. Create PPA: https://launchpad.net/~cflmy/+activate-ppa → name e.g. `marqdo`
2. OpenPGP key already on Launchpad (done)
3. Sign Ubuntu Code of Conduct if Launchpad asks
4. Enable the Ubuntu **series** you upload for (e.g. noble) under PPA → Change details
5. On the build machine:

```bash
sudo apt install build-essential debhelper devscripts dput lintian \
  dpkg-dev quilt cargo rustc pkg-config curl xz-utils
```

## Package layout

| File | Role |
|------|------|
| `debian/control` | Metadata (amd64; no distro cargo BD) |
| `debian/changelog` | Upload revisions (`1ppaN`) |
| `debian/rules` | `dh` + bundled/system cargo build |
| `scripts/ppa-build-source.sh` | Vendor + bundle Rust + `debuild -S` |
| `scripts/ppa-ship.sh` | apt deps + build + `dput` |

## Versioning

- Upstream SemVer: `Cargo.toml` / git tag `vX.Y.Z`
- Debian revision: `X.Y.Z-1ppaN~SERIES` (SERIES = `noble`, …) — bump `1ppaN` on packaging-only reuploads
- **Ext pack** SemVer (`ext/VERSION`) is **not** the deb version

## One-shot (recommended)

```bash
cd ~/work/marqdo
./scripts/ppa-ship.sh noble          # tools → debuild -S → dput
./scripts/ppa-ship.sh noble -n       # build only
```

If automation cannot enter sudo/GPG, tell the user these commands (do not invent alternatives):

```bash
./scripts/ppa-build-source.sh noble
dput ppa:cflmy/marqdo ../build-area/marqdo_*~noble_source.changes
```

Watch: https://launchpad.net/~cflmy/+archive/ubuntu/marqdo/+builds

## Failure notes

| Symptom | Fix |
|---------|-----|
| `lock file version 4 requires -Znext-lockfile-bump` | Rebuild with current `ppa-build-source.sh` (bundles Rust); bump `1ppaN` |
| `failed to calculate checksum of: …/vendor/…/Cargo.toml.orig` | `dh_clean` purged `*.orig`; fixed via `dh_clean -Xvendor/` (+ strip `.orig` only when **creating a new** orig); bump `1ppaN` with **`-sd`** |
| `orig.tar.xz already exists … different contents` | Never rewrite an uploaded upstream `.orig`; reuse PPA file (`ppa-build-source.sh` downloads it) and `debuild -S -sd` |
| Same version rejected by Launchpad | Bump `debian/changelog` (`1ppa2`, …) |
| Wrong series | Enable series on PPA; rebuild with that series name |

## Out of scope (v1 deb)

- Shipping `ext/native/*.so` inside the deb (use CDN `ext add`)
- WASM / VSIX / non-amd64
