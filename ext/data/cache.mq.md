---
title: ext/data/cache
description: Cache Resource (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/data: native web plugin not found
  > sys.exit code=1
****

# cache
    + `url`="memory:"

Key–value cache. Use `memory:` for in-process (tests / single process) or `redis://host:6379/0` for Redis.

> _load
*> web_cache_new url=`url`*

## get
    + `key`

**url = [url](self)**
*> web_cache_get url=`url` key=`key`*

## set
    + `key`
    + `value`
    + `ttl`=None

Optional `ttl` is seconds until expiry.

**url = [url](self)**
*> web_cache_set url=`url` key=`key` value=`value` ttl=`ttl`*

## del
    + `key`

**url = [url](self)**
*> web_cache_del url=`url` key=`key`*

## exists
    + `key`

**url = [url](self)**
*> web_cache_exists url=`url` key=`key`*

## ttl
    + `key`

**url = [url](self)**
*> web_cache_ttl url=`url` key=`key`*

