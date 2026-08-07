---
title: lib/plugin
description: Load optional native ABI plugins (.dll / .so / .dylib)
---

## load
    + `path`

**> host_plugin_load path=`path`**

## unload

**> host_plugin_unload**

## list

**> host_plugin_list**

## native_path
    + `name`

Resolve installed / local native plugin path (e.g. `agent`).

**> host_ext_native_path name=`name`**
