---
title: lib/sys
description: English system wrappers (load_dotenv accepts optional path via host alias)
---

English system wrappers (load_dotenv accepts optional path via host alias)

## env_get
    + `name`

Get environment variable name.

*> host_env_get name=`name`*

## env_set
    + `name`
    + `value`

Set environment variable name to value.

*> host_env_set name=`name` value=`value`*

## load_dotenv
    + `path`=None

Load `.env` (optional named arg `path=`). Existing process env is not overridden.

*> host_dotenv_load path=`path`*

## args

Process argv list.

*> host_args*

## cwd

Current working directory.

*> host_cwd*

## exit
    + `code`

Exit process with code.

*> host_exit code=`code`*

## exec
    + `cmd`
    + `args`=None
    + `capture`=False

Run a process. Default return is the exit code (int).

With `capture=True`, return a map `{code, stdout, stderr}` (stdout/stderr may be truncated for huge output).

Inside `**…**` prefer bare `args=args` (variable); `` args=`args` `` is also accepted.

*> host_exec cmd=`cmd` args=`args` capture=`capture`*

## stream_publish
    + `event`

Publish a stream event map onto the process EventBus (`marqdo view` SSE). No-op when nothing is subscribed.

*> host_stream_publish event=`event`*
