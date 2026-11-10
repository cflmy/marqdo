---
markup: occlusion
title: Observe and update GitHub release evidence
import:
  bridge: ./_bridge.mq.md
  model: ./model.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  fs: lib/fs.mq.md
---

GitHub release tasks keep source identity and publication state explicit. Start
with status for an existing version; notes is a separate mutation that consumes
a reviewed bilingual file. The same release metadata feeds assets.download and
assets.status, so name, size and digest checks do not drift between channels.

Every task returns Result.ok/value/error. Check that inner ok before consuming
value, then inspect complete for gates. A successful query may truthfully report
an incomplete release. plugin grants the native boundary OS/network
authority; credentials are resolved privately from the environment or existing
credential stores. They are never parameters, return values or journal bindings.

Example, from the checkout root:

```sh
marqdo call skills/marqdo-release/github.mq.md status 'version="2.6.0"' --effects plugin --fs-root . --json
```

The command is read-only. A future release uses the version the user actually
specified. A red EK gate, missing Linux build or wrong-commit green run keeps the
release incomplete even if a GitHub download page already exists.

## request

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | API path scoped to /repos/cflmy/marqdo/ |
| method | text | "GET" | GET for observation; PATCH for notes; POST for explicit publication |
| data | any | None | JSON request body; credentials are never arguments |
| missing | bool | False | Whether GET 404 is represented as a successful None |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing service JSON or a transport error |

Use `path`, `method`="GET", `data`=None, `missing`=False and `root`=".".
This function is the authenticated GitHub boundary used by the tasks below.
The proxy is tried before direct GitHub; GET and idempotent note PATCH may be
retried, while an uncertain POST is returned as failure. API failures retain a
code and service name without exposing tokens, headers or signed download URLs.
Reading requires plugin because the native engineering plugin owns HTTPS.

`payload` =

| path | method | data | missing |
|---|---|---|---|
| `path` | `method` | `data` | `missing` |

Return *[Request GitHub JSON through the maintained transport](bridge.request(operation="github", payload=`payload`, root=`root`))*.

## release

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing release metadata, or None when not published |

Use `version`, `mode`="full" and `root`=".".
The mode selects vX.Y.Z or ext-vX.Y.Z through the same pure plan used for
packaging. A missing release is an observable state, not proof that a previous
publish command failed before creating anything. Callers can distinguish it
from a network or authentication failure by inspecting Result.ok first.
This query does not create a release and never invokes the old Windows-only
publication script, whose assumptions predate embedded standard libraries.

We **planned = [Derive the correct release tag](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

We **plan_data = [Read the accepted tag contract](access(planned, "value"))** and
**path = "/repos/cflmy/marqdo/releases/tags/" + access(plan_data, "tag")**.
Return *[Observe this exact release without creating one](request(path=`path`, missing=True, root=`root`))*.

## commit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| tag | text | required | Exact tag from the release plan |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with the commit SHA targeted by a lightweight or annotated tag |

Use `tag` and `root`=".". Annotated tags point first to a tag object,
whereas lightweight tags already point to a commit. Follow that distinction
before matching a workflow run. Comparing a tag-object SHA with head_sha would
silently fail to find the correct green run. Only the single annotated object
created by this handbook is accepted; nested tag objects remain an error.
A missing tag is reported as a transport failure rather than guessed from main.

We **reference = [Read the exact remote tag reference](request(path="/repos/cflmy/marqdo/git/ref/tags/" + tag, root=`root`))**.
1. not access(`reference`, "ok")
  Return *reference* with its failure intact.

We **object = [Read the tag target](access(access(reference, "value"), "object"))**.
1. access(`object`, "type") == "tag"
  We **annotation = [Resolve the annotated tag object](request(path="/repos/cflmy/marqdo/git/tags/" + access(object, "sha"), root=`root`))**.
  1. not access(`annotation`, "ok")
    Return *annotation* without substituting main.
  We **object = [Read the annotated commit target](access(access(annotation, "value"), "object"))**.
1. access(`object`, "type") != "commit"
  Return *[Reject an unexpected tag target](result.error(code="release.tag_target", message="tag must resolve to a commit", operation="github.commit"))*.
Return *[The immutable release commit](result.ok(value=access(object, "sha")))*.

## select_run

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| runs | list | required | Workflow runs ordered newest first by the GitHub API |
| sha | text | required | Exact commit that this gate must cover |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Newest run for this commit, or None |

Use `runs` and `sha` to select one observed gate attempt. The first matching
head_sha is the most recent attempt provided by the API; an older successful
attempt cannot mask a later failure for the same commit. Runs for other commits
are irrelevant even if their badges are green. No API access or waiting occurs
here, making this decision directly testable with recorded service responses.
If the bounded query contains no matching run, return None and keep the gate
incomplete. The caller may fetch again after CI starts.

- [Inspect workflow runs in their returned order](for runs as run)
  1. access(`run`, "head_sha") == `sha`
    Return *run* as the observed attempt for this source.

Return *None* when this commit has no observed run.

## gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| workflow | text | required | release.yml, extension-release.yml or executable-knowledge.yml |
| sha | text | required | Commit that the workflow must cover |
| required | list | required | Required job names, such as windows and linux |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with completion, observed run and required-job evidence |

Use `workflow`, `sha`, `required` and `root`=".". Completion requires both a
successful run for the exact commit and every named job with conclusion success.
A skipped or missing Linux job cannot pass just because the Windows job created
a release. This query checks at most 100 recent runs, returns immediately for
queued/running CI, and does not rerun or cancel anything. Polling belongs to the
agent's bounded observation cycle, not an unbounded loop hidden in a tool.
Knowledge checks use the same rule, so uploaded assets never replace a green EK
gate. Missing evidence returns complete=False with the observed run retained.

We **response = [Read recent attempts for one workflow](request(path="/repos/cflmy/marqdo/actions/workflows/" + workflow + "/runs?per_page=100", root=`root`))**.
1. not access(`response`, "ok")
  Return *response* with its failure intact.

We **run = [Choose the latest attempt for this commit](select_run(runs=access(access(response, "value"), "workflow_runs"), sha=`sha`))**.
We **complete = False** and **jobs = [Start an empty required-job evidence list](table.empty_list())**.
1. `run` != None
  We **answer = [Read this attempt's actual jobs](request(path="/repos/cflmy/marqdo/actions/runs/" + access(run, "id") + "/jobs?per_page=100", root=`root`))**.
  1. not access(`answer`, "ok")
    Return *answer* with its failed observation.
  We **jobs = [Keep the observed jobs](access(access(answer, "value"), "jobs"))** and
  **complete = [Require a successful completed run](access(run, "status") == "completed" and access(run, "conclusion") == "success")**.
- [Check that every required platform or knowledge job passed](for required as name)
  We **matched = False** before examining this job name.
  - [Match this job against observed evidence](for jobs as job)
    1. access(`job`, "name") == `name` and access(`job`, "conclusion") == "success"
      We **matched = True** because the named job succeeded.
  We **complete = complete and matched** to preserve every missing-job failure.

`evidence` =

| complete | sha | run | jobs |
|---|---|---|---|
| `complete` | `sha` | `run` | `jobs` |

Return *[The observed gate and its evidence](result.ok(value=`evidence`))*.

## status

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with exact-source Release and executable-knowledge gates |

Use `version`, `mode`="full" and `root`="." to inspect an existing release.
The Release workflow is matched against the resolved release tag; executable
knowledge is matched against the current remote main tip. Both sources are
returned so a reviewer can see if main has advanced. Every mode requires both
Windows and Linux jobs on the tag commit. Independent packs use extension-release.yml;
full/cli use release.yml. Existing assets cannot replace missing build evidence.
A missing release or an unfinished gate leaves complete=False. This task does
not mutate GitHub, push local work, create tags, or change the requested version.
The returned service metadata is evidence for the next task, not a permission
to publish a different version or overwrite a release's existing files.

We **published = [Inspect the exact version's release](release(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`published`, "ok")
  Return *published* with its failure intact.

We **planned = [Read the pure release contract](model.plan(version=`version`, mode=`mode`))**.
1. not access(`planned`, "ok")
  Return *planned* with its failure intact.

We **release_data = [Keep the published metadata or None](access(published, "value"))** and
**tag = [Read the planned tag](access(access(planned, "value"), "tag"))**.
We **tagged = [Resolve its actual source commit](commit(tag=`tag`, root=`root`))**.
1. not access(`tagged`, "ok")
  Return *tagged* with its failure intact.

We **main = [Observe the remote main tip](request(path="/repos/cflmy/marqdo/git/ref/heads/main", root=`root`))**.
1. not access(`main`, "ok")
  Return *main* with its failure intact.

We **main_sha = [Read main's commit SHA](access(access(access(main, "value"), "object"), "sha"))**.

`knowledge_jobs` =

| name |
|---|
| gate |

We **knowledge = [Verify EK on the actual remote main commit](gate(workflow="executable-knowledge.yml", sha=`main_sha`, required=`knowledge_jobs`, root=`root`))**.
1. not access(`knowledge`, "ok")
  Return *knowledge* with its failure intact.

We **workflow = "release.yml"** for a CLI or full release.
1. `mode` == "ext"
  We **workflow = "extension-release.yml"** for independent extension builds.

`platform_jobs` =

| name |
|---|
| windows |
| linux |

We **platforms = [Require both platform builds for the tagged source](gate(workflow=`workflow`, sha=access(tagged, "value"), required=`platform_jobs`, root=`root`))**.
1. not access(`platforms`, "ok")
  Return *platforms* with its failed gate observation.
We **release_gate = [Keep actual platform evidence](access(platforms, "value"))** and **release_complete = [Read the exact-tag completion decision](access(release_gate, "complete"))**.

We **all_complete = [Require the published release and both source-specific gates](release_data != None and release_complete and access(access(knowledge, "value"), "complete"))**,
**release_sha = [Keep the resolved release commit](access(tagged, "value"))** and
**knowledge_state = [Keep the exact-main knowledge evidence](access(knowledge, "value"))** before forming the record.

`status_data` =

| complete | release | release_sha | main_sha | release_ci | knowledge_ci |
|---|---|---|---|---|---|
| `all_complete` | `release_data` | `release_sha` | `main_sha` | `release_gate` | `knowledge_state` |

Return *[The release's observed gates and source identities](result.ok(value=`status_data`))*.

## notes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | Stable X.Y.Z explicitly chosen by the user |
| path | text | required | Reviewed UTF-8 bilingual notes file under the filesystem root |
| mode | text | "full" | full, cli or ext; determines the deliverable contract |
| root | text | "." | Checkout directory; pass --fs-root . from its root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result of updating only the existing release body |

Use `version`, `path`, `mode`="full" and `root`=".". This is an explicit
GitHub mutation and needs the user's release instruction; reading this handbook
or asking for status does not authorize it. The prepared notes must contain
Chinese and English highlights, the actual asset matrix, installation routes,
core-surface delta and validation limits. The author remains responsible for
those claims; this function uploads the reviewed file rather than inventing it.
Reading uses fs.read in addition to plugin. UTF-8 text and its intentional
newlines travel in a structured JSON body, never through shell quoting.

We **published = [Locate the existing release to edit](release(version=`version`, mode=`mode`, root=`root`))**.
1. not access(`published`, "ok")
  Return *published* with its failure intact.

We **release_data = [Read its metadata](access(published, "value"))**.
1. `release_data` == None
  Return *[Do not invent a release when only editing notes](result.error(code="release.missing", message="release does not exist; wait for CI or attach an ext release", operation="github.notes"))*.
We **body = [Read the exact reviewed bilingual note file](fs.read_text(path=`path`))**.

We **data = [Keep the reviewed body as one named API field](table.put(in=None, at="body", value=`body`))**.

Return *[Publish the reviewed release body](request(path="/repos/cflmy/marqdo/releases/" + access(release_data, "id"), method="PATCH", data=`data`, root=`root`))*.


## create_ext

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | required | User-confirmed stable independent extension version |
| path | text | required | Reviewed bilingual notes committed with the release source |
| sha | text | required | Exact source commit expected from the authorized tag |
| root | text | "." | Checkout root matching the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing newly created or already existing release metadata |

Use `version`, `path`, `sha` and `root`="." only for authorized extension publication. This is an explicit GitHub mutation. First resolve ext-vX.Y.Z to its actual commit and require the caller's expected SHA. Read the committed note file through native filesystem operations before requesting creation. The file must describe this version's real changes and validation limits; the task does not synthesize release claims.

An existing release is returned unchanged so a resumed upload cannot overwrite its notes or assets. A missing release is created once through the authenticated service boundary, with its exact existing tag and reviewed body. Uncertain POST completion is a failure, never an automatic replay; inspect the release before resuming. CI uses this task after packaging and then uploads with the existing gh tool without clobbering prior assets.

We **planned = [Validate the independent extension version](model.plan(version=`version`, mode="ext"))**.
1. not access(`planned`, "ok")
  Return *planned*.
We **tag = [Read the exact immutable extension tag](access(access(planned, "value"), "tag"))** and **target = [Resolve the actual remote tag commit](commit(tag=`tag`, root=`root`))**.
1. not access(`target`, "ok")
  Return *target*.
1. access(`target`, "value") != `sha`
  Return *[Refuse publication from a different source](result.error(code="release.source_mismatch", message="extension tag differs from expected commit", operation="create_ext"))*.
We **body = [Read the reviewed version-specific bilingual notes](fs.read_text(path=`path`))**.
1. `body` == ""
  Return *[Require actual reviewed notes](result.error(code="release.notes_empty", message="extension notes must not be empty", operation="create_ext"))*.
We **existing = [Observe whether an earlier creation completed](release(version=`version`, mode="ext", root=`root`))**.
1. not access(`existing`, "ok")
  Return *existing*.
1. access(`existing`, "value") != None
  Return *existing*, preserving the existing release.

`data` =

| tag_name | name | target_commitish | body | draft | prerelease |
|---|---|---|---|---|---|
| `tag` | `tag` | `sha` | `body` | False | False |

Return *[Create this exact tagged release once](request(path="/repos/cflmy/marqdo/releases", method="POST", data=`data`, root=`root`))*.
