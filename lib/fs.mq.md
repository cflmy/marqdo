---
markup: occlusion
import:
  json: lib/json.mq.md
  csv: lib/csv.mq.md
title: lib/fs
description: English filesystem wrappers
---

English filesystem wrappers

Read, write, find and organize files under an explicit root.

Behavior and limits: Legacy remove recursively deletes directories. Prefer remove_file/remove_empty_dir. Atomic operations replace one file; directory batches are not transactions. Native-only operations state their CoW boundary.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Relative IO paths use --fs-root, or the called file directory when omitted. Writes require explicit fs.write grants, subprocesses require process.exec, and pure transforms require no IO grant.

Implementation: [src/host/fs.rs](../src/host/fs.rs)、[src/host/fs_ops.rs](../src/host/fs_ops.rs)、[src/host/sandbox_fs.rs](../src/host/sandbox_fs.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## read_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Read UTF-8 text from path |

Use `path` as the inputs for this operation.

Effects: fs.read.

Read UTF-8 text from path.

Return *[Read UTF-8 text from path](> host_read_text path=`path`)*.

## write_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `text` as the inputs for this operation.

Effects: fs.write.

Write text to path (overwrite).

Return *[Write text to path (overwrite)](> host_write_text path=`path` text=`text`)*.

## append_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `text` as the inputs for this operation.

Effects: fs.write.

Append text to path.

Return *[Append text to path](> host_append_text path=`path` text=`text`)*.

## exists

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether path exists |

Use `path` as the inputs for this operation.

Effects: fs.read.

Whether path exists.

Return *[Whether path exists](> host_exists path=`path`)*.

## list_dir

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | List directory entries at path |

Use `path` as the inputs for this operation.

Effects: fs.read.

List directory entries at path.

Return *[List directory entries at path](> host_list_dir path=`path`)*.

## make_dir

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Create directory at path.

Return *[Create directory at path](> host_make_dir path=`path`)*.

## make_dirs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Create directories recursively (`mkdir -p`).

Return *[Create directories recursively (mkdir -p)](> host_make_dirs path=`path`)*.

## remove

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Remove a file or recursively remove a directory. Legacy behavior; use remove_file or remove_empty_dir for narrow deletion.

Remove a file or recursively remove a directory. For narrow deletion use remove_file or remove_empty_dir.

Return *[Remove a file or recursively remove a directory](> host_remove path=`path`)*.

## remove_tree

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Remove a directory recursively. Errors if `path` is a file.

Return *[Remove a directory recursively](> host_remove_tree path=`path`)*.

## stat

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Returns `{size, mtime_unix, is_file, is_dir}` |

Use `path` as the inputs for this operation.

Effects: fs.read.

Returns `{size, mtime_unix, is_file, is_dir}`.

Return *[Returns {size, mtime_unix, is_file, is_dir}](> host_stat path=`path`)*.

## walk

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Depth-first listing of relative paths under `path` (forward slashes) |

Use `path` as the inputs for this operation.

Effects: fs.read.

Depth-first listing of relative paths under `path` (forward slashes).

Return *[Depth-first listing of relative paths under path (forward slashes)](> host_walk path=`path`)*.

## copy_file

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path under the granted root |
| dest | text | required | Destination path under the granted root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `src`, `dest` as the inputs for this operation.

Effects: fs.write.

Copy a file (overwrites `dest` if it exists). Paths are sandboxed like other fs ops.

Return *[Copy a file (overwrites dest if it exists)](> host_copy_file src=`src` dest=`dest`)*.

## move

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path under the granted root |
| dest | text | required | Destination path under the granted root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `src`, `dest` as the inputs for this operation.

Effects: fs.write.

Rename / move within the sandbox.

Return *[Rename / move within the sandbox](> host_move src=`src` dest=`dest`)*.

## make_temp

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prefix | any | None | Prefix added to the generated name or tested against text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Create an empty temp file under the program directory; return its relative path |

Use `prefix`=None as the inputs for this operation.

Effects: fs.write.

Create an empty temp file under the program directory; return its relative path.

Return *[Create an empty temp file under the program directory; return its relative path](> host_make_temp prefix=`prefix`)*.

## text_patch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |
| find | any | required | Exact text to locate before replacement |
| replace | any | required | Replacement text for each exact match |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Exact FIND→REPLACE once in a UTF-8 text file |

Use `path`, `find`, `replace` as the inputs for this operation.

Effects: fs.write.

Exact FIND→REPLACE once in a UTF-8 text file. `find` must match exactly once.

Return *[Exact FIND→REPLACE once in a UTF-8 text file](> host_text_patch path=`path` find=`find` replace=`replace`)*.

## apply_patch_blocks

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |
| text | text | required | Text supplied by the caller |
| soft | bool | False | Whether a missing patch match becomes a recoverable outcome |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply plan-style triple-angle FIND/REPLACE blocks from `text` to `path` |

Use `path`, `text`, `soft`=False as the inputs for this operation.

Effects: fs.write.

Apply plan-style triple-angle FIND/REPLACE blocks from `text` to `path`.
Also accepts fenced find/replace pairs and minimal Begin Patch hunks.
When `soft=True`, only “FIND not found” returns `0` (run continues). Empty FIND, multi-match, parse errors, and whole-file FIND still abort — whole-file rewrite is forbidden.

Return *[Apply plan-style triple-angle FIND/REPLACE blocks from text to path](> host_apply_patch_blocks path=`path` text=`text` soft=`soft`)*.

## try_read_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Read text as {ok,value,error}; file and permission failures are data |

Use `path` as the inputs for this operation.

Effects: fs.read.

Call this function directly:

```sh
marqdo call lib/fs.mq.md try_read_text 'path="lib/fs.mq.md"' --effects fs.read --fs-root . --json
```

Read at most 8 MiB of UTF-8 text as {ok,value,error}; file, encoding and permission failures are data. The function still requires fs.read.

Return *[Read at most 8 MiB of UTF-8 text as {ok,value,error}; file, encoding and permission failures are data](> host_fs_try_read_text path=`path`)*.

## try_write_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Overwrite UTF-8 text and return {ok,value,error}; requires fs |

Use `path`, `text` as the inputs for this operation.

Effects: fs.write.

Overwrite UTF-8 text and return {ok,value,error}; requires fs.write.

Return *[Overwrite UTF-8 text and return {ok,value,error}; requires fs.write](> host_fs_try_write_text path=`path` text=`text`)*.

## read_bytes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| max_bytes | number | 8388608 | Maximum bytes read or expanded; exceeding it fails |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Read binary data as a bytes record |

Use `path`, `max_bytes`=8388608 as the inputs for this operation.

Effects: fs.read.

Read binary data as a bytes record. Default limit is 8 MiB, maximum is 64 MiB; decoding is explicit via lib/bytes.

Return *[Read binary data as a bytes record](> host_fs_read_bytes path=`path` max_bytes=`max_bytes`)*.

## write_bytes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| data | map | required | Portable bytes record containing _type=bytes and base64 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `data` as the inputs for this operation.

Effects: fs.write.

Atomically replace a file with bytes; no UTF-8 conversion. Requires fs.write.

Return *[Atomically replace a file with bytes; no UTF-8 conversion](> host_fs_write_bytes path=`path` data=`data`)*.

## read_chunk

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| offset | number | 0 | Zero-based byte offset of the next chunk |
| size | number | 65536 | Number of bytes to read in this chunk |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Read at most size bytes from offset, returning data/next_offset/eof |

Use `path`, `offset`=0, `size`=65536 as the inputs for this operation.

Effects: fs.read.

Read at most size bytes from offset, returning data/next_offset/eof. Offsets are zero-based bytes; each call closes the file, so no handles escape sessions.

Return *[Read at most size bytes from offset, returning data/next_offset/eof](> host_fs_read_chunk path=`path` offset=`offset` size=`size`)*.

## append_bytes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| data | map | required | Portable bytes record containing _type=bytes and base64 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `data` as the inputs for this operation.

Effects: fs.write.

Append bytes, creating a file if absent; parent directory must exist.

Return *[Append bytes, creating a file if absent; parent directory must exist](> host_fs_append_bytes path=`path` data=`data`)*.

## atomic_write

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `text` as the inputs for this operation.

Effects: fs.write.

Replace text through a reserved sibling temporary file and rename; failures remove that temp file. This is one-file replacement, not a multi-file transaction.

Return *[Replace text through a reserved sibling temporary file and rename; failures remove that temp file](> host_fs_atomic_write path=`path` text=`text`)*.

## write_changed

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | bool | Write atomically only when content differs; return whether it was written |

Use `path`, `text` as the inputs for this operation.

Effects: fs.write.

Write atomically only when content differs; return whether it was written.

Return *[Write atomically only when content differs; return whether it was written](> host_fs_write_changed path=`path` text=`text`)*.

## write_checked

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| text | text | required | Text to read, transform or emit as described by this operation |
| expected_sha256 | text | required | Digest of the exact prior text; None disables the check where allowed |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Check current SHA-256 before atomic replacement, returning Result |

Use `path`, `text`, `expected_sha256` as the inputs for this operation.

Effects: fs.write.

Check current SHA-256 before atomic replacement, returning Result. Detects stale content; concurrent writers still need external coordination.

Return *[Check current SHA-256 before atomic replacement, returning Result](> host_fs_write_checked path=`path` text=`text` expected_sha256=`expected_sha256`)*.

## read_lines

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| max_bytes | number | 8388608 | Maximum bytes read or expanded; exceeding it fails |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Read bounded UTF-8 lines without terminators; CRLF is accepted and a final newline does not create a phantom row |

Use `path`, `max_bytes`=8388608 as the inputs for this operation.

Effects: fs.read.

Read bounded UTF-8 lines without terminators; CRLF is accepted and a final newline does not create a phantom row.

Return *[Read bounded UTF-8 lines without terminators; CRLF is accepted and a final newline does not create a phantom row](> host_fs_read_lines path=`path` max_bytes=`max_bytes`)*.

## write_lines

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| lines | list | required | List of text lines to write |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `lines` as the inputs for this operation.

Effects: fs.write.

Atomically write text lines using LF and one final newline for nonempty input.

Return *[Atomically write text lines using LF and one final newline for nonempty input](> host_fs_write_lines path=`path` lines=`lines`)*.

## remove_file

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Delete a file, refusing directories; missing paths fail. Legacy remove remains recursive for compatibility.

Return *[Delete a file, refusing directories; missing paths fail](> host_fs_remove_file path=`path`)*.

## remove_empty_dir

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path` as the inputs for this operation.

Effects: fs.write.

Delete only an empty directory; refuses nonempty directories. Native filesystem only.

Return *[Delete only an empty directory; refuses nonempty directories](> host_fs_remove_empty_dir path=`path`)*.

## temp_dir

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prefix | text | "mqtmp" | Prefix added to the generated name or tested against text |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Create a directory under the filesystem root and return its relative name |

Use `prefix`="mqtmp" as the inputs for this operation.

Effects: fs.write.

Create a directory under the filesystem root and return its relative name. Caller owns cleanup with remove_tree; never uses a global OS temp directory.

Return *[Create a directory under the filesystem root and return its relative name](> host_fs_temp_dir prefix=`prefix`)*.

## file_hash

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Compute SHA-256 with a 64 KiB read buffer; supports binary and large files without loading the whole file |

Use `path` as the inputs for this operation.

Effects: fs.read.

Compute SHA-256 with a 64 KiB read buffer; supports binary and large files without loading the whole file.

Return *[Compute SHA-256 with a 64 KiB read buffer; supports binary and large files without loading the whole file](> host_fs_file_hash path=`path`)*.

## find

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| pattern | text | "**" | Relative path glob supporting *, ** and ? |
| exclude | any | None | Glob pattern list pruned during traversal; None excludes nothing |
| max_depth | number | 64 | Maximum traversal depth |
| kind | text | "file" | Entry kind filter: file, dir or all (files and directories) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Find sorted relative paths with star, double-star and question-mark globs; exclude patterns prune directories; kind is file/dir/all |

Use `path`, `pattern`="**", `exclude`=None, `max_depth`=64, `kind`="file" as the inputs for this operation.

Effects: fs.read.

Find sorted relative paths with star, double-star and question-mark globs; exclude patterns prune directories; kind is file/dir/all. No symlink traversal; at most 100000 results. Native filesystem snapshot.

Return *[Find sorted relative paths with star, double-star and question-mark globs; exclude patterns prune directories; kind is file/dir/all](> host_fs_find path=`path` pattern=`pattern` exclude=`exclude` max_depth=`max_depth` kind=`kind`)*.

## copy_tree

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path; archive packing expects a directory |
| dest | text | required | Destination path; overwrite is allowed only where documented |
| overwrite | bool | False | Whether an existing destination file may be replaced |
| exclude | any | None | Glob pattern list pruned during traversal; None excludes nothing |
| dry_run | bool | False | Whether to return the plan without modifying files |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Copy files between disjoint trees |

Use `src`, `dest`, `overwrite`=False, `exclude`=None, `dry_run`=False as the inputs for this operation.

Effects: fs.write.

Copy files between disjoint trees. Preflight conflicts; overwrite is opt-in. Returns relative changed files; dry_run plans only. Does not copy permissions, symlinks or empty directories.

Return *[Copy files between disjoint trees](> host_fs_copy_tree src=`src` dest=`dest` overwrite=`overwrite` exclude=`exclude` dry_run=`dry_run`)*.

## sync_tree

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | text | required | Source path; archive packing expects a directory |
| dest | text | required | Destination path; overwrite is allowed only where documented |
| overwrite | bool | False | Whether an existing destination file may be replaced |
| exclude | any | None | Glob pattern list pruned during traversal; None excludes nothing |
| dry_run | bool | False | Whether to return the plan without modifying files |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Copy new or changed files with copy_tree conflict rules |

Use `src`, `dest`, `overwrite`=False, `exclude`=None, `dry_run`=False as the inputs for this operation.

Effects: fs.write.

Copy new or changed files with copy_tree conflict rules. Never deletes destination-only files; execution is not transactional.

Return *[Copy new or changed files with copy_tree conflict rules](> host_fs_sync_tree src=`src` dest=`dest` overwrite=`overwrite` exclude=`exclude` dry_run=`dry_run`)*.

## rename_many

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| moves | list | required | List of source/destination rename records |
| dry_run | bool | False | Whether to return the plan without modifying files |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Preflight src/dest file records, reject existing or duplicate destinations, then rename in input order |

Use `moves`, `dry_run`=False as the inputs for this operation.

Effects: fs.write.

Preflight src/dest file records, reject existing or duplicate destinations, then rename in input order. No cycles or rollback; dry_run reports the planned count.

Return *[Preflight src/dest file records, reject existing or duplicate destinations, then rename in input order](> host_fs_rename_many moves=`moves` dry_run=`dry_run`)*.

## read_json

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | 解析后的数据 |

Use `path` as the inputs for this operation.

Effects: fs.read.

Read UTF-8 then use the standard parser; errors abort, so batch tasks should first use try_read_text.

We retain **`body` = [Read UTF-8 text from the selected file](read_text(path=`path`))**.
Return *[Parse the JSON transport text](json.parse(text=`body`))*.

## write_json

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to filesystem root |
| value | any | required | Data to serialize |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `value` as the inputs for this operation.

Effects: fs.write.

Serialize then atomically replace the file; formatting and comments are not preserved.

We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`value`))**.
Return *[Atomic write](atomic_write(path=`path`, text=`body`))*.

## read_csv

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to filesystem root |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | 解析后的数据 |

Use `path` as the inputs for this operation.

Effects: fs.read.

Read UTF-8 then use the standard parser; errors abort, so batch tasks should first use try_read_text.

We retain **`body` = [Read UTF-8 text from the selected file](read_text(path=`path`))**.
Return *[Parse](csv.parse(text=`body`))*.

## write_csv

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to filesystem root |
| value | list | required | Data to serialize |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | None after completion; filesystem failures produce a diagnostic |

Use `path`, `value` as the inputs for this operation.

Effects: fs.write.

Serialize then atomically replace the file; formatting and comments are not preserved.

We retain **`body` = [Stringify](csv.stringify(rows=`value`))**.
Return *[Atomic write](atomic_write(path=`path`, text=`body`))*.

## resolve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root, or an absolute path inside it |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Normalized absolute logical path after the native root and symlink checks |

Use `path` to obtain an absolute address before giving a local file to another resource, such as a SQLite URL. The host resolves it using the same filesystem context as read_text and temp_dir, so a different process working directory does not change the selected root. The destination may not exist yet; its nearest existing ancestor still undergoes the native root and symlink checks. Requires fs.read and creates no file. The returned path remains logical, including permitted links; external plugins and processes retain their own operating-system authority.

Return *[Resolve the path through the native filesystem context](host_fs_resolve(path=`path`))*.
