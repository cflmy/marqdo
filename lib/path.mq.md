---
title: lib/path
description: Path algebra (Mid M2). No I/O; OS separators.
---

Path algebra only. No I/O; separators follow the host OS.

## join
    + `a`
    + `b`

Join path segments a and b.

Caller: [path.join] a="a" b="b"

*> host_path_join a=`a` b=`b`*

## split
    + `path`

Split path into parts.

*> host_path_split path=`path`*

## file_name
    + `path`

Final file name of path.

*> host_path_file_name path=`path`*

## parent
    + `path`

Parent directory of path.

*> host_path_parent path=`path`*

## extension
    + `path`

File extension of path.

*> host_path_extension path=`path`*

## normalize
    + `path`

Normalize path (dot segments).

*> host_path_normalize path=`path`*

## is_absolute
    + `path`

Whether path is absolute.

*> host_path_is_absolute path=`path`*
