---
markup: occlusion
title: Native audit of the maintained Marqdo repository
import:
  files: ./files.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

This repository task discovers the maintained source scope using native file
operations and passes it to the compiler-backed audit. It is an actual callable
workflow, not a generated inventory script. Read the development index for
bootstrap and the programming guide for how to resolve findings.

Only lib, ext, public, examples, skills and doc are selected. Intentional invalid
tests and protected man-test programs are outside this scope. Historical
.compat.mq.md examples remain compatibility evidence. Compiler warnings on
documented placeholders stay in the report; they do not turn into fake logic.
A report records the current source state, not a promise that the task is done.

## audit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| write_report | bool | False | Also write the actual Result to .marqdo/audit/handbooks.json |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual repository source audit, with evidence retained on failure |

Use `write_report`=False from the checkout root with plugin and fs.read.
Each selected directory is discovered natively; no Python, shell enumeration or
maintained list of hundreds of filenames is needed. Hidden staging trees,
dependency installs and build outputs are pruned. The historical compatibility
suffix is the only source-name exclusion within this maintained scope.

The audit does not run found programs or load their imports. It reports source
parser/contract errors, missing callable return contracts and executable-unit
narrative gaps. Actual dependency checks, behavior, rendering and service
publication require their own workflow tests. An incomplete source report is
returned as a failed Result so --require-ok stops automation.

Set write_report=True and grant fs.write to retain the same actual evidence
under the ignored audit directory. This intentionally replaces that generated
report; it never rewrites a program to silence a finding. A later audit reflects
later files, and a historical report does not establish compliance of new edits.
Missing selected directories retain a native error rather than silently narrowing
the task to make it appear complete.

`directories` =

| path |
|---|
| "lib" |
| "ext" |
| "public" |
| "examples" |
| "skills" |
| "doc" |

`excluded` =

| pattern |
|---|
| "**/.marqdo/**" |
| "**/node_modules/**" |
| "**/target/**" |

We **paths = [Start the currently discovered source scope](table.empty_list())**.
- [Discover maintained modules through native file operations](for directories as directory)
  We **names = [Discover actual Marqdo source files](fs.find(path=`directory`, pattern="**/*.mq.md", exclude=`excluded`))**.
  - [Keep current sources while preserving historical compatibility examples](for names as relative)
    1. not [This file is an explicitly historical compatibility source](text.ends_with(text=`relative`, suffix=".compat.mq.md"))
      We **path = directory + "/" + relative** and **paths = [Retain this actual maintained source](table.append(list=`paths`, item=`path`))**.
We **observed = [Audit the actual discovered sources without executing them](files.audit(paths=`paths`))**.
1. `write_report`
  We **directory = [Prepare only the ignored generated audit directory](fs.make_dirs(path=".marqdo/audit"))** and
  **saved = [Retain the same real success or failure evidence](fs.write_json(path=".marqdo/audit/handbooks.json", value=`observed`))**.
Return *observed* with all incomplete findings intact.
