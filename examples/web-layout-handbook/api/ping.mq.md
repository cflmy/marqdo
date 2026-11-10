---
markup: occlusion
type: endpoint
path: /api/ping
method: GET
request: none
response: json
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

`result` =

| ok | ping |
|----|------|
| True | pong |

After these steps, return *result*.
