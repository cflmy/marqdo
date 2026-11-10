---
markup: occlusion
title: Request-time database View resources
---

Use this single-file resource handbook when a recursively composed page needs current database rows on each request. Construction is pure: it records a query without connecting, selecting or rendering HTML. Place the result in component.resource and retain view: main for startup layout assembly. The application supplies its actual database resource when serving.

Do not combine view: main with automatic data_source metadata. Ordinary Documents keep that metadata path; explicit View pages use this query Resource instead. A missing database, invalid condition or backend failure remains a request error, never a fabricated empty list. See [the request-resource design](../../doc/design/web-view-resources.md).

## query

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| source | text | required | Existing table on the serving application's database |
| where | any | None | Native condition map or list; route placeholders resolve from actual request params |
| order | text | "" | Native database ordering, such as -created_at |
| limit | number | 50 | Integer from 1 through 1000 |
| detail | bool | False | False renders cards; True renders the first matching full article |
| link_prefix | text | "/post/" | Site-relative prefix applied to row slugs, or empty for no prefix |
| tenant_scope | bool | False | Require an attached tenant policy and current tenant filter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Pure data_query Resource; no query has executed yet |

Use `source`, `where`=None, `order`="", `limit`=50, `detail`=False, `link_prefix`="/post/" and `tenant_scope`=False. The native renderer validates the contract before listening, then selects current records for each request. Card titles and summaries are escaped; article content uses the existing Markdown renderer. Detail mode prefers the complete body/content even when a summary exists.

Rows follow the current Document convention: title, slug or href, summary, body or content, optional created_at/meta and tag. This fixed projection preserves the native card/article surface without restoring a private binding-table DSL. Layout remains in parent components; this resource only supplies one dynamic content region. For a different domain projection, keep a separate documented resource rather than passing arbitrary page configuration.

Conditions are copied before replacing route placeholders, so one request cannot mutate another's retained definition. Path parameters take precedence over query keys. A requested tenant scope requires actual tenant configuration and context; direct selectors do not prove user membership. Link visibility and data filtering do not replace authentication or server authorization.

This constructor needs no effects. Serving needs the actual web/database plugin and the launcher grants. Rendering the Resource offline without request context reports that requirement explicitly. Reading this file does not authorize a query, write or network listener.

`resource` =

| _type | source | where | order | limit | detail | link_prefix | tenant_scope |
|---|---|---|---|---|---|---|---|
| data_query | `source` | `where` | `order` | `limit` | `detail` | `link_prefix` | `tenant_scope` |

Once its intended request behavior is explicit, return *resource* for placement in a View resource node.
