---
markup: occlusion
title: Workspace-only Rust toolchain for real WASM validation
import:
  fs: lib/fs.mq.md
  net: lib/net.mq.md
  re: lib/re.mq.md
  text: lib/text.mq.md
  table: lib/table.mq.md
  sys: lib/sys.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
---

This development tool handles one concrete environment gap: a compatible Linux x86 compiler and WASM standard library. Native Marqdo handles files, HTTP, SHA256 and decisions; the existing tar executable handles the official xz archive format. Call rust_wasm_linux, then pass its compiler path as the child RUSTC for the ordinary Marqdo wasm build command. Other operating systems use their own supported toolchain setup.

## rust_wasm_linux

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | "1.93.1" | Pinned stable Rust version for the isolated Linux x86 toolchain |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with compiler path, matching host/WASM standard libraries and native digest receipts |

Use `version`="1.93.1" on Linux x86 when the current compiler has no compatible wasm32-unknown-unknown standard library. The task prepares a complete official compiler plus its matching host and WASM standard libraries in the reserved .marqdo/toolchain cache. A distribution compiler rebuilt from source can reject official precompiled metadata even when its visible release number is identical; installing only one target archive is therefore insufficient in that case. This task changes no system compiler, global environment, shell profile or Python setup. The returned compiler path can be passed as RUSTC in a single engineering process environment.

Run from the checkout root with plugin, process.exec, fs.read, fs.write and network.read. Each download is bounded and verified against its official checksum before tar extracts a component. Failures preserve their real evidence and stop the remaining sequence. Repeating the task rechecks cached bytes rather than assuming an existing filename is valid. The final compiler probe must report the requested version before a manifest is written. The result proves toolchain preparation; a separate wasm build and browser test must still verify this repository's artifact.

1. not [A stable numeric release](re.is_match(text=`version`, pattern="^[0-9]+[.][0-9]+[.][0-9]+$"))
  Return *[Explain the required version shape](result.error(code="toolchain.version", message="use a stable numeric Rust release"))*.
We **directory = ".marqdo/toolchain/rust-linux-x86-" + version** and **receipts = [Start verified distribution receipts](table.empty_list())**.
We **[Create this task-owned workspace directory](fs.make_dirs(path=`directory`))**.

`components` =

| @ | component | target |
|---|---|---|
| 1 | rustc | x86_64-unknown-linux-gnu |
| 2 | rust-std | x86_64-unknown-linux-gnu |
| 3 | rust-std | wasm32-unknown-unknown |

- [Prepare matching compiler and target components](for components as specification)
  We **installed = [Download, verify and extract this exact component](_component(version=`version`, component=access(specification, "component"), target=access(specification, "target"), directory=`directory`))**.
  1. not access(`installed`, "ok")
    Return *installed*.
  We **receipts = [Retain its native digest receipt](table.append(list=`receipts`, item=access(installed, "value")))**.
We **compiler = sys.cwd() + "/" + directory + "/bin/rustc"** and **probed = [Verify the extracted compiler actually starts](engineering.checked(argv=table.append(list=table.append(list=table.empty_list(), item=`compiler`), item="--version"), timeout_ms=30000, max_output=32768))**.
1. not access(`probed`, "ok")
  Return *probed*.
We **reported = [Read the actual compiler version](access(access(`probed`, "value"), "stdout"))**.
1. not [The requested release was installed](text.starts_with(text=`reported`, prefix=("rustc " + `version` + " ")))
  Return *[Retain the unexpected compiler identity](result.error(code="toolchain.compiler", message=`reported`))*.

`manifest` =

| version | compiler | components |
|---|---|---|
| `version` | `compiler` | `receipts` |

We **[Save the verified workspace toolchain receipt](fs.write_json(path=(directory + "/manifest.json"), value=`manifest`))**.
Return *[Expose the compiler for an explicit child environment](result.ok(value=`manifest`))*.

## _component

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable numeric Rust release |
| component | text | required | rustc or rust-std |
| target | text | required | x86_64-unknown-linux-gnu or wasm32-unknown-unknown |
| directory | text | required | Task-owned workspace toolchain directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with the verified component digest and extraction evidence |

Use `version`, `component`, `target` and `directory` only for the fixed compiler/standard-library components selected by rust_wasm_linux. Source bytes and checksums come from the official Rust distribution over HTTPS. A cached archive is reused only after its current native SHA256 matches the fetched checksum. A truncated transfer or HTTP failure is preserved as a failed Result; extraction never runs on an unchecked archive. Downloading, hashing and directory creation use native Marqdo. Tar is the explicit system boundary for the xz distribution format, with an argument array rather than generated shell code. Extraction is restricted to this component's own archive subtree and the selected workspace directory. No Python interpreter or system installation directory participates.

The extracted files retain their official executable modes, while --no-same-owner keeps archive ownership out of this workspace installation. The tar installer script is not executed. Component receipts identify the checked archive bytes; only the later compiler probe and repository build establish that those files work together. A failed extraction leaves the task-owned cache available for a verified retry. The caller's operating system still governs the external tar process, so this helper uses the fixed official component subtree rather than a caller-supplied archive member.

1. not [A stable numeric release](re.is_match(text=`version`, pattern="^[0-9]+[.][0-9]+[.][0-9]+$"))
  Return *[Reject an invalid release name](result.error(code="toolchain.version", message="stable numeric Rust release required"))*.
1. (`component` != "rustc" and `component` != "rust-std") or (`target` != "x86_64-unknown-linux-gnu" and `target` != "wasm32-unknown-unknown") or (`component` == "rustc" and `target` != "x86_64-unknown-linux-gnu")
  Return *[Reject an unsupported component](result.error(code="toolchain.component", message="this task supports the Linux x86 compiler and its host/WASM standard libraries"))*.
We **package = component + "-" + version + "-" + target** and **url = "https://static.rust-lang.org/dist/" + package + ".tar.xz"** and **cache = directory + "/downloads"**.
We **[Create the native download cache](fs.make_dirs(path=`cache`))**.
We **archive = cache + "/" + package + ".tar.xz"** and **checksum = [Read the official bounded checksum](net.get(url=(url + ".sha256"), max_bytes=4096))**.
1. not access(`checksum`, "ok")
  Return *checksum*.
We **body = [Read the checksum text](access(access(`checksum`, "value"), "body"))** and **expected = [Select its SHA256 field](table.first(list=text.str_split(s=text.str_trim(s=`body`), sep=" ")))**.
1. not [A complete SHA256 was returned](re.is_match(text=`expected`, pattern="^[a-f0-9]{64}$"))
  Return *[Retain the invalid checksum response](result.error(code="toolchain.checksum", message="official checksum is not a SHA256 digest"))*.
We **ready = False** before inspecting this cache entry.
1. [The selected archive is already cached](fs.exists(path=`archive`))
  We **actual = [Hash its actual cached bytes](fs.file_hash(path=`archive`))** and **ready = actual == expected**.
1. not `ready`
  We **downloaded = [Stream the official component into its cache path](net.download(url=`url`, path=`archive`, timeout_ms=120000, max_bytes=268435456))**.
  1. not access(`downloaded`, "ok")
    Return *downloaded*.
We **actual = [Verify the bytes before extraction](fs.file_hash(path=`archive`))**.
1. `actual` != `expected`
  Return *[Stop on a mismatching component](result.error(code="toolchain.digest", message="downloaded Rust component SHA256 differs from the official checksum"))*.
We **member = component**.
1. `component` == "rust-std"
  We **member = "rust-std-" + target**.
We **subtree = [Select the verified component path](package + "/" + member)**.

`argv` =

| argument |
|---|
| tar |
| "--extract" |
| "--xz" |
| "--no-same-owner" |
| "--file" |
| `archive` |
| "--directory" |
| `directory` |
| "--strip-components=2" |
| `subtree` |

We **extracted = [Extract only this verified distribution component](engineering.checked(argv=`argv`, timeout_ms=120000, max_output=32768))**.
1. not access(`extracted`, "ok")
  Return *extracted*.

`receipt` =

| package | sha256 |
|---|---|
| `package` | `actual` |

Return *[Retain the verified component identity](result.ok(value=`receipt`))*.
