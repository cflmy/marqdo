---
markup: occlusion
title: Engineering boundaries for executable handbooks
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  paths: lib/path.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
---

This extension supplies the small OS and service boundaries used by executable
engineering handbooks. File reads/writes, JSON/TOML, data decisions, retries and
release ordering remain Marqdo code. The native plugin provides bounded argv
process execution, private authenticated GitHub HTTPS, ZIP inspection and R2
Signature V4 uploads. No Python interpreter or generated shell program is used.

Read the task contract before calling it. Process children and native plugins
have the calling OS user's authority; a plugin grant is broader than a native
filesystem grant. The root parameter selects a working/artifact directory, not
an OS sandbox. Never treat reading this file as authorization to push refs,
change a release, upload assets or run arbitrary commands.

Build this repository's marqdo_plugin_engineering crate before first use; the
development handbook's prepare_engineering task performs that bootstrap through
marqdo call. MARQDO_ENGINEERING_PLUGIN may select a locally built library. Installed
packs use the same plugin resolver as other official extensions. This boundary
is native-only; browser WASM cannot start processes or load this library.

The public operation definitions and private request helper below share one
implementation. For the complete release workflow, read
[the executable release handbook](../../skills/marqdo-release/index.mq.md).
Normal filesystem work uses [the native file tools](../../skills/marqdo-dev/files.mq.md);
this extension is not an alternate file-write API.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load the installed or locally built engineering plugin |

Resolve the library through the existing extension resolver so an installed
pack and a development build share the same public calls. A missing library
produces the normal plugin diagnostic; build or install it rather than hiding
the failure behind a fixed success result. Repeated loads use the host loader's
existing ownership rules. This private boundary requires plugin effects and
never guesses a Python replacement or invents an external interpreter.

We **path = [Resolve the engineering native library](plugin.native_path(name="engineering"))**.
After locating it we return *[Load the bounded engineering operations](plugin.load(path=`path`))*.

## request

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation | text | required | command, git_inspect, github, archive, analyze, gzip_text or r2_put |
| payload | map | required | Operation-specific ordinary data; never credential values |
| root | text | "." | Existing working/artifact root for this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result.ok/value/error from the selected boundary |

Use `operation`, `payload` and `root`="." only at a documented native boundary.
Inputs travel as structured plugin values, not strings containing a generated
program. Failed process setup, HTTP, archive validation or signing remains a
failed Result. A successful command request still requires checking value.code
and value.timed_out. No private credential becomes a tool argument or result;
authentication is resolved inside the plugin from existing stores.

We **loaded = [Load this package's native boundary](_load())** and
**base = [Read the granted runtime working root](sys.cwd())** and **resolved_root = root**.
1. not [The requested root is already absolute](paths.is_absolute(path=`root`))
  We **resolved_root = base + "/" + root** so process and native filesystem locations agree.

`request_data` =

| operation | payload | root |
|---|---|---|
| `operation` | `payload` | `resolved_root` |

Return *[Perform the explicit native boundary operation](engineering_request(request=`request_data`))*.

## run

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| argv | list | required | Executable followed by individual text arguments |
| cwd | text | "." | Existing directory inside root |
| timeout_ms | number | 120000 | Positive limit through 3600000 milliseconds |
| max_output | number | 262144 | Per-stream capture limit through 16777216 bytes |
| stdin | any | None | Optional text sent to child stdin, then closed |
| env | any | None | Optional child-only text environment values |
| root | text | "." | Root used to resolve cwd and local Git identity |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing code/stdout/stderr/timed_out/truncated |

Use `argv`, `cwd`=".", `timeout_ms`=120000, `max_output`=262144, `stdin`=None,
`env`=None and `root`="." to invoke an existing OS tool. Arguments are passed
individually even when they contain quotes, spaces or shell metacharacters.
Inline shell programs, forced Git updates and hook bypass are rejected; a
checked-in packaging script remains a normal explicit external-tool invocation.

The return distinguishes transport completion from child success. Inspect code
and timed_out before consuming stdout. Output is drained concurrently and kept
within the requested cap so noisy children cannot deadlock their own pipes.
On Unix, timeout kills the child process group; Windows attempts taskkill's tree
termination. Output retained by surviving descendants cannot cause unbounded
waiting. A process environment is child-local and does not mutate the parent.

`payload` =

| argv | cwd | timeout_ms | max_output | stdin | env |
|---|---|---|---|---|---|
| `argv` | `cwd` | `timeout_ms` | `max_output` | `stdin` | `env` |

Return *[Execute the explicit argument vector with bounded capture](request(operation="command", payload=`payload`, root=`root`))*.

## github

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | API path under /repos/cflmy/marqdo/ |
| method | text | "GET" | GET, PATCH or POST; mutations need the user's task instruction |
| data | any | None | JSON body; no authentication data |
| missing | bool | False | Treat GET 404 as a successful None |
| root | text | "." | Checkout used by credential helpers |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing service JSON, None or a redacted transport error |

Use `path`, `method`="GET", `data`=None, `missing`=False and `root`=".".
The plugin resolves GH_TOKEN/GITHUB_TOKEN, Git's configured credential helper or
the checkout owner's existing gh store privately. It tries the configured
GitHub proxy before direct HTTPS. An uncertain POST is not replayed; a caller
must observe whether publication occurred before deciding to retry a mutation.

Only this repository's API paths are accepted. HTTP errors report their status
without reflecting credentials, signed URLs or arbitrary response bodies.
Missing evidence stays missing; a network failure is not a synthetic empty
release. Test endpoint overrides are explicit environment configuration and
belong only in isolated fixture runs.

`payload` =

| path | method | data | missing |
|---|---|---|---|
| `path` | `method` | `data` | `missing` |

Return *[Request private-authenticated repository service JSON](request(operation="github", payload=`payload`, root=`root`))*.

## inspect_zip

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Existing ZIP or VSIX inside root |
| root | text | "." | Artifact root; symlink escapes are refused |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with entries and selected UTF-8 package metadata |

Use `path` and `root`="." to inspect a binary format without extraction.
The plugin caps entry count and total expanded size, rejects escaping paths and
symlinks, and reads every entry to validate its content/CRC. It retains only
bounded VERSION, Cargo and VSIX metadata text for Marqdo's role-specific checks.
Hashing the archive itself remains lib/fs.file_hash, not an alternate Python IO
path. ZIP metadata cannot prove plugin ABI compatibility or execute a program.

We **payload = [Name the one archive path field](table.put(in=None, at="path", value=`path`))**.

Return *[Inspect the archive format without writing extracted files](request(operation="archive", payload=`payload`, root=`root`))*.

## put_file

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Verified upload file inside root |
| key | text | required | Explicit R2 key such as vX.Y.Z/filename.zip |
| sha256 | text | required | Previously verified SHA256; changed bytes are rejected |
| root | text | "." | Root containing the local upload file |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with written key, SHA256 and byte count |

Use `path`, `key`, `sha256` and `root`="." for an authorized publication.
The plugin transfers this bounded file across the authenticated service boundary
and rechecks its digest before signing; Marqdo decides which files and keys are
required. Credentials stay in existing R2 environment settings or ~/.marqdo/r2.env.
A configured credential home can select the maintainer's store when running under
another OS identity. Key material is never a parameter, result or command argument.

This is an external write. Successful PUT is evidence for that key only; public
CDN parity is checked separately. The release handbook uploads every pack before
writing VERSION pointers and never sweeps unrelated files from dist. A timeout
remains uncertain publication, so inspect the public object before retrying.

`payload` =

| path | key | sha256 |
|---|---|---|
| `path` | `key` | `sha256` |

Return *[Upload exactly the verified artifact with private R2 signing](request(operation="r2_put", payload=`payload`, root=`root`))*.

## put_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Explicit VERSION key selected by the release workflow |
| text | text | required | Small UTF-8 pointer body with its intended trailing newline |
| root | text | "." | Checkout used to locate existing credential stores |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with written key, SHA256 and byte count |

Use `key`, `text` and `root`="." only after all required publication bytes are
verified. This operation signs a small UTF-8 service body; it does not write a
local file or infer which version should become latest. The calling Marqdo
workflow owns that decision and its ordering. Keys remain explicit so a replay
cannot silently publish a different version from environment state.
As with put_file, service completion does not substitute for reading the public
pointer and checking the associated download hashes.

`payload` =

| key | text |
|---|---|
| `key` | `text` |

Return *[Write the explicitly selected publication pointer](request(operation="r2_put", payload=`payload`, root=`root`))*.
## analyze

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| source | text | required | Exact source text already read through lib/fs |
| include_body | bool | False | Include compiler AST debug evidence for migration review |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with compiler functions, contracts, diagnostics and narrative ratios |

Use `source` and `include_body`=False to inspect an authored document
without executing it. File access belongs to the caller's native filesystem
step; this operation receives text and uses the actual Marqdo parser and checker.
It does not invent a regular-expression grammar, load imports, run main, open
network resources or infer permission from prose. Full dependency checks still
use marqdo check on the real file.

The returned parameter names, defaults, heading spans and contracts support
editing without guessing the syntax. Narrative ratios come from the same
implementation used by the repository gate. AST debug evidence is optional
because it is large and intended for mechanical migration comparisons, while
the normal result remains a compact source description.

`payload` =

| source | include_body |
|---|---|
| `source` | `include_body` |

Return *[Inspect this text with the real compiler](request(operation="analyze", payload=`payload`))*.


## gzip_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Portable bytes returned by net.get(response="bytes") |
| max_bytes | number | 16777216 | Positive expanded UTF-8 limit, at most 64 MiB |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing decoded UTF-8 text or bounded format failure |

Use `data` and `max_bytes`=16777216 at a compressed data boundary.
Native net.get owns the network read and returns portable bytes; this operation
only interprets their gzip format. It accepts concatenated gzip members, validates
their checksums, caps compressed input and expanded output, and rejects non-UTF-8
text. No temporary file, shell pipeline or Python process is involved.

This is useful for a published apt Packages.gz index. A successful HTTP response
alone cannot establish its package version; decode and inspect its real stanzas.
Malformed data or an expansion beyond the cap remains an error. Binary payloads
are outside this text contract. Calling requires plugin, while the caller's
earlier download independently requires network.read.

`payload` =

| data | max_bytes |
|---|---|
| `data` | `max_bytes` |

Return *[Decode the bounded compressed text format](request(operation="gzip_text", payload=`payload`))*.


## check_exit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| outcome | map | required | Actual Result returned by engineering.run or its command boundary |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Observed process Result; failure retains its complete bounded execution evidence |

Use `outcome` from an actual process call. Starting a process successfully is
different from that process succeeding: command transport can return ok=true
with a nonzero child code. This pure adapter requires code zero and no timeout.
It never starts or retries a process, changes a command or discards failed output.

A setup/transport failure is retained unchanged. A nonzero exit or timeout becomes
a failed Result with the original execution record in value, including stdout,
stderr and truncation flags. This makes --require-ok useful for a complete
callable workflow without treating CLI transport success as a passed build.
Bounded output can be truncated even when the command succeeds; inspect that
flag before describing it as a full log.

This function only classifies supplied observations. It cannot prove that an
arbitrary caller-provided map came from a real command. Development tasks call
run and then this adapter in the same source flow, retaining their actual argv
and result. Release ordering and retry policy remain the caller's responsibility.

1. not access(`outcome`, "ok")
  Return *outcome* with its original native setup failure.
We **execution = [Read the actual bounded process observation](access(outcome, "value"))**.
1. access(`execution`, "code") != 0 or access(`execution`, "timed_out")
  We **failed = [Distinguish child failure from successful transport](result.error(code="engineering.process_failed", message="nonzero exit or timeout; inspect the actual execution in value", operation="check_exit"))**.
  Return *[Keep all observed process evidence on failure](table.put(in=`failed`, at="value", value=`execution`))*.
Return *outcome* after the observed child exits successfully.


## checked

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| argv | list | required | Existing executable and individual text arguments |
| cwd | text |  "."  | Existing working directory within root |
| timeout_ms | number | 120000 | Positive child duration limit up to one hour |
| max_output | number | 262144 | Bounded bytes retained per stdout/stderr stream |
| stdin | any | None | Optional text sent to child standard input |
| env | any | None | Optional child-only text environment record |
| root | text |  "."  | Working root matching the intended checkout |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result requiring a zero child exit; actual execution evidence is retained |

Use `argv`, `cwd`=".", `timeout_ms`=120000, `max_output`=262144,
`stdin`=None, `env`=None and `root`="." to execute one existing tool and require
its success in the same callable workflow. It composes run and check_exit rather
than regenerating process management, credential handling or a shell program.
The parameter vector, directory and resource limits keep their existing meaning.

A setup failure, nonzero exit or timeout returns ok=false. Its actual stdout,
stderr and truncation state remain available for diagnosis. A successful result
means the observed child exited successfully; interpreting its business output
still belongs to the caller. Use --require-ok when this single call controls a
dependent external automation. No retry or publication is implied.

This operation requires plugin. Its child retains normal OS authority, and the
native boundary keeps its existing argv guards and private credential rules.
For example, a unit-test task can pass cargo, test, --lib and --offline as four
separate text elements without needing a new generated command script.

After the bounded execution we return *[Require the actual child outcome and preserve all evidence](check_exit(outcome=run(argv=`argv`, cwd=`cwd`, timeout_ms=`timeout_ms`, max_output=`max_output`, stdin=`stdin`, env=`env`, root=`root`)))*.
