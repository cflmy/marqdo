---
markup: occlusion
title: Native artifact IO for the release handbook
import:
  engineering: ext/engineering/index.mq.md
  fs: lib/fs.mq.md
  net: lib/net.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  text: lib/text.mq.md
  re: lib/re.mq.md
  uuid: lib/uuid.mq.md
  sys: lib/sys.mq.md
---

These functions are the concrete IO used by assets.mq.md. Native Marqdo libraries read, hash, download, stage and clean files. The extension handles authenticated R2 writes and ZIP-format inspection. Source identity, release completeness and publication ordering remain visible Marqdo dataflow.

## verify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| asset | map | required | Exact GitHub asset record including size and digest |
| path | text | required | Local file selected inside the granted filesystem root |
| root | text | "." | Checkout root matching --fs-root; relative to the granted runtime cwd |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with native hash evidence and validated archive metadata |

Use `asset`, `path` and `root`="." to verify local bytes before publication.
Size and SHA256 come from lib/fs; a GitHub digest, when supplied, must match.
The native engineering extension inspects only the ZIP/VSIX format and CRCs.
Ordinary file facts stay in Marqdo instead of an external script. A mismatched
length or hash remains a failed Result and cannot advance the CDN pointer.

The format layer rejects escaping entries, duplicate names, symlinks and excessive
expansion. It does not extract or execute a CLI. Source pack version, native file
roles and VSIX version are checked by assets.content after this byte-level stage.
This function requires fs.read and plugin; executable-only assets use the same
size/hash checks without pretending they contain an archive.

We **information = [Read the native file information](fs.stat(path=`path`))** and
**size = [Read its actual byte count](access(information, "size"))** and
**sha256 = [Hash the actual file through the native filesystem](fs.file_hash(path=`path`))** and
**expected = [Read GitHub's optional digest metadata](table.get(map=`asset`, key="digest"))** and
**name = [Retain the asset's exact filename](access(asset, "name"))**.
1. `size` != access(`asset`, "size") or (`expected` != None and `expected` != "sha256:" + `sha256`)
  Return *[Reject bytes that differ from GitHub metadata](result.error(code="release.asset_digest", message=`name`, operation="asset_io.verify"))*.

`record` =

| name | size | sha256 |
|---|---|---|
| `name` | `size` | `sha256` |

1. [This asset is an archive format](text.ends_with(text=`name`, suffix=".zip") or text.ends_with(text=`name`, suffix=".vsix"))
  We **archive = [Inspect ZIP entries and their CRCs without extraction](engineering.inspect_zip(path=`path`, root=`root`))**.
  1. not access(`archive`, "ok")
    Return *archive* with the actual format error.
  We **record = [Retain the inspected format evidence](table.merge(a=`record`, b=access(archive, "value")))**.
Return *[The verified local byte identity](result.ok(value=`record`))*.

## fetch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| asset | map | required | GitHub asset record chosen by the release catalog |
| directory | text | required | Release-specific destination under --fs-root |
| root | text | "." | Checkout root matching --fs-root; relative to the granted runtime cwd |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with a reused or freshly downloaded verified local artifact |

Use `asset`, `directory` and `root`="." after assets.catalog has checked all
required filenames. A local file is reused only after its size, hash and archive
format are verified again. Otherwise native net.download streams into an atomic
replacement under the granted filesystem root. Failed HTTP transfers do not
replace an existing destination. The proxy route is tried before the original
GitHub asset URL; explicit fixture URLs remain visible in test metadata.

Downloads have a 64 MiB per-file limit and a 90-second request timeout here.
These limits cover the current release matrix; a larger future artifact needs
a deliberate documented limit change. No credential is put into a URL or native
download header. Role-specific contents are checked by the calling task.

We **name = [Read the exact catalog filename](access(asset, "name"))** and
**path = directory + "/" + name** and **prepared = [Create the explicit staging directory](fs.make_dirs(path=`directory`))**.
1. [A local candidate already exists](fs.exists(path=`path`))
  We **existing = [Recheck its actual byte identity](verify(asset=`asset`, path=`path`, root=`root`))**.
  1. access(`existing`, "ok")
    Return *existing* without downloading it again.
We **original = [Read the selected asset's original URL](access(asset, "browser_download_url"))** and
**proxy = [Select the documented GitHub mirror](text.replace(text=`original`, old="https://github.com/", new="https://proxy.cflmy.top/github.com/"))**.

`urls` =

| url |
|---|
| `proxy` |
| `original` |

We **headers = [Identify this public verification download](table.put(in=None, at="User-Agent", value="marqdo-engineering"))**.

We **last = [Start with no completed transfer](result.error(code="release.download", message=`name`, operation="asset_io.fetch"))**.
- [Try the bounded public download routes](for urls as url)
  We **downloaded = [Stream the official artifact through native IO](net.download(url=`url`, path=`path`, headers=`headers`, timeout_ms=90000, max_bytes=67108864))**.
  1. access(`downloaded`, "ok")
    We **last = [Check the downloaded bytes and archive format](verify(asset=`asset`, path=`path`, root=`root`))**.
    1. access(`last`, "ok")
      Return *last* with its real evidence.
  2. *
    We **last = downloaded** to preserve the last transport failure.
Return *last* when neither route produced a verified artifact.

## cdn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Versioned or latest VERSION/pack path from the release model |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing public pointer text or native download hash |

Use `path` only for a modeled CDN object. The public base is ext.marqdo.com
unless an explicit fixture override is configured. VERSION is read as UTF-8;
binary packs use native net.download, fs.file_hash and native cleanup in a
UUID-named directory under .marqdo/release-check. No Python body reader or local
file writer participates. A handled network failure cleans its owned directory.

A successful HTTP request proves only that URL's bytes; the caller compares the
hash with GitHub and decides whether the release is complete. Edge-cache delay
remains visible. Unexpected filesystem diagnostics may leave the uniquely owned
staging directory for inspection; no unrelated directory is removed.
This task needs network.read, fs.write, environment.read and random.read.

We **valid = [Accept only modeled public artifact paths](re.is_match(text=`path`, pattern="^(latest|v[0-9]+\\.[0-9]+\\.[0-9]+)/(VERSION|marqdo-[A-Za-z0-9._-]+\\.zip)$"))**.
1. not `valid`
  Return *[Reject an arbitrary CDN cache path](result.error(code="release.cdn_path", message="expected a modeled release path", operation="asset_io.cdn"))*.
We **base = [Read the explicit fixture override if configured](sys.env_get(name="MARQDO_RELEASE_CDN"))**.
1. `base` == None
  We **base = "https://ext.marqdo.com"** for normal public verification.
We **url = base + "/" + path**.

We **headers = [Identify this public verification download](table.put(in=None, at="User-Agent", value="marqdo-engineering"))**.

1. [This object is a version pointer](text.ends_with(text=`path`, suffix="/VERSION"))
  We **read = [Read the public UTF-8 version pointer](net.get(url=`url`, headers=`headers`, timeout_ms=30000))**.
  1. not access(`read`, "ok")
    Return *read* with its actual network error.
  We **response = [Read the HTTP result](access(read, "value"))**.
  1. access(`response`, "status") != 200
    Return *[Keep the public pointer HTTP failure](result.error(code="release.cdn_http", message=str(access(response, "status")), operation="asset_io.cdn"))*.
  We **version = [Trim only the pointer's outer whitespace](text.str_trim(s=access(response, "body")))**.

  We **pointer = [Keep the version as one named result field](table.put(in=None, at="version", value=`version`))**.

  Return *[The actual public version pointer](result.ok(value=`pointer`))*.
We **parent = [Prepare the ignored verification area](fs.make_dirs(path=".marqdo/release-check"))** and
**identifier = [Choose an independent staging identity](uuid.v4())** and **directory = ".marqdo/release-check/" + identifier** and
**created = [Reserve this verification directory](fs.make_dir(path=`directory`))** and **destination = directory + "/artifact.zip"**.
We **downloaded = [Stream public bytes into this owned native stage](net.download(url=`url`, path=`destination`, headers=`headers`, timeout_ms=90000, max_bytes=67108864))**.
1. not access(`downloaded`, "ok")
  We **removed = [Clean only this failed request's owned stage](fs.remove_tree(path=`directory`))**.
  Return *downloaded* with its real transport failure.
We **sha256 = [Hash the actual public bytes](fs.file_hash(path=`destination`))** and
**information = [Read the downloaded file information](fs.stat(path=`destination`))** and
**size = [Read their actual byte count](access(information, "size"))** and
**removed = [Remove this completed request's owned stage](fs.remove_tree(path=`directory`))**.

`evidence` =

| size | sha256 |
|---|---|
| `size` | `sha256` |

Return *[The measured public object identity](result.ok(value=`evidence`))*.

## upload

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| files | list | required | Exactly three verified path/name/sha256 records |
| root | text | "." | Checkout root matching --fs-root; relative to the granted runtime cwd |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the actual R2 write evidence |

Use `version`, `files` and `root`="." only for the release already authorized
by the user. The caller has checked all three source/native files against
GitHub before this operation begins. Marqdo controls the write order explicitly:
versioned and latest pack objects first, both VERSION pointers last. Private R2
signing is the only native service boundary. No whole-directory sweep occurs.

An interrupted upload keeps the old pointer until every required pack has
succeeded. Existing older versions are never deleted. The tool returns an error
on the first failed write and preserves already-written object evidence through
the service itself; it does not call a partial batch complete. Public byte-parity
checks still follow in assets.status.

We **count = [Require the source pack and both platform packs](len(`files`))**.
1. `count` != 3
  Return *[Reject an incomplete CDN upload set](result.error(code="release.cdn_files", message="exactly three verified packs are required", operation="asset_io.upload"))*.

`prefixes` =

| prefix |
|---|
| "v`version`" |
| latest |

We **written = [Start the service write evidence](table.empty_list())**.
- [Write both public pack namespaces before pointers](for prefixes as prefix)
  - [Upload only the preverified pack set](for files as file)
    We **key = prefix + "/" + access(file, "name")** and
    **uploaded = [Sign and upload the exact checked local bytes](engineering.put_file(path=access(file, "path"), key=`key`, sha256=access(file, "sha256"), root=`root`))**.
    1. not access(`uploaded`, "ok")
      Return *uploaded* before changing any VERSION pointer.
    We **written = [Retain the completed object identity](table.append(list=`written`, item=access(uploaded, "value")))**.
- [Advance pointers only after all six pack writes succeeded](for prefixes as prefix)
  We **uploaded = [Write the explicit version pointer](engineering.put_text(key=`prefix` + "/VERSION", text=`version` + "\n", root=`root`))**.
  1. not access(`uploaded`, "ok")
    Return *uploaded* with the actual pointer failure.
  We **written = [Retain this pointer write](table.append(list=`written`, item=access(uploaded, "value")))**.
Return *[All completed service writes](result.ok(value=`written`))*.
