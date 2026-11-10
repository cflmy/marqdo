---
markup: occlusion
title: resolve_config
capability: config.resolve
status: stable
---

Use when: final normalized configuration is required.

# config

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Object/type namespace for the child functions below; call the intended child directly |

## resolve_config

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Filesystem path under the granted root; this callback's prose identifies URL paths separately |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; this unit completes the operations described below without a data return |
The shared input protocol also accepts `path`. We **[Accept this documented input without changing the result](`path`)**.

Input `path`, used by the steps below.

`defaults` =

| host | port |
|------|------|
| localhost | 8080 |

* defaults
