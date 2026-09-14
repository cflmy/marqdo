---
title: lib/plugin
description: Load optional native ABI plugins (.dll / .so / .dylib)
---

Load optional native ABI plugins (.dll / .so / .dylib).

## load
    + `path`

Load a native plugin from path.

Caller: [plugin.load] path=`so`

*> host_plugin_load path=`path`*

## unload

Unload the current native plugin.

*> host_plugin_unload*

## list

List loaded native plugins.

*> host_plugin_list*

## native_path
    + `name`

Resolve installed or local native plugin path (e.g. agent).

*> host_ext_native_path name=`name`*
