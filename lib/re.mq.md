---
title: lib/re
description: Regular expressions (Mid M1). Unicode; no backreferences.
---

Regular expressions. Unicode; no backreferences. Prefer lib/text for literal replace.

## is_match
    + `text`
    + `pattern`

Whether text matches pattern.

Caller: [re.is_match] text=`s` pattern="a+"

*> host_re_is_match text=`text` pattern=`pattern`*

## find
    + `text`
    + `pattern`

First match of pattern in text, or None.

*> host_re_find text=`text` pattern=`pattern`*

## find_all
    + `text`
    + `pattern`

All matches of pattern in text.

*> host_re_find_all text=`text` pattern=`pattern`*

## replace
    + `text`
    + `pattern`
    + `with`
    + `count`=None

Replace matches of pattern in text with with; optional count limits replacements (None = all).

*> host_re_replace text=`text` pattern=`pattern` with=`with` count=`count`*

## split
    + `text`
    + `pattern`

Split text on pattern.

*> host_re_split text=`text` pattern=`pattern`*
