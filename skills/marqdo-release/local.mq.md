---
markup: occlusion
title: Prepare and publish a reviewed source release
import:
  bridge: ./_bridge.mq.md
  model: ./model.mq.md
  github: ./github.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  text: lib/text.mq.md
  toml: lib/toml.mq.md
  sys: lib/sys.mq.md
  datetime: lib/datetime.mq.md
  re: lib/re.mq.md
---

Local release tasks replace repeated command-writing with explicit callable
operations. Inspect first, use bump for version metadata, author and review the
bilingual release text, build docs, compile the editor branch separately, commit
reviewed paths, push main, wait for EK, then create the annotated tag.

These are separate tasks because their results are reviewable artifacts and
because waiting for CI is different from creating a new reference. Existing
user authorization carries through the chosen release; none of the tasks asks
for a second version choice. A new release version or a replacement tag needs
its own user decision. No task force-pushes or bypasses hooks.

Every process operation returns Result.ok/value/error. A failed compile, Git
hook or network request stops the dependent task. Multi-file metadata edits are
not a transaction: inspect any partial changes before retrying. No task cleans
or stages unrelated work, commits editor sources on main, or deploys gh-pages.

```sh
marqdo call skills/marqdo-release/local.mq.md inspect --effects plugin,fs.read --fs-root . --json
```

For mutation, use the chosen version and the effect grant described on that
heading. bump needs fs.write and plugin; the rest of the command tasks
use plugin, whose children retain normal OS authority. Do not interpret
--fs-root as a sandbox for external tools. GPG or sudo interaction is reported
at the Ubuntu boundary rather than hidden behind an unconditional success.

## inspect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "." | Checkout root matching --fs-root; relative to the granted runtime cwd |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with Git identity and versions read by native filesystem tools |

Use `root`="." to inspect a checkout without changing it. Git supplies branch,
HEAD, dirty paths and tags through the engineering boundary. Cargo manifests
and ext/VERSION are read with lib/fs and decoded with lib/toml and lib/text.
The version facts are therefore visible in this source rather than hidden in
an external file-reading script. No credential contents are returned.

The plugin root and --fs-root must identify the same directory; this check catches
a caller accidentally combining one checkout's Git state with another checkout's
version files. Local tags are evidence only; remote references are checked before
tagging. This operation requires plugin and fs.read but performs no publication.

We **payload = [Start an empty Git metadata request](table.empty_map())** and
**observed = [Read Git's actual repository state](bridge.request(operation="git_inspect", payload=`payload`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed* with its source error.
We **state = [Read the observed Git metadata](access(observed, "value"))** and
**cwd = [Read the native filesystem working root](sys.cwd())**.
1. access(`state`, "root") != `cwd`
  Return *[Refuse inconsistent process and filesystem roots](result.error(code="release.root_mismatch", message="root must match --fs-root", operation="local.inspect"))*.

We **cli = [Read and decode the actual interpreter manifest](toml.parse(text=fs.read_text(path="Cargo.toml")))** and
**wasm = [Read and decode the actual WASM manifest](toml.parse(text=fs.read_text(path="crates/marqdo-wasm/Cargo.toml")))** and
**cli_version = [Read the interpreter package version](access(access(cli, "package"), "version"))** and
**wasm_version = [Read the WASM package version](access(access(wasm, "package"), "version"))** and
**ext_version = [Read the independent pack version](text.str_trim(s=fs.read_text(path="ext/VERSION")))**.

`versions` =

| cli_version | wasm_version | ext_version |
|---|---|---|
| `cli_version` | `wasm_version` | `ext_version` |

We **combined = [Combine native file facts with Git identity](table.merge(a=`state`, b=`versions`))**.
Return *[The actual checkout facts](result.ok(value=`combined`))*.

## command

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| argv | list | required | Exact command and argument vector; never shell source |
| cwd | text | "." | Working directory inside the checkout |
| timeout | number | 120 | Maximum child duration in seconds |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with checked exit status and bounded stdout/stderr |

Use `argv`, `cwd`=".", `timeout`=120 and `root`="." for maintained commands.
This transport keeps arguments separate even when filenames contain spaces,
quotes, dollar signs or backticks. It refuses inline shell programs, forced Git
updates and hook bypass. Git credentials are attached privately when needed;
they never become parameters or command text. Nonzero exit and timeout remain
failed Results, so a later stage cannot silently continue after a failed build.
A child inherits the caller's OS capabilities. This helper does not claim that
--fs-root makes arbitrary subprocesses safe or confines their own file access.

`payload` =

| argv | cwd | timeout |
|---|---|---|
| `argv` | `cwd` | `timeout` |

Return *[Execute this exact maintained argument vector](bridge.request(operation="command", payload=`payload`, root=`root`))*.

## replace_version

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Cargo manifest relative to the granted filesystem root |
| old | text | required | Observed previous package version |
| version | text | required | Stable X.Y.Z explicitly chosen by the user |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Number of replaced manifest version fields |

Use `path`, `old` and `version` only after inspecting the version files.
The exact package-version line is replaced once. Other dependency versions and
surrounding documentation remain intact. The file helper refuses a missing or
ambiguous match, exposing drift instead of rewriting a whole manifest from a
model-generated template. Calling with the same version is a no-op.
This helper writes one file and is not a multi-file transaction. The calling
bump operation records which stage failed so the author can inspect any partial
changes before retrying. It needs fs.write under the supplied filesystem root.

1. `old` == `version`
  Return *0* because this manifest already has the requested version.
We **find = "version = \"" + old + "\""** and
**replacement = "version = \"" + version + "\""** as exact manifest lines.
Return *[Replace only the observed package version line](fs.text_patch(path=`path`, find=`find`, replace=`replacement`))*.

## bump

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with updated version metadata and Cargo check evidence |

Use `version`, `mode`="full" and `root`="." after the user chooses the release.
Full updates CLI, WASM and ext/VERSION; cli updates the two Cargo packages; ext
updates only ext/VERSION. Cargo check updates the lockfile for CLI/full and the
Debian stanza preserves the existing maintainer while dating the entry in
Asia/Shanghai. No tag, commit, remote push, R2 upload or PPA upload happens here.
The author still writes the bilingual CHANGELOG, README/tutorial changes and
release notes: version replacement cannot decide or verify their factual claims.
Run docs and the relevant regressions after those authored changes are reviewed.
The native filesystem calls need fs.write; Cargo and packaging transport need
plugin. A failure leaves intentional local changes available for review,
rather than pretending the multi-file edit was atomic or resetting other work.

We **planned = [Validate the user-chosen version and mode](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

We **observed = [Read current versions before changing any](inspect(root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed* with its failure intact.

We **state = [Keep the observed version metadata](access(observed, "value"))**.
1. access(`state`, "branch") != "main" or access(`state`, "extension_on_main") != ""
  Return *[Require the main checkout without tracked editor sources](result.error(code="release.branch", message="bump runs on main; editor sources belong only on vscode-extension", operation="local.bump"))*.
1. `mode` != "ext"
  We **cli_edit = [Update the interpreter package version](replace_version(path="Cargo.toml", old=access(state, "cli_version"), version=`version`))** and
  **wasm_edit = [Update the WASM package version](replace_version(path="crates/marqdo-wasm/Cargo.toml", old=access(state, "wasm_version"), version=`version`))**.

  `cargo_args` =

  | argument |
  |---|
  | "cargo" |
  | "check" |
  | "--offline" |

  We **checked = [Update Cargo.lock and verify the new package metadata](command(argv=`cargo_args`, root=`root`, timeout=600))**.
  1. not access(`checked`, "ok")
    Return *checked* with the local edits left for inspection.
  We **packaging = [Add the dated Debian stanza through native file IO](debian(version=`version`))**.
  1. not access(`packaging`, "ok")
    Return *packaging* before committing any metadata.
1. `mode` != "cli"
  We **ext_edit = [Write the independent extension pack version](fs.write_text(path="ext/VERSION", text=`version` + "\n"))**.
Return *[Read back all resulting metadata and dirty paths](inspect(root=`root`))*.

## docs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result from rebuilding the actual public documentation |

Use `root`="." after authoring public sources and release-facing documentation.
This command runs the checked-in documentation builder, which uses Marqdo View
on the real pages instead of copying stale generated HTML. Generated files stay
ignored and are packaged by CI. Library-only import helpers have no main entry;
their expected page execution status does not authorize inventing a main.
A failed builder is returned as an error and is not treated as proof that docs
were refreshed. This task does not deploy gh-pages; deployment is a separate
user request, even during a full release.

`argv` =

| argument |
|---|
| "bash" |
| "scripts/build-public.sh" |

Return *[Regenerate documentation from the authored Marqdo sources](command(argv=`argv`, root=`root`, timeout=900))*.

## review_index

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| paths | list | required | Exact reviewed repository-relative file paths |
| cwd | text |  "."  | Git checkout within root; editor stage or main |
| root | text |  "."  | Checkout root matching --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result whose value lists the already staged paths |

Use `paths`, `cwd`="." and `root`="." before staging or running a
version-changing tool. This inspection reads the actual Git index with explicit
unquoted path output; UTF-8 names and spaces remain ordinary path data.
Every existing staged path must appear in the supplied reviewed file list.
A refusal leaves both the index and working tree unchanged.

The release source and editor branch share this inspection so a pre-staged
unrelated edit cannot become part of either version commit. It does not inspect
unstaged file content, authorize a commit or reset another task's index.
Newline-bearing filenames are outside this release handbook's path contract.
Requires plugin for Git; there is no network request or filesystem write.

`argv` =

| argument |
|---|
| "git" |
| "-c" |
| "core.quotepath=false" |
| "diff" |
| "--cached" |
| "--name-only" |

We **observed = [Read the current staged file names before any change](command(argv=`argv`, cwd=`cwd`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed* with its Git diagnostic.
We **staged = [Split the observed index into individual file names](text.str_split(s=access(access(observed, "value"), "stdout"), sep="\n"))**.
- [Require each existing index path to be explicitly reviewed](for staged as path)
  1. `path` != "" and not [This staged path belongs to the review](table.contains(list=`paths`, item=`path`))
    Return *[Preserve unrelated staged work](result.error(code="release.staged_unrelated", message=`path`, operation="review_index"))*.
Return *[The inspected index paths](result.ok(value=`staged`))*.

## commit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| paths | list | required | Reviewed explicit paths; never use a blanket git add . |
| message | text | required | Commit message describing the actual reviewed release |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with commit output; no remote mutation |

Use `paths`, `message` and `root`="." after reviewing authored changes and
running the relevant checks. This function stages only the supplied paths,
checks the staged patch for whitespace errors, and creates an ordinary commit
with hooks enabled. It does not infer that every dirty file belongs to a release.
Editor sources remain on their own branch: do not include vscode-extension in
these paths. The current branch must be main. Only individual reviewed files are accepted;
a directory path is refused before staging. Deleted files can still be reviewed
by their exact repository path. Retries preserve already-staged deletions: Git no longer knows those missing paths in its index, so they must not be handed to git add again. Existing staged paths must all
appear in that same explicit file list before staging begins, so another task's
index work cannot be swept into the release commit. Broad pathspecs, parent
traversal and nested editor paths are rejected. These checks preserve the index
on refusal; a later hook failure keeps intentional staging available for review. The author can inspect staged changes before
this explicit commit task; a hook failure stays visible and is never bypassed.

`branch_args` =

| argument |
|---|
| "git" |
| "branch" |
| "--show-current" |

We **observed = [Require the source branch before touching its index](command(argv=`branch_args`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed* with the command diagnostic.
We **branch = [Read the actual source branch](text.str_trim(s=access(access(observed, "value"), "stdout")))**.
1. `branch` != "main"
  Return *[Keep release source commits on main](result.error(code="release.commit_branch", message="source commit requires main", operation="local.commit"))*.

1. len(`paths`) == 0 or [Editor sources are not main-branch release files](table.contains(list=`paths`, item="vscode-extension"))
  Return *[Require explicit reviewed main-branch paths](result.error(code="release.commit_paths", message="provide reviewed paths and exclude vscode-extension", operation="local.commit"))*.

- [Validate individual reviewed repository paths](for paths as path)
  1. `path` == "." or `path` == "" or [This path is broad, outside the checkout or editor-only](re.is_match(text=`path`, pattern="(^/|(^|/)\\.\\.(/|$)|(^|/)vscode-extension(/|$)|[:*?])"))
    Return *[Require concrete main-branch file paths](result.error(code="release.commit_paths", message=`path`, operation="local.commit"))*.
  1. [Inspect an existing reviewed path without opening it](fs.exists(path=`path`))
    We **metadata = [Require a file rather than a directory pathspec](fs.stat(path=`path`))**.
    1. access(`metadata`, "is_dir")
      Return *[Do not stage an entire reviewed directory](result.error(code="release.commit_paths", message=`path`, operation="local.commit"))*.

We **reviewed = [Protect all previously staged work](review_index(paths=`paths`, root=`root`))**.
1. not access(`reviewed`, "ok")
  Return *reviewed* before changing the index.

`deleted_args` =

| argument |
|---|
| "git" |
| "-c" |
| "core.quotepath=false" |
| "diff" |
| "--cached" |
| "--name-only" |
| "--diff-filter=D" |
| "--no-renames" |

We **removed = [Read deletions already retained by the reviewed index](command(argv=`deleted_args`, root=`root`))**.
1. not access(`removed`, "ok")
  Return *removed* with the original index diagnostic.
We **deleted = [Keep exact already-staged deletion paths](text.str_split(s=access(access(removed, "value"), "stdout"), sep="\n"))** and **stage_paths = [Start only files requiring staging](table.empty_list())**.
- [Preserve reviewed deletions when retrying a failed commit](for paths as path)
  1. fs.exists(path=`path`) or not table.contains(list=`deleted`, item=`path`)
    We **stage_paths = [Stage existing files and unstaged removals](table.append(list=`stage_paths`, item=`path`))**.
1. len(`stage_paths`) > 0

  `prefix` =

  | argument |
  |---|
  | "git" |
  | "add" |
  | -- |

  We **argv = [Append only paths still requiring staging](table.concat(a=`prefix`, b=`stage_paths`))** and
  **staged = [Stage those exact reviewed paths](command(argv=`argv`, root=`root`))**.
  1. not access(`staged`, "ok")
    Return *staged* with its failure intact.

`check_args` =

| argument |
|---|
| "git" |
| "diff" |
| "--cached" |
| "--check" |

We **checked = [Check the exact staged patch](command(argv=`check_args`, root=`root`))**.
1. not access(`checked`, "ok")
  Return *checked* with its failure intact.

`commit_args` =

| argument |
|---|
| "git" |
| "commit" |
| "-m" |
| `message` |

Return *[Create an ordinary commit with hooks](command(argv=`commit_args`, root=`root`))*.

## push

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| refspec | text | "HEAD:refs/heads/main" | main, vscode-extension or a same-name stable tag; force syntax is rejected |
| cwd | text | "." | Checkout directory inside root; editor staging may override it |
| root | text | "." | Checkout root matching --fs-root; relative to the granted runtime cwd |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result confirming the exact expected remote ref SHA |

Use `refspec`="HEAD:refs/heads/main", `cwd`="." and `root`="." only for
the authorized source publication. The argument vector and retry order are
visible here: proxy first, then direct GitHub, with two attempts per route.
After each response, including an uncertain timeout, read the remote ref before
deciding to retry. Only its exact expected SHA proves that this push arrived.

A branch must match its destination; stable tags can only push to the same name.
The native boundary also rejects force and hook bypass. A test-only configured
remote can replace the network routes in an isolated fixture. Production
credentials remain private inside the engineering Git boundary. This task never
changes origin and never treats stale remote state as success. If both network
routes fail, retain local refs and use an explicitly configured SSH/HK transport
rather than force-pushing or silently skipping publication.

MARQDO_RELEASE_GIT_REMOTE is an explicit transport override for an already
reviewed SSH/HK route or an isolated fixture. The task does not invent that
route, rewrite origin or generate a jump program. The same branch checks and
exact-SHA readback apply to an override; transport success alone is insufficient.
Configure SSH identities in the existing OS store, never in tool arguments.

We **valid = [Accept only reviewed release refspec shapes](re.is_match(text=`refspec`, pattern="^(HEAD:refs/heads/(main|vscode-extension)|refs/tags/(ext-)?v[0-9]+\\.[0-9]+\\.[0-9]+:refs/tags/(ext-)?v[0-9]+\\.[0-9]+\\.[0-9]+)$"))**.
1. not `valid`
  Return *[Reject arbitrary or forced ref updates](result.error(code="release.refspec", message="unsupported release refspec", operation="local.push"))*.
We **parts = [Separate the source and destination ref](text.str_split(s=`refspec`, sep=":"))** and
**source = [Read its local source](access(parts, 1))** and **destination = [Read its remote target](access(parts, 2))**.
1. `source` != "HEAD" and `source` != `destination`
  Return *[Keep tag names immutable across push](result.error(code="release.refspec", message="tag destination must equal its source", operation="local.push"))*.

`rev_args` =

| argument |
|---|
| "git" |
| "rev-parse" |
| `source` |

1. `source` == "HEAD"
  `branch_args` =

  | argument |
  |---|
  | "git" |
  | "branch" |
  | "--show-current" |

  We **observed = [Read the current source branch before any push](command(argv=`branch_args`, cwd=`cwd`, root=`root`))**.
  1. not access(`observed`, "ok")
    Return *observed*.
  We **branch = [Keep the actual source branch](text.str_trim(s=access(access(observed, "value"), "stdout")))**.
  1. `destination` != "refs/heads/" + `branch`
    Return *[Refuse a different branch destination](result.error(code="release.push_branch", message="source branch must match destination", operation="local.push"))*.

We **local = [Resolve the exact local ref before transport](command(argv=`rev_args`, cwd=`cwd`, root=`root`))**.
1. not access(`local`, "ok")
  Return *local* before any remote mutation.
We **expected = [Keep the expected local SHA](text.str_trim(s=access(access(local, "value"), "stdout")))** and
**override = [Read an explicitly configured fixture remote](sys.env_get(name="MARQDO_RELEASE_GIT_REMOTE"))**.

`remotes` =

| remote |
|---|
| "https://proxy.cflmy.top/github.com/cflmy/marqdo.git" |
| "https://github.com/cflmy/marqdo.git" |

1. `override` != None

  `remotes` =

  | remote |
  |---|
  | `override` |

`attempts` =

| attempt |
|---|
| 1 |
| 2 |

- [Try each configured route without changing origin](for remotes as remote)
  - [Bound retries and verify after uncertain responses](for attempts as attempt)

    `push_args` =

    | argument |
    |---|
    | "git" |
    | "push" |
    | `remote` |
    | `refspec` |

    We **pushed = [Send the explicit ref without force](command(argv=`push_args`, cwd=`cwd`, root=`root`, timeout=180))**.

    `read_args` =

    | argument |
    |---|
    | "git" |
    | "ls-remote" |
    | `remote` |
    | `destination` |

    We **read = [Observe whether that exact ref arrived](command(argv=`read_args`, cwd=`cwd`, root=`root`, timeout=90))**.
    1. access(`read`, "ok")
      We **observed = [Read the server's ref response](text.str_trim(s=access(access(read, "value"), "stdout")))** and
      **fields = [Separate the SHA from the ref name](text.str_split(s=`observed`, sep="\t"))**.
      1. access(`fields`, 1) == `expected`

        `verified` =

        | ref | sha | verified |
        |---|---|---|
        | `destination` | `expected` | True |

        Return *[The confirmed remote source identity](result.ok(value=`verified`))*.
Return *[Preserve an unconfirmed network outcome](result.error(code="release.push_unconfirmed", message="proxy retries and direct route did not confirm the expected SHA; retain local refs and inspect the configured transport", operation="local.push"))*.

## tag

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result confirming a new annotated tag on the checked remote commit |

Use `version`, `mode`="full" and `root`="." only for the already authorized
release. Main must be clean, contain no tracked editor source, and carry the
mode's matching versions. A local existing tag is rejected. The remote main SHA
must equal local HEAD, and its latest EK gate must be green before this task
creates anything. These conditions preserve the release skill's hard rules.
The remote source identity is retained as evidence for CI matching.
The tag is annotated and then pushed through the same bounded proxy transport.
If pushing fails after local creation, call push with that exact tag refspec;
do not rerun tag, delete the tag or move it to a new commit. An ext tag triggers
extension-release.yml, which runs packs.ci and packs.attach_ci on Windows and Linux.
Commit reviewed bilingual doc/releases/ext-X.Y.Z.md notes before creating that tag.
The notes travel in the tagged checkout, so CI reads the reviewed content rather
than reconstructing publication text. Existence and nonempty checks are only a
preflight: the author must still verify the actual changes, asset scope and test
limits before committing. A clean checkout ensures the note file is included in
the source decision; the later CI creation task verifies the same commit again.
This function creates an immutable release reference. Its invocation needs the
release authorization; merely reading this file or running inspect is read-only.

We **planned = [Validate the authorized version and release mode](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

1. `mode` == "ext"
  We **notes_path = "doc/releases/ext-" + version + ".md"**.
  1. not [The independent pack's reviewed notes exist](fs.exists(path=`notes_path`))
    Return *[Require committed version-specific release notes](result.error(code="release.notes_missing", message=`notes_path`, operation="local.tag"))*.
  1. [The independent pack's note file is empty](fs.read_text(path=`notes_path`) == "")
    Return *[Require actual bilingual release content](result.error(code="release.notes_empty", message=`notes_path`, operation="local.tag"))*.

We **observed = [Require an actual clean main checkout](inspect(root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed* with its failure intact.

We **state = [Read observed branch and version metadata](access(observed, "value"))** and
**tag_name = [Read the mode's immutable tag name](access(access(planned, "value"), "tag"))**.
1. access(`state`, "branch") != "main" or access(`state`, "changes") != "" or access(`state`, "extension_on_main") != ""
  Return *[Reject a dirty or wrong-branch release](result.error(code="release.dirty", message="tagging requires clean main without tracked editor sources", operation="local.tag"))*.
1. [This tag already exists locally](table.contains(list=access(`state`, "tags"), item=`tag_name`))
  Return *[Preserve the existing immutable tag](result.error(code="release.tag_exists", message="existing tag requires inspection; never replace it", operation="local.tag"))*.
1. `mode` != "ext" and (access(`state`, "cli_version") != `version` or access(`state`, "wasm_version") != `version`)
  Return *[Reject mismatched CLI metadata](result.error(code="release.versions", message="CLI and WASM must match the chosen version", operation="local.tag"))*.
1. `mode` != "cli" and access(`state`, "ext_version") != `version`
  Return *[Reject a mismatched extension pack](result.error(code="release.versions", message="ext/VERSION must match the chosen pack version", operation="local.tag"))*.
We **remote = [Read the current remote main reference](github.request(path="/repos/cflmy/marqdo/git/ref/heads/main", root=`root`))**.
1. not access(`remote`, "ok")
  Return *remote* with its failure intact.

We **sha = [Read the remote source identity](access(access(access(remote, "value"), "object"), "sha"))**.
1. `sha` != access(`state`, "commit")
  Return *[Require the reviewed commit to be pushed first](result.error(code="release.unpushed", message="remote main differs from local HEAD", operation="local.tag"))*.

`required` =

| name |
|---|
| gate |

We **knowledge = [Require green EK before publishing a tag](github.gate(workflow="executable-knowledge.yml", sha=`sha`, required=`required`, root=`root`))**.
1. not access(`knowledge`, "ok")
  Return *knowledge* with its failure intact.

1. not access(access(`knowledge`, "value"), "complete")
  Return *[Keep a red or running gate visible](result.error(code="release.ek_incomplete", message="wait for or fix EK on the exact main commit", operation="local.tag"))*.

`argv` =

| argument |
|---|
| "git" |
| "tag" |
| "-a" |
| `tag_name` |
| "-m" |
| `tag_name` |

We **created = [Create the new annotated tag without replacement](command(argv=`argv`, root=`root`))**.
1. not access(`created`, "ok")
  Return *created* with its failure intact.

Return *[Push the same immutable tag and verify the remote SHA](push(refspec="refs/tags/" + tag_name + ":refs/tags/" + tag_name, root=`root`))*.

## editor_workspace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Validated release version naming the isolated editor stage |
| root | text | "." | Checkout root matching --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing the isolated vscode-extension directory |

Use `version` and `root`="." after model.plan validates the release.
This task creates a separate clone under .marqdo/release/editor-X.Y.Z. The main
checkout never receives editor files. Git clones only the vscode-extension branch;
proxy transport is attempted before direct GitHub. A failed clone that leaves a
directory is retained for diagnosis rather than deleted or reset to retry.

An existing stage is reusable only when its branch is vscode-extension and its
origin is one of this project's explicit release transports. That check prevents
npm from being sent into an unrelated repository merely because its path exists.
The stage is deliberately persistent: inspect a failed install or compile there,
then call extension again. No source merge, hard reset, stash or force checkout
is performed. Requires plugin and fs.write under the same root. Normal Git
credential stores remain private inside the engineering boundary.

The returned path is only local preparation evidence, not a pushed editor
commit. The extension task below performs package versioning, dependency install,
compilation, narrowly scoped staging and normal push. Publishing on the editor
branch remains part of the user's authorized CLI/full release; simply reading
this file or running preflight authorizes none of those mutations.

We **directory = ".marqdo/release/editor-" + version** and **parent = [Prepare the ignored release stage parent](fs.make_dirs(path=".marqdo/release"))**.

`remotes` =

| remote |
|---|
| "https://proxy.cflmy.top/github.com/cflmy/marqdo.git" |
| "https://github.com/cflmy/marqdo.git" |

1. not [An existing stage needs inspection before reuse](fs.exists(path=`directory`))
  - [Try maintained transports without replacing an existing stage](for remotes as remote)
    `clone_argv` =

    | argument |
    |---|
    | "git" |
    | "clone" |
    | "--single-branch" |
    | "--branch" |
    | "vscode-extension" |
    | `remote` |
    | `directory` |

    We **cloned = [Clone only the editor-source branch](command(argv=`clone_argv`, root=`root`, timeout=300))**.
    1. access(`cloned`, "ok")
      Return *[Validate the newly created stage before using npm](editor_workspace(version=`version`, root=`root`))*.
    1. [A failed partial clone needs explicit diagnosis](fs.exists(path=`directory`))
      Return *cloned* with its preserved directory.
  Return *[Report exhausted clone transports](result.error(code="release.editor_clone", message="both editor clone transports failed", operation="editor_workspace"))*.

`branch_argv` =

| argument |
|---|
| "git" |
| "branch" |
| "--show-current" |

We **observed = [Read the stage's real branch](command(argv=`branch_argv`, cwd=`directory`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed*.
We **branch = [Trim the actual branch name](text.str_trim(s=access(access(observed, "value"), "stdout")))**.
1. `branch` != "vscode-extension"
  Return *[Preserve a stage on another branch](result.error(code="release.editor_branch", message="stage must remain on vscode-extension", operation="editor_workspace"))*.

`origin_argv` =

| argument |
|---|
| "git" |
| "remote" |
| "get-url" |
| "origin" |

We **observed = [Read the stage's source repository](command(argv=`origin_argv`, cwd=`directory`, root=`root`))**.
1. not access(`observed`, "ok")
  Return *observed*.
We **origin = [Keep the actual origin URL](text.str_trim(s=access(access(observed, "value"), "stdout")))**.
1. not [The source belongs to this release project](table.contains(list=`remotes`, item=`origin`))
  Return *[Refuse an unrelated editor stage](result.error(code="release.editor_origin", message="stage origin is outside the maintained project transports", operation="editor_workspace"))*.

`stage` =

| directory | branch |
|---|---|
| `directory` | `branch` |

Return *[The inspected isolated editor checkout](result.ok(value=`stage`))*.


## extension

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with an isolated editor-branch commit and confirmed remote SHA |

Use `version` and `root`="." for an authorized full/cli release. This handbook
prepares a separate clone under .marqdo/release and checks out only the
vscode-extension branch there. Main never receives the ignored editor source.
A failed existing stage is resumed without reset or deletion so local evidence
remains reviewable. npm updates package.json and its lockfile; install and
compile run before that version commit is pushed. Unrelated staged files are
refused before npm modifies anything. Repeating a successful build with unchanged
version files skips the empty commit and resumes exact-SHA push verification.
An editor-branch failure leaves main unchanged and retains its separate workspace.
This task requires Node/npm and network access through plugin. CI packages
the VSIX from the pushed branch; this local task verifies compilation but does
not upload a separate Marketplace package. A compile or push failure propagates
and leaves the stage available for inspection. No model-generated terminal
program is needed to repeat these same mechanics for the next release.

We **planned = [Accept a valid confirmed editor version](model.plan(version=`version`, mode="cli"))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

We **workspace = [Prepare or resume the isolated editor-source checkout](editor_workspace(version=`version`, root=`root`))**.
1. not access(`workspace`, "ok")
  Return *workspace* with its failure intact.

We **cwd = [Keep the editor checkout separate from main](access(access(workspace, "value"), "directory"))**.

`reviewed_paths` =

| path |
|---|
| "vscode-extension/package.json" |
| "vscode-extension/package-lock.json" |

We **reviewed = [Protect unrelated editor index work before npm changes versions](review_index(paths=`reviewed_paths`, cwd=`cwd`, root=`root`))**.
1. not access(`reviewed`, "ok")
  Return *reviewed* before installing or versioning.


`version_argv` =

| argument |
|---|
| "npm" |
| "version" |
| `version` |
| "--no-git-tag-version" |
| "--allow-same-version" |
| "--prefix" |
| "vscode-extension" |

We **updated = [Update the editor package and lockfile version](command(argv=`version_argv`, cwd=`cwd`, root=`root`))**.
1. not access(`updated`, "ok")
  Return *updated* with its failure intact.

`install_argv` =

| argument |
|---|
| "npm" |
| "ci" |
| "--no-audit" |
| "--prefix" |
| "vscode-extension" |

We **installed = [Install the declared editor build dependencies](command(argv=`install_argv`, cwd=`cwd`, root=`root`, timeout=600))**.
1. not access(`installed`, "ok")
  Return *installed* with its failure intact.

`compile_argv` =

| argument |
|---|
| "npm" |
| "run" |
| "compile" |
| "--prefix" |
| "vscode-extension" |

We **compiled = [Compile the editor source before publishing it](command(argv=`compile_argv`, cwd=`cwd`, root=`root`, timeout=300))**.
1. not access(`compiled`, "ok")
  Return *compiled* with its failure intact.

`stage_argv` =

| argument |
|---|
| "git" |
| "add" |
| -- |
| "vscode-extension/package.json" |
| "vscode-extension/package-lock.json" |

We **staged = [Stage only the editor version files](command(argv=`stage_argv`, cwd=`cwd`, root=`root`))**.
1. not access(`staged`, "ok")
  Return *staged* with its failure intact.

`changed_args` =

| argument |
|---|
| "git" |
| "diff" |
| "--cached" |
| "--name-only" |

We **changes = [Observe whether the successful rebuild changed the version files](command(argv=`changed_args`, cwd=`cwd`, root=`root`))**.
1. not access(`changes`, "ok")
  Return *changes* with the index diagnostic.
We **changed = [Retain the actual staged changes](text.str_trim(s=access(access(changes, "value"), "stdout")))**.
1. `changed` != ""
  `commit_argv` =

  | argument |
  |---|
  | "git" |
  | "commit" |
  | "-m" |
  | "vscode-extension: bump to `version` for release" |

  We **committed = [Commit the compiled version when there is a real change](command(argv=`commit_argv`, cwd=`cwd`, root=`root`))**.
  1. not access(`committed`, "ok")
    Return *committed* with its hook or commit failure.

Return *[Push only the editor branch and verify its SHA](push(refspec="HEAD:refs/heads/vscode-extension", cwd=`cwd`, root=`root`))*.

## debian

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result reporting whether a Debian source stanza was added |

Use `version` after its release plan has been validated. The existing
changelog supplies the maintainer identity; Marqdo reads it, checks whether that
upstream already has a revision, and creates the dated stanza through native
file IO. Asia/Shanghai supplies the release date, independent of the execution
host's default timezone. This is a local metadata edit, not a source upload.

An existing revision for this upstream is retained, including packaging fixes.
The source archive rules remain the packaging tool's responsibility: never
rewrite an orig archive already accepted by Launchpad. This helper needs fs.write
and time.read, preserves older changelog entries, and returns a recoverable
failure if the old maintainer line cannot be identified.

We **previous = [Read the actual Debian changelog](fs.read_text(path="debian/changelog"))**.
1. [This upstream already has a packaging revision](text.starts_with(text=`previous`, prefix="marqdo (" + `version` + "-"))
  Return *[Preserve its reviewed revision](result.ok(value=False))*.
We **maintainer = [Retain the changelog's maintainer identity](re.find(text=`previous`, pattern="(?m)^ -- .+?  "))**.
1. `maintainer` == None
  Return *[Require an existing maintainer identity](result.error(code="release.debian_maintainer", message="no changelog maintainer line", operation="local.debian"))*.
We **now = [Read the actual release clock](datetime.now())** and
**local = [Use the release timezone](datetime.in_zone(dt=`now`, zone="Asia/Shanghai"))** and
**date = [Format the Debian timestamp](datetime.format(dt=`local`, style="%a, %d %b %Y %H:%M:%S %z"))** and
**stanza = "marqdo (" + version + "-1ppa1) noble; urgency=medium\n\n  * Release " + version + "; see the reviewed upstream CHANGELOG.\n\n" + maintainer + date + "\n\n"**.
We **written = [Atomically prepend this packaging entry](fs.atomic_write(path="debian/changelog", text=`stanza` + previous))**.
Return *[Report the actual local metadata change](result.ok(value=True))*.
