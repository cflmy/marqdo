---
title: lib/foreign
description: Run bound code fences and inline sources via local interpreters
---

## set_cmd
    + `lang`
    + `cmd`

**> host_foreign_set_cmd lang=`lang` cmd=`cmd`**

## run
    + `code`
    + `stdin`=None

**> host_foreign_run code=`code` stdin=`stdin`**

## run_lang
    + `lang`
    + `source`
    + `stdin`=None

**> host_foreign_run_lang lang=`lang` source=`source` stdin=`stdin`**

## langs

**> host_foreign_langs**
