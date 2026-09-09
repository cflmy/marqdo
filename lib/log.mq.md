---
title: lib/log
description: Level-filtered one-line logs (Mid M6 + Mid2 M10 fields). Default level info.
---

## set_level
    + `level`

level is debug / info / warn / error.

**> host_log_set_level level=`level`**

## debug
    + `text`
    + `fields`=None

*line = > host_log_line level="debug" text=`text` fields=`fields`*
1. `line`
  > print text=`line`
2. *
  ****

## info
    + `text`
    + `fields`=None

*line = > host_log_line level="info" text=`text` fields=`fields`*
1. `line`
  > print text=`line`
2. *
  ****

## warn
    + `text`
    + `fields`=None

*line = > host_log_line level="warn" text=`text` fields=`fields`*
1. `line`
  > print text=`line`
2. *
  ****

## error
    + `text`
    + `fields`=None

*line = > host_log_line level="error" text=`text` fields=`fields`*
1. `line`
  > print text=`line`
2. *
  ****
