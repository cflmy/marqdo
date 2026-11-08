---
markup: occlusion
title: ext/data/cache
description: Cache Resource (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

Cache values belong to the selected backend and process lifetime. Use memory: only for process-local demonstration; choose and configure persistent backends explicitly. [The data entry](index.mq.md) demonstrates set, get and deletion through one receiver. Retain native ok/error fields and expiry behavior when extending this leaf; a cache miss is not evidence that a remote write succeeded.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/data: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

# cache

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | "memory:" | Resource connection URL; scheme selects the documented backend |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Key–value cache. Use `memory:` for in-process (tests / single process) or `redis://host:6379/0` for Redis. |

Use `url`="memory:" as the inputs for this operation.

Key–value cache. Use `memory:` for in-process (tests / single process) or `redis://host:6379/0` for Redis.

We **[Load the native plugin required by this operation](_load())**.
Return *[Key–value cache. Use memory: for in-process (tests / single process) or redis://host:6379/0 for Redis.](web_cache_new(url=`url`))*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read the cached value associated with a key; a miss is reported by the cache result. |

Use `key` as the inputs for this operation.

Read the cached value associated with a key; a miss is reported by the cache result.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Read the cached value associated with a key; a miss is reported by the cache result.](web_cache_get(url=`url`, key=`key`))*.

## set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |
| value | any | required | Value supplied to this operation; immutable transformations return a new value |
| ttl | any | None | Expiry duration in seconds; None uses the backend default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Optional `ttl` is seconds until expiry. |

Use `key`, `value`, `ttl`=None as the inputs for this operation.

Optional `ttl` is seconds until expiry.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Optional ttl is seconds until expiry.](web_cache_set(url=`url`, key=`key`, value=`value`, ttl=`ttl`))*.

## del

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delete a cached key without changing other entries. |

Use `key` as the inputs for this operation.

Delete a cached key without changing other entries.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Delete a cached key without changing other entries.](web_cache_del(url=`url`, key=`key`))*.

## exists

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Test whether the cache currently holds the selected key. |

Use `key` as the inputs for this operation.

Test whether the cache currently holds the selected key.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Test whether the cache currently holds the selected key.](web_cache_exists(url=`url`, key=`key`))*.

## ttl

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read the remaining lifetime of a cached key using the backend TTL convention. |

Use `key` as the inputs for this operation.

Read the remaining lifetime of a cached key using the backend TTL convention.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Read the remaining lifetime of a cached key using the backend TTL convention.](web_cache_ttl(url=`url`, key=`key`))*.
