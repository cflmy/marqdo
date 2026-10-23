---
title: agent-sandbox-okf — trial then commit
description: >
  O1 example: opt-in sandbox trial with GFM-table returns and link-index reads.
import:
  agent: ext/ai/agent.mq.md
  fs: lib/fs.mq.md
---

# main

Short demo: write via `agent.trial`, prove the host is clean until commit.

**had = fs.exists path="_example_sbx"**
1. `had`
  > fs.remove path="_example_sbx"
2. *
  **_ = 0**

> fs.make_dir path="_example_sbx"

**dry = agent.trial root="_example_sbx" write_path="note.txt" text=sandbox-okf commit=False**
**dry_ok = [ok](dry)**
**dry_committed = [committed](dry)**
1. `dry_ok`
  1. not `dry_committed`
    > print text=dry-ok
  2. *
    > print text=dry-bad-commit
2. *
  > print text=dry-fail

**leaked = fs.exists path="_example_sbx/note.txt"**
1. `leaked`
  > print text=host-leaked
2. *
  > print text=host-clean

**wet = agent.trial root="_example_sbx" write_path="note.txt" text=sandbox-okf commit=True**
**wet_ok = [ok](wet)**
**wet_committed = [committed](wet)**
1. `wet_ok`
  1. `wet_committed`
    > print text=wet-ok
  2. *
    > print text=wet-no-commit
2. *
  > print text=wet-fail

**body = fs.read_text path="_example_sbx/note.txt"**
1. `body` == "sandbox-okf"
  > print text=proof-visible
2. *
  > print text=proof-missing

**pname = [name]([profile](wet))**
1. `pname` == "default"
  > print text=profile-default
2. *
  > print text=profile-other
