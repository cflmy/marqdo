---
markup: occlusion
---

Optional ZIP tasks: native defaults include ZIP; minimal/browser builds report unavailable. Returns Result.

ZIP tasks are optional native capabilities, with no standalone plugin to install.
The pack/unpack contracts below define both operations. Both return
Result and require fs.write. Source and destination paths stay under the granted
root; output must not already exist. Extraction preflights paths, duplicate
entries, symlinks and the uncompressed byte limit before creating its directory.
Failed extraction removes only the newly-created destination. Minimal/browser
builds report unavailable; they never pretend to have written an archive.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Relative IO paths use --fs-root, or the called file directory when omitted. Writes require explicit fs.write grants, subprocesses require process.exec, and pure transforms require no IO grant.

Implementation: [src/host/archive.rs](../src/host/archive.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## pack

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path; archive packing expects a directory |
| dest | text | required | Destination path; overwrite is allowed only where documented |
| max_bytes | number | 1073741824 | Maximum bytes read or expanded; exceeding it fails |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Result containing the count of archived regular files, or a structured error |

Use `src`, `dest`, `max_bytes`=1073741824 as the inputs for this operation.

Effects: fs.write.

For this example, set MQ_WORK to an existing disposable directory with regular files in an input subdirectory. bundle.zip and output must not exist. Keep or clean the produced artifacts explicitly.

```sh
marqdo call lib/archive.mq.md pack 'src="input"' 'dest="bundle.zip"' --fs-root "$MQ_WORK" --effects fs.write --json
marqdo call lib/archive.mq.md unpack 'src="bundle.zip"' 'dest="output"' --fs-root "$MQ_WORK" --effects fs.write --json
```

Destination must not exist. Caps total uncompressed bytes and 100000 entries; rejects symlinks and escaping paths. Pack includes regular files; failed unpack removes only its newly-created destination.

Return *[Pack regular files into ZIP without overwriting an existing destination](> host_zip_pack src=`src` dest=`dest` max_bytes=`max_bytes`)*.

## unpack

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path; archive packing expects a directory |
| dest | text | required | Destination path; overwrite is allowed only where documented |
| max_bytes | number | 1073741824 | Maximum bytes read or expanded; exceeding it fails |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Result containing the extracted entry count, or a structured error |

Use `src`, `dest`, `max_bytes`=1073741824 as the inputs for this operation.

Effects: fs.write.

Destination must not exist. Caps total uncompressed bytes and 100000 entries; rejects symlinks and escaping paths. Pack includes regular files; failed unpack removes only its newly-created destination.

Return *[Validate and extract ZIP entries without escaping the root or overwriting a destination](> host_zip_unpack src=`src` dest=`dest` max_bytes=`max_bytes`)*.
