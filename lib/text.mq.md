---
title: lib/text
description: Text helpers (Mid M4 thicken). Literal replace — use lib/re for regex.
---

## str_trim
    + `s`

**> trim `s`**

## str_split
    + `s`
    + `sep`

**> split value=`s` sep=`sep`**

## str_join
    + `xs`
    + `sep`

**> join value=`xs` sep=`sep`**

## contains
    + `text`
    + `sub`

**> host_text_contains text=`text` sub=`sub`**

## starts_with
    + `text`
    + `prefix`

**> host_text_starts_with text=`text` prefix=`prefix`**

## ends_with
    + `text`
    + `suffix`

**> host_text_ends_with text=`text` suffix=`suffix`**

## replace
    + `text`
    + `old`
    + `new`
    + `count`=None

Literal substring replace (`count` optional).

**> host_text_replace text=`text` old=`old` new=`new` count=`count`**

## to_upper
    + `text`

**> host_text_to_upper text=`text`**

## to_lower
    + `text`

**> host_text_to_lower text=`text`**

## repeat
    + `text`
    + `n`

**> host_text_repeat text=`text` n=`n`**

## pad
    + `text`
    + `width`
    + `fill`=None
    + `align`=None

align: left (default), right, or center.

**> host_text_pad text=`text` width=`width` fill=`fill` align=`align`**
