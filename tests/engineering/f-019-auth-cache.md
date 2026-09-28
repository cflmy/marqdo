---
type: Failure
id: F-019
title: Do not cache authorization by user only
task: add_cache_layer
status: resolved
applies_to:
  - config.resolve
---

# Do not cache authorization result

## Failed approach

Caching permission checks by user only.

## Failure

Permissions changed during role update.

## Correct approach

Cache by user_id, resource_id, and permission_version.
