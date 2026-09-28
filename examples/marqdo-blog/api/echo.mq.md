---
type: endpoint
method: POST
path: /api/echo
request: json
response: json
auth: none
---

# main

Echo request fields bound via Artifact Metadata (`arg` overlays).

**msg = message**
1. not `msg`
  **msg = ""**
2. *

`out` =

| ok | message |
|----|---------|
| True | `msg` |

*out*
