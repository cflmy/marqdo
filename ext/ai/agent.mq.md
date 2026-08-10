---
title: ext/ai/agent
description: Document-driven agent — step / plan; tools via lib/subtask; runtime via ABI v2 agent plugin.
> ext/ai/llm.mq.md
> lib/json.mq.md
> lib/sys.mq.md
> lib/fs.mq.md
> lib/time.mq.md
> lib/writeback.mq.md
> lib/subtask.mq.md
> lib/plugin.mq.md
---

## build_step_context
    + `agent`
    + `task`

Assemble a readable prompt: standing, task, tools, call site, source, skill, and the act protocol (human-visible).

*`standing` = > json.get value=`agent` key=standing *
1. `standing`
  *`up` = `standing`*
2. *
  *`up` = None*

*`tools` = > json.get value=`agent` key=tools *
*`tools_s` = > agent_format_tools tools=`tools` *
*`src` = > agent_module_source *
*`site` = > agent_call_site *
*`site_s` = > json.stringify value=`site` *
*`skill` = > agent_marqdo_skill *
*`task_s` = > json.stringify value=`task` *

*`esc` = > json.parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- task ---\n","c":"\n\n--- tools ---\n","d":"\n\n--- call site ---\n","e":"\n\n--- source (.mq.md) ---\n","f":"\n\n--- marqdo skill ---\n","g":"\n\n--- how to act ---\nTools are ## functions in the source above. To call one, reply with exactly one line: CALL:<name>\n(Chinese ok: 调用：<name>)\nOtherwise reply with the final answer only.\nTools run via Marqdo subtask (spawn fn / wait).\n"} *
*`a` = > json.get value=`esc` key=a *
*`b` = > json.get value=`esc` key=b *
*`c` = > json.get value=`esc` key=c *
*`d` = > json.get value=`esc` key=d *
*`e` = > json.get value=`esc` key=e *
*`f` = > json.get value=`esc` key=f *
*`g` = > json.get value=`esc` key=g *

*`p` = You are a Marqdo agent. The runbook source is your ground truth — code is documentation. *
*`p` = `p` + `a` + `up` + `b` + `task_s` + `c` + `tools_s` + `g` + `d` + `site_s` + `e` + `src` + `f` + `skill` *
**`p`**

---

## extract_tool_name
    + `reply`

Find a human-readable call line: `CALL:<name>` or `调用：<name>` / `调用:<name>`.

*`esc` = > json.parse text={"nl":"\n"} *
*`nl` = > json.get value=`esc` key=nl *
*`text` = > trim value=`reply` *
*`lines` = > split value=`text` sep=`nl` *
*`found` = None*

- [`line`](`lines`)
  *`t` = > trim value=`line` *
  *`parts` = > split value=`t` sep=: *
  *`n` = > len value=`parts` *
  1. `n` > 1
    *`head` = > at value=`parts` index=0 *
    *`head` = > trim value=`head` *
    *`rest` = > at value=`parts` index=1 *
    *`rest` = > trim value=`rest` *
    1. `head` == CALL
      *`found` = `rest`*
    2. `head` == 调用
      *`found` = `rest`*
    3. *
      *`_` = 1*
  2. *
    *`_` = 1*

**`found`**

---

## run_tool
    + `tools`
    + `name`

Allowlist check, then invoke the runbook `##` via `lib/subtask` (`spawn fn=` → `wait`).

*`allowed` = > agent_tool_allowed tools=`tools` name=`name` *
1. `allowed`
  *`id` = > subtask.spawn fn=`name` *
  **> subtask.wait id=`id`**
2. *
  *`msg` = Tool not allowed: *
  **`msg` + `name`**

## render_workbook_skeleton
    + `goal`
    + `skeleton`=single

Build a runnable workbook. `skeleton=single` (default): one worker `step`. `skeleton=dual`: research then write agents. On success the parent chooses DONE; runtime `agent_workbook_solidify` freezes the answer.

*`q` = > json.quote text=`goal` *
*`esc` = > json.parse text={"h":"---\ntitle: agent workbook\n> ext/ai/llm.mq.md\n> ext/ai/agent.mq.md\n> lib/json.mq.md\n> lib/sys.mq.md\n> lib/writeback.mq.md\n> lib/subtask.mq.md\n---\n\n# Goal\n\nParent goal is embedded as JSON in # main.\n\n## Solidify\n\nWhen the child returns a good value, parent DECISION: DONE — runtime agent_workbook_solidify freezes the answer as **return** (do not invent long FIND/REPLACE).\n\n# main\n\n> llm.load_env\n\n","s1":"\u002a\u0060model\u0060 = > llm.llm \u002a\n","s2":"\u002a\u0060tools\u0060 = > json.parse text=[] \u002a\n","s3":"\u002a\u0060worker\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You are a workbook worker. Finish the task; do not invent tools. Prefer a direct answer. \u002a\n","s4":"\u002a\u0060wrap\u0060 = > json.parse text={\"task\":","s5":"} \u002a\n","s6":"\u002a\u0060task\u0060 = > json.get value=\u0060wrap\u0060 key=task \u002a\n","s7":"\u002a\u0060out\u0060 = > \u0060worker\u0060.step task=\u0060task\u0060 \u002a\n","s7b":"\u002a\u0060text\u0060 = > json.get value=\u0060out\u0060 key=result \u002a\n","s8":"\u002a\u002a\u0060text\u0060\u002a\u002a\n","d3":"\u002a\u0060research\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You gather facts for the goal. Be concise. Do not invent tools. \u002a\n","d3b":"\u002a\u0060writer\u0060 = > agent.agent model=\u0060model\u0060 tools=\u0060tools\u0060 standing=You write the final answer from research notes. Do not invent tools. \u002a\n","d4":"\u002a\u0060wrap\u0060 = > json.parse text={\"task\":","d5":"} \u002a\n","d6":"\u002a\u0060task\u0060 = > json.get value=\u0060wrap\u0060 key=task \u002a\n","d7":"\u002a\u0060notes\u0060 = > \u0060research\u0060.step task=\u0060task\u0060 \u002a\n","d8":"\u002a\u0060ns\u0060 = > json.stringify value=\u0060notes\u0060 \u002a\n","d9":"\u002a\u0060wt\u0060 = Write the final answer. Research notes: \u002a\n","d10":"\u002a\u0060wt\u0060 = \u0060wt\u0060 + \u0060ns\u0060 \u002a\n","d11":"\u002a\u0060out\u0060 = > \u0060writer\u0060.step task=\u0060wt\u0060 \u002a\n","d11b":"\u002a\u0060text\u0060 = > json.get value=\u0060out\u0060 key=result \u002a\n","d12":"\u002a\u002a\u0060text\u0060\u002a\u002a\n"} *
*`h` = > json.get value=`esc` key=h *
*`s1` = > json.get value=`esc` key=s1 *
*`s2` = > json.get value=`esc` key=s2 *

1. `skeleton` == dual
  *`s3` = > json.get value=`esc` key=d3 *
  *`s3b` = > json.get value=`esc` key=d3b *
  *`s4` = > json.get value=`esc` key=d4 *
  *`s5` = > json.get value=`esc` key=d5 *
  *`s6` = > json.get value=`esc` key=d6 *
  *`s7` = > json.get value=`esc` key=d7 *
  *`s8` = > json.get value=`esc` key=d8 *
  *`s9` = > json.get value=`esc` key=d9 *
  *`s10` = > json.get value=`esc` key=d10 *
  *`s11` = > json.get value=`esc` key=d11 *
  *`s11b` = > json.get value=`esc` key=d11b *
  *`s12` = > json.get value=`esc` key=d12 *
  **`h` + `s1` + `s2` + `s3` + `s3b` + `s4` + `q` + `s5` + `s6` + `s7` + `s8` + `s9` + `s10` + `s11` + `s11b` + `s12`**
2. *
  *`s3` = > json.get value=`esc` key=s3 *
  *`s4` = > json.get value=`esc` key=s4 *
  *`s5` = > json.get value=`esc` key=s5 *
  *`s6` = > json.get value=`esc` key=s6 *
  *`s7` = > json.get value=`esc` key=s7 *
  *`s7b` = > json.get value=`esc` key=s7b *
  *`s8` = > json.get value=`esc` key=s8 *
  **`h` + `s1` + `s2` + `s3` + `s4` + `q` + `s5` + `s6` + `s7` + `s7b` + `s8`**

---

## inspect_workbook
    + `path`
    + `exit_code`=None
    + `value`=None

Structured observation for the parent developer-agent: source, named writeback slots, exit code, and optional child return value.

*`source` = > fs.read_text path=`path` *
*`slots` = > writeback.scan_path path=`path` *
*`obs` = > json.parse text={"path":""} *
*`obs` = > json.set map=`obs` key=path value=`path` *
*`obs` = > json.set map=`obs` key=source value=`source` *
*`obs` = > json.set map=`obs` key=slots value=`slots` *
*`obs` = > json.set map=`obs` key=exit_code value=`exit_code` *
1. `value`
  *`obs` = > json.set map=`obs` key=value value=`value` *
2. *
  *`_` = 1*

*`last_ok` = None*
*`last_error` = None*
- [`slot`](`slots`)
  *`k` = > json.get value=`slot` key=key *
  *`body` = > json.get value=`slot` key=body *
  1. `k` == ok
    *`last_ok` = `body`*
  2. `k` == error
    *`last_error` = `body`*
  3. *
    *`_` = 1*

1. `last_ok`
  *`obs` = > json.set map=`obs` key=last_ok value=`last_ok` *
2. *
  *`_` = 1*

1. `last_error`
  *`obs` = > json.set map=`obs` key=last_error value=`last_error` *
2. *
  *`_` = 1*

**`obs`**

---

## await_workbook
    + `path`

Spawn a workbook file (quiet by default), wait for `{code,value}`, and inspect. Parent consumes `value` — not child stdout.

*`id` = > subtask.spawn path=`path` *
*`waited` = > subtask.wait id=`id` *
*`code` = > json.get value=`waited` key=code *
*`value` = > json.get value=`waited` key=value *
*`obs` = > inspect_workbook path=`path` exit_code=`code` value=`value` *
*`out` = > json.parse text={"code":0} *
*`out` = > json.set map=`out` key=code value=`code` *
*`out` = > json.set map=`out` key=value value=`value` *
*`out` = > json.set map=`out` key=observation value=`obs` *
**`out`**

---

## extract_plan_decision
    + `reply`

Parse `DECISION: DONE` / `CONTINUE` / `RUN` (Chinese: `决定：完成` / `继续` / `运行`).

*`esc` = > json.parse text={"nl":"\n","fw":"："} *
*`nl` = > json.get value=`esc` key=nl *
*`fw` = > json.get value=`esc` key=fw *
*`text` = > trim value=`reply` *
*`lines` = > split value=`text` sep=`nl` *
*`found` = None*

- [`line`](`lines`)
  *`t` = > trim value=`line` *
  *`parts` = > split value=`t` sep=: *
  *`n` = > len value=`parts` *
  1. `n` > 1
    *`head` = > at value=`parts` index=0 *
    *`head` = > trim value=`head` *
    *`rest` = > at value=`parts` index=1 *
    *`rest` = > trim value=`rest` *
    1. `head` == DECISION
      *`found` = `rest`*
    2. `head` == 决定
      *`found` = `rest`*
    3. *
      *`_` = 1*
  2. *
    *`parts` = > split value=`t` sep=`fw` *
    *`n` = > len value=`parts` *
    1. `n` > 1
      *`head` = > at value=`parts` index=0 *
      *`head` = > trim value=`head` *
      *`rest` = > at value=`parts` index=1 *
      *`rest` = > trim value=`rest` *
      1. `head` == 决定
        *`found` = `rest`*
      2. `head` == DECISION
        *`found` = `rest`*
      3. *
        *`_` = 1*
    2. *
      *`_` = 1*

1. `found` == DONE
  **DONE**
2. `found` == CONTINUE
  **CONTINUE**
3. `found` == RUN
  **RUN**
4. `found` == 完成
  **DONE**
5. `found` == 继续
  **CONTINUE**
6. `found` == 运行
  **RUN**
7. *
  **`found`**

---

## extract_plan_summary
    + `reply`

First `SUMMARY:` / `汇总:` / `汇总：` line body, else trimmed reply.

*`esc` = > json.parse text={"nl":"\n","fw":"："} *
*`nl` = > json.get value=`esc` key=nl *
*`fw` = > json.get value=`esc` key=fw *
*`text` = > trim value=`reply` *
*`lines` = > split value=`text` sep=`nl` *
*`found` = None*

- [`line`](`lines`)
  *`t` = > trim value=`line` *
  *`parts` = > split value=`t` sep=: *
  *`n` = > len value=`parts` *
  1. `n` > 1
    *`head` = > at value=`parts` index=0 *
    *`head` = > trim value=`head` *
    *`rest` = > at value=`parts` index=1 *
    *`rest` = > trim value=`rest` *
    1. `head` == SUMMARY
      *`found` = `rest`*
    2. `head` == 汇总
      *`found` = `rest`*
    3. *
      *`_` = 1*
  2. *
    *`parts` = > split value=`t` sep=`fw` *
    *`n` = > len value=`parts` *
    1. `n` > 1
      *`head` = > at value=`parts` index=0 *
      *`head` = > trim value=`head` *
      *`rest` = > at value=`parts` index=1 *
      *`rest` = > trim value=`rest` *
      1. `head` == 汇总
        *`found` = `rest`*
      2. `head` == SUMMARY
        *`found` = `rest`*
      3. *
        *`_` = 1*
    2. *
      *`_` = 1*

1. `found`
  **`found`**
2. *
  **`text`**

---

## build_plan_context
    + `agent`
    + `goal`
    + `observation`
    + `explore_attempt`=None
    + `explore_n`=None
    + `phase`=revise

Parent developer-agent prompt. `phase=decompose`: pre-run (design step ③) before any spawn. `phase=revise` (default): observe → DONE or CONTINUE with patches.

*`standing` = > json.get value=`agent` key=standing *
1. `standing`
  *`up` = `standing`*
2. *
  *`up` = None*

*`skill` = > agent_marqdo_skill *

Omit full workbook `source` and large slot/value bodies from the parent prompt — they bloat context and invite endless solidify guessing. Runtime `agent_workbook_solidify` runs on DECISION: DONE.

*`obs_path` = > json.get value=`observation` key=path *
*`obs_code` = > json.get value=`observation` key=exit_code *
*`obs_val` = > json.get value=`observation` key=value *
*`obs_slots` = > json.get value=`observation` key=slots *
*`obs_src` = > json.get value=`observation` key=source *
*`src_len` = 0*
*`has_step` = False*
1. `obs_src`
  *`src_len` = > len value=`obs_src` *
  *`step_parts` = > split value=`obs_src` sep=worker.step *
  *`n_step` = > len value=`step_parts` *
  1. `n_step` > 1
    *`has_step` = True*
  2. *
    *`_` = 1*
2. *
  *`_` = 1*

*`compact` = > json.parse text={"note":"source/slot bodies omitted. If exit_code is 0 and has_value, prefer DECISION: DONE immediately — runtime solidifies worker.step. CONTINUE only to fix failures or redesign structure. Never paste user prose into REPLACE."} *
*`compact` = > json.set map=`compact` key=path value=`obs_path` *
*`compact` = > json.set map=`compact` key=exit_code value=`obs_code` *
*`compact` = > json.set map=`compact` key=source_len value=`src_len` *
*`compact` = > json.set map=`compact` key=has_worker_step value=`has_step` *
*`compact` = > json.set map=`compact` key=solidify_on_done value=True *

*`val_txt` = > str value=`obs_val` *
*`val_len` = > len value=`val_txt` *
1. `obs_val`
  *`compact` = > json.set map=`compact` key=has_value value=True *
  *`compact` = > json.set map=`compact` key=value_len value=`val_len` *
  1. `val_len` > 200
    *`_` = 1*
  2. *
    *`compact` = > json.set map=`compact` key=value value=`obs_val` *
2. *
  *`compact` = > json.set map=`compact` key=has_value value=False *

*`slots_brief` = > json.parse text=[] *
1. `obs_slots`
  - [`slot`](`obs_slots`)
    *`sk` = > json.get value=`slot` key=key *
    *`sl` = > json.get value=`slot` key=line *
    *`sb` = > json.get value=`slot` key=body *
    *`sb_txt` = > str value=`sb` *
    *`sb_len` = > len value=`sb_txt` *
    *`brief` = > json.parse text={"key":""} *
    *`brief` = > json.set map=`brief` key=key value=`sk` *
    *`brief` = > json.set map=`brief` key=line value=`sl` *
    *`brief` = > json.set map=`brief` key=body_len value=`sb_len` *
    *`slots_brief` = > json.append list=`slots_brief` item=`brief` *
2. *
  *`_` = 1*
*`compact` = > json.set map=`compact` key=slots value=`slots_brief` *

*`last_ok` = > json.get value=`observation` key=last_ok *
*`last_error` = > json.get value=`observation` key=last_error *
1. `last_ok`
  *`ok_txt` = > str value=`last_ok` *
  *`ok_len` = > len value=`ok_txt` *
  *`compact` = > json.set map=`compact` key=last_ok_len value=`ok_len` *
  1. `ok_len` > 200
    *`compact` = > json.set map=`compact` key=last_ok_omitted value=True *
  2. *
    *`compact` = > json.set map=`compact` key=last_ok value=`last_ok` *
2. *
  *`_` = 1*
1. `last_error`
  *`err_txt` = > str value=`last_error` *
  *`err_len` = > len value=`err_txt` *
  1. `err_len` > 200
    *`compact` = > json.set map=`compact` key=last_error_omitted value=True *
    *`compact` = > json.set map=`compact` key=last_error_len value=`err_len` *
  2. *
    *`compact` = > json.set map=`compact` key=last_error value=`last_error` *
2. *
  *`_` = 1*

*`obs_s` = > json.stringify value=`compact` *
*`goal_s` = > json.stringify value=`goal` *
*`esc` = > json.parse text={"a":"\n\n--- standing ---\n","b":"\n\n--- goal ---\n","c":"\n\n--- workbook observation ---\n","d":"\n\n--- marqdo skill ---\n","e_rev":"\n\n--- how to act ---\nYou are a Marqdo agent-development master.\nPriority order (STOP RULES — follow top-down):\n1) SUCCESS STOP: if exit_code is 0 AND has_value is true, reply DECISION: DONE with a one-line SUMMARY immediately. Do NOT invent FIND/REPLACE. Runtime agent_workbook_solidify removes worker.step on DONE — has_worker_step true is NOT a reason to CONTINUE.\n2) FAILURE ONLY: CONTINUE with short FIND/REPLACE (<20 lines) only when exit_code is non-zero, has_value is false, or the returned value clearly fails the goal. Prefer structure edits; never paste itineraries/schedules/observation prose into REPLACE.\n3) Code-first for fixed answers (e.g. pong): solidify via DONE (runtime) or a tiny structural patch — not by rewriting child prose.\n4) If exploring an alternate path, try a meaningfully DIFFERENT structure than prior attempts. Do not merely re-run the same step.\n5) Prefer multiple narrow agents over one giant standing when roles differ.\nNever rewrite the whole .mq.md file.\nThe moment success is observed, emit DONE and stop — do not keep being helpful.\nReply with EXACTLY one protocol:\nDECISION: DONE\nSUMMARY: <one line>\nOR\nDECISION: CONTINUE\nPATCH:\n<<<\nFIND\n<exact old snippet>\n===\nREPLACE\n<new snippet>\n>>>\n(You may repeat <<< blocks.)\n","e_dec":"\n\n--- how to act ---\nYou are a Marqdo agent-development master. PRE-RUN DECOMPOSE: the workbook has NOT been executed yet.\nPriority order:\n1) If the skeleton is fine to execute as-is, reply DECISION: RUN (zero patches OK). Prefer RUN over inventing user-facing prose.\n2) Code first: if the goal is a fixed answer, PATCH to a **return** (no worker.step / .单步), then DECISION: RUN or DECISION: DONE with SUMMARY after solidify.\n3) To reshape before the first run, DECISION: CONTINUE with short FIND/REPLACE (<20 lines); runtime applies them then spawns.\n4) Prefer multiple narrow agents when roles differ. Never paste long itineraries into REPLACE.\nNever rewrite the whole .mq.md file.\nReply with EXACTLY one protocol:\nDECISION: RUN\nOR\nDECISION: DONE\nSUMMARY: <one line>\nOR\nDECISION: CONTINUE\nPATCH:\n<<<\nFIND\n<exact old snippet>\n===\nREPLACE\n<new snippet>\n>>>\n(You may repeat <<< blocks.)\n","f":"\n\n--- explore attempt ---\n"} *
*`a` = > json.get value=`esc` key=a *
*`b` = > json.get value=`esc` key=b *
*`c` = > json.get value=`esc` key=c *
*`d` = > json.get value=`esc` key=d *
*`f` = > json.get value=`esc` key=f *
1. `phase` == decompose
  *`e` = > json.get value=`esc` key=e_dec *
  *`p` = You decompose the workbook before the first run. *
2. *
  *`e` = > json.get value=`esc` key=e_rev *
  *`p` = You develop and revise Marqdo workbooks with surgical patches. *
*`p` = `p` + `a` + `up` + `b` + `goal_s` + `c` + `obs_s` + `d` + `skill` + `e` *
1. `explore_attempt`
  *`p` = `p` + `f` *
  *`p` = `p` + Attempt *
  *`p` = `p` + `explore_attempt` *
  *`p` = `p` + of *
  *`p` = `p` + `explore_n` *
  *`p` = `p` + . Try a different path; prefer code when the answer is fixed. *
2. *
  *`_` = 1*
**`p`**

---

## plan_echo_decompose
    + `workbook`
    + `stream`=False
    + `echo`=False

TTY marker for pre-run parent decompose (before any child spawn).

1. `stream`
  1. `echo`
    > print text=plan:decompose
  2. *
    *`_` = 1*
2. *
  *`_` = 1*

****

## plan_echo_await
    + `round`
    + `workbook`
    + `stream`=False
    + `echo`=False

TTY-only marker before `await_workbook` so `stream+echo` is not silent during the (often long) child run.

1. `stream`
  1. `echo`
    > print text=plan:await
    > print text=`round`
  2. *
    *`_` = 1*
2. *
  *`_` = 1*

****

## workbook_has_worker_step
    + `path`

True when source still contains `worker.step` or `.单步`.

*`src` = > fs.read_text path=`path` *
*`a` = > split value=`src` sep=worker.step *
*`na` = > len value=`a` *
*`b` = > split value=`src` sep=.单步 *
*`nb` = > len value=`b` *
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

Append a `round` event when `stream=True`. Optional `result` is the child `# main` return (for view Child cards). Optional `echo` prints a short TTY marker (not the full workbook path).

1. `stream`
  *`ev` = > json.parse text={"type":"round"} *
  *`ev` = > json.set map=`ev` key=round value=`round` *
  *`ev` = > json.set map=`ev` key=workbook value=`workbook` *
  *`ev` = > json.set map=`ev` key=exit_code value=`exit_code` *
  1. `result`
    *`ev` = > json.set map=`ev` key=result value=`result` *
  2. *
    *`_` = 1*
  *`events` = > json.append list=`events` item=`ev` *
  > sys.stream_publish event=`ev`
  1. `echo`
    > print text=plan:round
  2. *
    *`_` = 1*
2. *
  *`_` = 1*

**`events`**

## plan_append_decision
    + `events`
    + `decision`
    + `stream`=False
    + `summary`=None

1. `stream`
  *`ev` = > json.parse text={"type":"decision"} *
  *`ev` = > json.set map=`ev` key=decision value=`decision` *
  1. `summary`
    *`ev` = > json.set map=`ev` key=summary value=`summary` *
  2. *
    *`_` = 1*
  *`events` = > json.append list=`events` item=`ev` *
  > sys.stream_publish event=`ev`
2. *
  *`_` = 1*

**`events`**

## plan_merge_deltas
    + `events`
    + `from`
    + `stream`=False

Bubble parent `complete stream=True` reasoning/deltas (and errors) into the plan event list. Skip nested `done` so plan owns the final `done`.

1. `stream`
  - [`ev`](`from`)
    *`t` = > json.get value=`ev` key=type *
    1. `t` == delta
      *`events` = > json.append list=`events` item=`ev` *
    2. `t` == reasoning
      *`events` = > json.append list=`events` item=`ev` *
    3. `t` == error
      *`events` = > json.append list=`events` item=`ev` *
    4. *
      *`_` = 1*
2. *
  *`_` = 1*

**`events`**

## plan_finish_stream
    + `out`
    + `events`
    + `stream`=False
    + `trace`=False
    + `result`=None

When `stream=True`, append `done` and attach `events` on the result map. Optional `trace=True` writes the event list to writeback slot `trace`.

1. `stream`
  *`ev` = > json.parse text={"type":"done"} *
  1. `result`
    *`ev` = > json.set map=`ev` key=result value=`result` *
  2. *
    *`_` = 1*
  *`events` = > json.append list=`events` item=`ev` *
  > sys.stream_publish event=`ev`
  *`out` = > json.set map=`out` key=events value=`events` *
2. *
  *`_` = 1*

1. `trace`
  *`body` = > json.stringify value=`events` *
  > writeback.record value=`body` key=trace
2. *
  *`_` = 1*

**`out`**

---

# agent
    + `model`
    + `tools`
    + `standing`

Load ABI v2 agent plugin once (session bag + context queries).

*`p` = > plugin.native_path name=agent *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)
  > sys.exit code=1

*`id` = > agent_alloc *
*`h` = > json.parse text={"_type":"agent"} *
*`h` = > json.set map=`h` key=id value=`id` *
*`h` = > json.set map=`h` key=model value=`model` *
*`h` = > json.set map=`h` key=tools value=`tools` *

1. `standing`
  *`h` = > json.set map=`h` key=standing value=`standing` *
2. *
  *`_` = 1*

**`h`**

## clear_history

Clear the plugin session bag for this agent id.

*`id` = > json.get value=`self` key=id *
> agent_history_clear id=`id`
****

## step
    + `task`
    + `writeback`=True
    + `stream`=False
    + `echo`=False

One atomic turn: context → LLM → optional tool via subtask. Returns a **map** (`status`, `task`, `decision`, and on success `result` plus optional `tool` / `tool_result`; on failure `error`). By default (`writeback=True`) persists that map under named slots `ok` / `error` at the call site; pass `writeback=False` to skip.

With `stream=True`, the model call uses SSE; `echo=True` prints delta text to stdout as it arrives. The returned map is unchanged (final `result` string).

*`ctx` = > build_step_context agent=`self` task=`task` *
*`id` = > json.get value=`self` key=id *
*`model` = > json.get value=`self` key=model *
*`tools` = > json.get value=`self` key=tools *

*`user_turn` = > json.parse text={"role":"user"} *
*`user_turn` = > json.set map=`user_turn` key=content value=`task` *
> agent_history_append id=`id` item=`user_turn`

1. `stream`
  *`evs` = > `model`.complete prompt=`ctx` stream=True echo=`echo` *
  *`reply` = > llm.stream_result events=`evs` *
2. *
  *`reply` = > `model`.complete prompt=`ctx` *

*`reply` = > trim value=`reply` *
*`decision` = `reply`*
*`tool_name` = > extract_tool_name reply=`reply` *
*`out` = > json.parse text={"status":"ok"} *
*`out` = > json.set map=`out` key=task value=`task` *
*`out` = > json.set map=`out` key=decision value=`decision` *

1. `tool_name`
  *`tool_out` = > run_tool tools=`tools` name=`tool_name` *
  *`tool_s` = > json.stringify value=`tool_out` *
  *`deny` = > split value=`tool_s` sep=Tool not allowed *
  *`deny_n` = > len value=`deny` *
  1. `deny_n` > 1
    *`out` = > json.set map=`out` key=status value=error *
    *`out` = > json.set map=`out` key=error value=`tool_s` *
    *`reply` = `tool_s`*
  2. *
    *`task_s` = > json.stringify value=`task` *
    *`fp` = The user asked: *
    *`fp` = `fp` + `task_s` *
    *`fp` = `fp` + . Tool *
    *`fp` = `fp` + `tool_name` *
    *`fp` = `fp` + ran via subtask and returned: *
    *`fp` = `fp` + `tool_s` *
    *`fp` = `fp` + . Reply to the user briefly. *
    1. `stream`
      *`evs2` = > `model`.complete prompt=`fp` stream=True echo=`echo` *
      *`reply` = > llm.stream_result events=`evs2` *
    2. *
      *`reply` = > `model`.complete prompt=`fp` *
    *`out` = > json.set map=`out` key=tool value=`tool_name` *
    *`out` = > json.set map=`out` key=tool_result value=`tool_s` *
    *`out` = > json.set map=`out` key=result value=`reply` *
2. *
  *`out` = > json.set map=`out` key=result value=`reply` *

*`as_turn` = > json.parse text={"role":"assistant"} *
*`as_turn` = > json.set map=`as_turn` key=content value=`reply` *
> agent_history_append id=`id` item=`as_turn`

1. `writeback`
  *`body` = > json.stringify value=`out` *
  *`st` = > json.get value=`out` key=status *
  1. `st` == ok
    > writeback.record value=`body` key=ok
  2. *
    > writeback.record value=`body` key=error
2. *
  *`_` = 1*

**`out`**

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
    + `promote`=True
    + `kb_dir`=.marqdo/agent-kb
    + `improve_every`=3
    + `explore_n`=3
    + `stream`=False
    + `echo`=False
    + `trace`=False

Multi-step with OKF agent-kb. Default workbook is `kb_dir/resources/<slug>.mq.md`. While task file count `< explore_n` and skill is not llm_free, force a new explore variant under `kb_dir/explore/<slug>/`. Code-first: llm_free hits skip parent LLM. File children return via `# main`; `plan` exposes that as `result`.

Non-hit path runs parent **decompose** before the first child spawn (`DECISION: RUN` / `CONTINUE`+patch / solidified `DONE`). Then `await` → revise loop. With `stream=True`, emit parent `delta` / `decision` / `round` / `done` on `events`. `echo=True` prints `plan:decompose` / `plan:await` / deltas. `trace=True` writes events to writeback slot `trace`. Quiet child subtasks stay quiet.

*`tools` = > json.get value=`self` key=tools *
*`cache` = miss*
*`path` = None*
*`improve` = None*
*`explore` = None*
*`explore_attempt` = None*
*`skel_kind` = `skeleton`*
*`events` = > json.parse text=[] *

*`tf` = > agent_kb_task_files kb_dir=`kb_dir` goal=`goal` tools=`tools` *
*`nfiles` = > json.get value=`tf` key=count *

1. `force`
  *`_` = 1*
2. `optimize`
  *`_` = 1*
3. `reuse`
  *`hit` = > agent_kb_lookup kb_dir=`kb_dir` goal=`goal` tools=`tools` *
  1. `hit`
    *`match_kind` = > json.get value=`hit` key=match *
    1. `match_kind` == alias
      *`cache_label` = soft-hit*
    2. *
      *`cache_label` = hit*
    *`lf` = > json.get value=`hit` key=llm_free *
    1. `lf`
      *`path` = > json.get value=`hit` key=resource *
      *`aw` = > await_workbook path=`path` *
      *`code` = > json.get value=`aw` key=code *
      *`child_val` = > json.get value=`aw` key=value *
      *`last_obs` = > json.get value=`aw` key=observation *
      1. `code` == 0
        > agent_kb_record_hit kb_dir=`kb_dir` goal=`goal` tools=`tools` improve_every=`improve_every`
        *`cache` = `cache_label`*
        *`out` = > json.parse text={"status":"ok"} *
        *`out` = > json.set map=`out` key=status value=ok *
        *`out` = > json.set map=`out` key=goal value=`goal` *
        *`out` = > json.set map=`out` key=workbook value=`path` *
        *`out` = > json.set map=`out` key=rounds value=1 *
        *`out` = > json.set map=`out` key=cache value=`cache_label` *
        *`sk` = > json.get value=`hit` key=skill *
        *`out` = > json.set map=`out` key=skill value=`sk` *
        *`st` = > json.get value=`hit` key=status *
        *`out` = > json.set map=`out` key=skill_status value=`st` *
        *`sum` = OKF llm_free skill hit; spawned resource *
        *`out` = > json.set map=`out` key=summary value=`sum` *
        *`out` = > json.set map=`out` key=observation value=`last_obs` *
        *`out` = > json.set map=`out` key=result value=`child_val` *
        *`events` = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo` *
        1. `writeback`
          *`body` = > json.stringify value=`out` *
          > writeback.record value=`body` key=ok
        2. *
          *`_` = 1*
        *`out` = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`child_val` *
        **`out`**
      2. *
        *`path` = None*
    2. `nfiles` < `explore_n`
      *`explore` = 1*
      *`cache` = explore*
      *`path` = None*
      *`explore_attempt` = `nfiles` + 1*
      1. `explore_attempt` == 2
        *`skel_kind` = dual*
      2. *
        *`skel_kind` = `skeleton`*
    3. *
      *`path` = > json.get value=`hit` key=resource *
      *`aw` = > await_workbook path=`path` *
      *`code` = > json.get value=`aw` key=code *
      *`child_val` = > json.get value=`aw` key=value *
      *`last_obs` = > json.get value=`aw` key=observation *
      1. `code` == 0
        *`rec` = > agent_kb_record_hit kb_dir=`kb_dir` goal=`goal` tools=`tools` improve_every=`improve_every` *
        *`due` = > json.get value=`rec` key=improve_due *
        1. `due`
          *`improve` = 1*
          *`cache` = improve*
        2. *
          *`cache` = `cache_label`*
          *`out` = > json.parse text={"status":"ok"} *
          *`out` = > json.set map=`out` key=status value=ok *
          *`out` = > json.set map=`out` key=goal value=`goal` *
          *`out` = > json.set map=`out` key=workbook value=`path` *
          *`out` = > json.set map=`out` key=rounds value=1 *
          *`out` = > json.set map=`out` key=cache value=`cache_label` *
          *`sk` = > json.get value=`hit` key=skill *
          *`out` = > json.set map=`out` key=skill value=`sk` *
          *`st` = > json.get value=`hit` key=status *
          *`out` = > json.set map=`out` key=skill_status value=`st` *
          *`sum` = OKF skill hit; spawned resource *
          *`out` = > json.set map=`out` key=summary value=`sum` *
          *`out` = > json.set map=`out` key=observation value=`last_obs` *
          *`out` = > json.set map=`out` key=result value=`child_val` *
          *`events` = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo` *
          1. `writeback`
            *`body` = > json.stringify value=`out` *
            > writeback.record value=`body` key=ok
          2. *
            *`_` = 1*
          *`out` = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`child_val` *
          **`out`**
      2. *
        *`path` = None*
  2. *
    1. `nfiles` < `explore_n`
      *`explore` = 1*
      *`cache` = explore*
      *`explore_attempt` = `nfiles` + 1*
      1. `explore_attempt` == 2
        *`skel_kind` = dual*
      2. *
        *`skel_kind` = `skeleton`*
    2. *
      *`_` = 1*
4. *
  1. `nfiles` < `explore_n`
    *`explore` = 1*
    *`cache` = explore*
    *`explore_attempt` = `nfiles` + 1*
  2. *
    *`_` = 1*

*`skel` = > render_workbook_skeleton goal=`goal` skeleton=`skel_kind` *

1. `workbook`
  *`path` = `workbook`*
  *`ex` = > fs.exists path=`path` *
  1. `ex`
    *`_` = 1*
  2. *
    > fs.write_text path=`path` text=`skel`
2. *
  1. `path`
    *`_` = 1*
  2. *
    *`slug` = > agent_goal_slug goal=`goal` *
    1. `workbook_dir`
      > fs.make_dir path=`workbook_dir`
      *`ts` = > time.now_ms *
      *`parts` = > json.parse text={"a":"/workbook-","b":"-","c":".mq.md"} *
      *`a` = > json.get value=`parts` key=a *
      *`b` = > json.get value=`parts` key=b *
      *`c` = > json.get value=`parts` key=c *
      *`path` = `workbook_dir` + `a` + `slug` + `b` + `ts` + `c` *
    2. `explore`
      *`parts` = > json.parse text={"a":"/explore/","b":"/","c":".mq.md"} *
      *`a` = > json.get value=`parts` key=a *
      *`b` = > json.get value=`parts` key=b *
      *`c` = > json.get value=`parts` key=c *
      *`path` = `kb_dir` + `a` + `slug` + `b` + `explore_attempt` + `c` *
    3. *
      *`parts` = > json.parse text={"a":"/resources/","b":".mq.md"} *
      *`a` = > json.get value=`parts` key=a *
      *`b` = > json.get value=`parts` key=b *
      *`path` = `kb_dir` + `a` + `slug` + `b` *
    > fs.write_text path=`path` text=`skel`

1. `confirm`
  *`out` = > json.parse text={"status":"pending"} *
  *`out` = > json.set map=`out` key=goal value=`goal` *
  *`out` = > json.set map=`out` key=workbook value=`path` *
  *`out` = > json.set map=`out` key=summary value=workbook created; confirm to run *
  *`out` = > json.set map=`out` key=cache value=bypass *
  1. `writeback`
    *`body` = > json.stringify value=`out` *
    > writeback.record value=`body` key=ok
  2. *
    *`_` = 1*
  *`out` = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` *
  **`out`**
2. *
  *`_` = 1*

*`model` = > json.get value=`self` key=model *
*`round` = 0*
*`last_obs` = None*
*`last_reply` = None*
*`child_val` = None*
*`done` = None*
*`summary` = None*
*`status` = error*
*`err` = max_rounds exhausted *
*`skip_loop` = None*

> plan_echo_decompose workbook=`path` stream=`stream` echo=`echo`
*`last_obs` = > inspect_workbook path=`path` *
*`ctx` = > build_plan_context agent=`self` goal=`goal` observation=`last_obs` explore_attempt=`explore_attempt` explore_n=`explore_n` phase=decompose *
1. `stream`
  *`evs` = > `model`.complete prompt=`ctx` stream=True echo=`echo` *
  *`last_reply` = > llm.stream_result events=`evs` *
  *`events` = > plan_merge_deltas events=`events` from=`evs` stream=`stream` *
2. *
  *`last_reply` = > `model`.complete prompt=`ctx` *
*`last_reply` = > trim value=`last_reply` *
*`dec` = > extract_plan_decision reply=`last_reply` *
*`events` = > plan_append_decision events=`events` decision=`dec` stream=`stream` *

1. `dec` == CONTINUE
  *`n` = > fs.apply_patch_blocks path=`path` text=`last_reply` *
  *`_` = 1*
2. `dec` == RUN
  *`_` = 1*
3. `dec` == DONE
  *`has_step` = > workbook_has_worker_step path=`path` *
  1. `has_step`
    *`_` = 1*
  2. *
    > plan_echo_await round=1 workbook=`path` stream=`stream` echo=`echo`
    *`aw` = > await_workbook path=`path` *
    *`code` = > json.get value=`aw` key=code *
    *`child_val` = > json.get value=`aw` key=value *
    *`last_obs` = > json.get value=`aw` key=observation *
    *`round` = 1*
    *`events` = > plan_append_round events=`events` round=1 workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo` *
    1. `code` == 0
      *`done` = 1*
      *`status` = ok*
      *`summary` = > extract_plan_summary reply=`last_reply` *
      *`err` = None*
      *`skip_loop` = 1*
    2. *
      *`_` = 1*
4. *
  *`done` = 1*
  *`status` = error*
  *`err` = unrecognized plan decision *
  *`summary` = `last_reply`*
  *`skip_loop` = 1*

1. `skip_loop`
  *`left` = 0*
2. `improve`
  *`left` = 1*
3. *
  *`left` = `max_rounds`*

- `left` > 0
  1. `done`
    *`left` = 0*
  2. *
    *`round` = `round` + 1*
    > plan_echo_await round=`round` workbook=`path` stream=`stream` echo=`echo`
    *`aw` = > await_workbook path=`path` *
    *`code` = > json.get value=`aw` key=code *
    *`child_val` = > json.get value=`aw` key=value *
    *`last_obs` = > json.get value=`aw` key=observation *
    *`events` = > plan_append_round events=`events` round=`round` workbook=`path` exit_code=`code` result=`child_val` stream=`stream` echo=`echo` *

Deterministic success stop (loop engineering): when the child already returned a value, do not ask the parent LLM to invent solidify patches — DONE + runtime solidify.

    1. `code` == 0
      1. `child_val`
        1. `improve`
          *`auto_done` = None*
        2. *
          *`auto_done` = 1*
      2. *
        *`auto_done` = None*
    2. *
      *`auto_done` = None*

    1. `auto_done`
      > agent_workbook_solidify path=`path` observation=`last_obs`
      *`done` = 1*
      *`status` = ok*
      *`summary` = child returned value; solidified *
      *`err` = None*
      *`dec` = DONE*
      *`events` = > plan_append_decision events=`events` decision=`dec` stream=`stream` summary=`summary` *
      *`left` = 0*
    2. *
      *`ctx` = > build_plan_context agent=`self` goal=`goal` observation=`last_obs` explore_attempt=`explore_attempt` explore_n=`explore_n` phase=revise *
      1. `stream`
        *`evs` = > `model`.complete prompt=`ctx` stream=True echo=`echo` *
        *`last_reply` = > llm.stream_result events=`evs` *
        *`events` = > plan_merge_deltas events=`events` from=`evs` stream=`stream` *
      2. *
        *`last_reply` = > `model`.complete prompt=`ctx` *
      *`last_reply` = > trim value=`last_reply` *
      *`dec` = > extract_plan_decision reply=`last_reply` *
      *`events` = > plan_append_decision events=`events` decision=`dec` stream=`stream` *
      1. `dec` == DONE
        > agent_workbook_solidify path=`path` observation=`last_obs`
        *`done` = 1*
        *`status` = ok*
        *`summary` = > extract_plan_summary reply=`last_reply` *
        *`err` = None*
        *`left` = 0*
      2. `dec` == CONTINUE
        *`n` = > fs.apply_patch_blocks path=`path` text=`last_reply` *
        1. `n`
          *`left` = `left` - 1*
        2. *
          *`done` = 1*
          *`status` = error*
          *`err` = no patches applied *
          *`summary` = > extract_plan_summary reply=`last_reply` *
          *`left` = 0*
      3. `dec` == RUN
        *`left` = `left` - 1*
      4. *
        *`done` = 1*
        *`status` = error*
        *`err` = unrecognized plan decision *
        *`summary` = `last_reply`*
        *`left` = 0*

1. `status` == ok
  1. `promote`
    *`prom` = > agent_kb_promote kb_dir=`kb_dir` goal=`goal` workbook=`path` tools=`tools` *
    *`cache` = refreshed*
  2. *
    *`cache` = miss*
2. *
  *`_` = 1*

*`out` = > json.parse text={"status":"ok"} *
*`out` = > json.set map=`out` key=status value=`status` *
*`out` = > json.set map=`out` key=goal value=`goal` *
*`out` = > json.set map=`out` key=workbook value=`path` *
*`out` = > json.set map=`out` key=rounds value=`round` *
*`out` = > json.set map=`out` key=cache value=`cache` *
1. `summary`
  *`out` = > json.set map=`out` key=summary value=`summary` *
2. *
  *`_` = 1*

1. `err`
  *`out` = > json.set map=`out` key=error value=`err` *
2. *
  *`_` = 1*

1. `last_obs`
  *`out` = > json.set map=`out` key=observation value=`last_obs` *
2. *
  *`_` = 1*

1. `child_val`
  *`out` = > json.set map=`out` key=result value=`child_val` *
  *`plan_result` = `child_val`*
2. `summary`
  *`out` = > json.set map=`out` key=result value=`summary` *
  *`plan_result` = `summary`*
3. *
  *`plan_result` = None*

1. `writeback`
  *`body` = > json.stringify value=`out` *
  1. `status` == ok
    > writeback.record value=`body` key=ok
  2. *
    > writeback.record value=`body` key=error
2. *
  *`_` = 1*

*`out` = > plan_finish_stream out=`out` events=`events` stream=`stream` trace=`trace` result=`plan_result` *
**`out`**
