---
title: ext/ai/agent
description: Document-driven agent — step / plan; tools via lib/subtask; runtime via ABI v2 agent plugin.
import llm:ext/ai/llm.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
import fs:lib/fs.mq.md
import time:lib/time.mq.md
import writeback:lib/writeback.mq.md
import subtask:lib/subtask.mq.md
import plugin:lib/plugin.mq.md
---

## build_step_context
    + `agent`
    + `task`

Assemble a readable prompt: standing, task, tools, call site, source, skill, and the act protocol (human-visible).

*standing = > json.get value=`agent` key="standing"*
1. `standing`
  *up = standing*
2. *
  *up = None*

*tools = > json.get value=`agent` key="tools"*
*tools_s = > agent_format_tools tools=`tools`*
*src = > agent_module_source*
*site = > agent_call_site*
*site_s = > json.stringify value=`site`*
*skill = > agent_marqdo_skill*
*task_s = > json.stringify value=`task`*

*esc = > json.parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- task ---\n","c":"\n\n--- tools ---\n","d":"\n\n--- call site ---\n","e":"\n\n--- source (.mq.md) ---\n","f":"\n\n--- marqdo skill ---\n","g":"\n\n--- how to act ---\nCall ONLY names listed under --- tools --- (whitelist). If that section is empty or says none, do NOT emit CALL / 调用 — reply with the final answer only.\nNever CALL internal framework helpers (build_step_context, plan, inspect_workbook, etc.).\nWhen calling an allowed tool, reply with exactly one line: CALL:<name>\n(Chinese ok: 调用：<name>)\nOtherwise reply with the final answer only.\nTools run via Marqdo subtask (spawn fn / wait).\n"}*
*a = > json.get value=`esc` key="a"*
*b = > json.get value=`esc` key="b"*
*c = > json.get value=`esc` key="c"*
*d = > json.get value=`esc` key="d"*
*e = > json.get value=`esc` key="e"*
*f = > json.get value=`esc` key="f"*
*g = > json.get value=`esc` key="g"*

*p = "You are a Marqdo agent. The runbook source is your ground truth — code is documentation."*
*p = p + a + up + b + task_s + c + tools_s + g + d + site_s + e + src + f + skill*
**p**

---

## extract_tool_name
    + `reply`

Find a human-readable call line: `CALL:<name>` or `调用：<name>` / `调用:<name>`.

*esc = > json.parse text={"nl":"\n"}*
*nl = > json.get value=`esc` key="nl"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*found = None*

- [`line`](`lines`)
  *t = > trim value=`line`*
  *parts = > split value=`t` sep=":"*
  *n = > len value=`parts`*
  1. `n` > 1
    *head = > at value=`parts` index=0*
    *head = > trim value=`head`*
    *rest = > at value=`parts` index=1*
    *rest = > trim value=`rest`*
    1. `head` == CALL
      *found = rest*
    2. `head` == 调用
      *found = rest*
    3. *
      *_ = 1*
  2. *
    *_ = 1*

**found**

---

## run_tool
    + `tools`
    + `name`

Allowlist check, then invoke the runbook `##` via `lib/subtask` (`spawn fn=` → `wait`).

*allowed = > agent_tool_allowed tools=`tools` name=`name`*
1. `allowed`
  *id = > subtask.spawn fn=`name`*
  **> subtask.wait id=`id`**
2. *
  *msg = "Tool not allowed:"*
  **msg + name**

## render_workbook_skeleton
    + `goal`
    + `skeleton`=single

Build a runnable workbook. `skeleton=single` (default): one worker `step`. `skeleton=dual`: research then write agents. On success the parent chooses DONE; runtime `agent_workbook_solidify` freezes the answer.

*q = > json.quote text=`goal`*
*esc = > json.parse text={"h":"---\ntitle: agent workbook\nimport llm:ext/ai/llm.mq.md\nimport agent:ext/ai/agent.mq.md\nimport json:lib/json.mq.md\nimport sys:lib/sys.mq.md\nimport writeback:lib/writeback.mq.md\nimport subtask:lib/subtask.mq.md\n---\n\n# Goal\n\nParent goal is embedded as JSON in # main.\n\n## Solidify\n\nWhen the child returns a good value, parent DECISION: DONE — runtime agent_workbook_solidify freezes the answer as **return** (do not invent long FIND/REPLACE).\n\n# main\n\n> llm.load_env\n\n","s1":"\u002a\u0060model\u0060 = > llm.llm \u002a\n","s2":"\u002a\u0060tools\u0060 = > json.parse text=[] \u002a\n","s3":"\u002a\u0060worker\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You are a workbook worker. tools=[] means never CALL. Finish the task with a direct final answer; do not invent tools. \u002a\n","s4":"\u002a\u0060wrap\u0060 = > json.parse text={\"task\":","s5":"} \u002a\n","s6":"\u002a\u0060task\u0060 = > json.get value=\u0060wrap\u0060 key=task \u002a\n","s7":"\u002a\u0060out\u0060 = > \u0060worker\u0060.step task=\u0060task\u0060 stream=True \u002a\n","s7b":"\u002a\u0060text\u0060 = > json.get value=\u0060out\u0060 key=result \u002a\n","s8":"\u002a\u002a\u0060text\u0060\u002a\u002a\n","d3":"\u002a\u0060research\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You gather facts for the goal. Be concise. Do not invent tools. \u002a\n","d3b":"\u002a\u0060writer\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You write the final answer from research notes. Do not invent tools. \u002a\n","d4":"\u002a\u0060wrap\u0060 = > json.parse text={\"task\":","d5":"} \u002a\n","d6":"\u002a\u0060task\u0060 = > json.get value=\u0060wrap\u0060 key=task \u002a\n","d7":"\u002a\u0060notes\u0060 = > \u0060research\u0060.step task=\u0060task\u0060 stream=True \u002a\n","d8":"\u002a\u0060ns\u0060 = > json.stringify value=\u0060notes\u0060 \u002a\n","d9":"\u002a\u0060wt\u0060 = Write the final answer. Research notes: \u002a\n","d10":"\u002a\u0060wt\u0060 = \u0060wt\u0060 + \u0060ns\u0060 \u002a\n","d11":"\u002a\u0060out\u0060 = > \u0060writer\u0060.step task=\u0060wt\u0060 stream=True \u002a\n","d11b":"\u002a\u0060text\u0060 = > json.get value=\u0060out\u0060 key=result \u002a\n","d12":"\u002a\u002a\u0060text\u0060\u002a\u002a\n"}*
*h = > json.get value=`esc` key="h"*
*s1 = > json.get value=`esc` key="s1"*
*s2 = > json.get value=`esc` key="s2"*

1. `skeleton` == dual
  *s3 = > json.get value=`esc` key="d3"*
  *s3b = > json.get value=`esc` key="d3b"*
  *s4 = > json.get value=`esc` key="d4"*
  *s5 = > json.get value=`esc` key="d5"*
  *s6 = > json.get value=`esc` key="d6"*
  *s7 = > json.get value=`esc` key="d7"*
  *s8 = > json.get value=`esc` key="d8"*
  *s9 = > json.get value=`esc` key="d9"*
  *s10 = > json.get value=`esc` key="d10"*
  *s11 = > json.get value=`esc` key="d11"*
  *s11b = > json.get value=`esc` key="d11b"*
  *s12 = > json.get value=`esc` key="d12"*
  **h + s1 + s2 + s3 + s3b + s4 + q + s5 + s6 + s7 + s8 + s9 + s10 + s11 + s11b + s12**
2. *
  *s3 = > json.get value=`esc` key="s3"*
  *s4 = > json.get value=`esc` key="s4"*
  *s5 = > json.get value=`esc` key="s5"*
  *s6 = > json.get value=`esc` key="s6"*
  *s7 = > json.get value=`esc` key="s7"*
  *s7b = > json.get value=`esc` key="s7b"*
  *s8 = > json.get value=`esc` key="s8"*
  **h + s1 + s2 + s3 + s4 + q + s5 + s6 + s7 + s7b + s8**

---

## inspect_workbook
    + `path`
    + `exit_code`=None
    + `value`=None
    + `stdout`=None
    + `stderr`=None

Structured observation for the parent developer-agent: source, named writeback slots, exit code, optional child return value, and quiet-captured I/O.

*source = > fs.read_text path=`path`*
*slots = > writeback.scan_path path=`path`*
*obs = > json.parse text={"path":""}*
*obs = > json.set map=`obs` key="path" value=`path`*
*obs = > json.set map=`obs` key="source" value=`source`*
*obs = > json.set map=`obs` key="slots" value=`slots`*
*obs = > json.set map=`obs` key="exit_code" value=`exit_code`*
1. `value`
  *obs = > json.set map=`obs` key="value" value=`value`*
2. *
  *_ = 1*
1. `stdout`
  *obs = > json.set map=`obs` key="stdout" value=`stdout`*
2. *
  *_ = 1*
1. `stderr`
  *obs = > json.set map=`obs` key="stderr" value=`stderr`*
2. *
  *_ = 1*

*last_ok = None*
*last_error = None*
- [`slot`](`slots`)
  *k = > json.get value=`slot` key="key"*
  *body = > json.get value=`slot` key="body"*
  1. `k` == ok
    *last_ok = body*
  2. `k` == error
    *last_error = body*
  3. *
    *_ = 1*

1. `last_ok`
  *obs = > json.set map=`obs` key="last_ok" value=`last_ok`*
2. *
  *_ = 1*

1. `last_error`
  *obs = > json.set map=`obs` key="last_error" value=`last_error`*
2. *
  *_ = 1*

**obs**

---

## await_workbook
    + `path`

Spawn a workbook file (quiet by default), wait for `{code,value,stdout?,stderr?}`, and inspect.

*id = > subtask.spawn path=`path`*
*waited = > subtask.wait id=`id`*
*code = > json.get value=`waited` key="code"*
*value = > json.get value=`waited` key="value"*
*stdout = > json.get value=`waited` key="stdout"*
*stderr = > json.get value=`waited` key="stderr"*
*obs = > inspect_workbook path=`path` exit_code=`code` value=`value` stdout=`stdout` stderr=`stderr`*
*out = > json.parse text={"code":0}*
*out = > json.set map=`out` key="code" value=`code`*
*out = > json.set map=`out` key="value" value=`value`*
*out = > json.set map=`out` key="observation" value=`obs`*
**out**

---

## text_head_lines
    + `text`
    + `max_lines`=80
    + `max_chars`=4000

*esc = > json.parse text={"nl":"\n"}*
*nl = > json.get value=`esc` key="nl"*
*raw = > str value=`text`*
*lines = > split value=`raw` sep=`nl`*
*acc = > json.parse text=[]*
*chars = 0*
*count = 0*
- [`line`](`lines`)
  1. `count` < `max_lines`
    *ln = > len value=`line`*
    *next = chars + ln*
    1. `next` > `max_chars`
      *count = max_lines*
    2. *
      *acc = > json.append list=`acc` item=`line`*
      *chars = next*
      *count = count + 1*
  2. *
    *_ = 1*
**> join value=`acc` sep=`nl`**

---

## text_tail_lines
    + `text`
    + `max_lines`=40
    + `max_chars`=2000

*esc = > json.parse text={"nl":"\n"}*
*nl = > json.get value=`esc` key="nl"*
*raw = > str value=`text`*
*lines = > split value=`raw` sep=`nl`*
*n = > len value=`lines`*
*start = n - max_lines*
1. `start` < 0
  *start = 0*
2. *
  *_ = 1*
*acc = > json.parse text=[]*
*i = 0*
*chars = 0*
- [`line`](`lines`)
  1. `i` >= `start`
    *ln = > len value=`line`*
    *next = chars + ln*
    1. `next` > `max_chars`
      *i = n*
    2. *
      *acc = > json.append list=`acc` item=`line`*
      *chars = next*
  2. *
    *_ = 1*
  *i = i + 1*
**> join value=`acc` sep=`nl`**

---

## strip_html_comments
    + `text`

Drop `<!-- … -->` blocks (including marqdo-out bodies) so parent excerpts stay structural.

*esc = > json.parse text={"nl":"\n"}*
*nl = > json.get value=`esc` key="nl"*
*raw = > str value=`text`*
*lines = > split value=`raw` sep=`nl`*
*acc = > json.parse text=[]*
*skip = None*
- [`line`](`lines`)
  1. `skip`
    *end_parts = > split value=`line` sep="-->"*
    *en = > len value=`end_parts`*
    1. `en` > 1
      *skip = None*
    2. *
      *_ = 1*
  2. *
    *start_parts = > split value=`line` sep="<!--"*
    *sn = > len value=`start_parts`*
    1. `sn` > 1
      *before = > at value=`start_parts` index=0*
      *before = > trim value=`before`*
      1. `before`
        *acc = > json.append list=`acc` item=`before`*
      2. *
        *_ = 1*
      *tail = > at value=`start_parts` index=1*
      *end_parts = > split value=`tail` sep="-->"*
      *en = > len value=`end_parts`*
      1. `en` > 1
        *skip = None*
      2. *
        *skip = 1*
    2. *
      *acc = > json.append list=`acc` item=`line`*
**> join value=`acc` sep=`nl`**

---

## workbook_source_excerpt
    + `source`
    + `max_lines`=80
    + `max_chars`=4000

*clean = > strip_html_comments text=`source`*
**> text_head_lines text=`clean` max_lines=`max_lines` max_chars=`max_chars`**

---

## workbook_excerpt
    + `path`
    + `max_lines`=80
    + `max_chars`=4000

*src = > fs.read_text path=`path`*
**> workbook_source_excerpt source=`src` max_lines=`max_lines` max_chars=`max_chars`**

---

## workbook_read
    + `path`
    + `depth`=default

*src = > fs.read_text path=`path`*
1. `depth` == deep
  **> workbook_source_excerpt source=`src` max_lines=200 max_chars=12000**
2. *
  **> workbook_source_excerpt source=`src` max_lines=80 max_chars=4000**

---

## lib_catalog

Callable dotted names (whitelist for `CALL:lib…`) plus static module file list for workbook imports.

*out = > json.parse text={"note":"Direct CALL:lib.<mod>.<fn> + optional ARGS:{json} for whitelist only. Or import via frontmatter > lib/<file>.mq.md then PATCH / scratch_tool_write.","files":["fs.mq.md","json.mq.md","subtask.mq.md","sys.mq.md","writeback.mq.md","text.mq.md","time.mq.md","math.mq.md","net.mq.md","table.mq.md","plugin.mq.md","foreign.mq.md"],"callable":[{"name":"lib.fs.read_text","desc":"Read UTF-8 text file"},{"name":"lib.fs.exists","desc":"Path exists?"},{"name":"lib.fs.list_dir","desc":"List directory entries"},{"name":"lib.json.parse","desc":"Parse JSON text"},{"name":"lib.json.stringify","desc":"Serialize JSON"},{"name":"lib.sys.cwd","desc":"Process working directory"}]}*
**out**

---

## extract_call_args
    + `reply`=None

Parse `ARGS:{…}` JSON line, or a bare `{…}` line after CALL.

*esc = > json.parse text={"nl":"\n","mark":"ARGS:","brace":"{"}*
*nl = > json.get value=`esc` key="nl"*
*mark = > json.get value=`esc` key="mark"*
*brace = > json.get value=`esc` key="brace"*
*empty = > json.parse text={}*
1. `reply`
  *text = > trim value=`reply`*
  *lines = > split value=`text` sep=`nl`*
  *found = None*
  - [`line`](`lines`)
    1. `found`
      *_ = 1*
    2. *
      *t = > trim value=`line`*
      *chunks = > split value=`t` sep=`mark`*
      *cn = > len value=`chunks`*
      1. `cn` > 1
        *rest = > at value=`chunks` index=1*
        *found = > trim value=`rest`*
      2. *
        *chunks = > split value=`t` sep=`brace`*
        *cn = > len value=`chunks`*
        *prefix = > at value=`chunks` index=0*
        1. `cn` > 1
          1. `prefix`
            *_ = 1*
          2. *
            *found = t*
        2. *
          *_ = 1*
  1. `found`
    **> json.parse text=`found`**
  2. *
    **empty**
2. *
  **empty**

---

## tool_result_brief
    + `value`=None
    + `max_chars`=1024

Truncate a tool result for `tool_end` events (~1KiB).

1. `value`
  *s = > json.stringify value=`value`*
  **> text_head_lines text=`s` max_lines=40 max_chars=`max_chars`**
2. *
  ****

---

## plan_append_tool
    + `events`
    + `type`
    + `name`
    + `kind`
    + `result`=None
    + `stream`=False

Always append `tool_start` / `tool_end` to `events` for audit / view process cards. SSE publish only when `stream=True`.

*ev = > json.parse text={}*
*ev = > json.set map=`ev` key="type" value=`type`*
*ev = > json.set map=`ev` key="name" value=`name`*
*ev = > json.set map=`ev` key="kind" value=`kind`*
1. `result`
  *ev = > json.set map=`ev` key="result" value=`result`*
2. *
  *_ = 1*
*events = > json.append list=`events` item=`ev`*
1. `stream`
  > sys.stream_publish event=`ev`
2. *
  *_ = 1*

**events**

---

## run_lib_call
    + `name`
    + `reply`=None

Whitelist `CALL:lib.<mod>.<fn>` (+ ARGS). Rejects net/exec/write/delete.

*args = > extract_call_args reply=`reply`*
*errs = > json.parse text={"deny":"Lib CALL not allowed: "}*

1. `name` == lib.fs.exists
  *path = > json.get value=`args` key="path"*
  **> fs.exists path=`path`**
2. `name` == lib.fs.read_text
  *path = > json.get value=`args` key="path"*
  **> fs.read_text path=`path`**
3. `name` == lib.fs.list_dir
  *path = > json.get value=`args` key="path"*
  **> fs.list_dir path=`path`**
4. `name` == lib.json.parse
  *text = > json.get value=`args` key="text"*
  **> json.parse text=`text`**
5. `name` == lib.json.stringify
  *value = > json.get value=`args` key="value"*
  **> json.stringify value=`value`**
6. `name` == lib.sys.cwd
  **> sys.cwd**
7. *
  *deny = > json.get value=`errs` key="deny"*
  **deny + name**

---

## scratch_tool_write
    + `name`
    + `text`

Write a scratch tool workbook under `.marqdo/agent-runs/tools/<name>.mq.md`.

*esc = > json.parse text={"a":".marqdo/agent-runs/tools/","b":".mq.md"}*
*a = > json.get value=`esc` key="a"*
*b = > json.get value=`esc` key="b"*
*dir = ".marqdo/agent-runs/tools"*
> fs.make_dir path=`dir`
*path = a + name + b*
> fs.write_text path=`path` text=`text`
*out = > json.parse text={"ok":True}*
*out = > json.set map=`out` key="path" value=`path`*
**out**

---

## skill_brief
    + `max_chars`=1200

*skill = > agent_marqdo_skill*
**> text_head_lines text=`skill` max_lines=40 max_chars=`max_chars`**

---

## extract_plan_read
    + `reply`

Parse READ:source|stderr|stdout|slots (Chinese: 读取：…).

*esc = > json.parse text={"nl":"\n","fw":"："}*
*nl = > json.get value=`esc` key="nl"*
*fw = > json.get value=`esc` key="fw"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*found = None*
- [`line`](`lines`)
  *t = > trim value=`line`*
  *parts = > split value=`t` sep=":"*
  *n = > len value=`parts`*
  1. `n` > 1
    *head = > at value=`parts` index=0*
    *head = > trim value=`head`*
    *rest = > at value=`parts` index=1*
    *rest = > trim value=`rest`*
    1. `head` == READ
      *found = rest*
    2. `head` == 读取
      *found = rest*
    3. *
      *_ = 1*
  2. *
    *parts = > split value=`t` sep=`fw`*
    *n = > len value=`parts`*
    1. `n` > 1
      *head = > at value=`parts` index=0*
      *head = > trim value=`head`*
      *rest = > at value=`parts` index=1*
      *rest = > trim value=`rest`*
      1. `head` == 读取
        *found = rest*
      2. `head` == READ
        *found = rest*
      3. *
        *_ = 1*
    2. *
      *_ = 1*
**found**

---

## extract_plan_act
    + `reply`

Priority: CALL → READ → DECISION. Returns `{kind,name}`.

*tool = > extract_tool_name reply=`reply`*
1. `tool`
  *out = > json.parse text={"kind":"call"}*
  *out = > json.set map=`out` key="name" value=`tool`*
  **out**
2. *
  *_ = 1*
*rk = > extract_plan_read reply=`reply`*
1. `rk`
  *out = > json.parse text={"kind":"read"}*
  *out = > json.set map=`out` key="name" value=`rk`*
  **out**
2. *
  *_ = 1*
*dec = > extract_plan_decision reply=`reply`*
1. `dec`
  *out = > json.parse text={"kind":"decision"}*
  *out = > json.set map=`out` key="name" value=`dec`*
  **out**
2. *
  *out = > json.parse text={"kind":"unknown"}*
  **out**

---

## extract_scratch_tool
    + `reply`

Parse NAME: id plus a triple-angle body after CALL:scratch_tool_write.

*esc = > json.parse text={"nl":"\n","open":"<<<","close":">>>"}*
*nl = > json.get value=`esc` key="nl"*
*open = > json.get value=`esc` key="open"*
*close = > json.get value=`esc` key="close"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*name = None*
*body_lines = > json.parse text=[]*
*in_body = None*
- [`line`](`lines`)
  1. `in_body`
    *t = > trim value=`line`*
    1. `t` == `close`
      *in_body = None*
    2. *
      *body_lines = > json.append list=`body_lines` item=`line`*
  2. *
    *t = > trim value=`line`*
    *parts = > split value=`t` sep=":"*
    *n = > len value=`parts`*
    1. `n` > 1
      *head = > at value=`parts` index=0*
      *head = > trim value=`head`*
      *rest = > at value=`parts` index=1*
      *rest = > trim value=`rest`*
      1. `head` == NAME
        *name = rest*
      2. `head` == 名称
        *name = rest*
      3. *
        *_ = 1*
    2. *
      *_ = 1*
    1. `t` == `open`
      *in_body = 1*
    2. *
      *_ = 1*
*body = > join value=`body_lines` sep=`nl`*
*out = > json.parse text={}*
*out = > json.set map=`out` key="name" value=`name`*
*out = > json.set map=`out` key="text" value=`body`*
**out**

---

## run_parent_tool
    + `name`
    + `path`
    + `observation`=None
    + `reply`=None

Parent Plan-and-Move tools (helpers + whitelist `CALL:lib…`).

1. `name` == workbook_read
  **> workbook_read path=`path` depth="deep"**
2. `name` == workbook_excerpt
  **> workbook_excerpt path=`path`**
3. `name` == lib_catalog
  **> lib_catalog**
4. `name` == scratch_tool_write
  *parsed = > extract_scratch_tool reply=`reply`*
  *tn = > json.get value=`parsed` key="name"*
  *tt = > json.get value=`parsed` key="text"*
  *errs = > json.parse text={"a":"scratch_tool_write needs NAME line and fenced body","b":"scratch_tool_write needs NAME id"}*
  1. `tn`
    1. `tt`
      **> scratch_tool_write name=`tn` text=`tt`**
    2. *
      **> json.get value=`errs` key="a"**
  2. *
    **> json.get value=`errs` key="b"**
5. *
  *dots = > split value=`name` sep=".*"*
  *head = > at value=`dots` index=0*
  1. `head` == lib
    **> run_lib_call name=`name` reply=`reply`**
  2. *
    *msg = "Parent tool not allowed:"*
    **msg + name**

---

## plan_read_deepen
    + `observation`
    + `kind`
    + `path`

*obs = observation*
1. `kind` == source
  *ex = > workbook_read path=`path` depth="deep"*
  *obs = > json.set map=`obs` key="source_excerpt" value=`ex`*
  *obs = > json.set map=`obs` key="read_source" value=True*
2. `kind` == stderr
  *err = > json.get value=`obs` key="stderr"*
  *tail = > text_tail_lines text=`err` max_lines=80 max_chars=6000*
  *obs = > json.set map=`obs` key="stderr_tail" value=`tail`*
  *obs = > json.set map=`obs` key="read_stderr" value=True*
3. `kind` == stdout
  *out = > json.get value=`obs` key="stdout"*
  *tail = > text_tail_lines text=`out` max_lines=80 max_chars=6000*
  *obs = > json.set map=`obs` key="stdout_tail" value=`tail`*
  *obs = > json.set map=`obs` key="read_stdout" value=True*
4. `kind` == slots
  *slots = > json.get value=`obs` key="slots"*
  *obs = > json.set map=`obs` key="slots_detail" value=`slots`*
  *obs = > json.set map=`obs` key="read_slots" value=True*
5. *
  *_ = 1*
**obs**

---

## compact_plan_observation
    + `observation`

Bounded observation for the parent prompt (actionable, short).

*obs_path = > json.get value=`observation` key="path"*
*obs_code = > json.get value=`observation` key="exit_code"*
*obs_val = > json.get value=`observation` key="value"*
*obs_slots = > json.get value=`observation` key="slots"*
*obs_src = > json.get value=`observation` key="source"*
*obs_stdout = > json.get value=`observation` key="stdout"*
*obs_stderr = > json.get value=`observation` key="stderr"*
*src_len = 0*
*has_step = False*
1. `obs_src`
  *src_len = > len value=`obs_src`*
  *step_parts = > split value=`obs_src` sep="worker.step"*
  *n_step = > len value=`step_parts`*
  1. `n_step` > 1
    *has_step = True*
  2. *
    *_ = 1*
2. *
  *_ = 1*

*compact = > json.parse text={"note":"Plan-and-Move: CALL/READ then DECISION. Success (exit 0 + has_value) → DONE. solidify_on_done. No long prose."}*
*compact = > json.set map=`compact` key="path" value=`obs_path`*
*compact = > json.set map=`compact` key="exit_code" value=`obs_code`*
*compact = > json.set map=`compact` key="source_len" value=`src_len`*
*compact = > json.set map=`compact` key="has_worker_step" value=`has_step`*
*compact = > json.set map=`compact` key="solidify_on_done" value=True*

1. `obs_src`
  *ex = > json.get value=`observation` key="source_excerpt"*
  1. `ex`
    *compact = > json.set map=`compact` key="source_excerpt" value=`ex`*
  2. *
    *ex = > workbook_source_excerpt source=`obs_src`*
    *compact = > json.set map=`compact` key="source_excerpt" value=`ex`*
2. *
  *_ = 1*

1. `obs_stderr`
  *tail = > json.get value=`observation` key="stderr_tail"*
  1. `tail`
    *compact = > json.set map=`compact` key="stderr_tail" value=`tail`*
  2. *
    *tail = > text_tail_lines text=`obs_stderr`*
    *compact = > json.set map=`compact` key="stderr_tail" value=`tail`*
2. *
  *_ = 1*
1. `obs_stdout`
  *tail = > json.get value=`observation` key="stdout_tail"*
  1. `tail`
    *compact = > json.set map=`compact` key="stdout_tail" value=`tail`*
  2. *
    *tail = > text_tail_lines text=`obs_stdout`*
    *compact = > json.set map=`compact` key="stdout_tail" value=`tail`*
2. *
  *_ = 1*

*val_txt = > str value=`obs_val`*
*val_len = > len value=`val_txt`*
1. `obs_val`
  *compact = > json.set map=`compact` key="has_value" value=True*
  *compact = > json.set map=`compact` key="value_len" value=`val_len`*
  1. `val_len` > 200
    *preview = > text_head_lines text=`val_txt` max_lines=4 max_chars=200*
    *compact = > json.set map=`compact` key="value_preview" value=`preview`*
  2. *
    *compact = > json.set map=`compact` key="value" value=`obs_val`*
2. *
  *compact = > json.set map=`compact` key="has_value" value=False*

*slots_brief = > json.parse text=[]*
1. `obs_slots`
  - [`slot`](`obs_slots`)
    *sk = > json.get value=`slot` key="key"*
    *sl = > json.get value=`slot` key="line"*
    *sb = > json.get value=`slot` key="body"*
    *sb_txt = > str value=`sb`*
    *sb_len = > len value=`sb_txt`*
    *brief = > json.parse text={"key":""}*
    *brief = > json.set map=`brief` key="key" value=`sk`*
    *brief = > json.set map=`brief` key="line" value=`sl`*
    *brief = > json.set map=`brief` key="body_len" value=`sb_len`*
    1. `sk` == error
      *prev = > text_head_lines text=`sb_txt` max_lines=8 max_chars=400*
      *brief = > json.set map=`brief` key="body_preview" value=`prev`*
    2. *
      *_ = 1*
    *slots_brief = > json.append list=`slots_brief` item=`brief`*
2. *
  *_ = 1*
*compact = > json.set map=`compact` key="slots" value=`slots_brief`*
*detail = > json.get value=`observation` key="slots_detail"*
1. `detail`
  *compact = > json.set map=`compact` key="slots_detail" value=`detail`*
2. *
  *_ = 1*

*tool_result = > json.get value=`observation` key="tool_result"*
1. `tool_result`
  *tr = > str value=`tool_result`*
  *tr = > text_head_lines text=`tr` max_lines=40 max_chars=3000*
  *compact = > json.set map=`compact` key="tool_result" value=`tr`*
2. *
  *_ = 1*

*last_error = > json.get value=`observation` key="last_error"*
1. `last_error`
  *err_txt = > str value=`last_error`*
  *err_len = > len value=`err_txt`*
  1. `err_len` > 200
    *prev = > text_head_lines text=`err_txt` max_lines=8 max_chars=400*
    *compact = > json.set map=`compact` key="last_error_preview" value=`prev`*
    *compact = > json.set map=`compact` key="last_error_len" value=`err_len`*
  2. *
    *compact = > json.set map=`compact` key="last_error" value=`last_error`*
2. *
  *_ = 1*

**compact**

---

## extract_plan_decision
    + `reply`

Parse `DECISION: DONE` / `CONTINUE` / `RUN` (Chinese: `决定：完成` / `继续` / `运行`).
**First** matching DECISION / 决定 line wins (ignore later conflicting decisions in the same reply).

*esc = > json.parse text={"nl":"\n","fw":"："}*
*nl = > json.get value=`esc` key="nl"*
*fw = > json.get value=`esc` key="fw"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*found = None*

- [`line`](`lines`)
  1. `found`
    *_ = 1*
  2. *
    *t = > trim value=`line`*
    *parts = > split value=`t` sep=":"*
    *n = > len value=`parts`*
    1. `n` > 1
      *head = > at value=`parts` index=0*
      *head = > trim value=`head`*
      *rest = > at value=`parts` index=1*
      *rest = > trim value=`rest`*
      1. `head` == DECISION
        *found = rest*
      2. `head` == 决定
        *found = rest*
      3. *
        *_ = 1*
    2. *
      *parts = > split value=`t` sep=`fw`*
      *n = > len value=`parts`*
      1. `n` > 1
        *head = > at value=`parts` index=0*
        *head = > trim value=`head`*
        *rest = > at value=`parts` index=1*
        *rest = > trim value=`rest`*
        1. `head` == 决定
          *found = rest*
        2. `head` == DECISION
          *found = rest*
        3. *
          *_ = 1*
      2. *
        *_ = 1*

1. `found` == DONE
  **"DONE"**
2. `found` == CONTINUE
  **"CONTINUE"**
3. `found` == RUN
  **"RUN"**
4. `found` == REUSE
  **"REUSE"**
5. `found` == NEW
  **"NEW"**
6. `found` == 完成
  **"DONE"**
7. `found` == 继续
  **"CONTINUE"**
8. `found` == 运行
  **"RUN"**
9. `found` == 复用
  **"REUSE"**
10. `found` == 新建
  **"NEW"**
11. *
  **found**

---

## extract_soft_match_slug
    + `reply`

Parse `SLUG: …` / `标识：…` after a soft-match REUSE decision.

*esc = > json.parse text={"nl":"\n","fw":"："}*
*nl = > json.get value=`esc` key="nl"*
*fw = > json.get value=`esc` key="fw"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*found = None*

- [`line`](`lines`)
  *t = > trim value=`line`*
  *parts = > split value=`t` sep=":"*
  *n = > len value=`parts`*
  1. `n` > 1
    *head = > at value=`parts` index=0*
    *head = > trim value=`head`*
    *rest = > at value=`parts` index=1*
    *rest = > trim value=`rest`*
    1. `head` == SLUG
      *found = rest*
    2. `head` == 标识
      *found = rest*
    3. *
      *_ = 1*
  2. *
    *parts = > split value=`t` sep=`fw`*
    *n = > len value=`parts`*
    1. `n` > 1
      *head = > at value=`parts` index=0*
      *head = > trim value=`head`*
      *rest = > at value=`parts` index=1*
      *rest = > trim value=`rest`*
      1. `head` == 标识
        *found = rest*
      2. `head` == SLUG
        *found = rest*
      3. *
        *_ = 1*
    2. *
      *_ = 1*

**found**

---

## build_soft_match_prompt
    + `goal`
    + `tasks`

Short parent prompt: REUSE+SLUG or NEW only. Candidates may include lexical `score` (prefer higher). `list_tasks` rows may also carry `description` / `status` / `llm_free`.

*esc = > json.parse text={"a":"\n\n--- goal ---\n","b":"\n\n--- candidate tasks (slug | title | meta) ---\n","c":"\n\n--- how to act ---\nExact reuse miss; candidates are ranked by local n-gram score when present. Prefer higher scores and llm_free=true skills. Reply with ONE protocol only:\nDECISION: REUSE\nSLUG: <exact-slug-from-list>\nor\nDECISION: NEW\nNo other prose.\n","nl":"\n","sep":" | ","sp":"score=","st":"status=","lf":"llm_free="}*
*a = > json.get value=`esc` key="a"*
*b = > json.get value=`esc` key="b"*
*c = > json.get value=`esc` key="c"*
*nl = > json.get value=`esc` key="nl"*
*sep = > json.get value=`esc` key="sep"*
*sp = > json.get value=`esc` key="sp"*
*st = > json.get value=`esc` key="st"*
*lf = > json.get value=`esc` key="lf"*
*lines = > json.parse text=[]*
- [`t`](`tasks`)
  *slug = > json.get value=`t` key="slug"*
  *title = > json.get value=`t` key="title"*
  *row = slug + sep + title*
  *score = > json.get value=`t` key="score"*
  1. `score`
    *row = row + sep + sp + score*
  2. *
    *_ = 1*
  *status = > json.get value=`t` key="status"*
  1. `status`
    *row = row + sep + st + status*
  2. *
    *_ = 1*
  *free = > json.get value=`t` key="llm_free"*
  1. `free` == True
    *row = row + sep + lf + "true"*
  2. `free` == true
    *row = row + sep + lf + "true"*
  3. `free` == False
    *row = row + sep + lf + "false"*
  4. `free` == false
    *row = row + sep + lf + "false"*
  5. *
    *_ = 1*
  *desc = > json.get value=`t` key="description"*
  1. `desc`
    *row = row + sep + desc*
  2. *
    *_ = 1*
  *lines = > json.append list=`lines` item=`row`*
*catalog = > join value=`lines` sep=`nl`*
*p = "Soft-match: same task intent?"*
**p + a + goal + b + catalog + c**

---

## extract_plan_summary
    + `reply`

First `SUMMARY:` / `汇总:` / `汇总：` line body, else trimmed reply.

*esc = > json.parse text={"nl":"\n","fw":"："}*
*nl = > json.get value=`esc` key="nl"*
*fw = > json.get value=`esc` key="fw"*
*text = > trim value=`reply`*
*lines = > split value=`text` sep=`nl`*
*found = None*

- [`line`](`lines`)
  *t = > trim value=`line`*
  *parts = > split value=`t` sep=":"*
  *n = > len value=`parts`*
  1. `n` > 1
    *head = > at value=`parts` index=0*
    *head = > trim value=`head`*
    *rest = > at value=`parts` index=1*
    *rest = > trim value=`rest`*
    1. `head` == SUMMARY
      *found = rest*
    2. `head` == 汇总
      *found = rest*
    3. *
      *_ = 1*
  2. *
    *parts = > split value=`t` sep=`fw`*
    *n = > len value=`parts`*
    1. `n` > 1
      *head = > at value=`parts` index=0*
      *head = > trim value=`head`*
      *rest = > at value=`parts` index=1*
      *rest = > trim value=`rest`*
      1. `head` == 汇总
        *found = rest*
      2. `head` == SUMMARY
        *found = rest*
      3. *
        *_ = 1*
    2. *
      *_ = 1*

1. `found`
  **found**
2. *
  **text**

---

## build_plan_context
    + `agent`
    + `goal`
    + `observation`
    + `explore_attempt`=None
    + `explore_n`=None
    + `phase`=revise

Parent Plan-and-Move prompt. Short protocol only — no long monologues.

*standing = > json.get value=`agent` key="standing"*
1. `standing`
  *up = standing*
2. *
  *up = None*

*skill = > skill_brief*
*compact = > compact_plan_observation observation=`observation`*
*obs_s = > json.stringify value=`compact`*
*goal_s = > json.stringify value=`goal`*
*esc = > json.parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- goal ---\n","c":"\n\n--- observation ---\n","d":"\n\n--- skill brief ---\n","tools":"\n\n--- parent tools ---\nCALL:workbook_read | workbook_excerpt | lib_catalog | scratch_tool_write\nCALL:lib.fs.read_text|exists|list_dir | lib.json.parse|stringify | lib.sys.cwd (+ ARGS:{json})\nREAD:source | stderr | stdout | slots\nCreate tools: CONTINUE PATCH add ##, or CALL:scratch_tool_write with NAME line and fenced body.\n","e_rev":"\n\n--- how to act ---\nPlan-and-Move parent. Reply with ONE protocol only (no long reasoning). Exactly one DECISION line.\n1) exit_code=0 and has_value → DECISION: DONE + one-line SUMMARY. solidify_on_done. Do not CONTINUE just because has_worker_step.\n2) Need more evidence → READ:kind or CALL:tool (then you will be re-prompted).\n3) Failure / wrong value → DECISION: CONTINUE + short PATCH (<20 lines). Never paste user prose into REPLACE.\n4) Do NOT CONTINUE only to rewrite frontmatter imports; skeleton already uses import llm:/agent:/json:….\nPATCH must use triple-angle blocks only:\n<<<\nFIND\n<exact old>\n===\nREPLACE\n<new>\n>>>\nDo not use \u0060\u0060\u0060find, \u0060\u0060\u0060replace, or *** Begin Patch ***.\nProtocols:\nCALL:name\nREAD:kind\nDECISION: DONE\nSUMMARY: one line\nDECISION: CONTINUE\n","e_dec":"\n\n--- how to act ---\nPRE-RUN decompose. Reply with ONE protocol only (no long reasoning). Exactly one DECISION line.\n1) Skeleton OK → DECISION: RUN (preferred). Do not rewrite imports; they are already import llm:/agent:/….\n2) Need evidence → READ:source or CALL:workbook_read / lib_catalog.\n3) Reshape body/logic only → DECISION: CONTINUE + short PATCH (<20 lines).\n4) Fixed answer without LLM → PATCH to return, then RUN or DONE.\nPATCH must use <<< FIND === REPLACE >>> only (no \u0060\u0060\u0060find / Begin Patch).\nProtocols:\nCALL:name\nREAD:kind\nDECISION: RUN\nDECISION: DONE\nSUMMARY: one line\nDECISION: CONTINUE\n","f":"\n\n--- explore ---\n"}*
*a = > json.get value=`esc` key="a"*
*b = > json.get value=`esc` key="b"*
*c = > json.get value=`esc` key="c"*
*d = > json.get value=`esc` key="d"*
*tools = > json.get value=`esc` key="tools"*
*f = > json.get value=`esc` key="f"*
1. `phase` == decompose
  *e = > json.get value=`esc` key="e_dec"*
  *p = "Plan-and-Move: decompose before first run."*
2. *
  *e = > json.get value=`esc` key="e_rev"*
  *p = "Plan-and-Move: revise after child run."*
*p = p + a + up + b + goal_s + c + obs_s + tools + d + skill + e*
1. `explore_attempt`
  *p = p + f*
  *p = p + "Attempt"*
  *p = p + explore_attempt*
  *p = p + "of"*
  *p = p + explore_n*
  *p = p + ". Different path; prefer code when fixed."*
2. *
  *_ = 1*
**p**

---

## plan_llm_act_loop
    + `agent`
    + `goal`
    + `path`
    + `observation`
    + `phase`=revise
    + `explore_attempt`=None
    + `explore_n`=None
    + `stream`=False
    + `echo`=False
    + `events`
    + `max_acts`=6

LLM Plan → CALL/READ/DECISION loop. Returns `{decision,reply,observation,events}`.

*model = > json.get value=`agent` key="model"*
*last_obs = observation*
*left = max_acts*
*decision = None*
*last_reply = None*
*evs_all = events*

- `left` > 0
  1. `decision`
    *left = 0*
  2. *
    *ctx = > build_plan_context agent=`agent` goal=`goal` observation=`last_obs` explore_attempt=`explore_attempt` explore_n=`explore_n` phase=`phase`*
    1. `stream`
      *evs = > model.complete prompt=`ctx` stream=True echo=`echo`*
      *last_reply = > llm.stream_result events=`evs`*
      *evs_all = > plan_merge_deltas events=`evs_all` from=`evs` stream=`stream`*
    2. *
      *last_reply = > model.complete prompt=`ctx`*
    *last_reply = > trim value=`last_reply`*
    *act = > extract_plan_act reply=`last_reply`*
    *kind = > json.get value=`act` key="kind"*
    *name = > json.get value=`act` key="name"*
    1. `kind` == call
      *evs_all = > plan_append_tool events=`evs_all` type="tool_start" name=`name` kind="call" stream=`stream`*
      *tool_out = > run_parent_tool name=`name` path=`path` observation=`last_obs` reply=`last_reply`*
      *brief = > tool_result_brief value=`tool_out`*
      *evs_all = > plan_append_tool events=`evs_all` type="tool_end" name=`name` kind="call" result=`brief` stream=`stream`*
      *last_obs = > json.set map=`last_obs` key="tool_result" value=`tool_out`*
      *left = left - 1*
    2. `kind` == read
      *evs_all = > plan_append_tool events=`evs_all` type="tool_start" name=`name` kind="read" stream=`stream`*
      *last_obs = > plan_read_deepen observation=`last_obs` kind=`name` path=`path`*
      *brief = > tool_result_brief value=`last_obs`*
      *evs_all = > plan_append_tool events=`evs_all` type="tool_end" name=`name` kind="read" result=`brief` stream=`stream`*
      *left = left - 1*
    3. `kind` == decision
      *decision = name*
      *evs_all = > plan_append_decision events=`evs_all` decision=`decision` stream=`stream`*
      *left = 0*
    4. *
      *decision = "unknown"*
      *left = 0*

*out = > json.parse text={}*
*out = > json.set map=`out` key="decision" value=`decision`*
*out = > json.set map=`out` key="reply" value=`last_reply`*
*out = > json.set map=`out` key="observation" value=`last_obs`*
*out = > json.set map=`out` key="events" value=`evs_all`*
**out**

---

## plan_echo_decompose
    + `workbook`
    + `stream`=False
    + `echo`=False

Pre-run parent decompose marker. With `stream=True`, publish `{type:decompose}` so view is not silent before the first child.

1. `stream`
  *ev = > json.parse text={"type":"decompose"}*
  *ev = > json.set map=`ev` key="workbook" value=`workbook`*
  > sys.stream_publish event=`ev`
  1. `echo`
    > print text=plan:decompose
  2. *
    *_ = 1*
  **ev**
2. *
  ****

## plan_echo_await
    + `round`
    + `workbook`
    + `stream`=False
    + `echo`=False

Marker before `await_workbook`. With `stream=True`, publish `{type:await,round,workbook}` so the Stream panel shows progress while the quiet child runs (TTY `echo` optional).

1. `stream`
  *ev = > json.parse text={"type":"await"}*
  *ev = > json.set map=`ev` key="round" value=`round`*
  *ev = > json.set map=`ev` key="workbook" value=`workbook`*
  > sys.stream_publish event=`ev`
  1. `echo`
    > print text=plan:await
    > print text=`round`
  2. *
    *_ = 1*
  **ev**
2. *
  ****

## workbook_has_worker_step
    + `path`

True when source still contains `worker.step` or `.单步`.

*src = > fs.read_text path=`path`*
*a = > split value=`src` sep="worker.step"*
*na = > len value=`a`*
*b = > split value=`src` sep=".单步"*
*nb = > len value=`b`*
1. `na` > 1
  **1**
2. `nb` > 1
  **1**
3. *
  **0**

## plan_append_round
    + `events`
    + `round`
    + `workbook`
    + `exit_code`
    + `result`=None
    + `stream`=False
    + `echo`=False

Always append a `round` event (child workbook finished) for audit / view. Optional `result` is the child `# main` return. SSE + optional TTY `plan:round` only when `stream` / `echo`.

*ev = > json.parse text={"type":"round"}*
*ev = > json.set map=`ev` key="round" value=`round`*
*ev = > json.set map=`ev` key="workbook" value=`workbook`*
*ev = > json.set map=`ev` key="exit_code" value=`exit_code`*
1. `result`
  *ev = > json.set map=`ev` key="result" value=`result`*
2. *
  *_ = 1*
*events = > json.append list=`events` item=`ev`*
1. `stream`
  > sys.stream_publish event=`ev`
2. *
  *_ = 1*
1. `echo`
  > print text=plan:round
2. *
  *_ = 1*

**events**

## plan_append_decision
    + `events`
    + `decision`
    + `stream`=False
    + `summary`=None

Always append a `decision` event. SSE publish only when `stream=True`.

*ev = > json.parse text={"type":"decision"}*
*ev = > json.set map=`ev` key="decision" value=`decision`*
1. `summary`
  *ev = > json.set map=`ev` key="summary" value=`summary`*
2. *
  *_ = 1*
*events = > json.append list=`events` item=`ev`*
1. `stream`
  > sys.stream_publish event=`ev`
2. *
  *_ = 1*

**events**

## plan_merge_deltas
    + `events`
    + `from`
    + `stream`=False

Bubble parent `complete stream=True` reasoning/deltas (and errors) into the plan event list. Skip nested `done` so plan owns the final `done`. Always merge into `events` when present; `stream` reserved for callers that also SSE-publish upstream.

- [`ev`](`from`)
  *t = > json.get value=`ev` key="type"*
  1. `t` == delta
    *events = > json.append list=`events` item=`ev`*
  2. `t` == reasoning
    *events = > json.append list=`events` item=`ev`*
  3. `t` == error
    *events = > json.append list=`events` item=`ev`*
  4. *
    *_ = 1*

**events**

## plan_finish_stream
    + `out`
    + `events`
    + `stream`=False
    + `trace`=False
    + `result`=None

Always append `done` and attach `events` on the result map (process audit even when not streaming). SSE publish of `done` only when `stream=True`. Optional `trace=True` writes the event list to writeback slot `trace`.

*ev = > json.parse text={"type":"done"}*
1. `result`
  *ev = > json.set map=`ev` key="result" value=`result`*
2. *
  *_ = 1*
*events = > json.append list=`events` item=`ev`*
1. `stream`
  > sys.stream_publish event=`ev`
2. *
  *_ = 1*
*out = > json.set map=`out` key="events" value=`events`*

1. `trace`
  *body = > json.stringify value=`events`*
  > writeback.record value=`body` key=trace
2. *
  *_ = 1*

**out**

---

# agent
    + `model`
    + `tools`
    + `standing`

Load ABI v2 agent plugin once (session bag + context queries).

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)
  > sys.exit code=1

*id = > agent_alloc*
*h = > json.parse text={"_type":"agent"}*
*h = > json.set map=`h` key="id" value=`id`*
*h = > json.set map=`h` key="model" value=`model`*
*h = > json.set map=`h` key="tools" value=`tools`*

1. `standing`
  *h = > json.set map=`h` key="standing" value=`standing`*
2. *
  *_ = 1*

**h**

## clear_history

Clear the plugin session bag for this agent id.

*id = > json.get value=`self` key="id"*
> agent_history_clear id=`id`
****

## step
    + `task`
    + `writeback`=True
    + `stream`=False
    + `echo`=False

One atomic turn: context → LLM → optional tool via subtask. Returns a **map** (`status`, `task`, `decision`, and on success `result` plus optional `tool` / `tool_result`; on failure `error`). By default (`writeback=True`) persists that map under named slots `ok` / `error` at the call site; pass `writeback=False` to skip.

With `stream=True`, the model call uses SSE; `echo=True` prints delta text to stdout as it arrives. The returned map is unchanged (final `result` string).

*ctx = > build_step_context agent=`self` task=`task`*
*id = > json.get value=`self` key="id"*
*model = > json.get value=`self` key="model"*
*tools = > json.get value=`self` key="tools"*

*user_turn = > json.parse text={"role":"user"}*
*user_turn = > json.set map=`user_turn` key="content" value=`task`*
> agent_history_append id=`id` item=`user_turn`

1. `stream`
  *evs = > model.complete prompt=`ctx` stream=True echo=`echo`*
  *reply = > llm.stream_result events=`evs`*
2. *
  *reply = > model.complete prompt=`ctx`*

*reply = > trim value=`reply`*
*decision = reply*
*tool_name = > extract_tool_name reply=`reply`*
*out = > json.parse text={"status":"ok"}*
*out = > json.set map=`out` key="task" value=`task`*
*out = > json.set map=`out` key="decision" value=`decision`*

1. `tool_name`
  *tool_out = > run_tool tools=`tools` name=`tool_name`*
  *tool_s = > json.stringify value=`tool_out`*
  *deny = > split value=`tool_s` sep="Tool not allowed"*
  *deny_n = > len value=`deny`*
  1. `deny_n` > 1
    *task_s = > json.stringify value=`task`*
    *fp = "Tool CALL was rejected (not on the tools whitelist). Do NOT emit CALL. Answer the user task directly with the final answer only.\n\nUser task:\n"*
    *fp = fp + task_s*
    1. `stream`
      *evs_retry = > model.complete prompt=`fp` stream=True echo=`echo`*
      *reply = > llm.stream_result events=`evs_retry`*
    2. *
      *reply = > model.complete prompt=`fp`*
    *out = > json.set map=`out` key="status" value="ok"*
    *out = > json.set map=`out` key="decision" value=`decision`*
    *out = > json.set map=`out` key="denied_tool" value=`tool_name`*
    *out = > json.set map=`out` key="result" value=`reply`*
  2. *
    *task_s = > json.stringify value=`task`*
    *fp = "The user asked:"*
    *fp = fp + task_s*
    *fp = fp + ". Tool"*
    *fp = fp + tool_name*
    *fp = fp + "ran via subtask and returned:"*
    *fp = fp + tool_s*
    *fp = fp + ". Reply to the user briefly."*
    1. `stream`
      *evs2 = > model.complete prompt=`fp` stream=True echo=`echo`*
      *reply = > llm.stream_result events=`evs2`*
    2. *
      *reply = > model.complete prompt=`fp`*
    *out = > json.set map=`out` key="tool" value=`tool_name`*
    *out = > json.set map=`out` key="tool_result" value=`tool_s`*
    *out = > json.set map=`out` key="result" value=`reply`*
2. *
  *out = > json.set map=`out` key="result" value=`reply`*

*as_turn = > json.parse text={"role":"assistant"}*
*as_turn = > json.set map=`as_turn` key="content" value=`reply`*
> agent_history_append id=`id` item=`as_turn`

1. `writeback`
  *body = > json.stringify value=`out`*
  *st = > json.get value=`out` key="status"*
  1. `st` == ok
    > writeback.record value=`body` key=ok
  2. *
    > writeback.record value=`body` key=error
2. *
  *_ = 1*

**out**

## plan
    + `goal`
    + `workbook_dir`=None
    + `workbook`=None
    + `max_rounds`=4
    + `confirm`=False
    + `writeback`=True
    + `skeleton`=single
    + `reuse`=True
    + `optimize`=False
    + `force`=False
    + `soft_match`=False
    + `near_match`=True
    + `near_threshold`=0.78
    + `promote`=True
    + `kb_dir`=.marqdo/agent-kb
    + `improve_every`=3
    + `explore_n`=3
    + `stream`=False
    + `echo`=False
    + `trace`=False

Multi-step with OKF agent-kb. Default workbook is `kb_dir/resources/<slug>.mq.md`. While task file count `< explore_n` and skill is not llm_free, force a new explore variant under `kb_dir/explore/<slug>/`. Code-first: llm_free hits skip parent LLM. File children return via `# main`; `plan` exposes that as `result`.

Reuse lookup: exact → alias → canonicalize → optional local n-gram `near` when `near_match=True` and score ≥ `near_threshold`. Non-hit path: optional `soft_match=True` parent REUSE/NEW over ranked `agent_kb_near_match` candidates; else **decompose** before first child spawn (`DECISION: RUN` / `CONTINUE`+patch / solidified `DONE`). Then `await` → revise loop. Process events (`decision` / `round` / `done` / tools) are **always** attached on the result map for audit and view process cards; `stream=True` additionally SSE-publishes them (and parent `delta`). `echo=True` prints `plan:decompose` / `plan:await` / deltas. `trace=True` writes events to writeback slot `trace`. Quiet child subtasks stay quiet.

*tools = > json.get value=`self` key="tools"*
*cache = "miss"*
*path = None*
*improve = None*
*explore = None*
*explore_attempt = None*
*skel_kind = skeleton*
*events = > json.parse text=[]*

*tf = > agent_kb_task_files kb_dir=`kb_dir` goal=`goal` tools=`tools`*
*nfiles = > json.get value=`tf` key="count"*

1. `force`
  *_ = 1*
2. `optimize`
  *_ = 1*
3. `reuse`
  *hit = > agent_kb_lookup kb_dir=`kb_dir` goal=`goal` tools=`tools` near_match=`near_match` near_threshold=`near_threshold`*
  1. `hit`
    *match_kind = > json.get value=`hit` key="match"*
    1. `match_kind` == alias
      *cache_label = "soft-hit"*
    2. `match_kind` == canonical
      *cache_label = "soft-hit"*
      *hit_slug = > json.get value=`hit` key="slug"*
      > agent_kb_add_alias kb_dir=`kb_dir` slug=`hit_slug` alias=`goal`
    3. `match_kind` == near
      *cache_label = "soft-hit"*
      *hit_slug = > json.get value=`hit` key="slug"*
      > agent_kb_add_alias kb_dir=`kb_dir` slug=`hit_slug` alias=`goal`
    4. `match_kind` == soft
      *cache_label = "soft-hit"*
    5. *
      *cache_label = "hit"*
    *lf = > json.get value=`hit` key="llm_free"*
    1. `lf`
      *path = > json.get value=`hit` key="resource"*
      *aw = > await_workbook path=`path`*
      *code = > json.get value=`aw` key="code"*
      *child_val = > json.get value=`aw` key="value"*
      *last_obs = > json.get value=`aw` key="observation"*
      1. `code` == 0
        > agent_kb_record_hit kb_dir=`kb_dir` goal=`goal` tools=`tools` improve_every=`improve_every`
        *cache = cache_label*
        *out = > json.parse text={"status":"ok"}*
        *out = > json.set map=`out` key="status" value="ok"*
        *out = > json.set map=`out` key="goal" value=`goal`*
        *out = > json.set map=`out` key="workbook" value=`path`*
        *out = > json.set map=`out` key="rounds" value=1*
        *out = > json.set map=`out` key="cache" value=`cache_label`*
        *out = > json.set map=`out` key="match" value=`match_kind`*
        *hit_score = > json.get value=`hit` key="score"*
        1. `hit_score`
          *out = > json.set map=`out` key="score" value=`hit_score`*
        2. *
          *_ = 1*
        *sk = > json.get value=`hit` key="skill"*
        *out = > json.set map=`out` key="skill" value=`sk`*
        *st = > json.get value=`hit` key="status"*
        *out = > json.set map=`out` key="skill_status" value=`st`*
        *sum = > json.parse text={"a":"OKF llm_free skill hit (","b":"); spawned resource"}*
        *suma = > json.get value=`sum` key="a"*
        *sumb = > json.get value=`sum` key="b"*
        *sum = suma + match_kind + sumb*
        *out = > json.set map=`out` key="summary" value=`sum`*
        *out = > json.set map=`out` key="observation" value=`last_obs`*
        *out = > json.set map=`out` key="result" value=`child_val`*
        *events = > plan_append_decision events=`events` decision="REUSE" stream=`stream` summary=`sum`*
        *events = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo`*
        1. `writeback`
          *body = > json.stringify value=`out`*
          > writeback.record value=`body` key=ok
        2. *
          *_ = 1*
        *out = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`child_val`*
        **out**
      2. *
        *path = None*
    2. `nfiles` < `explore_n`
      *explore = 1*
      *cache = "explore"*
      *path = None*
      *explore_attempt = nfiles + 1*
      1. `explore_attempt` == 2
        *skel_kind = "dual"*
      2. *
        *skel_kind = skeleton*
    3. *
      *path = > json.get value=`hit` key="resource"*
      *aw = > await_workbook path=`path`*
      *code = > json.get value=`aw` key="code"*
      *child_val = > json.get value=`aw` key="value"*
      *last_obs = > json.get value=`aw` key="observation"*
      1. `code` == 0
        *rec = > agent_kb_record_hit kb_dir=`kb_dir` goal=`goal` tools=`tools` improve_every=`improve_every`*
        *due = > json.get value=`rec` key="improve_due"*
        1. `due`
          *improve = 1*
          *cache = "improve"*
        2. *
          *cache = cache_label*
          *out = > json.parse text={"status":"ok"}*
          *out = > json.set map=`out` key="status" value="ok"*
          *out = > json.set map=`out` key="goal" value=`goal`*
          *out = > json.set map=`out` key="workbook" value=`path`*
          *out = > json.set map=`out` key="rounds" value=1*
          *out = > json.set map=`out` key="cache" value=`cache_label`*
          *out = > json.set map=`out` key="match" value=`match_kind`*
          *hit_score = > json.get value=`hit` key="score"*
          1. `hit_score`
            *out = > json.set map=`out` key="score" value=`hit_score`*
          2. *
            *_ = 1*
          *sk = > json.get value=`hit` key="skill"*
          *out = > json.set map=`out` key="skill" value=`sk`*
          *st = > json.get value=`hit` key="status"*
          *out = > json.set map=`out` key="skill_status" value=`st`*
          *sum = > json.parse text={"a":"OKF skill hit (","b":"); spawned resource"}*
          *suma = > json.get value=`sum` key="a"*
          *sumb = > json.get value=`sum` key="b"*
          *sum = suma + match_kind + sumb*
          *out = > json.set map=`out` key="summary" value=`sum`*
          *out = > json.set map=`out` key="observation" value=`last_obs`*
          *out = > json.set map=`out` key="result" value=`child_val`*
          *events = > plan_append_decision events=`events` decision="REUSE" stream=`stream` summary=`sum`*
          *events = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo`*
          1. `writeback`
            *body = > json.stringify value=`out`*
            > writeback.record value=`body` key=ok
          2. *
            *_ = 1*
          *out = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`child_val`*
          **out**
      2. *
        *path = None*
  2. *
    1. `soft_match`
      *near = > agent_kb_near_match kb_dir=`kb_dir` goal=`goal`*
      *tasks = > json.get value=`near` key="candidates"*
      *tn = > len value=`tasks`*
      1. `tn` < 1
        *listed = > agent_kb_list_tasks kb_dir=`kb_dir`*
        *tasks = > json.get value=`listed` key="tasks"*
        *tn = > len value=`tasks`*
      2. *
        *_ = 1*
      1. `tn` > 0
        *soft_prompt = > build_soft_match_prompt goal=`goal` tasks=`tasks`*
        *model = > json.get value=`self` key="model"*
        1. `stream`
          *evs = > model.complete prompt=`soft_prompt` stream=True echo=`echo`*
          *soft_reply = > llm.stream_result events=`evs`*
          *events = > plan_merge_deltas events=`events` from=`evs` stream=`stream`*
        2. *
          *soft_reply = > model.complete prompt=`soft_prompt`*
        *soft_reply = > trim value=`soft_reply`*
        *soft_dec = > extract_plan_decision reply=`soft_reply`*
        *events = > plan_append_decision events=`events` decision=`soft_dec` stream=`stream`*
        1. `soft_dec` == REUSE
          *reuse_slug = > extract_soft_match_slug reply=`soft_reply`*
          1. `reuse_slug`
            *hit = > agent_kb_lookup kb_dir=`kb_dir` slug=`reuse_slug` goal=`goal` tools=`tools`*
            1. `hit`
              > agent_kb_add_alias kb_dir=`kb_dir` slug=`reuse_slug` alias=`goal`
              *cache_label = "soft-hit"*
              *path = > json.get value=`hit` key="resource"*
              *aw = > await_workbook path=`path`*
              *code = > json.get value=`aw` key="code"*
              *child_val = > json.get value=`aw` key="value"*
              *last_obs = > json.get value=`aw` key="observation"*
              1. `code` == 0
                > agent_kb_record_hit kb_dir=`kb_dir` goal=`goal` tools=`tools` improve_every=`improve_every`
                *cache = "soft-hit"*
                *out = > json.parse text={"status":"ok"}*
                *out = > json.set map=`out` key="status" value="ok"*
                *out = > json.set map=`out` key="goal" value=`goal`*
                *out = > json.set map=`out` key="workbook" value=`path`*
                *out = > json.set map=`out` key="rounds" value=1*
                *out = > json.set map=`out` key="cache" value="soft-hit"*
                *out = > json.set map=`out` key="match" value="soft"*
                *sk = > json.get value=`hit` key="skill"*
                *out = > json.set map=`out` key="skill" value=`sk`*
                *st = > json.get value=`hit` key="status"*
                *out = > json.set map=`out` key="skill_status" value=`st`*
                *sum = "OKF soft_match REUSE; spawned resource"*
                *out = > json.set map=`out` key="summary" value=`sum`*
                *out = > json.set map=`out` key="observation" value=`last_obs`*
                *out = > json.set map=`out` key="result" value=`child_val`*
                *events = > plan_append_decision events=`events` decision="REUSE" stream=`stream` summary=`sum`*
                *events = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo`*
                1. `writeback`
                  *body = > json.stringify value=`out`*
                  > writeback.record value=`body` key=ok
                2. *
                  *_ = 1*
                *out = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`child_val`*
                **out**
              2. *
                *path = None*
            2. *
              *_ = 1*
          2. *
            *_ = 1*
        2. *
          *_ = 1*
      2. *
        *_ = 1*
    2. *
      *_ = 1*
    1. `nfiles` < `explore_n`
      *explore = 1*
      *cache = "explore"*
      *explore_attempt = nfiles + 1*
      1. `explore_attempt` == 2
        *skel_kind = "dual"*
      2. *
        *skel_kind = skeleton*
    2. *
      *_ = 1*
4. *
  1. `nfiles` < `explore_n`
    *explore = 1*
    *cache = "explore"*
    *explore_attempt = nfiles + 1*
  2. *
    *_ = 1*

*skel = > render_workbook_skeleton goal=`goal` skeleton=`skel_kind`*

1. `workbook`
  *path = workbook*
  *ex = > fs.exists path=`path`*
  1. `ex`
    *_ = 1*
  2. *
    > fs.write_text path=`path` text=`skel`
2. *
  1. `path`
    *_ = 1*
  2. *
    *slug = > agent_goal_slug goal=`goal`*
    1. `workbook_dir`
      > fs.make_dir path=`workbook_dir`
      *ts = > time.now_ms*
      *parts = > json.parse text={"a":"/workbook-","b":"-","c":".mq.md"}*
      *a = > json.get value=`parts` key="a"*
      *b = > json.get value=`parts` key="b"*
      *c = > json.get value=`parts` key="c"*
      *path = workbook_dir + a + slug + b + ts + c*
    2. `explore`
      *parts = > json.parse text={"a":"/explore/","b":"/","c":".mq.md"}*
      *a = > json.get value=`parts` key="a"*
      *b = > json.get value=`parts` key="b"*
      *c = > json.get value=`parts` key="c"*
      *path = kb_dir + a + slug + b + explore_attempt + c*
    3. *
      *parts = > json.parse text={"a":"/resources/","b":".mq.md"}*
      *a = > json.get value=`parts` key="a"*
      *b = > json.get value=`parts` key="b"*
      *path = kb_dir + a + slug + b*
    > fs.write_text path=`path` text=`skel`

1. `confirm`
  *out = > json.parse text={"status":"pending"}*
  *out = > json.set map=`out` key="goal" value=`goal`*
  *out = > json.set map=`out` key="workbook" value=`path`*
  *out = > json.set map=`out` key="summary" value="workbook created; confirm to run"*
  *out = > json.set map=`out` key="cache" value="bypass"*
  1. `writeback`
    *body = > json.stringify value=`out`*
    > writeback.record value=`body` key=ok
  2. *
    *_ = 1*
  *out = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace`*
  **out**
2. *
  *_ = 1*

*model = > json.get value=`self` key="model"*
*round = 0*
*last_obs = None*
*last_reply = None*
*child_val = None*
*done = None*
*summary = None*
*status = "error"*
*err = "max_rounds exhausted"*
*skip_loop = None*

> plan_echo_decompose workbook=`path` stream=`stream` echo=`echo`
*last_obs = > inspect_workbook path=`path`*
*turn = > plan_llm_act_loop agent=`self` goal=`goal` path=`path` observation=`last_obs` phase="decompose" explore_attempt=`explore_attempt` explore_n=`explore_n` stream=`stream` echo=`echo` events=`events`*
*dec = > json.get value=`turn` key="decision"*
*last_reply = > json.get value=`turn` key="reply"*
*last_obs = > json.get value=`turn` key="observation"*
*events = > json.get value=`turn` key="events"*

1. `dec` == CONTINUE
  *n = > fs.apply_patch_blocks path=`path` text=`last_reply` soft=True*
  1. `n`
    *_ = 1*
  2. *
    *done = 1*
    *status = "error"*
    *err = "no patches applied"*
    *summary = > extract_plan_summary reply=`last_reply`*
    *skip_loop = 1*
2. `dec` == RUN
  *_ = 1*
3. `dec` == DONE
  *has_step = > workbook_has_worker_step path=`path`*
  1. `has_step`
    *_ = 1*
  2. *
    > plan_echo_await round=1 workbook=`path` stream=`stream` echo=`echo`
    *aw = > await_workbook path=`path`*
    *code = > json.get value=`aw` key="code"*
    *child_val = > json.get value=`aw` key="value"*
    *last_obs = > json.get value=`aw` key="observation"*
    *round = 1*
    *events = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo`*
    1. `code` == 0
      *done = 1*
      *status = "ok"*
      *summary = > extract_plan_summary reply=`last_reply`*
      *err = None*
      *skip_loop = 1*
    2. *
      *_ = 1*
4. *
  *done = 1*
  *status = "error"*
  *err = "unrecognized plan decision"*
  *summary = last_reply*
  *skip_loop = 1*

1. `skip_loop`
  *left = 0*
2. `improve`
  *left = 1*
3. *
  *left = max_rounds*

- `left` > 0
  1. `done`
    *left = 0*
  2. *
    *round = round + 1*
    > plan_echo_await round=`round` workbook=`path` stream=`stream` echo=`echo`
    *aw = > await_workbook path=`path`*
    *code = > json.get value=`aw` key="code"*
    *child_val = > json.get value=`aw` key="value"*
    *last_obs = > json.get value=`aw` key="observation"*
    *events = > plan_append_round events=`events` round=`round` workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo`*

Deterministic success stop (loop engineering): when the child already returned a value, do not ask the parent LLM to invent solidify patches — DONE + runtime solidify.

    1. `code` == 0
      1. `child_val`
        1. `improve`
          *auto_done = None*
        2. *
          *auto_done = 1*
      2. *
        *auto_done = None*
    2. *
      *auto_done = None*

    1. `auto_done`
      > agent_workbook_solidify path=`path` observation=`last_obs`
      *done = 1*
      *status = "ok"*
      *summary = "child returned value; solidified"*
      *err = None*
      *dec = "DONE"*
      *events = > plan_append_decision events=`events` decision=`dec` stream=`stream` summary=`summary`*
      *left = 0*
    2. *
      *turn = > plan_llm_act_loop agent=`self` goal=`goal` path=`path` observation=`last_obs` phase="revise" explore_attempt=`explore_attempt` explore_n=`explore_n` stream=`stream` echo=`echo` events=`events`*
      *dec = > json.get value=`turn` key="decision"*
      *last_reply = > json.get value=`turn` key="reply"*
      *last_obs = > json.get value=`turn` key="observation"*
      *events = > json.get value=`turn` key="events"*
      1. `dec` == DONE
        > agent_workbook_solidify path=`path` observation=`last_obs`
        *done = 1*
        *status = "ok"*
        *summary = > extract_plan_summary reply=`last_reply`*
        *err = None*
        *left = 0*
      2. `dec` == CONTINUE
        *n = > fs.apply_patch_blocks path=`path` text=`last_reply` soft=True*
        1. `n`
          *left = left - 1*
        2. *
          *done = 1*
          *status = "error"*
          *err = "no patches applied"*
          *summary = > extract_plan_summary reply=`last_reply`*
          *left = 0*
      3. `dec` == RUN
        *left = left - 1*
      4. *
        *done = 1*
        *status = "error"*
        *err = "unrecognized plan decision"*
        *summary = last_reply*
        *left = 0*

1. `status` == ok
  1. `promote`
    *prom = > agent_kb_promote kb_dir=`kb_dir` goal=`goal` workbook=`path` tools=`tools`*
    *cache = "refreshed"*
  2. *
    *cache = "miss"*
2. *
  *_ = 1*

*out = > json.parse text={"status":"ok"}*
*out = > json.set map=`out` key="status" value=`status`*
*out = > json.set map=`out` key="goal" value=`goal`*
*out = > json.set map=`out` key="workbook" value=`path`*
*out = > json.set map=`out` key="rounds" value=`round`*
*out = > json.set map=`out` key="cache" value=`cache`*
1. `summary`
  *out = > json.set map=`out` key="summary" value=`summary`*
2. *
  *_ = 1*

1. `err`
  *out = > json.set map=`out` key="error" value=`err`*
2. *
  *_ = 1*

1. `last_obs`
  *out = > json.set map=`out` key="observation" value=`last_obs`*
2. *
  *_ = 1*

1. `child_val`
  *out = > json.set map=`out` key="result" value=`child_val`*
  *plan_result = child_val*
2. `summary`
  *out = > json.set map=`out` key="result" value=`summary`*
  *plan_result = summary*
3. *
  *plan_result = None*

1. `writeback`
  *body = > json.stringify value=`out`*
  1. `status` == ok
    > writeback.record value=`body` key=ok
  2. *
    > writeback.record value=`body` key=error
2. *
  *_ = 1*

*out = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`plan_result`*
**out**
