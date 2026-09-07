---
title: lib/re
description: Regular expressions (Mid M1). Unicode; no backreferences.
---

## is_match
    + `text`
    + `pattern`

**> host_re_is_match text=`text` pattern=`pattern`**

## find
    + `text`
    + `pattern`

**> host_re_find text=`text` pattern=`pattern`**

## find_all
    + `text`
    + `pattern`

**> host_re_find_all text=`text` pattern=`pattern`**

## replace
    + `text`
    + `pattern`
    + `with`
    + `count`=None

Optional `count=` limits replacements (`None` = all).

**> host_re_replace text=`text` pattern=`pattern` with=`with` count=`count`**

## split
    + `text`
    + `pattern`

**> host_re_split text=`text` pattern=`pattern`**
