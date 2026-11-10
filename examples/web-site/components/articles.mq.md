---
markup: occlusion
title: Current article list and slug detail
import:
  component: ext/web/component/index.mq.md
  query: ext/data/view.mq.md
  table: lib/table.mq.md
---

Both views read the same application-owned articles table for each request. Cards use summaries and slug links; detail uses the complete Markdown body. These pure descriptors do not open a second database or cache startup records. Their renderer retains the native card/article classes consumed by the original theme.

## cards

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Current article cards linked to /post/slug |

Construct this query without effects. The serving application owns its database resource and actual request execution. Cards keep the original stable id order and use the existing summary/title/slug projection. Titles and summaries are escaped text, while the full body is intentionally reserved for detail. A publish through either mounted form becomes visible on the next request.

We **resource = [Describe the actual current article list](query.query(source="articles", order="id", link_prefix="/post/"))**.
Return *[Place one request-time card responsibility](component.resource(value=`resource`, slot="articles"))*.

## detail

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | First article matching the actual captured slug, rendered with full Markdown body |

The declared /post/{slug} Document supplies the path parameter when the server resolves this query. A same-named query-string value cannot override that capture. The native renderer uses the body even when a summary is present, and it does not mutate the retained condition between requests. Missing database resources remain errors; a missing record remains an empty result under the existing query contract.

We **condition = [Keep a native record rather than a one-column list](table.put(in=None, at="slug", value="{slug}"))** and **resource = [Describe full-body slug detail](query.query(source="articles", where=`condition`, detail=True, limit=1))**.
Return *[Place the actual requested article](component.resource(value=`resource`, slot="article"))*.
