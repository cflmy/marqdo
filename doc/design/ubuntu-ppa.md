# Ubuntu PPA: executable packaging and publication

The maintained entry is [the release handbook](../../skills/marqdo-release/index.mq.md). [ubuntu.mq.md](../../skills/marqdo-release/ubuntu.mq.md) provides publication and observation tasks; [package.mq.md](../../skills/marqdo-release/package.mq.md) owns native metadata and staged file preparation. Their source contracts replace a separate command playbook.

The PPA is ppa:cflmy/marqdo. Apt installs the interpreter; extensions remain independently installed by marqdo ext add. Upload acceptance alone is incomplete: the exact source must be published, its amd64 build must succeed and apt must expose the same Debian version.

## Exact version and artifact

| Value | Meaning |
|---|---|
| X.Y.Z | Cargo package.version and the authorized upstream |
| 1ppaN | Reviewed debian/changelog packaging revision |
| ~SERIES | Explicit Ubuntu target, normally noble |
| marqdo_X.Y.Z-1ppaN~SERIES_source.changes | Exact signed source artifact |
| ext/VERSION | Independent extension-pack version |

Native upstream and debian_version tasks parse the actual files. The latter retains the reviewed revision, removes its old series suffix and selects the current upstream/target series. A packaging-only reupload needs an explicit changelog revision edit. Upload selects the exact computed filename, never the newest unrelated package from a previous release or another series.

## Callable workflow

Run from the checkout root after the user has authorized this CLI/full release. Replace X.Y.Z with that version:
```sh
marqdo call skills/marqdo-release/ubuntu.mq.md ship 'version="X.Y.Z"' 'series="noble"' --require-ok --effects plugin,fs.read --fs-root . --json
marqdo call skills/marqdo-release/ubuntu.mq.md status 'version="X.Y.Z"' 'series="noble"' --require-ok --effects plugin,net --fs-root . --json
marqdo call skills/marqdo-release/index.mq.md finish 'version="X.Y.Z"' 'mode="full"' --require-ok --effects plugin,net,fs.write --fs-root . --json
```

Status is observational: inspect complete as well as Result.ok. Finish turns incomplete publication into a failed Result, allowing --require-ok to stop automation while keeping evidence. A green GitHub run does not establish Launchpad completion or apt availability.

## Native files and existing tools

Marqdo reads version metadata and writes the staged Cargo configuration, Debian changelog and vendor-checksum repairs. Native filesystem, TOML and JSON functions carry the data. No generated Python reader or writer is used. Vendor repair removes backup entries that Debian cleaning would delete while retaining package digests and ordinary source hashes.

The existing Debian boundary uses Cargo vendor, tar/xz, the configured Rust installation, debuild, GPG and dput. The source archive includes vendored crates and the selected Rust toolchain for an offline build. Its script chooses a fixed default toolchain and exposes the existing override; it does not assume the build host compiler is sufficient.

A new upstream may create an orig archive. Once Launchpad accepts that upstream, reuse the identical bytes and upload only the Debian revision. Never rebuild an accepted orig archive to work around a checksum rejection. A failed stage remains available for inspection.

## Recovery evidence

| Failure | Required action |
|---|---|
| sudo/GPG interaction unavailable | Complete the same concrete Debian boundary in the user's terminal, keeping version and series |
| Complete package version already used | Review and increment the Debian revision before another authorized upload |
| orig bytes differ | Retrieve and reuse the previously accepted archive |
| Offline vendor checksum failure | Inspect sanitize_vendor evidence and the retained staged tree |
| Source accepted, build pending/failed | Inspect the actual Launchpad build |
| Build green, apt metadata stale | Repeat apt observation for the exact version |

Public apt observation reads Packages.gz with native HTTP bytes and the bounded gzip boundary, then inspects actual package records. Isolated fixtures cover metadata selection, configuration writing and vendor-checksum preservation without uploading to a PPA.
