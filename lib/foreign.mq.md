---
title: lib/foreign
description: Run bound code fences and inline sources via local interpreters
---

Run bound code fences and inline sources via local interpreters.

## set_cmd
    + `lang`
    + `cmd`
    + `args`=None

Bind interpreter command cmd for language lang; optional extra argv.

*> host_foreign_set_cmd lang=`lang` cmd=`cmd` args=`args`*

## run
    + `code`
    + `stdin`=None
    + `env`=None
    + `cwd`=None
    + `timeout`=None
    + `args`=None
    + `max_output`=None

Run a bound code fence value; optional stdin, env map, cwd, timeout (seconds),
extra argv and output cap. Returns stdout text; non-zero exit is an error.

Caller: [foreign.run] code=`block`

*> host_foreign_run code=`code` stdin=`stdin` env=`env` cwd=`cwd` timeout=`timeout` args=`args` max_output=`max_output`*

## run_lang
    + `lang`
    + `source`
    + `stdin`=None
    + `env`=None
    + `cwd`=None
    + `timeout`=None
    + `args`=None
    + `max_output`=None

Run source text with language lang; same options as run.

*> host_foreign_run_lang lang=`lang` source=`source` stdin=`stdin` env=`env` cwd=`cwd` timeout=`timeout` args=`args` max_output=`max_output`*

## invoke
    + `code`=None
    + `lang`=None
    + `source`=None
    + `stdin`=None
    + `input`=None
    + `env`=None
    + `cwd`=None
    + `timeout`=None
    + `args`=None
    + `output`=auto
    + `max_output`=None
    + `cmd`=None
    + `keep`=False

Run code or lang+source and return a record map instead of raising on process
failure. Fields: ok, code, stdout, stderr, ms, lang, cmd, timed_out, truncated,
result, result_error, error, script, cached.

`input` is encoded as JSON on stdin (mutually exclusive with `stdin`). `output`
is auto | text | json | lines. `cmd` overrides the interpreter for this call.

*> host_foreign_invoke code=`code` lang=`lang` source=`source` stdin=`stdin` input=`input` env=`env` cwd=`cwd` timeout=`timeout` args=`args` output=`output` max_output=`max_output` cmd=`cmd` keep=`keep`*

## probe
    + `lang`=None

Probe one language, or every known language when lang is omitted. Returns
{lang, cmd, path, ok, version, error} (a list when lang is omitted).

*> host_foreign_probe lang=`lang`*

## register
    + `lang`
    + `cmd`=None
    + `args`=None
    + `template`=None
    + `ext`=None
    + `env`=None
    + `version_args`=None

Register a language invocation. Provide cmd (+args) or template (placeholders
{file}, {dir}, {base}, {inline}, {lang}, {cwd}); they are mutually exclusive.

*> host_foreign_register lang=`lang` cmd=`cmd` args=`args` template=`template` ext=`ext` env=`env` version_args=`version_args`*

## unset
    + `lang`

Remove a process-local registration. Presets/config/env still apply.

*> host_foreign_unset lang=`lang`*

## load_config
    + `path`=None

Load .marqdo/foreign.toml (or an explicit path); returns the number of language
tables read.

*> host_foreign_load_config path=`path`*

## langs

List configured foreign languages.

*> host_foreign_langs*
