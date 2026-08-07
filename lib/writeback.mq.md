---
title: lib/writeback
description: Persist run output in .mq.md (Jupyter-style writeback)
---

## record
    + `value`
    + `at_end`=False
    + `key`=None
    + `line`=None

Write `value` to `<!-- marqdo-out … -->` in the entry file. Default: insert or replace the block immediately below the call; `at_end=true` replaces the unkeyed EOF block. With `key=` (e.g. `ok` / `error`), replace or create that **named** slot only — slots do not overwrite each other. Optional `line=` anchors the block under that source line (needed when writing from a subtask).

**> host_writeback_record value=`value` at_end=`at_end` key=`key` line=`line`**

## get
    + `at_end`=False
    + `key`=None
    + `line`=None

Read persisted output for the current call site (adjacent unkeyed block), the unkeyed EOF block when `at_end=true`, or a named slot when `key=` is set.

**> host_writeback_get at_end=`at_end` key=`key` line=`line`**

## clear
    + `at_end`=False
    + `key`=None
    + `line`=None

Remove the adjacent, EOF, or named output block.

**> host_writeback_clear at_end=`at_end` key=`key` line=`line`**

## ensure
    + `key`
    + `placeholder`=pending
    + `line`=None

Create a named slot with `placeholder` if missing. Never overwrites an existing slot body.

**> host_writeback_ensure key=`key` placeholder=`placeholder` line=`line`**

## list

List all output blocks in the entry file (includes `key` when named).

**> host_writeback_list**
