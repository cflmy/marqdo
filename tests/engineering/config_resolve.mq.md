---
title: Resolve configuration
capability: config.resolve
status: stable
---

Config resolution capability for Engineering Knowledge demos.

Use when: The caller needs the final normalized configuration.

Do not use when: Only raw configuration should be inspected.

# config

## resolve_config
    + `path`

`defaults` =

| host | port |
|------|------|
| localhost | 8080 |

* defaults

## merge_config
    + `base`
    + `overlay`

Merge overlay keys onto base configuration map.

* overlay
