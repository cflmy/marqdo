---
title: lib/fs
description: English filesystem wrappers
---

## read_text
    + `path`

**> host_read_text path=`path`**

## write_text
    + `path`
    + `text`

**> host_write_text path=`path` text=`text`**

## append_text
    + `path`
    + `text`

**> host_append_text path=`path` text=`text`**

## exists
    + `path`

**> host_exists path=`path`**

## list_dir
    + `path`

**> host_list_dir path=`path`**

## make_dir
    + `path`

**> host_make_dir path=`path`**

## remove
    + `path`

**> host_remove path=`path`**

## text_patch
    + `path`
    + `find`
    + `replace`

Exact FIND→REPLACE once in a UTF-8 text file. `find` must match exactly once.

**> host_text_patch path=`path` find=`find` replace=`replace`**

## apply_patch_blocks
    + `path`
    + `text`
    + `soft`=False

Apply plan-style triple-angle FIND/REPLACE blocks from `text` to `path`.
Also accepts fenced find/replace pairs and minimal Begin Patch hunks.
When `soft=True`, FIND failures return `0` instead of aborting the run.

**> host_apply_patch_blocks path=`path` text=`text` soft=`soft`**
