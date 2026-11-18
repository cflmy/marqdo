---
markup: occlusion
title: Release assets: bytes, contents and CDN parity
import:
  bridge: ./_bridge.mq.md
  native: ./asset_io.mq.md
  github: ./github.mq.md
  model: ./model.mq.md
  result: lib/result.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
  json: lib/json.mq.md
  toml: lib/toml.mq.md
---

Artifacts have a lifecycle: discover the exact GitHub metadata, download to a
release-specific directory, verify bytes and package role, publish only the
three extension packs, then compare every public copy. Full mode requires ten
GitHub assets; Linux CLI and Linux native packages are separate obligations.

The local staging directory is dist/release-X.Y.Z by default. Downloads use
atomic .partial replacement and bounded retries. Verification reads ZIP entries
without extraction. Existing verified files are reusable; a failed operation
never changes an old tag or treats an incomplete upload as completion.

Read-only public checks and local downloads use plugin,fs.write,network.read,environment.read because the checked-in
native transport owns those service and filesystem operations. Publishing is an
explicit external mutation authorized by a release request. Credentials remain
in existing environment or ~/.marqdo/r2.env stores and are not call arguments.

```sh
marqdo call skills/marqdo-release/assets.mq.md download 'version="2.6.0"' --effects plugin,fs.write,network.read,environment.read --fs-root . --json
marqdo call skills/marqdo-release/assets.mq.md status 'version="2.6.0"' --effects plugin,fs.write,network.read,environment.read --fs-root . --json
```

These commands inspect the existing release; they do not republish it. Use
publish only for the version and mode actually authorized. On missing keys,
configure the documented local credential store and call publish again. Never
put credentials in a parameter, notes file, source tree or session binding.

## named

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| assets | list | required | GitHub asset records from one exact release |
| name | text | required | Exact filename to look up |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Matching asset record or None |

Use `assets` and `name` to find one exact release artifact. Prefix matches
are insufficient because source packs, CLI bundles and native packs serve
different roles. This pure helper preserves the GitHub-provided size, digest and
download URL together. It neither creates an asset nor substitutes a file from
a previous release. None means the caller must keep the release incomplete,
even if other files are already downloadable.

- [Find the exact filename in this release](for assets as asset)
  1. access(`asset`, "name") == `name`
    Return *asset* with its original metadata.
Return *None* when no asset has that exact name.

## catalog

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the accepted plan and complete GitHub asset records |

Use `version`, `mode`="full" and `root`="." before downloading or publishing.
All required names are checked first, so an incomplete release cannot partially
advance the CDN pointer. Extra GitHub assets are allowed; they do not replace
any required filename. A missing release and a missing platform pack have
separate error codes. Only metadata is fetched here; no local archive is trusted
merely because it has the right filename. Content and digest checks occur next.
For ext mode the same rule requires source, Windows native and Linux native packs.

We **planned = [Read the exact deliverable contract](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

We **published = [Read this tag's actual GitHub assets](github.release(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`published`, "ok")
  Return *published* with its failure intact.

We **release_data = [Read published metadata or None](access(published, "value"))**.
1. `release_data` == None
  Return *[Keep a missing release visible](result.error(code="release.missing", message="GitHub release is not published", operation="assets.catalog"))*.
We **plan_data = [Keep the validated mode and names](access(planned, "value"))** and
**assets = [Read GitHub asset records](access(release_data, "assets"))** and
**required = [Read the required filenames](access(plan_data, "required"))**.
- [Require every declared deliverable before continuing](for required as name)
  We **asset = [Find this exact artifact](named(assets=`assets`, name=`name`))**.
  1. `asset` == None
    Return *[Report the missing deliverable](result.error(code="release.asset_missing", message=`name`, operation="assets.catalog"))*.

`catalog_data` =

| plan | release | assets |
|---|---|---|
| `plan_data` | `release_data` | `assets` |

Return *[The complete metadata catalog](result.ok(value=`catalog_data`))*.

## bundle_content

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| entries | list | required | Validated archive entry names |
| platform | text | required | windows or linux, chosen from the catalog filename |
| name | text | required | Exact bundle filename retained in errors |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result confirming CLI, readable libraries and every embedded native plugin |

Use `entries`, `platform` and `name` for an already CRC-checked CLI bundle.
A standalone executable has a different role: the bundle must also carry its
readable standard and extension sources. This task requires the interpreter,
file/table libraries, extension version, web/quantum handbooks and all baseline
native libraries. When engineering sources are present, its native library is
required in the same bundle.

Platform names are explicit data. Windows uses marqdo.exe and DLL names, Linux
uses marqdo and lib-prefixed shared objects. A missing library cannot be excused
by success on the other platform. Archive paths use their portable forward-slash
representation; no extraction or ABI execution happens in this pure layer.

Hash/size/CRC identity comes from the earlier byte check, and the correct-commit
CI gate proves the platform build. This function verifies packaging coherence,
not scientific correctness, loader ABI compatibility or a machine's installed
system dependencies. Its successful Result means every listed bundle obligation
was observed in these actual entry names.

We **binary = "marqdo"** and **prefix = "lib"** and **suffix = ".so"** for Linux.
1. `platform` == "windows"
  We **binary = "marqdo.exe"** and **prefix = ""** and **suffix = ".dll"**.
2. `platform` != "linux"
  Return *[Reject an unknown bundle platform](result.error(code="release.platform", message=`platform`, operation="bundle_content"))*.

`required` =

| entry |
|---|
| `binary` |
| "lib/fs.mq.md" |
| "lib/table.mq.md" |
| "ext/VERSION" |
| "ext/web/index.mq.md" |
| "ext/quantum/index.mq.md" |

`plugins` =

| plugin |
|---|
| "agent" |
| "linalg" |
| "llm" |
| "quantum" |
| "web" |

1. [This bundle includes engineering sources](table.contains(list=`entries`, item="ext/engineering/index.mq.md"))
  We **plugins = [Require the same new native capability](table.append(list=`plugins`, item="engineering"))**.
- [Add each embedded native dependency](for plugins as plugin)
  We **required = [Retain the platform-specific native path](table.append(list=`required`, item="ext/native/" + prefix + plugin + suffix))**.
- [Check each actual bundle obligation](for required as entry)
  1. not [The bundle contains this required file](table.contains(list=`entries`, item=`entry`))
    Return *[Retain the missing packaging entry](result.error(code="release.bundle_content", message=`name` + ": " + `entry`, operation="bundle_content"))*.
Return *[The actual packaging checks passed](result.ok(value=True))*.


## matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Complete catalog-selected records after individual size/digest/role checks |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result retaining records only when declared engineering sources have both native platforms |

Use `records` after individual content checks on the complete release set.
The source pack declares which extension sources it actually ships. If it contains
engineering/index.mq.md, both native packs must contain that capability too.
A new source pack combined with older native packs would otherwise pass separate
filename checks and fail only when an agent tries to load its new tool.

This check follows package contents rather than guessing a version threshold.
Historical packs without engineering sources retain their original five-plugin
contract. CLI bundles are checked independently by bundle_content, including
their embedded source/native correspondence. The release model and catalog still
own the complete filename matrix for full/cli/ext; this helper does not turn an
arbitrary partial list into proof of an entire release.

The function is pure. It neither downloads, extracts nor executes an archive.
Missing new native capability remains a failed Result before publication, and
a successful result preserves all previously verified records for summarization.

We **engineering = False** and **windows = False** and **linux = False** before inspecting the declared source footprint.
- [Read capability declarations from the source pack](for records as record)
  We **name = [Read this verified filename](access(record, "name"))**.
  1. [This is the extension source pack](text.ends_with(text=`name`, suffix="-ext.zip"))
    We **engineering = [Observe whether engineering sources actually ship](table.contains(list=access(record, "entries"), item="ext/engineering/index.mq.md"))**.
1. not `engineering`
  Return *[Keep the historical source/native contract](result.ok(value=`records`))*.
- [Require new capability on each native platform](for records as record)
  We **name = [Read this native candidate's filename](access(record, "name"))** and **required = None**.
  1. [This is the Windows native pack](text.ends_with(text=`name`, suffix="-native-x86_64-pc-windows-msvc.zip"))
    We **windows = True** and **required = "native/engineering.dll"**.
  1. [This is the Linux native pack](text.ends_with(text=`name`, suffix="-native-x86_64-unknown-linux-gnu.zip"))
    We **linux = True** and **required = "native/libengineering.so"**.
  1. `required` != None
    1. not [This platform contains its declared engineering library](table.contains(list=access(`record`, "entries"), item=`required`))
      Return *[Reject source/native capability drift](result.error(code="release.native_capability", message=`name`, operation="assets.matrix"))*.
1. not `windows` or not `linux`
  Return *[Require both native platforms](result.error(code="release.native_platform", message="engineering sources require Windows and Linux native packs", operation="assets.matrix"))*.
Return *[The consistent source/native records](result.ok(value=`records`))*.


## summaries

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Verified records retained after matrix checks |
| directory | text | required | Actual local staging directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Compact name/path/size/SHA256/entry-count evidence rows |

Use `records` and `directory` after verification, not as a replacement for it.
An archive can contain thousands of source paths. Returning every path and full
package metadata at the end of a release obscures the handful of facts an agent
needs to act: exact local path, byte length, digest and entry count.

This pure projection keeps those observed facts in one row per artifact.
It does not invent a verified flag, rehash a file or silently discard an earlier
failure. Executable-only files have zero archive entries. Detailed inspection
remains available from asset_io.verify and engineering.inspect_zip when needed.
The reduced final evidence is also suitable for saving through native fs.write_json,
without requiring another language to parse command output or reconstruct a report.

We **rows = [Start concise artifact evidence](table.empty_list())**.
- [Project already-checked artifacts into readable rows](for records as record)
  We **name = [Keep the exact filename](access(record, "name"))** and **size = [Keep the measured size](access(record, "size"))** and **sha256 = [Keep the measured digest](access(record, "sha256"))** and **entries = [Read optional archive details](table.get(map=`record`, key="entries"))** and **entry_count = 0** and **path = directory + "/" + name**.
  1. `entries` != None
    We **entry_count = [Count the inspected entries](len(entries))**.

  `row` =

  | name | path | size | sha256 | entry_count |
  |---|---|---|---|---|
  | `name` | `path` | `size` | `sha256` | `entry_count` |

  We **rows = [Retain this measured artifact](table.append(list=`rows`, item=`row`))**.
Return *rows* after projecting each verified record.


## content

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| record | map | required | Verified archive record produced by the byte transport |
| version | text | required | Stable X.Y.Z explicitly chosen by the user |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result retaining archive evidence after role-specific content checks |

Use `record` and `version` after a SHA256/size/CRC check. This second layer
checks useful contents: extension sources identify their pack version; native
packs retain their baseline plugins and source-declared new capabilities; bundles include the embedded-library
sources and a CLI; VSIX metadata declares its editor version; the source archive
contains the tagged Cargo version. These checks complement CI and do not claim
to prove ABI compatibility or scientific correctness of every plugin.
An executable-only asset has no ZIP entries and relies on its GitHub digest and
build evidence. A source pack without either handbook entry is unusable for the
document-first workflow even when it decompresses correctly. Native filenames
are checked individually because one missing shared library would otherwise
surface only after installation, far from the publication step that caused it. No archive is extracted or executed while reading its metadata.

Version text is trimmed for comparison, so Windows CRLF and Unix LF identify
the same declared version without changing any archive bytes. SHA256 still
covers the original byte sequence. A source snapshot is checked through its
actual TOML package.version field; a matching dependency version cannot satisfy
that role. Missing metadata remains a failure rather than a guessed version.

We **name = [Read the verified filename](access(record, "name"))**.
1. [This is an executable-only asset](text.ends_with(text=`name`, suffix=".exe"))
  Return *[Retain its verified digest evidence](result.ok(value=`record`))*.
We **entries = [Read the archive entry list](access(record, "entries"))** and
**metadata = [Read bounded metadata files](access(record, "metadata"))** and
**required = [Start the role-specific entry list](table.empty_list())**.
1. [This is the source extension pack](text.ends_with(text=`name`, suffix="-ext.zip"))

  `required` =

  | entry |
  |---|
  | ext/VERSION |
  | ext/quantum/index.mq.md |
  | ext/web/index.mq.md |

  We **declared = [Read the pack version without assuming platform newlines](table.get(map=`metadata`, key="ext/VERSION"))**.
  1. `declared` == None
    Return *[Report missing version metadata](result.error(code="release.pack_version", message=`name`, operation="assets.content"))*.
  We **pack_version = [Normalize the source pack version field](text.str_trim(s=`declared`))**.
  1. `pack_version` != `version`
    Return *[Reject a mismatched source pack version](result.error(code="release.pack_version", message=`name`, operation="assets.content"))*.
1. [This is a Linux native pack](text.contains(text=`name`, sub="-native-x86_64-unknown-linux-gnu"))

  `required` =

  | entry |
  |---|
  | native/libagent.so |
  | native/liblinalg.so |
  | native/libllm.so |
  | native/libquantum.so |
  | native/libweb.so |

1. [This is a Windows native pack](text.contains(text=`name`, sub="-native-x86_64-pc-windows-msvc"))

  `required` =

  | entry |
  |---|
  | native/agent.dll |
  | native/linalg.dll |
  | native/llm.dll |
  | native/quantum.dll |
  | native/web.dll |

1. [This is the standard-library source pack](text.ends_with(text=`name`, suffix="-stdlib.zip"))

  `required` =

  | entry |
  |---|
  | lib/fs.mq.md |
  | lib/table.mq.md |
  | lib/文件.mq.md |

1. [This is the offline documentation pack](text.ends_with(text=`name`, suffix="-public.zip"))
  We **required = [Require the documentation entry](table.append(list=`required`, item="index.html"))**.
1. [This is the tagged source snapshot](text.ends_with(text=`name`, suffix="-source.zip"))
  We **cargo_source = [Read the selected manifest metadata](table.get(map=`metadata`, key="Cargo.toml"))**.
  1. `cargo_source` == None
    Return *[Report missing source manifest](result.error(code="release.source_metadata", message=`name`, operation="assets.content"))*.
  We **manifest = [Parse the actual source package declaration](toml.parse(text=`cargo_source`))** and **package_version = [Read the package version](access(access(manifest, "package"), "version"))**.
  1. `package_version` != `version`
    Return *[Reject a source snapshot from another version](result.error(code="release.source_version", message=`name`, operation="assets.content"))*.
1. [This is the editor extension](text.ends_with(text=`name`, suffix=".vsix"))
  We **package = [Decode the VSIX package metadata](json.parse(text=access(metadata, "extension/package.json")))**.
  1. access(`package`, "version") != `version`
    Return *[Reject a mismatched VSIX version](result.error(code="release.vsix_version", message=`name`, operation="assets.content"))*.
- [Check every entry required by this role](for required as entry)
  1. not [The entry is present](table.contains(list=`entries`, item=`entry`))
    Return *[Report missing archive content](result.error(code="release.archive_content", message=`name` + ": " + entry, operation="assets.content"))*.
1. not [This is a standalone native pack](text.contains(text=`name`, sub="-native-"))
  We **platform = None**.
  1. [This is a Windows CLI bundle](text.ends_with(text=`name`, suffix="-x86_64-pc-windows-msvc.zip"))
    We **platform = "windows"**.
  1. [This is a Linux CLI bundle](text.ends_with(text=`name`, suffix="-x86_64-unknown-linux-gnu.zip"))
    We **platform = "linux"**.
  1. `platform` != None
    We **checked = [Verify the embedded source/native bundle](bundle_content(entries=`entries`, platform=`platform`, name=`name`))**.
    1. not access(`checked`, "ok")
      Return *checked*.
Return *[Keep the verified digest and content evidence](result.ok(value=`record`))*.

## download

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| directory | any | None | Relative destination; None selects dist/release-X.Y.Z |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with locally downloaded, byte-verified and content-checked assets |

Use `version`, `mode`="full", `directory`=None and `root`=".".
The destination stays inside the checkout. Files are reused only after their
size and GitHub digest are checked again; a .partial file never counts as an
artifact. Network retries try the configured GitHub proxy then the original
asset URL. The download adapter validates archive paths and CRCs without
extracting an executable. Role-specific checks below use the same version.
A failure stops the batch with its real error. Successfully verified files
remain in the destination for a retry, while no R2 pointer or PPA is changed.
plugin,fs.write,network.read,environment.read includes the adapter's local writes and network authority; the
working root is a location constraint, not a subprocess permission sandbox.

We **catalog_result = [Require complete GitHub metadata first](catalog(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`catalog_result`, "ok")
  Return *catalog_result* with its failure intact.

We **catalog_data = [Read the accepted metadata](access(catalog_result, "value"))** and
**required = [Read required names](access(access(catalog_data, "plan"), "required"))** and
**assets = [Read their GitHub size and digest records](access(catalog_data, "assets"))**.
1. `directory` == None
  We **directory = "dist/release-" + version** for the release staging directory.
We **checked = [Start the verified artifact list](table.empty_list())**.
- [Download and verify each required artifact](for required as name)
  We **asset = [Select its unmodified metadata](named(assets=`assets`, name=`name`))**.

  `payload` =

  | asset | directory |
  |---|---|
  | `asset` | `directory` |

  We **fetched = [Transfer exact GitHub bytes and validate digest and CRC](native.fetch(asset=`asset`, directory=`directory`, root=`root`))**.
  1. not access(`fetched`, "ok")
    Return *fetched* with the completed earlier files retained.
  We **verified = [Check the package's actual role and version](content(record=access(fetched, "value"), version=`version`))**.
  1. not access(`verified`, "ok")
    Return *verified* before any publication step.
  We **checked = [Append the checked evidence](table.append(list=`checked`, item=access(verified, "value")))**.
We **consistent = [Check source/native capabilities across the complete set](matrix(records=`checked`))**.
1. not access(`consistent`, "ok")
  Return *consistent* before publication.
We **evidence = [Keep concise observed paths and hashes](summaries(records=`checked`, directory=`directory`))**.
Return *[The real verified local artifacts](result.ok(value=`evidence`))*.

## status

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with CDN parity, version pointers and checked URLs |

Use `version`, `mode`="full" and `root`="." to verify published extension
bytes. CLI mode has no CDN obligation and reports that explicitly. Other modes
require both the versioned and latest download for each of the three packs, plus
both VERSION pointers. Their SHA256 values must match the GitHub-provided digest
for the exact release. Missing digests remain a failure; the task does not infer
integrity from a filename, HTTP status or a previous version's pointer.
This operation is read-only but downloads the bytes to compute their hashes.
A pointer can be published before every edge cache refreshes; any mismatch keeps
complete=False and retains the URL evidence so a later query can verify it.

We **catalog_result = [Read the same release's asset metadata](catalog(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`catalog_result`, "ok")
  Return *catalog_result* with its failure intact.

We **catalog_data = [Read the accepted catalog](access(catalog_result, "value"))** and
**checks = [Start public endpoint evidence](table.empty_list())** and **complete = True**.
1. `mode` != "cli"
  We **names = [Read the three CDN deliverables](access(access(catalog_data, "plan"), "cdn"))** and
  **assets = [Read the exact GitHub asset digests](access(catalog_data, "assets"))**.

  `prefixes` =

  | prefix |
  |---|
  | "v`version`" |
  | latest |

  - [Verify versioned and latest publication independently](for prefixes as prefix)

    We **pointer_path = prefix + "/VERSION"** as the explicit public pointer.

    We **pointer = [Read the public version pointer](native.cdn(path=`pointer_path`))**.
    1. not access(`pointer`, "ok")
      Return *pointer* with its service failure.
    We **complete = [Keep a mismatched pointer incomplete](complete and access(access(pointer, "value"), "version") == version)**.
    - [Compare every pack with its GitHub SHA256](for names as name)
      We **asset = [Keep GitHub metadata as the source of truth](named(assets=`assets`, name=`name`))**.

      We **asset_path = prefix + "/" + name** as the exact public object.

      We **observed = [Hash the actual public download](native.cdn(path=`asset_path`))**.
      1. not access(`observed`, "ok")
        Return *observed* with its failed public read.
      We **digest = [Read the observed SHA256](access(access(observed, "value"), "sha256"))** and
      **matches = [Require the same exact release bytes](table.get(map=`asset`, key="digest") == "sha256:" + digest)** and
      **complete = complete and matches**.

      `evidence` =

      | path | sha256 | matches |
      |---|---|---|
      | "`prefix`/`name`" | `digest` | `matches` |

      We **checks = [Retain this endpoint's evidence](table.append(list=`checks`, item=`evidence`))**.

We **needed = [Decide whether this mode publishes extension packs](mode != "cli")**.

`state` =

| complete | required | checks |
|---|---|---|
| `complete` | `needed` | `checks` |

Return *[The measured CDN publication state](result.ok(value=`state`))*.

## publish

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| directory | any | None | Verified staging directory; None uses dist/release-X.Y.Z |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result of verified R2 upload followed by public byte-parity checks |

Use `version`, `mode`="full", `directory`=None and `root`="." only after the
user has instructed this release. CLI-only mode has no extension publication
step. The three local ZIPs must already exist, match GitHub metadata and pass
content checks before any R2 write begins. Their combined capability matrix
must also agree: source-declared plugins require both matching native libraries.
Individually valid archives cannot hide a missing runtime on one platform. Missing credentials produce an error
with the existing uploader command available in this handbook's recovery notes.
The maintained uploader writes all versioned/latest packs before VERSION, keeping
pointer updates after content. It does not delete older versions. After upload,
this function verifies public parity; an edge-cache delay is incomplete evidence,
not a reason to upload unrelated files from the whole dist directory.

We **catalog_result = [Validate the intended release and all required filenames](catalog(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`catalog_result`, "ok")
  Return *catalog_result* with its failure intact.

1. `mode` == "cli"
  Return *[Report that this mode has no CDN publication](status(version=`version`, mode=`mode`, root=`root`))*.
1. `directory` == None
  We **directory = "dist/release-" + version** as the staging directory.
We **catalog_data = [Read the verified metadata catalog](access(catalog_result, "value"))** and
**names = [Read the exact three CDN packs](access(access(catalog_data, "plan"), "cdn"))** and
**assets = [Read their GitHub digests](access(catalog_data, "assets"))** and
**files = [Start the explicit upload path list](table.empty_list())** and
**records = [Retain the complete source and platform content evidence](table.empty_list())**.
- [Verify all local packs before touching R2](for names as name)
  We **asset = [Choose the same release asset](named(assets=`assets`, name=`name`))** and
  **path = directory + "/" + name**.

  `payload` =

  | asset | path |
  |---|---|
  | `asset` | `path` |

  We **verified = [Require local size, SHA256 and archive CRC parity](native.verify(asset=`asset`, path=`path`, root=`root`))**.
  1. not access(`verified`, "ok")
    Return *verified* before any CDN write.
  We **checked = [Require the expected contents and version](content(record=access(verified, "value"), version=`version`))**.
  1. not access(`checked`, "ok")
    Return *checked* with the source or platform error.
  We **records = [Retain the verified archive for capability matching](table.append(list=`records`, item=access(verified, "value")))** and
  **sha256 = [Retain this file's checked native hash](access(access(verified, "value"), "sha256"))**.

  `file` =

  | path | name | sha256 |
  |---|---|---|
  | `path` | `name` | `sha256` |

  We **files = [Keep only the verified upload record](table.append(list=`files`, item=`file`))**.

We **matched = [Require the source-declared plugins in both native platform packs](matrix(records=`records`))**.
1. not access(`matched`, "ok")
  Return *matched* before publishing any pack or pointer.

`upload` =

| version | files |
|---|---|
| `version` | `files` |

We **uploaded = [Upload only the three verified packs using private R2 credentials](native.upload(version=`version`, files=`files`, root=`root`))**.
1. not access(`uploaded`, "ok")
  Return *uploaded* with its failure intact.

Return *[Verify the public bytes after uploading](status(version=`version`, mode=`mode`, root=`root`))*.
