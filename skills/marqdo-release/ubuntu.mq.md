---
markup: occlusion
title: Ubuntu publication: source, build and apt evidence
import:
  net: lib/net.mq.md
  json: lib/json.mq.md
  text: lib/text.mq.md
  table: lib/table.mq.md
  re: lib/re.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
  local: ./local.mq.md
  model: ./model.mq.md
---

Ubuntu 发布分为源码接收、构建成功和 apt 索引发布三个状态。这里用 Marqdo 固有网络库读取公开证据；gzip 格式交给工程拓展解码，不启动 Python。ship 是独立的外部写入任务，只有实际发版授权才能调用。status 只读，不会尝试重新上传已经存在的版本。

源包准备和文件修订在 package.mq.md；Debian 工具链、GPG 和 dput 通过维护中的系统脚本外联。sudo、签名口令或 Launchpad 拒绝都是真实失败，需要报告确切阶段与修复命令。已接收的 orig 归档不能改字节重传，打包修复只递增 Debian 修订号。

## read_json

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Public Launchpad JSON URL |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing decoded response JSON |

Use `url` for an explicitly selected public metadata endpoint. Native net.get
returns an HTTP response separately from transport success. Require status 200
before decoding its body: a 404 HTML page must never become an empty publication
record. The helper reads at most one MiB with a bounded timeout and keeps the
original transport failure. No authentication, subprocess or temporary file is
needed for these public facts.

This helper does not guess whether an upstream version has shipped. Its caller
selects matching source publications and builds from the returned records.
A schema change remains visible to that caller. Reading a successful response
is evidence for that endpoint at this time, not permission to upload a new
package. Call with network.read; further gzip decoding is a separate capability.

We **fetched = [Read bounded public Launchpad JSON](net.get(url=`url`, timeout_ms=30000, max_bytes=1048576))**.
1. not access(`fetched`, "ok")
  Return *fetched* with the transport diagnostic.
We **response = [Inspect the actual HTTP response](access(fetched, "value"))**.
1. access(`response`, "status") != 200
  Return *[Retain the unsuccessful HTTP status](result.error(code="ubuntu.http", message=str(access(response, "status")), operation="read_json"))*.
We **body = [Decode the successful JSON body](json.parse(text=access(response, "body")))**.
Return *[The service's actual metadata](result.ok(value=`body`))*.

## source

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z |
| series | text | "noble" | Ubuntu series being observed |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the newest matching published source or None |

Use `version` and `series`="noble" to locate a published upstream package.
The pure release model first validates the upstream version. Launchpad returns
publication history ordered by date; select the first entry with this upstream
version followed by a Debian revision and the exact target distro series.
A newer unrelated release or another Ubuntu series cannot satisfy this check.

None is a successful observation that this version is not currently published.
An HTTP failure remains a failed Result and must not be reported as absence.
A source publication alone does not establish successful binary compilation:
builds and apt are separate tasks below. This query is public and requires
network.read. It changes neither Launchpad state nor local version metadata,
and it never infers the user's next version from the newest remote package.

We **planned = [Validate the selected upstream version](model.plan(version=`version`))**.
1. not access(`planned`, "ok")
  Return *planned*.
We **observed = [Read ordered publication history](read_json(url="https://api.launchpad.net/1.0/~cflmy/+archive/ubuntu/marqdo?ws.op=getPublishedSources&source_name=marqdo&exact_match=true&status=Published&order_by_date=true"))**.
1. not access(`observed`, "ok")
  Return *observed*.
We **entries = [Read actual publication records](access(access(observed, "value"), "entries"))**.
- [Choose the exact upstream and distro series](for entries as entry)
  1. [The Debian revision belongs to this upstream](text.starts_with(text=access(`entry`, "source_package_version"), prefix=`version` + "-")) and access(`entry`, "distro_series_link") == "https://api.launchpad.net/1.0/ubuntu/" + `series`
    Return *[The matching source publication](result.ok(value=`entry`))*.
Return *[No matching publication exists](result.ok(value=None))*.

## builds

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| publication | any | required | Source publication returned by source, or None |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with complete and the selected source's actual builds |

Use `publication` from the source task, rather than a build number copied
from a previous release. Launchpad's source record supplies the build collection.
The required amd64 build must report Successfully built. Keep all returned build
records for diagnosis, including logs and a pending or failed status.

A missing source or an empty build collection remains incomplete. Do not treat
the successful metadata request as successful compilation. The task deliberately
does not retry or rescore Launchpad jobs; those are separate external mutations
that need a concrete packaging fix and release authorization. This read requires
network.read. Successful compilation still leaves the apt publication check:
the publisher may need time to expose the binary package in the repository
index after a build turns green.

We **entries = [Start with no invented build records](table.empty_list())** and **complete = False**.
1. `publication` != None
  We **observed = [Read builds belonging to this source publication](read_json(url=access(publication, "self_link") + "?ws.op=getBuilds"))**.
  1. not access(`observed`, "ok")
    Return *observed*.
  We **entries = [Keep the real build evidence](access(access(observed, "value"), "entries"))**.
  - [Require the intended Linux architecture](for entries as build)
    1. access(`build`, "arch_tag") == "amd64" and access(`build`, "buildstate") == "Successfully built"
      We **complete = True** because this source's amd64 binary compiled.

`state` =

| complete | entries |
|---|---|
| `complete` | `entries` |

Return *[The actual source-specific build state](result.ok(value=`state`))*.

## apt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z |
| series | text | "noble" | Ubuntu series being observed |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with complete, package_version and the public index URL |

Use `version` and `series`="noble" to inspect the actual public amd64 apt
index. Launchpad publishes Packages.gz; the uncompressed URL may return 404.
Native net.get reads portable bytes, then the engineering format boundary
validates gzip and decodes bounded UTF-8 text. No shell pipeline, local apt cache
or Python parser substitutes for the remote index.

Find the marqdo package stanza and its exact upstream prefix plus Debian revision.
A different package's Version line cannot satisfy this check. A successful build
may precede index publication, so complete=False is an honest intermediate state.
Transport, gzip and schema failures retain their diagnostics. The task needs
network.read and plugin and writes no file. It proves repository visibility at
this instant; installation on a particular user's machine additionally depends
on their configured source and apt refresh.

We **url = "https://ppa.launchpadcontent.net/cflmy/marqdo/ubuntu/dists/" + series + "/main/binary-amd64/Packages.gz"** and
**fetched = [Read the actual compressed apt index](net.get(url=`url`, response="bytes", max_bytes=16777216))**.
1. not access(`fetched`, "ok")
  Return *fetched*.
We **response = [Inspect its HTTP status](access(fetched, "value"))**.
1. access(`response`, "status") != 200
  Return *[Retain index unavailability](result.error(code="ubuntu.apt_http", message=str(access(response, "status")), operation="apt"))*.
We **decoded = [Validate and decode the gzip format](engineering.gzip_text(data=access(response, "body")))**.
1. not access(`decoded`, "ok")
  Return *decoded*.
We **stanzas = [Keep package fields in their own records](text.str_split(s=access(decoded, "value"), sep="\n\n"))** and **complete = False** and **package_version = None**.
- [Inspect only the marqdo package's version](for stanzas as stanza)
  1. [This is the marqdo package stanza](text.starts_with(text=`stanza`, prefix="Package: marqdo\n"))
    We **line = [Read its Version field](re.find(text=`stanza`, pattern="(?m)^Version: [^\n]+"))**.
    1. `line` != None
      We **candidate = [Remove the field label](text.replace(text=`line`, old="Version: ", new="", count=1))**.
      1. [This index contains the requested upstream](text.starts_with(text=`candidate`, prefix=`version` + "-"))
        We **package_version = candidate** and **complete = True**.

`state` =

| complete | package_version | url |
|---|---|---|
| `complete` | `package_version` | `url` |

Return *[The observed apt publication state](result.ok(value=`state`))*.

## status

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z |
| series | text | "noble" | Ubuntu series being observed |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result combining source publication, build and apt completion |

Use `version` and `series`="noble" to verify an existing Ubuntu release.
Each observation remains separate in the returned record so the user can tell
whether upload, compilation or apt publication is pending. Completion requires
a published source, its successful amd64 build, and the matching apt package.

This is a read-only task requiring network.read and plugin. It does not invoke
sudo, GPG, debuild or dput. Repeating status observes newer external state without
creating another upload. If a service request fails, return its real failure
rather than manufacturing a pending state. An ext-only release has no Ubuntu
obligation; the top-level release handbook handles that mode explicitly. Keep
the version provided by the caller: querying status never selects a next release
or edits the local changelog.

We **published = [Locate this upstream's source publication](source(version=`version`, series=`series`))**.
1. not access(`published`, "ok")
  Return *published*.
We **publication = [Retain its matching source record](access(published, "value"))** and
**built = [Check the source-specific amd64 build](builds(publication=`publication`))**.
1. not access(`built`, "ok")
  Return *built*.
We **indexed = [Verify actual apt repository visibility](apt(version=`version`, series=`series`))**.
1. not access(`indexed`, "ok")
  Return *indexed*.
We **build_state = [Keep build evidence](access(built, "value"))** and **apt_state = [Keep index evidence](access(indexed, "value"))** and
**complete = publication != None and access(build_state, "complete") and access(apt_state, "complete")**.

`state` =

| complete | publication | build | apt |
|---|---|---|---|
| `complete` | `publication` | `build_state` | `apt_state` |

Return *[The independently measured Ubuntu release state](result.ok(value=`state`))*.

## ship

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable X.Y.Z |
| series | text | "noble" | Ubuntu series being observed |
| root | text | "." | Checkout matching --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result from the signed Ubuntu source publication boundary |

Use `version`, `series`="noble" and `root`="." only for an authorized
CLI/full release. Verify the checked-out interpreter has that exact upstream
version before invoking the maintained Debian tools. Their signatures, orig
archive rules and dput behavior remain the operating-system boundary. File
preparation in that script delegates to package.mq.md.

This task requires plugin and fs.read and may need interactive OS credentials.
A sudo, GPG or dput failure is incomplete publication. Report the exact failing
stage and give the concrete recovery call, rather than hiding it behind success
or inventing another version. After the command succeeds, run status: upload
acceptance alone does not establish green builds or apt availability. Never
rewrite an orig archive already accepted by Launchpad.

We **observed = [Require actual checkout version metadata](local.inspect(root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed*.
1. access(access(`observed`, "value"), "cli_version") != `version`
  Return *[Refuse a different source version](result.error(code="ubuntu.version", message="checkout version differs from requested upstream", operation="ship"))*.

`argv` =

| argument |
|---|
| "bash" |
| "scripts/ppa-ship.sh" |
| `series` |

Return *[Build, sign and upload through the maintained Debian tools](local.command(argv=`argv`, timeout=3600, root=`root`))*.
