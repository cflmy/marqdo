# Ubuntu PPA packaging (Marqdo)

| | |
|---|---|
| Status | **Scaffold** — local `debian/` ready; first upload needs Launchpad PPA + vendored crates |
| Maintainer | cflmy \<pingan@cflmy.cn\> |
| GPG | `505943294D04C803` (fingerprint `52A1…C803`) |
| Suggested PPA | `ppa:cflmy/marqdo` |

## Goal

Users install with:

```bash
sudo add-apt-repository ppa:cflmy/marqdo
sudo apt update
sudo apt install marqdo
```

CLI and extension packs stay independent: the `.deb` ships the **interpreter**; `marqdo ext add …` still pulls from **https://ext.marqdo.com** (then GitHub / proxy).

## One-time Launchpad setup

1. Create PPA: https://launchpad.net/~cflmy/+activate-ppa → name e.g. `marqdo`
2. OpenPGP key already on Launchpad (done)
3. Sign Ubuntu Code of Conduct if Launchpad asks
4. On the build machine:

```bash
sudo apt install build-essential debhelper devscripts dput lintian \
  dh-cargo cargo rustc pkg-config
```

## Package layout

Debian packaging lives under repo-root `debian/` (this tree).

| File | Role |
|------|------|
| `debian/control` | Package metadata / deps |
| `debian/changelog` | Upload revisions (`dch`) |
| `debian/rules` | `dh` + cargo build |
| `debian/copyright` | Apache-2.0 |
| `debian/source/format` | `3.0 (quilt)` |
| `debian/watch` | Optional upstream tarball watch |
| `scripts/ppa-build-source.sh` | Vendor + `debuild -S` helper |

## Versioning

- Upstream SemVer: `Cargo.toml` / git tag `vX.Y.Z`
- Debian revision: `X.Y.Z-1ppa1~SERIES` (SERIES = `noble`, `jammy`, …)
- **Ext pack** SemVer (`ext/VERSION`) is **not** the deb version; bump deb only when CLI/stdlib packaging changes.

## Build source package (local)

```bash
# from repo root, clean tree preferred
./scripts/ppa-build-source.sh noble   # or jammy
# → ../build-area/marqdo_X.Y.Z-1ppa1~noble_source.changes
```

The script vendors Cargo crates into `vendor/` (Launchpad builders have **no network**).

## Upload

```bash
dput ppa:cflmy/marqdo ../build-area/marqdo_*_source.changes
```

Wait for https://launchpad.net/~cflmy/+archive/ubuntu/marqdo/+builds

## Series strategy

Start with **one** series you use daily (e.g. `noble` 24.04 or `resolute` 26.04). Copy the changelog stanza and rebuild per series, or use `backportpackage` later.

## Out of scope (v1 deb)

- Shipping `ext/native/*.so` inside the deb (use CDN `ext add`)
- WASM / VSIX
- Multi-arch cross builds beyond amd64

## Checklist for first upload

- [ ] PPA `marqdo` exists under `~cflmy`
- [ ] `./scripts/ppa-build-source.sh <series>` succeeds
- [ ] `lintian` on `.changes` is acceptable
- [ ] `dput` accepted; amd64 build green on Launchpad
- [ ] Fresh VM: `add-apt-repository` + `apt install marqdo` + `marqdo version`
