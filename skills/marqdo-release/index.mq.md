---
markup: occlusion
title: Marqdo release as an executable handbook
import:
  model: ./model.mq.md
  local: ./local.mq.md
  github: ./github.mq.md
  assets: ./assets.mq.md
  ubuntu: ./ubuntu.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
---

这份入口替代需要智能体重新拼装命令的发版 SKILL。版本和 full/cli/ext 模式必须由用户选择；已有明确选择时沿用授权，不重复询问。读取本手册或检查旧版本不会授权再次发布。当前维护版本的真实只读验证可调用 status 或 verify，并传入 version="2.6.0"。

先调用 plan 查看版本对应的产物与步骤，再由 local.inspect 查看本地状态。local.bump 修订版本，作者用开发手册的原生文件工具撰写双语 CHANGELOG、发布说明与用户文档，local.docs 构建真实文档。需要 CLI 的模式先编译独立编辑器分支，再显式提交审核过的文件并推送 main；EK 在实际提交上变绿后才创建新的不可变标签。每一步保留真实 Result，不通过资产存在来掩盖失败。

多文件工程按职责分开：model 是纯产物契约；local 是本地元数据、Git 与编辑器任务；github 是提交身份及 CI 证据；assets 与 asset_io 是同字节下载、内容检查和 R2 发布；ubuntu 与 package 是 noble 源码构建及 apt 验证。标准库的文件读写、JSON/TOML、哈希、网络、日期和表格负责组合，工程插件仅承担进程、认证和压缩格式边界。开发流程不需要 Python。独立拓展包在 packs.mq.md 中通过原生文件能力暂存与检查，并外联既有 ZIP 工具；ext-v 标签触发 extension-release.yml，两个平台均使用 packs.ci 与 packs.attach_ci 完成构建、精确来源检查、创建 Release 和不覆盖已有资产的上传。创建标签前须提交 doc/releases/ext-X.Y.Z.md 双语说明。

发布必须保留以下约束：main 不跟踪 vscode-extension 源码；标签不可覆盖；不得强推或绕过钩子；full 必须具备 Windows 和 Linux 产物。拓展包同时发布到 GitHub 和 R2，所有 ZIP 先于 VERSION 指针上传。full/cli 还需 Ubuntu PPA 源码上传、构建和 apt 发布证据。Release 与 EK 都应在正确来源提交上成功。双语说明要写实际核心表面变化及验证限制，不能只用模板宣称兼容。

凭据留在原有私密存储，不作为调用参数、源码或会话绑定。网络失败先走配置代理并有界重试，再直连；仍失败时保留本地证据，使用明确配置的外联传输，不能假装完成。sudo、GPG 或 dput 若需要人工输入，报告失败阶段及具体的 marqdo call 恢复命令。gh-pages 部署需要另外的用户指令。

命令形式是 marqdo call skills/marqdo-release/index.mq.md plan 'version="2.6.0"' --json。源码仓库开发时使用已构建的 target/debug/marqdo；先按开发入口 prepare_engineering 准备插件。status 和 verify 都使用 plugin,fs.write,network.read,environment.read，用于本地暂存和公开下载的校验；preflight 只需 plugin,fs.read，plan 不需要效应授权。自动化串接可加 --require-ok：函数返回失败 Result 时退出码非零，并保留真实返回值。所有命令都从仓库根目录传 --fs-root .。

## plan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z; inspection never chooses another version |
| mode | text | "full" | full, cli or ext |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with exact deliverable names and task routing |

Use `version` and `mode`="full" after the user's version decision.
This pure entry derives filenames and publication obligations; it grants no
effect and writes no resource. Full has ten GitHub deliverables and both native
platforms, cli keeps the independent pack version, and ext has three packs on
both GitHub and R2. Unknown modes and malformed versions fail before IO.

Follow the returned task routing to the actual callable files, whose headings
specify their inputs, outputs and failures. The routing is not a shell program
for the model to reconstruct. Inspecting the plan for an existing version is
safe and does not authorize a replacement release or a new immutable tag.

Return *[Derive the confirmed release contract](model.plan(version=`version`, mode=`mode`))*.

## preflight

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z; inspection never chooses another version |
| mode | text | "full" | full, cli or ext |
| root | text | "." | Checkout/artifact root matching --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with ready, plan and actual local checkout facts |

Use `version`, `mode`="full" and `root`="." before preparing a
release. This reads actual Git state and native version files; it does not fetch,
reset, stage, sign or publish. A clean main checkout without tracked editor
sources and without the selected local tag is ready for preparation. Existing
tags require inspection of their release state rather than replacement.

A successful Result may contain ready=False. That is a real observation, not
permission to discard dirty files or overwrite refs. During authorized release
preparation, intentional edits remain visible until the explicit commit task.
The later tag task checks matching versions, remote main identity and EK again,
so preflight cannot become stale authorization. Call with plugin,fs.read and a
filesystem root that identifies the same checkout as the plugin's root.

We **planned = [Validate the requested release contract](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned*.
We **observed = [Read the real checkout without changing it](local.inspect(root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed*.
We **plan_data = [Keep the pure release plan](access(planned, "value"))** and **state = [Keep the actual Git and version facts](access(observed, "value"))**.
We **exists = [Preserve any existing immutable tag](table.contains(list=access(state, "tags"), item=access(plan_data, "tag")))** and
**ready = access(state, "branch") == "main" and access(state, "changes") == "" and access(state, "extension_on_main") == "" and not exists**.

`inspection` =

| ready | plan | checkout |
|---|---|---|
| `ready` | `plan_data` | `state` |

Return *[The preparation decision with its evidence](result.ok(value=`inspection`))*.

## status

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z; inspection never chooses another version |
| mode | text | "full" | full, cli or ext |
| root | text | "." | Checkout/artifact root matching --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with source-specific GitHub CI, CDN parity and Ubuntu state |

Use `version`, `mode`="full" and `root`="." for an existing release.
This entry independently checks GitHub publication and its exact source commits,
the versioned and latest CDN bytes when packs ship, and Ubuntu publication when
the CLI ships. It creates no tag, release, upload or apt mutation. Temporary
download files are owned and cleaned by the native CDN task.

The result keeps each component's evidence and requires all applicable components
to complete. A missing or red gate, different digest, pending build or stale apt
index remains incomplete. Network failures are failed Results, not fabricated
absence. Call with plugin,fs.write,network.read,environment.read: local writes
are only bounded verification staging. An ext-only release skips Ubuntu; a
cli-only release skips CDN. Existing GitHub assets alone never prove completion.

We **source = [Observe publication and CI on the actual source identities](github.status(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`source`, "ok")
  Return *source*.
We **cdn = [Verify published pack hashes and VERSION pointers](assets.status(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`cdn`, "ok")
  Return *cdn*.
We **ppa = None** and **ppa_complete = True** for an extension-only release.
1. `mode` != "ext"
  We **observed = [Check source publication, builds and the actual apt index](ubuntu.status(version=`version`))**.
  1. not access(`observed`, "ok")
    Return *observed*.
  We **ppa = [Retain the real Ubuntu evidence](access(observed, "value"))** and **ppa_complete = [Read its independent completion state](access(ppa, "complete"))**.
We **github_state = [Keep GitHub evidence](access(source, "value"))** and **cdn_state = [Keep CDN evidence](access(cdn, "value"))** and
**complete = access(github_state, "complete") and access(cdn_state, "complete") and ppa_complete**.

`state` =

| complete | github | cdn | ubuntu |
|---|---|---|---|
| `complete` | `github_state` | `cdn_state` | `ppa` |

Return *[The independently measured release state](result.ok(value=`state`))*.

## verify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z; inspection never chooses another version |
| mode | text | "full" | full, cli or ext |
| root | text | "." | Checkout/artifact root matching --fs-root |
| directory | any | None | Staging directory; None means dist/release-X.Y.Z |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with all verified local artifacts and published completion evidence |

Use `version`, `mode`="full", `root`="." and `directory`=None to
verify an existing release from its actual bytes. First download or reuse every
required GitHub artifact, check size and digest, inspect archive paths and CRCs,
and verify each package's role and version. Then status checks independent
publication channels. Existing valid local files may be reused after rechecking.

This is verification, not publication. It needs plugin,fs.write,network.read,
environment.read and can consume network time for all platform packs. Failed
checks preserve their source error and completed local files for diagnosis;
none triggers a re-upload. Full completion includes Linux and Windows, green
source-specific CI, matching CDN copies and Ubuntu's actual apt index. Keep the
returned hashes as release evidence. A later release still needs its own user
version decision and source review.

We **downloaded = [Validate all required GitHub asset bytes and package contents](assets.download(version=`version`, mode=`mode`, directory=`directory`, root=`root`))**.
1. not access(`downloaded`, "ok")
  Return *downloaded*.
We **observed = [Verify all applicable published channels](status(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed*.
We **artifacts = [Retain exact local hash evidence](access(downloaded, "value"))** and **publication = [Retain independent remote evidence](access(observed, "value"))** and **complete = [Read the real completion decision](access(publication, "complete"))**.

`evidence` =

| complete | artifacts | publication |
|---|---|---|
| `complete` | `artifacts` | `publication` |

Return *[The verified bytes and actual publication state](result.ok(value=`evidence`))*.


## finish

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z |
| mode | text | "full" | full, cli or ext |
| root | text | "." | Checkout/artifact root matching --fs-root |
| directory | any | None | Local artifact staging directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result succeeding only when all verified release obligations are complete |

Use `version`, `mode`="full", `root`="." and `directory`=None at the
final verification step of an authorized release. This runs verify on actual
artifact bytes and independent published channels. It does not create another
release, overwrite assets or repeat an upload.

Unlike an observational status query, this completion gate returns a failed
Result when any applicable obligation is pending. The failure retains its
evidence record so a reviewer can identify the unfinished channel. Use
--require-ok when this task is the last step of external automation; a pending
release then produces a nonzero exit status. It needs the same native staging,
network, environment and plugin grants as verify. A service failure retains its
original source diagnostic. Only this successful final gate permits reporting
the chosen release complete; changing the version still requires the user's
explicit decision.

We **observed = [Verify actual artifacts and every applicable publication channel](verify(version=`version`, mode=`mode`, root=`root`, directory=`directory`))**.
1. not access(`observed`, "ok")
  Return *observed* with its original failure.
We **evidence = [Retain the complete verification record](access(observed, "value"))**.
1. not access(`evidence`, "complete")
  We **failure = [Keep pending publication distinct from successful completion](result.error(code="release.incomplete", message="inspect evidence for the unfinished release obligation", operation="finish"))**.
  Return *[Retain evidence with the failed completion gate](table.put(in=`failure`, at="evidence", value=`evidence`))*.
Return *observed* when every required obligation is verified.
