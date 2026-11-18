---
markup: occlusion
title: Mid stdlib — datetime and url
description: Calendar moments + URL/query parse (Mid2 M7)
import:
  dt: lib/datetime.mq.md
  url: lib/url.mq.md
---

# Time and addresses without plugins


Moments are maps `{unix, zone, iso}`. Parse HTTPS URLs and round-trip query strings in `lib/url` (segment encoding stays in `net.url_encode`).

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **epoch = [Compute epoch](dt.from_unix(unix=0, zone="Asia/Shanghai"))** for the next step.
We now **[Display this step's result](> print text=[iso](access(`epoch`, "iso")))**.

We first **u = [Compute u](url.parse(text="https://example.com/path?q=1#top"))** for the next step.
We now **[Display this step's result](> print text=[host](access(`u`, "host")))**.
We now **[Display this step's result](> print text=[query](access(`u`, "query")))**.
