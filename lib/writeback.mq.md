---
title: lib/writeback
description: Persist run output in .mq.md (Jupyter-style writeback)
---

## record
    + `value`
    + `at_end`=False

Write `value` to `<!-- marqdo-out … -->` in the entry file. Default: insert or replace the block immediately below the call; `at_end=true` appends a single block at EOF.

**> host_writeback_record value=`value` at_end=`at_end`**

## get
    + `at_end`=False

Read persisted output for the current call site (adjacent block), or the EOF block when `at_end=true`.

**> host_writeback_get at_end=`at_end`**

## clear
    + `at_end`=False

Remove the adjacent or EOF output block.

**> host_writeback_clear at_end=`at_end`**

## list

List all output blocks in the entry file.

**> host_writeback_list**
