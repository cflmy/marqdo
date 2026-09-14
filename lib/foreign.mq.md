---
title: lib/foreign
description: Run bound code fences and inline sources via local interpreters
---

Run bound code fences and inline sources via local interpreters.

## set_cmd
    + `lang`
    + `cmd`

Bind interpreter command cmd for language lang.

*> host_foreign_set_cmd lang=`lang` cmd=`cmd`*

## run
    + `code`
    + `stdin`=None

Run a bound code fence value; optional stdin.

Caller: [foreign.run] code=`block`

*> host_foreign_run code=`code` stdin=`stdin`*

## run_lang
    + `lang`
    + `source`
    + `stdin`=None

Run source text with language lang; optional stdin.

*> host_foreign_run_lang lang=`lang` source=`source` stdin=`stdin`*

## langs

List configured foreign languages.

*> host_foreign_langs*
