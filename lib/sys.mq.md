---
title: lib/sys
description: English system wrappers (load_dotenv accepts optional path via host alias)
---

## env_get
    + `name`

**> host_env_get name=`name`**

## env_set
    + `name`
    + `value`

**> host_env_set name=`name` value=`value`**

## load_dotenv

Load `.env` (optional named arg `path=`). Existing process env is not overridden.

**> host_dotenv_load**

## args

**> host_args**

## cwd

**> host_cwd**

## exit
    + `code`

**> host_exit code=`code`**

## exec
    + `cmd`

**> host_exec cmd=`cmd`**
