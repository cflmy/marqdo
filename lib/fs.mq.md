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

## make_dirs
    + `path`

Create directories recursively (`mkdir -p`).

**> host_make_dirs path=`path`**

## remove
    + `path`

**> host_remove path=`path`**

## remove_tree
    + `path`

Remove a directory recursively. Errors if `path` is a file.

**> host_remove_tree path=`path`**

## stat
    + `path`

Returns `{size, mtime_unix, is_file, is_dir}`.

**> host_stat path=`path`**

## walk
    + `path`

Depth-first listing of relative paths under `path` (forward slashes).

**> host_walk path=`path`**

## copy_file
    + `src`
    + `dest`

Copy a file (overwrites `dest` if it exists). Paths are sandboxed like other fs ops.

**> host_copy_file src=`src` dest=`dest`**

## move
    + `src`
    + `dest`

Rename / move within the sandbox.

**> host_move src=`src` dest=`dest`**

## make_temp
    + `prefix`=None

Create an empty temp file under the program directory; return its relative path.

**> host_make_temp prefix=`prefix`**

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
When `soft=True`, **only** “FIND not found” returns `0` (run continues). Empty FIND, multi-match, parse errors, and whole-file FIND still abort — whole-file rewrite is forbidden.

**> host_apply_patch_blocks path=`path` text=`text` soft=`soft`**
