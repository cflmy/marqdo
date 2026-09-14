---
title: lib/text
description: Text helpers (Mid M4). Literal replace — use lib/re for regex.
---

Small text helpers. Prefer these over hand-rolled splits; regex belongs in lib/re.

## str_trim
    + `s`

Trim ends of string s.

*> trim `s`*

## str_split
    + `s`
    + `sep`

Split string s on separator sep.

*> split value=`s` sep=`sep`*

## str_join
    + `xs`
    + `sep`

Join list xs with separator sep.

*> join value=`xs` sep=`sep`*

## contains
    + `text`
    + `sub`

Whether text contains substring sub.

Caller: [text.contains] text="hello" sub="ell"

*> host_text_contains text=`text` sub=`sub`*

## starts_with
    + `text`
    + `prefix`

Whether text starts with prefix.

*> host_text_starts_with text=`text` prefix=`prefix`*

## ends_with
    + `text`
    + `suffix`

Whether text ends with suffix.

*> host_text_ends_with text=`text` suffix=`suffix`*

## replace
    + `text`
    + `old`
    + `new`
    + `count`=None

Literal replace in text: replace old with new, optional count.

*> host_text_replace text=`text` old=`old` new=`new` count=`count`*

## to_upper
    + `text`

Uppercase text.

*> host_text_to_upper text=`text`*

## to_lower
    + `text`

Lowercase text.

*> host_text_to_lower text=`text`*

## repeat
    + `text`
    + `n`

Repeat text n times.

*> host_text_repeat text=`text` n=`n`*

## pad
    + `text`
    + `width`
    + `fill`=None
    + `align`=None

Pad text to width with fill and align (left / right / center).

*> host_text_pad text=`text` width=`width` fill=`fill` align=`align`*
