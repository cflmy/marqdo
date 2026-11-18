---
markup: occlusion
title: Release model and deliverable contract
import:
  re: lib/re.mq.md
  result: lib/result.mq.md
  table: lib/table.mq.md
---

This is the pure release model: version choice belongs to the user, while artifact
names and stage requirements are derived here once. Calling plan never edits a
file, contacts a service, tags a commit or publishes a package. It is suitable
for an agent to inspect before requesting any effect grant.

Full releases unify CLI, WASM, extension pack and VSIX versions. CLI releases keep
the independently versioned extension pack. Ext releases publish source and both
native packs to GitHub and R2 without retagging the CLI or uploading an Ubuntu CLI.
A tag is a reference to an immutable source decision; an existing tag is not an
invitation to overwrite it. The transport enforces the no-force constraint too.

Use marqdo call skills/marqdo-release/model.mq.md plan 'version="2.6.0"' --json.
The example describes the already published version; it grants no authorization
to publish it again. Inspecting an existing release uses index.status instead.

## plan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z; never infer the next release |
| mode | text | "full" | full, cli or ext; this selects required deliverables |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Derive required assets and stages from a confirmed version and release mode. |

Derive required assets and stages from a confirmed version and release mode.
Use `version` and `mode`="full". Stable versions have three nonnegative decimal
components without leading zeroes. Unknown modes and malformed versions return
release.input failures before any path or command can be constructed.
The result includes exact filenames; platform completeness is therefore a data
check rather than a model's recollection of what should have been uploaded.

We **valid = [Accept only stable X.Y.Z version input](re.is_match(text=`version`, pattern="^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$"))**.
1. not `valid`
  Return *[Report an invalid confirmed version](result.error(code="release.input", message="version must be stable X.Y.Z without leading zeroes", operation="plan"))*.
1. `mode` != "full" and `mode` != "cli" and `mode` != "ext"
  Return *[Report an unsupported release mode](result.error(code="release.input", message="mode must be full, cli or ext", operation="plan"))*.

We **cdn = [Read the maintained cdn contract](cdn_names(version=`version`))**.

We **cli = [Read the maintained cli contract](cli_names(version=`version`))**.

1. `mode` == "ext"
  We **required = cdn** and **tag = "ext-v" + version** for an independent pack.
2. `mode` == "cli"
  We **required = cli** and **tag = "v" + version** for the interpreter.
3. *
  We **required = [Require both CLI and cross-platform extension assets](table.concat(a=`cli`, b=`cdn`))** and **tag = "v" + version**.

We **stages = [Read the maintained stages contract](stages_names())**.

We **needs_ppa = [Select Ubuntu publication for CLI and full modes](mode != "ext")**.

`plan_data` =

| version | mode | tag | required | cdn | ppa | stages |
|---|---|---|---|---|---|---|
| `version` | `mode` | `tag` | `required` | `cdn` | `needs_ppa` | `stages` |

After establishing the contract, return *[The pure release plan](result.ok(value=`plan_data`))*.

## cdn_names

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable version already accepted by plan |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Exact cdn release asset names |

Exact cdn release asset names.
Source and native assets are three separate deliverables. The source ZIP contains
readable extension modules; each native ZIP contains platform-specific plugins.
Windows success cannot stand in for Linux support. Publishing uses all three
files from the same GitHub release and writes VERSION pointers only after uploads.

Use `version` for these filenames. This helper performs no IO.

`cdn` =

| filename |
|---|
| "marqdo-`version`-ext.zip" |
| "marqdo-`version`-native-x86_64-pc-windows-msvc.zip" |
| "marqdo-`version`-native-x86_64-unknown-linux-gnu.zip" |

We return *cdn* once this contract is assembled.

## cli_names

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable version already accepted by plan |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Exact cli release asset names |

Exact cli release asset names.
The executable is convenient for an existing installation; each platform bundle
also carries readable libraries and the platform runtime dependencies. The
standard-library ZIP supports source inspection, the source ZIP identifies the
tagged repository, the VSIX is installed into the editor, and the public ZIP is
offline documentation. These roles remain distinct even when some content overlaps.

Use `version` for these filenames. This helper performs no IO.

`cli` =

| filename |
|---|
| "marqdo-`version`-x86_64-pc-windows-msvc.exe" |
| "marqdo-`version`-x86_64-pc-windows-msvc.zip" |
| "marqdo-`version`-x86_64-unknown-linux-gnu.zip" |
| "marqdo-`version`-stdlib.zip" |
| "marqdo-`version`-source.zip" |
| "marqdo-`version`.vsix" |
| "marqdo-`version`-public.zip" |

We return *cli* once this contract is assembled.

## stages_names

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Task routing for the release lifecycle |

Task routing for the release lifecycle.
The routing table is a map for the agent, not a shell script to rewrite.
Each task resolves to a callable heading in this package. Mode-specific work
is selected by plan.mode and plan.ppa; an ext release skips the CLI and Ubuntu
stages. Any external-state failure remains visible to the next invocation.
Review bilingual notes before the notes task, and retain the returned commit
and artifact hashes with the release record. The table orders responsibilities;
it never authorizes a version the user has not chosen.
CI waiting is observational and bounded by the caller, so asking for status does
not queue another release or repeat an upload.

`stages` =

| @ | task | responsibility |
|---|---|---|
| 1 | local.inspect / index.preflight | versions, branch, tree and tag checks |
| 2 | local.bump / local.docs | version files, reviewed notes and generated documentation |
| 3 | local.extension | VSIX source version on its own branch |
| 4 | local.commit / local.push / local.tag | reviewed commit, proxy push, immutable annotated tag |
| 5 | packs.ci / packs.attach_ci (ext); github.status / github.notes / assets.download | both-platform exact-tag builds, reviewed bilingual notes and verified GitHub assets |
| 6 | assets.publish / assets.status | exact same extension bytes on R2 and GitHub |
| 7 | ubuntu.ship / ubuntu.status | signed noble source upload, build and apt publication |

We return *stages* once this contract is assembled.
