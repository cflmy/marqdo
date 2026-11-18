---
markup: occlusion
title: Ubuntu 源码包的原生文件处理
import:
  fs: lib/fs.mq.md
  json: lib/json.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
  re: lib/re.mq.md
  result: lib/result.mq.md
  toml: lib/toml.mq.md
---

本文件由已维护的 Debian 打包脚本调用。文件枚举、JSON 修订和 changelog 更新使用 Marqdo 固有库；cargo vendor、tar、debuild、GPG 和 dput 是已有工具的外联边界。任务失败应立即中止外层脚本，不能继续签名或上传不完整的源码包。

元数据读取任务使用仓库根目录和 fs.read；暂存修订任务要求把 --fs-root 指向实际暂存源码根目录，并授权 fs.write。它们不会选择版本、创建 Git 标签或上传文件。发布者必须先检查所需版本及打包修订号，已经被 Launchpad 接受的 orig 归档必须复用原字节。

## upstream

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Current interpreter version read from Cargo package metadata |

Run this task at the checkout root with fs.read. It reads Cargo.toml through
the native filesystem and TOML parser, then selects package.version rather than
matching an arbitrary dependency version. The release preflight checks that this
actual version is the one the user requested; this helper never chooses a version.

Without --json, marqdo call prints this text on stdout. The maintained Debian
tool boundary captures that exact output; it does not run sed over the manifest
or regenerate a Python reader. Invalid files retain their native diagnostics.
No network, Git mutation, changelog edit or package upload occurs here.

We **manifest = [Read and decode the actual interpreter manifest](toml.parse(text=fs.read_text(path="Cargo.toml")))**.
Return *[The current Cargo package version](access(access(manifest, "package"), "version"))*.

## debian_version

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| series | text | "noble" | Ubuntu series appended to the existing packaging revision |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Full Debian version text, or failed Result for invalid input |

Use `series`="noble" at the checkout root. Read the current upstream version
and first Debian changelog header through native Marqdo file operations. Retain
the existing packaging revision, remove its old series suffix and append the
selected series. If the changelog has no revision, use 1ppa1, matching the
maintained packaging rule. This is a read-only projection, not a revision bump.

A missing marqdo header or invalid series returns a failed Result. The shell
boundary uses --require-ok, so that failure stops before a staging directory or
signature is created. A successful non-JSON call prints only the version text.
Use local.debian or an explicit reviewed changelog edit to choose a new packaging
revision; never silently increment one to bypass an existing upstream archive.

1. not [The Ubuntu series has the expected shape](re.is_match(text=`series`, pattern="^[a-z][a-z0-9]*$"))
  Return *[Reject invalid packaging input](result.error(code="package.series", message=`series`, operation="debian_version"))*.
We **version = [Read the actual interpreter version](upstream())** and **source = [Read the existing Debian changelog](fs.read_text(path="debian/changelog"))** and
**header = [Require its first package declaration](re.find(text=`source`, pattern="^marqdo \\([^)]+\\)"))**.
1. `header` == None
  Return *[Keep a missing changelog header visible](result.error(code="package.changelog", message="missing marqdo changelog header", operation="debian_version"))*.
We **observed = [Extract the declared package version](re.replace(text=`header`, pattern="^marqdo \\(([^)]+)\\)$", with="$1"))** and
**base = [Remove the previously selected series suffix](re.replace(text=`observed`, pattern="~.*$", with=""))** and **revision = "1ppa1"**.
1. [The existing version contains a packaging revision](re.is_match(text=`base`, pattern="-"))
  We **revision = [Keep the reviewed revision after the upstream component](re.replace(text=`base`, pattern="^[^-]+-", with=""))**.
Return *[The packaging version for this upstream and Ubuntu series](version + "-" + revision + "~" + series)*.

## vendor_config

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the natively written staging Cargo configuration |

Run after cargo vendor in a disposable source-package root. The explicit
configuration makes Cargo use that root's vendor directory when Launchpad builds
offline. Directory creation and UTF-8 writing use the native filesystem rather
than a shell heredoc. Grant fs.write and point --fs-root at the staged package.

The file is replaced atomically, as the prior packaging script did. This helper
is for the generated staging tree, not for silently changing a developer's
personal Cargo settings. It neither downloads crates nor changes vendor checksums;
sanitize_vendor remains a separate reviewed task. A write failure propagates and
the packaging boundary must stop before building or signing the source archive.

We **directory = [Prepare the staged Cargo configuration directory](fs.make_dirs(path=".cargo"))** and
**configuration = "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n"**.
We **saved = [Atomically write the explicit offline source configuration](fs.atomic_write(path=".cargo/config.toml", text=`configuration`))**.
Return *[The actual prepared path](result.ok(value=".cargo/config.toml"))*.

## sanitize_vendor

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：移除的备份文件数与修改的 checksum 文件数 |

cargo vendor 完成后执行此任务。Debian 的清理步骤会删除 .orig 和 .rej 文件；若 Cargo 的 vendor 清单仍引用这些文件，离线构建就会失败。因此先枚举每个 crate 的真实 checksum，删除对应条目及备份文件，再原子保存清单。普通源码文件和 package 校验值保持原样。

路径来自生成的清单，但仍需验证没有父目录跳转或绝对路径。一个 crate 完成后才处理下一个；发生错误时已完成的修订留在暂存目录供诊断，不能宣称整个目录修改具有事务性。重复运行不会继续增加计数，也不应引入 Python 作为备用写入器。

The package checksum and the individual file checksums serve different purposes.
Retain the upstream package digest and every ordinary source entry. Removing
only backup suffixes mirrors the existing Debian clean rule; it is not a general
vendor rewrite or permission to replace code inside third-party crates.
After this task, the packaging boundary must still run the normal offline
build checks. A successful count only records this local preparation step.
Review the resulting checksum changes when diagnosing a Launchpad failure,
and preserve the staged tree if any later signing or upload operation fails.

We **checksums = [枚举 vendor 的实际校验清单](fs.find(path="vendor", pattern="**/.cargo-checksum.json"))** and **removed = 0** and **changed = 0**。
- [逐个处理 crate 的校验清单](for checksums as relative)
  We **path = "vendor/" + relative** and **source = [读取校验清单](fs.read_text(path=`path`))** and **data = [解码 JSON 边界文本](json.parse(text=`source`))** and **files = [取得文件散列映射](access(data, "files"))** and **dirty = False**。
  We **keys = [保留遍历开始时的文件名](table.keys(map=`files`))** and **directory = [取得此清单所在目录](re.replace(text=`path`, pattern="/[^/]+$", with=""))**。
  - [仅移除 Debian 清理器会删除的备份](for keys as name)
    1. [该路径是备份或补丁拒绝文件](text.ends_with(text=`name`, suffix=".orig")) or [该路径是补丁拒绝文件](text.ends_with(text=`name`, suffix=".rej"))
      1. [清单路径越出 crate 边界](re.is_match(text=`name`, pattern="(^/|(^|/)\\.\\.(/|$)|\\\\|:)"))
        返回*[拒绝不可信的清单路径](result.error(code="package.vendor_path", message=`name`, operation="sanitize_vendor"))*。
      We **target = directory + "/" + name**。
      1. [备份文件实际存在](fs.exists(path=`target`))
        We **deleted = [通过固有库删除备份文件](fs.remove_file(path=`target`))** and **removed = removed + 1**。
      We **files = [从校验清单删除同名项](table.delete(map=`files`, key=`name`))** and **dirty = True**。
  1. `dirty`
    We **data = [保留其他元数据并替换文件表](table.put(in=`data`, at="files", value=`files`))** and **encoded = [序列化修改后的边界文本](json.stringify(value=`data`, indent=4))**。
    We **written = [原子保存此 crate 的清单](fs.atomic_write(path=`path`, text=`encoded` + "\n"))** and **changed = changed + 1**。

`counts` =

| removed | changed |
|---|---|
| `removed` | `changed` |

处理结束后返回*[实际修改计数](result.ok(value=`counts`))*。

## changelog

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | 包含 Debian 修订号的完整源码包版本 |
| series | text | "noble" | 目标 Ubuntu series |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result：保存后的第一行 |

以 `version` 和 `series`="noble" 修订暂存目录中的 Debian changelog。这里传入的是类似 2.6.0-1ppa1~noble 的完整打包版本，不能只传上游版本。任务只替换现有首行的包版本和发行系列，维护者、签名时间和旧条目由先前准备步骤保留。

实际调用的 version 需要保留完整修订号和发行系列后缀，例如 2.6.0-1ppa1~noble。
输入必须符合 Debian 版本和 series 的字符范围；缺少现有 marqdo 首行时返回错误，避免悄悄生成一份内容不足的 changelog。这里不递增打包修订，也不重建已经接收的 orig 归档。写入采用固有库原子替换，需要 fs.write。

1. not [打包版本只包含允许的字符](re.is_match(text=`version`, pattern="^[0-9][0-9A-Za-z.+:~_-]*$")) or not [发行系列合法](re.is_match(text=`series`, pattern="^[a-z][a-z0-9]*$"))
  返回*[拒绝无效打包元数据](result.error(code="package.input", message="invalid Debian version or series", operation="changelog"))*。
We **source = [读取已准备的 changelog](fs.read_text(path="debian/changelog"))** and **first = [查找现有包声明](re.find(text=`source`, pattern="^marqdo \\([^)]+\\) [^;]+;"))**。
1. `first` == None
  返回*[保留缺失首行的诊断](result.error(code="package.changelog", message="missing marqdo changelog header", operation="changelog"))*。
We **header = "marqdo (" + version + ") " + series + ";"** and **updated = [仅替换已确认的首行](text.replace(text=`source`, old=`first`, new=`header`, count=1))**。
We **saved = [原子保存暂存包元数据](fs.atomic_write(path="debian/changelog", text=`updated`))**。
完成后返回*[实际保存的包声明](result.ok(value=`header`))*。
