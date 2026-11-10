---
markup: occlusion
title: Independent extension-pack build tasks
import:
  model: ./model.mq.md
  github: ./github.mq.md
  fs: lib/fs.mq.md
  path: lib/path.mq.md
  sys: lib/sys.mq.md
  text: lib/text.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
---

This leaf builds source and one platform's native extension archives for an authorized version. It does not publish, choose a version, create a tag or edit version files. The extension-release workflow runs the same tasks on Windows and Linux after local.tag pushes ext-vX.Y.Z. Its attach_ci task creates the exact tagged release through the native service boundary and attaches three assets with the existing gh upload tool; assets.publish subsequently verifies those exact bytes before R2 publication.

Build all six maintained native plugins first. The native_dir contract is a directory containing their platform library names, normally target/release. Source discovery and staging use native Marqdo filesystem operations. Only ZIP encoding uses the existing platform tool: Info-ZIP on Linux, the installed bsdtar on Windows. Compiler and format boundaries do not require Python or a generated shell program.

## libraries

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| platform | text | required | x86_64-pc-windows-msvc or x86_64-unknown-linux-gnu |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the six exact native library filenames |

Use `platform` to select the format names rather than guessing suffixes from one local file. Both platform packs must contain web, agent, quantum, linalg, llm and engineering. This pure contract neither proves ABI compatibility nor substitutes for compiling on the selected platform. Unknown targets fail before a staging directory is created.

1. `platform` == "x86_64-pc-windows-msvc"

  `names` =

  | name |
  |---|
  | web.dll |
  | agent.dll |
  | quantum.dll |
  | linalg.dll |
  | llm.dll |
  | engineering.dll |

  Return *[The complete Windows native contract](result.ok(value=`names`))*.
1. `platform` == "x86_64-unknown-linux-gnu"

  `names` =

  | name |
  |---|
  | libweb.so |
  | libagent.so |
  | libquantum.so |
  | liblinalg.so |
  | libllm.so |
  | libengineering.so |

  Return *[The complete Linux native contract](result.ok(value=`names`))*.
Return *[Reject an unsupported native target](result.error(code="packs.platform", message=`platform`, operation="libraries"))*.

## _archive

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| stage | text | required | Owned staging root with one selected ext or native directory |
| entry | text | required | ext or native directory to encode |
| output | text | required | New ZIP path under the same filesystem root |
| platform | text | required | Already validated platform target |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Checked ZIP encoder result; an existing output is never replaced |

Use `stage`, `entry`, `output` and `platform` only after build preflight. Native resolution validates the stage and output paths first; the encoder receives individual arguments and one named staging subtree. It cannot expand a generated shell expression. A failed encoder can leave the selected incomplete archive for inspection; this task never reuses it as successful output or deletes unrelated files.

The Windows runner supplies bsdtar with ZIP format support; Linux supplies Info-ZIP. Format encoding is the only external operation here. Reads, directory creation, copies, hashes and final ZIP validation remain separate explicit native tasks. The caller removes only its owned temporary stage after the child exits. Existing archives require inspection or a different output directory, keeping reruns from silently replacing publication candidates.

1. [This output already belongs to an earlier attempt](fs.exists(path=`output`))
  Return *[Preserve the existing archive](result.error(code="packs.output_exists", message=`output`, operation="_archive"))*.
We **destination = [Resolve the new archive inside the granted root](fs.resolve(path=`output`))**.

`argv` =

| argument |
|---|
| zip |
| "-q" |
| "-r" |
| `destination` |
| `entry` |

1. `platform` == "x86_64-pc-windows-msvc"

  `argv` =

  | argument |
  |---|
  | tar |
  | "--format=zip" |
  | "-cf" |
  | `destination` |
  | `entry` |

Return *[Encode only the explicitly staged format subtree](engineering.checked(argv=`argv`, cwd=`stage`, timeout_ms=300000, max_output=65536))*.

## _receipt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| file | text | required | Completed archive selected by build |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with path, SHA256, size and validated archive entries |

Use `file` after the encoder exits successfully. Hash the resulting bytes through the native filesystem, then inspect ZIP contents with the maintained archive boundary. CRC, traversal, duplicate-path and expansion checks apply to actual archive bytes; a process exit alone does not establish a valid publication candidate. A later upload still rechecks this digest.

A receipt is local evidence, not a declaration that both platforms have been built. The format boundary reads the named archive without extracting it. Failure remains a failed Result with its original diagnostic. Keep completed archives and receipts for review; the build task cleans temporary staging, not its explicitly requested outputs.

We **sha256 = [Hash the actual encoded bytes](fs.file_hash(path=`file`))** and **metadata = [Read the native file size](fs.stat(path=`file`))** and **inspected = [Validate actual ZIP entries and CRCs](engineering.inspect_zip(path=`file`))**.
1. not access(`inspected`, "ok")
  Return *inspected*.
We **size = [Retain the actual byte count](access(metadata, "size"))** and **archive = [Retain validated archive evidence](access(inspected, "value"))**.

`receipt` =

| path | sha256 | size | archive |
|---|---|---|---|
| `file` | `sha256` | `size` | `archive` |

Return *[Expose the verified local candidate](result.ok(value=`receipt`))*.

## _stage_sources

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| stage | text | required | Newly created temporary root owned by build |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None after native source copies; filesystem failures remain diagnostics |

Use `stage` with fs.write on the checkout root. Discover source files under ext while excluding native binaries and generated caches. Each returned path stays relative to ext, preserving module organization, language mirrors and VERSION. Copies use the native root and symlink policy; this task does not call cp, PowerShell file commands or an archive tool to enumerate authoring data.

The stage is private to this invocation. No existing distribution tree is cleared, and no source file is edited. Parent directories are created only for selected copies. Keeping enumeration separate from ZIP encoding makes the source list visible in Marqdo rather than embedding repository policy in a shell script.

`excluded` =

| pattern |
|---|
| native/** |
| **/target/** |
| **/node_modules/** |
| **/.git/** |

We **files = [Find maintained extension source files](fs.find(path="ext", pattern="**", exclude=`excluded`))**.
- [Copy only the discovered source entries](for files as file)
  We **target = stage + "/ext/" + file** and **parent = [Find the selected native destination parent](path.parent(path=`target`))**.
  We **[Create that destination directory](fs.make_dirs(path=`parent`))** and **[Copy this exact source file](fs.copy_file(src="ext/" + file, dest=`target`))**.
Return *None* after all explicit source copies finish.

## build

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable extension version already present in ext/VERSION |
| platform | text | required | One supported native target compiled by this runner |
| directory | text | "dist/ext-release" | New output directory under the granted checkout root |
| native_dir | text | "target/release" | Existing directory containing all six compiled native libraries |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing source and current-platform native archive receipts |

Use `version`, `platform`, `directory`="dist/ext-release" and `native_dir`="target/release". Grant plugin and fs.write from the checkout root. Stable version, supported platform, ext/VERSION, all six library files and both absent destination names are checked before any stage is created. This task packages previously compiled libraries; it does not claim to cross-compile Windows from Linux.

The source archive contains the readable ext tree without native binaries. The platform archive contains six libraries under native. Every file copy and output digest uses native filesystem tools, with only ZIP encoding at the existing external-tool boundary. Stage cleanup happens after encoding returns; a native filesystem diagnostic may retain the owned stage for inspection. Existing output archives are preserved rather than overwritten on a retry.

We **plan = [Validate the confirmed independent-pack version](model.plan(version=`version`, mode="ext"))** and **selected = [Select the complete platform library contract](libraries(platform=`platform`))**.
1. not access(`plan`, "ok")
  Return *plan*.
1. not access(`selected`, "ok")
  Return *selected*.
1. [The actual extension version differs](text.str_trim(s=fs.read_text(path="ext/VERSION")) != `version`)
  Return *[Require reviewed extension metadata first](result.error(code="packs.version", message="ext/VERSION differs from the requested pack", operation="build"))*.
We **names = [Read the six required library names](access(selected, "value"))**.
- [Preflight all actual native files before staging](for names as name)
  1. not [The compiled library exists](fs.exists(path=(`native_dir` + "/" + `name`)))
    Return *[Keep the missing native capability visible](result.error(code="packs.library", message=`name`, operation="build"))*.
We **source = directory + "/marqdo-" + version + "-ext.zip"** and **native = directory + "/marqdo-" + version + "-native-" + platform + ".zip"**.
1. [The source output already exists](fs.exists(path=`source`)) or [The native output already exists](fs.exists(path=`native`))
  Return *[Preserve all existing output candidates](result.error(code="packs.output_exists", message="inspect existing archives or choose a new directory", operation="build"))*.
We **[Create the requested output directory](fs.make_dirs(path=`directory`))** and **stage = [Allocate an owned native staging directory](fs.temp_dir(prefix=".release-pack"))**.
We **[Copy readable extension sources](_stage_sources(stage=`stage`))** and **[Create the native library subtree](fs.make_dirs(path=stage + "/native"))**.
- [Copy each selected compiled library](for names as name)
  We **[Retain the exact compiled bytes](fs.copy_file(src=native_dir + "/" + name, dest=stage + "/native/" + name))**.
We **packed = [Encode the source subtree](_archive(stage=`stage`, entry="ext", output=`source`, platform=`platform`))**.
1. access(`packed`, "ok")
  We **packed = [Encode the platform native subtree](_archive(stage=`stage`, entry="native", output=`native`, platform=`platform`))**.
We **[Remove only this invocation's completed staging tree](fs.remove_tree(path=`stage`))**.
1. not access(`packed`, "ok")
  Return *packed*.
We **source_receipt = [Validate the actual source archive](_receipt(file=`source`))** and **native_receipt = [Validate the actual platform archive](_receipt(file=`native`))**.
1. not access(`source_receipt`, "ok")
  Return *source_receipt*.
1. not access(`native_receipt`, "ok")
  Return *native_receipt*.
We **sources = [Keep source candidate evidence](access(source_receipt, "value"))** and **platform_pack = [Keep native candidate evidence](access(native_receipt, "value"))**.

`receipt` =

| version | platform | source | native |
|---|---|---|---|
| `version` | `platform` | `sources` | `platform_pack` |

Return *[Expose both verified local pack candidates](result.ok(value=`receipt`))*.

## ci

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result from building the current runner's user-tagged independent pack |

Run this task only in the extension-release workflow after its compiler steps. Grant environment.read, fs.write and plugin. The runner provides GITHUB_REF_NAME and RUNNER_OS; neither is interpolated into generated shell source. Require the ext-v prefix, validate its stable version through build, and map only the two known runner operating systems to their supported targets.

This entry does not create a remote release. The workflow's separate attach_ci task attaches the exact output names after local validation. Windows contributes the source and Windows native pack; Linux contributes its native pack. CI status must subsequently cover both jobs on the exact annotated-tag commit. A failed task stops the upload step through --require-ok; no stale package replaces it.

We **tag = [Read the actual pushed tag](sys.env_get(name="GITHUB_REF_NAME"))** and **runner = [Read the actual compilation runner](sys.env_get(name="RUNNER_OS"))**.
1. `tag` == None or not [The tag selects independent extension publication](text.starts_with(text=`tag`, prefix="ext-v"))
  Return *[Reject an unrelated ref](result.error(code="packs.tag", message="expected ext-vX.Y.Z", operation="ci"))*.
We **version = [Read the tag's stable version component](text.replace(text=`tag`, old="ext-v", new="", count=1))** and **platform = ""**.
1. `runner` == "Windows"
  We **platform = "x86_64-pc-windows-msvc"**.
1. `runner` == "Linux"
  We **platform = "x86_64-unknown-linux-gnu"**.
We **built = [Build the exact runner and tag contract](build(version=`version`, platform=`platform`))**.
1. not access(`built`, "ok")
  Return *built*.
We **[Create the receipt directory through native IO](fs.make_dirs(path=".marqdo/release"))** and **[Retain exact bytes for the separate upload step](fs.write_json(path=".marqdo/release/packs.json", value=access(built, "value")))**.
Return *built*.

## _unchanged

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| receipt | map | required | Local path/SHA256 evidence returned by this runner's build |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether the selected archive still has the recorded byte identity |

Use `receipt` before an upload mutation. Native filesystem hashing checks the exact file selected by the build rather than trusting its name, timestamp or successful encoder exit. A missing file remains a filesystem diagnostic. A changed digest returns False and prevents release creation as well as upload. This is a preflight drift check, not a lock against concurrent writers; CI owns its output directory during publication.

Return *[Recheck the recorded candidate's actual bytes](fs.file_hash(path=access(receipt, "path")) == access(receipt, "sha256"))*.

## attach_ci

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Checked result of attaching this runner's unchanged archives without replacement |

Run after ci in the independent extension workflow, with plugin, fs.read and environment.read. The GitHub runner supplies its existing GH_TOKEN privately to the installed gh tool; credentials never become arguments or journal values. Read the saved build receipt, pushed tag and source SHA first. Only the two supported runner names are accepted, and the receipt must match that runner and exact version.

Before any service mutation, rehash both local archives and read the reviewed notes at doc/releases/ext-X.Y.Z.md. The release source must commit those bilingual notes before its immutable tag is pushed. create_ext verifies that remote tag's commit equals GITHUB_SHA, creates a missing release once, and preserves an existing release. An uncertain creation remains failed for explicit observation, not automatic retry.

Windows attaches the source archive and Windows native archive. Linux attaches only its native archive. The existing gh release upload command is an explicit service-tool boundary; no --clobber flag is used, so rerunning cannot silently replace a prior asset. An already attached file requires inspection and byte comparison rather than deleting or overwriting it. Both platform CI jobs and later GitHub/R2 parity still must pass before finish can declare the release complete.

We **tag = [Read the actual pushed tag](sys.env_get(name="GITHUB_REF_NAME"))** and **runner = [Read the actual build runner](sys.env_get(name="RUNNER_OS"))** and **sha = [Read the runner's exact source commit](sys.env_get(name="GITHUB_SHA"))**.
1. `tag` == None or `sha` == None or not [The ref selects extension publication](text.starts_with(text=`tag`, prefix="ext-v"))
  Return *[Require an explicit extension CI context](result.error(code="packs.ci_context", message="missing extension tag or commit", operation="attach_ci"))*.
We **version = [Read the selected stable version](text.replace(text=`tag`, old="ext-v", new="", count=1))** and **plan = [Validate that version before choosing paths](model.plan(version=`version`, mode="ext"))**.
1. not access(`plan`, "ok")
  Return *plan*.
We **platform = ""**.
1. `runner` == "Windows"
  We **platform = "x86_64-pc-windows-msvc"**.
1. `runner` == "Linux"
  We **platform = "x86_64-unknown-linux-gnu"**.
We **supported = [Reject unsupported compilation runners](libraries(platform=`platform`))**.
1. not access(`supported`, "ok")
  Return *supported*.
We **receipt = [Read this runner's actual native build receipt](fs.read_json(path=".marqdo/release/packs.json"))**.
1. access(`receipt`, "version") != `version` or access(`receipt`, "platform") != `platform`
  Return *[Reject stale or foreign runner evidence](result.error(code="packs.receipt", message="receipt version/platform mismatch", operation="attach_ci"))*.
We **source = [Read the source archive identity](access(receipt, "source"))** and **native = [Read the platform archive identity](access(receipt, "native"))**.
1. access(`source`, "path") != ("dist/ext-release/marqdo-" + `version` + "-ext.zip") or access(`native`, "path") != ("dist/ext-release/marqdo-" + `version` + "-native-" + `platform` + ".zip")
  Return *[Require the exact runner output names](result.error(code="packs.receipt_path", message="receipt paths differ from the selected outputs", operation="attach_ci"))*.
1. not [_unchanged source bytes](_unchanged(receipt=`source`)) or not [Unchanged native bytes](_unchanged(receipt=`native`))
  Return *[Stop before publication when local bytes drift](result.error(code="packs.changed", message="archive changed after build", operation="attach_ci"))*.
We **published = [Create or observe the exact release from reviewed committed notes](github.create_ext(version=`version`, path="doc/releases/ext-" + version + ".md", sha=`sha`))**.
1. not access(`published`, "ok")
  Return *published*.

`argv` =

| argument |
|---|
| gh |
| release |
| upload |
| `tag` |
| "--repo" |
| "cflmy/marqdo" |

1. `runner` == "Windows"
  We **argv = [Attach the source pack from the Windows runner](table.append(list=`argv`, item=access(source, "path")))**.
We **argv = [Attach only this runner's native pack](table.append(list=`argv`, item=access(native, "path")))**.
Return *[Upload the verified candidates without replacing existing assets](engineering.checked(argv=`argv`, timeout_ms=300000, max_output=65536))*.
