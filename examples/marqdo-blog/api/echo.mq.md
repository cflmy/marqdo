---
markup: occlusion
type: endpoint
method: POST
path: /api/echo
request: json
response: json
auth: none
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Result or browser effect record assembled below |

Echo request fields bound via Artifact Metadata (`arg` overlays).

We first **msg = `message`** for the next step.
1. not `msg`
  We first **msg = ""** for the next step.
2. *

`out` =

| ok | message |
|----|---------|
| True | `msg` |

After these steps, return *out*.
