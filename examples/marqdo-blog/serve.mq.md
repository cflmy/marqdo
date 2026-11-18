---
markup: occlusion
title: Callable Artifact blog launcher
import:
  web: ext/web/web.mq.md
  posts: db/posts.mq.md
---

This launcher starts the ordinary Markdown/data-source Documents and declared JSON Endpoints. It owns data preparation and listener lifetime; the index remains readable page content. From the checkout use marqdo call examples/marqdo-blog/serve.mq.md main with --effects plugin,fs.write,network.write and --fs-root examples/marqdo-blog.

## main

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/marqdo-blog" | Explicit discovery root relative to the checkout |
| port | number | 18081 | Local loopback listen port |
| url | any | None | None uses the retained site-root database; tests supply an owned URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener after database preparation and declared-route discovery |

Use `root`="examples/marqdo-blog", `port`=18081 and `url`=None from the checkout. The ordinary callable heading makes this workflow available through call, while Document bodies remain Markdown. Native filesystem resolution keeps SQLite under the actual granted site root, independently of the terminal working directory. Existing articles are retained and only an empty table receives demonstration rows.

Discovery follows the actual route metadata for Home, About, slug detail, Ping and Echo. Ordinary Documents keep their existing request-time data-source behavior. Starting this workflow is an explicit local network action, not a side effect of reading a library. The listener blocks until the supervising caller stops the process; no detached daemon or database reset is implied.

We **store = [Prepare the retained current blog data](posts.open(url=`url`))**.
Return *[Discover and listen with the actual site context](web.serve(root=`root`, host="127.0.0.1", port=`port`, db=`store`, static_dir="public"))*.
