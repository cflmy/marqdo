---
title: agent trial sandbox opt-in — host unchanged until commit
description: >
  O1: agent.trial write stays upper until commit=True.
  Prefer link index over json.get.
import:
  agent: ext/ai/agent.mq.md
  sandbox: ext/ai/sandbox.mq.md
  fs: lib/fs.mq.md
---

# main

Reset leftover workspace from prior runs.

**had = fs.exists path="_sbx_optin"**
1. `had`
  > fs.remove path="_sbx_optin"
2. *
  **_ = 0**

> fs.make_dir path="_sbx_optin"

Trial with `commit=False`: upper write must not touch the host tree.

**dry = agent.trial root="_sbx_optin" write_path="proof.txt" text=optin-ok commit=False**
**dry_ok = [ok](dry)**
**dry_committed = [committed](dry)**
> print text=`dry_ok`
> print text=`dry_committed`

**before = fs.exists path="_sbx_optin/proof.txt"**
> print text=`before`

Trial with `commit=True`: host sees the file after success.

**wet = agent.trial root="_sbx_optin" write_path="proof.txt" text=optin-ok commit=True**
**wet_ok = [ok](wet)**
**wet_committed = [committed](wet)**
> print text=`wet_ok`
> print text=`wet_committed`

**after = fs.read_text path="_sbx_optin/proof.txt"**
> print text=`after`

Profile row is attached (link-index readable).

**prof = [profile](wet)**
**pname = [name](prof)**
> print text=`pname`
