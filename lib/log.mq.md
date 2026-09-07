---
title: lib/log
description: Level-filtered one-line logs (Mid M6). Default level info.
---

## set_level
    + `level`

level is debug / info / warn / error.

**> host_log_set_level level=`level`**

## debug
    + `text`

*line = > host_log_line level="debug" text=`text`*
1. `line`
  > print text=`line`
2. *
  ****

## info
    + `text`

*line = > host_log_line level="info" text=`text`*
1. `line`
  > print text=`line`
2. *
  ****

## warn
    + `text`

*line = > host_log_line level="warn" text=`text`*
1. `line`
  > print text=`line`
2. *
  ****

## error
    + `text`

*line = > host_log_line level="error" text=`text`*
1. `line`
  > print text=`line`
2. *
  ****
