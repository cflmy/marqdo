---
title: Normalize settings (near-duplicate)
status: stable
---

Near-duplicate of resolve_config for duplicate detection demos.

Use when: Caller wants normalized settings map.

# settings

## normalize_settings
    + `path`

Normalize settings from a path with host/port defaults.

`defaults` =

| host | port |
|------|------|
| localhost | 8080 |

* defaults

## merge_settings
    + `base`
    + `overlay`

Merge overlay onto base settings — semantically close to merge_config.

* overlay
