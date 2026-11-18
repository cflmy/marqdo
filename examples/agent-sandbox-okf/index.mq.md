---
markup: occlusion
title: agent-sandbox-okf — trial then commit
description: >
  O1 example: opt-in sandbox trial with GFM-table returns and link-index reads.
import:
  agent: ext/ai/agent.mq.md
  fs: lib/fs.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

The two trials intentionally differ only in commit. First inspect a successful dry trial and verify that note.txt has not appeared in the host directory. Then run the committed trial, read the host file and compare its actual content. This separates an observed proposal from an observed filesystem change; neither ok alone nor a proposed path establishes that a commit happened.

Use a disposable filesystem root: the example replaces its named _example_sbx directory and leaves the committed evidence there for inspection. It does not promise to roll back a committed external effect. Read the returned profile as part of the trial evidence, and keep failures visible rather than manufacturing a success marker when the expected file is absent.

Short demo: write via `agent.trial`, prove the host is clean until commit.

We first **had = [Compute had](fs.exists(path="_example_sbx"))** for the next step.
1. `had`
  We now **[Call fs.remove](> fs.remove path="_example_sbx")**.
2. *
  We first **_ = 0** for the next step.

We now **[Call fs.make dir](> fs.make_dir path="_example_sbx")**.

We first **dry = [Compute dry](agent.trial(root="_example_sbx", write_path="note.txt", text="sandbox-okf", commit=False))** for the next step.
We first **dry_ok = [ok](access(`dry`, "ok"))** for the next step.
We first **dry_committed = [committed](access(`dry`, "committed"))** for the next step.
1. `dry_ok`
  1. not `dry_committed`
    We now **[Display this step's result](> print text="dry-ok")**.
  2. *
    We now **[Display this step's result](> print text="dry-bad-commit")**.
2. *
  We now **[Display this step's result](> print text="dry-fail")**.

We first **leaked = [Compute leaked](fs.exists(path="_example_sbx/note.txt"))** for the next step.
1. `leaked`
  We now **[Display this step's result](> print text="host-leaked")**.
2. *
  We now **[Display this step's result](> print text="host-clean")**.

We first **wet = [Compute wet](agent.trial(root="_example_sbx", write_path="note.txt", text="sandbox-okf", commit=True))** for the next step.
We first **wet_ok = [ok](access(`wet`, "ok"))** for the next step.
We first **wet_committed = [committed](access(`wet`, "committed"))** for the next step.
1. `wet_ok`
  1. `wet_committed`
    We now **[Display this step's result](> print text="wet-ok")**.
  2. *
    We now **[Display this step's result](> print text="wet-no-commit")**.
2. *
  We now **[Display this step's result](> print text="wet-fail")**.

We first **body = [Compute body](fs.read_text(path="_example_sbx/note.txt"))** for the next step.
1. `body` == "sandbox-okf"
  We now **[Display this step's result](> print text="proof-visible")**.
2. *
  We now **[Display this step's result](> print text="proof-missing")**.

We first **pname = [name](access([profile](access(`wet`, "profile")), "name"))** for the next step.
1. `pname` == "default"
  We now **[Display this step's result](> print text="profile-default")**.
2. *
  We now **[Display this step's result](> print text="profile-other")**.
