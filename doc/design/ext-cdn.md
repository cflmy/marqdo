# Extension CDN: native publication and byte parity

This page describes storage. The executable tasks are in [the release entry](../../skills/marqdo-release/index.mq.md), [assets.mq.md](../../skills/marqdo-release/assets.mq.md) and [asset_io.mq.md](../../skills/marqdo-release/asset_io.mq.md). Read the source contracts before calling them. Version, mode and publication authorization come from the user; inspecting an existing release grants no new remote write.

## Versioned packs

Extension versions come from ext/VERSION and can advance independently of the interpreter. An extension release uses its own ext-vX.Y.Z tag and three packs. A full release also includes the CLI, source, WASM and editor deliverables listed in model.mq.md.

| Object | Meaning |
|---|---|
| vX.Y.Z/marqdo-X.Y.Z-ext.zip | Readable extension sources |
| vX.Y.Z/marqdo-X.Y.Z-native-x86_64-pc-windows-msvc.zip | Windows plugins |
| vX.Y.Z/marqdo-X.Y.Z-native-x86_64-unknown-linux-gnu.zip | Linux plugins |
| vX.Y.Z/VERSION | Version echo for this pack set |
| latest/marqdo-X.Y.Z-*.zip | Identical current pack bytes |
| latest/VERSION | Recommended current extension version |

Public reads use https://ext.marqdo.com. [src/ext_cli.rs](../../src/ext_cli.rs) is authoritative for client override variables, version resolution and download fallback order. Ordinary installation does not need storage credentials.

## Callable publication

After GitHub contains every required asset, download to dist/release-X.Y.Z. Validation checks names, GitHub size/SHA256, archive CRCs, versions and native capabilities. Source-declared plugins require matching libraries on both native platforms. Independently valid ZIPs cannot hide a missing DLL or shared library.

Run from the checkout root with the version already chosen by the user; use the actual full or ext mode:
```sh
marqdo call skills/marqdo-release/assets.mq.md download 'version="X.Y.Z"' 'mode="ext"' --require-ok --effects plugin,net,fs.write --fs-root . --json
marqdo call skills/marqdo-release/assets.mq.md publish 'version="X.Y.Z"' 'mode="ext"' --require-ok --effects plugin,net,fs.write --fs-root . --json
marqdo call skills/marqdo-release/index.mq.md finish 'version="X.Y.Z"' 'mode="ext"' --require-ok --effects plugin,net,fs.write --fs-root . --json
```

Publication rechecks all three packs and their combined capability matrix before the first PUT. All three versioned ZIPs and three latest ZIPs are written before either VERSION pointer. A failed ZIP stops the sequence. Historical versions are retained.

Finish reads both public pointers and compares downloaded versioned/latest bytes with GitHub SHA256 values. HTTP 200 or one sample URL is insufficient. Edge propagation may leave verification incomplete; repeat observation rather than rebuilding unrelated archives or claiming completion from an upload response.

## Explicit boundaries

Filesystem operations, hashing, public HTTP downloads, release decisions and upload order are Marqdo code. The engineering extension supplies bounded ZIP inspection and private R2 Signature V4 transport. No Python uploader is part of this path; older standalone scripts remain compatibility utilities.

The native boundary resolves credentials privately from existing stores or environment. Missing credentials fail explicitly. Never put access keys or secrets in tool parameters, source or reports. A plugin grant has normal OS authority; --fs-root does not sandbox native plugins or child processes.

Isolated loopback tests use fake credentials to verify ZIP-before-VERSION ordering and failure stops. Read-only verification of the published 2.6.0 release has also exercised the actual GitHub/CDN parity path without republishing its assets.
