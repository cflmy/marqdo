---
markup: occlusion
title: ext/ai/agent
description: Document-driven agent v2 — step / plan / run; Skill Compilation via ABI plugin (episodes → learn → llm_free).
import:
  llm: ext/ai/llm.mq.md
  json: lib/json.mq.md
  sys: lib/sys.mq.md
  fs: lib/fs.mq.md
  time: lib/time.mq.md
  text: lib/text.mq.md
  wb: lib/writeback.mq.md
  subtask: lib/subtask.mq.md
  plugin: lib/plugin.mq.md
  sandbox: ext/ai/sandbox.mq.md
  table: lib/table.mq.md
---

This file owns agent orchestration: call-site context, explicit tools, routing, workbooks, observations and continuation. Start with [the AI workflows](index.mq.md) for package choices and native dependencies. Use step for one turn, run for reuse and learning, and plan for a bounded workbook lifecycle. Internal helpers are documented for development but are not automatically model-callable tools. Changes must preserve source authority, effect deduplication and the location of writeback.

## build_step_context

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| agent | any | required | Agent configuration or receiver created by this package |
| task | any | required | Structured task or text supplied to the agent |
| source_depth | any | "default" | Source-context depth; default follows the documented context budget |
| skill_depth | any | "default" | Skill-context depth; default follows the documented context budget |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Assemble a readable prompt: standing, task, call site, tools, budgeted source + skill, and the act protocol. The runbook `.mq.md` is ground truth (code as documentation). Default budgets keep long files from exploding the window; emit `READ:source` / `READ:skill` (Chinese `读取：`) to deepen, then answer or `CALL:`. |

Use `agent`, `task`, `source_depth`="default", `skill_depth`="default" as the inputs for this operation.

Assemble a readable prompt: standing, task, call site, tools, budgeted source + skill, and the act protocol.
The runbook `.mq.md` is ground truth (code as documentation). Default budgets keep long files from exploding the window; emit `READ:source` / `READ:skill` (Chinese `读取：`) to deepen, then answer or `CALL:`.

The sections have different authority. Standing text sets the agent's role, task text states the current request, and the call site identifies the exact source location that requested it. Source and skill excerpts are bounded evidence for that location. They are not a replacement for the author's runbook. The returned prompt is plain text; constructing it does not call a model or execute any listed tool.

Tool descriptions come from the supplied whitelist. The act instructions restrict CALL names to that section, so an internal helper mentioned elsewhere is not automatically callable. A truncated excerpt can request a deeper source or skill read through the explicit READ protocol. The caller must enforce its read budget and tool permissions; these instructions alone are not an authorization mechanism.

We retain **`standing` = [Read the standing field](json.get(value=`agent`, key="standing"))**.
1. `standing`
  We retain **up = standing**.
2. *
  We retain **up = None**.

We retain **`tools` = [Read the tools field](json.get(value=`agent`, key="tools"))**.
We retain **`tools_s` = [Format tools](agent_format_tools(tools=`tools`))**.
We retain **`src` = [Source brief](source_brief(depth=`source_depth`))**.
We retain **`site` = [Call site](agent_call_site())**.
We retain **`site_s` = [Encode the structured value as JSON text](json.stringify(value=`site`))**.
We retain **`skill` = [Skill brief](skill_brief(depth=`skill_depth`))**.
We retain **`task_s` = [Encode the structured value as JSON text](json.stringify(value=`task`))**.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_g_lines` =

| line |
|---|
| "" |
| "" |
| "--- how to act ---" |
| "Call ONLY names listed under --- tools --- (whitelist). If that section is empty or says none, do NOT emit CALL / 调用 — reply with the final answer only." |
| "Never CALL internal framework helpers (build_step_context, plan, inspect_workbook, etc.)." |
| "Context is budgeted: if you see a [truncated …] marker, you may first emit exactly one line READ:source or READ:skill (Chinese: 读取：source\|skill) to deepen, then you will be re-prompted." |
| "When calling an allowed tool, reply with exactly one line: CALL:<name>" |
| "(Chinese ok: 调用：<name>)" |
| "Otherwise reply with the final answer only." |
| "If a capability cannot truly be wired, do NOT fake it silently and never claim it works: write an occlusion with an explicit status, e.g. [按关键词检索，接口未接，待开发](None). Stubs must run without error and must state their return value (None is acceptable). Before finishing, check your own declarations with labels.search." |
| "Tools run via Marqdo subtask (spawn fn / wait)." |
| "" |

Join exactly those lines into **_literal_esc_g_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_g_lines`, sep="\n"))**.

The following record names each literal field without a JSON construction chain.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "a" | "\n\n--- standing ---\n" |
| 2 | "b" | "\n\n--- task ---\n" |
| 3 | "c" | "\n\n--- tools ---\n" |
| 4 | "d" | "\n\n--- call site ---\n" |
| 5 | "e" | "\n\n--- source (.mq.md) ---\n" |
| 6 | "f" | "\n\n--- marqdo skill ---\n" |
| 7 | "g" | `_literal_esc_g_text` |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`a` = [Read the a field](json.get(value=`esc`, key="a"))**.
We retain **`b` = [Read the b field](json.get(value=`esc`, key="b"))**.
We retain **`c` = [Read the c field](json.get(value=`esc`, key="c"))**.
We retain **`d` = [Read the d field](json.get(value=`esc`, key="d"))**.
We retain **`e` = [Read the e field](json.get(value=`esc`, key="e"))**.
We retain **`f` = [Read the f field](json.get(value=`esc`, key="f"))**.
We retain **`g` = [Read the g field](json.get(value=`esc`, key="g"))**.

We retain **p = "You are a Marqdo agent. Code is documentation: this runbook source and call site are your ground truth — not a hidden system prompt. When you cannot wire a capability, declare it in the description (待开发 / 模拟) instead of faking success."**.
We retain **p = p + a + up + b + task_s + d + site_s + c + tools_s + e + src + f + skill + g**.
Return *p*.

---

## model_turn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | required | Model resource or provider model identifier, as required by this operation |
| prompt | any | required | Prompt text or structured prompt expected by this adapter |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | One LLM turn via the semantic surface (`ask` / `stream`+`collect`). Returns `{text,llm_calls,tokens}` for episode metrics; transport failures set `error` + `llm_calls=0` (never abort). |

Use `model`, `prompt`, `stream`=False, `echo`=False, `stream_kind`="delta" as the inputs for this operation.

One LLM turn via the semantic surface (`ask` / `stream`+`collect`). Returns `{text,llm_calls,tokens}` for episode metrics; transport failures set `error` + `llm_calls=0` (never abort).
`stream_kind=plan` publishes protocol tokens onto the process trail (not the answer bubble).

1. `stream`
  We retain **`evs` = [Stream](> `model`.stream prompt=`prompt` echo=`echo` stream_kind=`stream_kind` )**.
  We retain **`text` = [Collect](llm.collect(events=`evs`))**.
  We retain **`tokens` = [Collect usage](llm.collect_usage(events=`evs`))**.
  1. `tokens` == None
    We retain **tokens = 0**.
  2. *
    **_ = 1**
  We retain **err = None**.
  We retain **calls = 1**.
  1. not `text`
    We retain **`e0` = [At](at(value=`evs`, index=0))**.
    We retain **`e0_s` = [Encode the structured value as JSON text](json.stringify(value=`e0`))**.
    We retain **`e0_map` = [Starts with](text.starts_with(text=`e0_s`, prefix="{"))**.
    1. `e0_map`
      We retain **`err` = [Read the message field](json.get(value=`e0`, key="message"))**.
      1. `err`
        We retain **calls = 0**.
    2. *
      **_ = 1**
2. *
  We retain **`r` = [Ask](> `model`.ask prompt=`prompt` )**.
  We retain **`r_s` = [Encode the structured value as JSON text](json.stringify(value=`r`))**.
  We retain **`r_map` = [Starts with](text.starts_with(text=`r_s`, prefix="{"))**.
  1. `r_map`
    We retain **`text` = [Read the text field](json.get(value=`r`, key="text"))**.
    We retain **`tokens` = [Read the tokens field](json.get(value=`r`, key="tokens"))**.
    We retain **`err` = [Read the error field](json.get(value=`r`, key="error"))**.
    We retain **`fin` = [Read the finish field](json.get(value=`r`, key="finish"))**.
    1. `tokens` == None
      We retain **`usage` = [Read the usage field](json.get(value=`r`, key="usage"))**.
      1. `usage`
        We retain **`tokens` = [Read the total_tokens field](json.get(value=`usage`, key="total_tokens"))**.
  2. *
    We retain **`text` = [Str](str(value=`r`))**.
    We retain **tokens = 0**.
    We retain **err = None**.
    We retain **fin = None**.
  1. `tokens` == None
    We retain **tokens = 0**.
  2. *
    **_ = 1**
  1. `err`
    We retain **calls = 0**.
  2. `fin` == "error"
    We retain **calls = 0**.
    1. not `err`
      We retain **err = "llm transport error"**.
  3. *
    We retain **calls = 1**.

`out` =

| text | llm_calls | tokens | error |
|------|-----------|--------|-------|
| `text` | `calls` | `tokens` | `err` |

Return *out*.

---

## dump_step_context

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| agent | any | required | Agent configuration or receiver created by this package |
| task | any | "" | Structured task or text supplied to the agent |
| source_depth | any | "default" | Source-context depth; default follows the documented context budget |
| skill_depth | any | "default" | Skill-context depth; default follows the documented context budget |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Inspect what `build_step_context` would send — no LLM. Returns a map: `prompt`, `call_site`, `prompt_chars`, `truncated` (true when budget markers appear). Use for harness / debugging the code-as-doc pipeline. |

Use `agent`, `task`="", `source_depth`="default", `skill_depth`="default" as the inputs for this operation.

Inspect what `build_step_context` would send — no LLM. Returns a map: `prompt`, `call_site`, `prompt_chars`, `truncated` (true when budget markers appear). Use for harness / debugging the code-as-doc pipeline.

We retain **`prompt` = [Build step context](build_step_context(agent=`agent`, task=`task`, source_depth=`source_depth`, skill_depth=`skill_depth`))**.
We retain **`site` = [Call site](agent_call_site())**.
We retain **`chars` = [Len](len(value=`prompt`))**.
We retain **`parts` = [Split](split(value=`prompt`, sep="[truncated"))**.
We retain **`n` = [Len](len(value=`parts`))**.
We retain **truncated = False**.
1. `n` > 1
  We retain **truncated = True**.
2. *
  **_ = 1**

`out` =

| prompt | call_site | prompt_chars | truncated |
|--------|-----------|--------------|-----------|
| `prompt` | `site` | `chars` | `truncated` |

Return *out*.

---

## extract_tool_name

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Find a human-readable call line: `CALL:<name>` or `调用：<name>` / `调用:<name>`. |

Use `reply` as the inputs for this operation.

Find a human-readable call line: `CALL:<name>` or `调用：<name>` / `调用:<name>`.

Keep the literal separator as **esc = [Keep the exact separator](table.put(in=None, at="nl", value="\n"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **found = None**.

- [line](遍历 `lines` 为 `line`)
  We retain **`t` = [Trim](trim(value=`line`))**.
  We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
  We retain **`n` = [Len](len(value=`parts`))**.
  1. `n` > 1
    We retain **`head` = [At](at(value=`parts`, index=0))**.
    We retain **`head` = [Trim](trim(value=`head`))**.
    We retain **`rest` = [At](at(value=`parts`, index=1))**.
    We retain **`rest` = [Trim](trim(value=`rest`))**.
    1. `head` == CALL
      We retain **found = rest**.
    2. `head` == 调用
      We retain **found = rest**.
    3. *
      **_ = 1**
  2. *
    **_ = 1**

Return *found*.

---

## run_tool

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| tools | any | required | Explicit tool whitelist or tool-specification records |
| name | any | required | Name registered or looked up by this operation |
| arguments | any | None | Named argument record for the selected registered tool |
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |
| logical_run_id | any | None | Stable logical execution identifier shared by retries |
| step_id | any | None | Identifier of the current workbook step |
| call_id | any | None | Identifier of this tool invocation |
| operation_id | any | None | Stable idempotent operation identifier |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Allowlist and ToolSpec prepare (M-A2), then invoke the runbook section via lib/subtask. Unauthorized or invalid-args calls return ToolResultV1 failure without spawning (0 executions). Mutating effects (write / execute / external_write) go through EffectLedger (M-A5a). State flow is prepared then started then spawn then committed, failed_no_effect, or unknown. Read tools skip the ledger. Matching idempotency_key and args_digest that already committed skips re-exec. |

Use `tools`, `name`, `arguments`=None, `ledger_dir`=".marqdo/agent-effects", `logical_run_id`=None, `step_id`=None, `call_id`=None, `operation_id`=None as the inputs for this operation.

Allowlist and ToolSpec prepare (M-A2), then invoke the runbook section via lib/subtask.
Unauthorized or invalid-args calls return ToolResultV1 failure without spawning (0 executions).
Mutating effects (write / execute / external_write) go through EffectLedger (M-A5a).
State flow is prepared then started then spawn then committed, failed_no_effect, or unknown.
Read tools skip the ledger. Matching idempotency_key and args_digest that already committed skips re-exec.

Preparation and execution are separate stages. A registered ToolSpec is validated before a child task is created. If preparation fails, its structured failure is returned immediately. The operation identifiers connect retries to the same logical effect; changing arguments under an old identity must not be treated as a successful replay. Supplying these identifiers is the orchestration caller's responsibility.

For effects that need a ledger, an existing committed record yields a receipt instead of another child execution. A previously started record without a completion record is uncertain: the code marks it unknown and requires reconciliation. It cannot infer that the external write failed merely because the process stopped. Read-only tools follow the shorter path and return the awaited child result.

The compatibility path when no native agent plugin is found is narrower: it checks the name whitelist and spawns the runbook section. It does not provide ToolSpec validation or durable effect deduplication. Applications that require those guarantees must prepare the native plugin and inspect the returned status and receipt, rather than treating every nonempty value as proof of completion.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  **_ = 1**

1. not `arguments`
  We retain **`arguments` = [Empty map](table.empty_map())**.
2. *
  **_ = 1**

We retain **prepared = None**.
We retain **needs = False**.
We retain **opid = None**.
We retain **digest = None**.
We retain **idem = None**.
We retain **effect = "read"**.
We retain **receipt = None**.
We retain **raw = None**.

1. not `p`
  We retain **`allowed` = [Tool allowed](agent_tool_allowed(tools=`tools`, name=`name`))**.
  1. not `allowed`
    We retain **msg = "Tool not allowed:"**.
    Return *msg + name*.
  2. *
    We retain **`id` = [Spawn](subtask.spawn(fn=`name`))**.
    Return *[Wait](subtask.wait(id=`id`))*.

We retain **`prepared` = [Tools prepare call](agent_tools_prepare_call(tool_id=`name`, tools=`tools`, arguments=`arguments`, call_id=`call_id`, operation_id=`operation_id`, logical_run_id=`logical_run_id`, step_id=`step_id`))**.
We retain **`pok` = [Read the ok field](json.get(value=`prepared`, key="ok"))**.
1. not `pok`
  Return *prepared*.
2. *
  **_ = 1**

We retain **`needs` = [Read the needs_ledger field](json.get(value=`prepared`, key="needs_ledger"))**.
We retain **`call` = [Read the call field](json.get(value=`prepared`, key="call"))**.
We retain **`opid` = [Read the operation_id field](json.get(value=`call`, key="operation_id"))**.
We retain **`digest` = [Read the args_digest field](json.get(value=`prepared`, key="args_digest"))**.
We retain **`idem` = [Read the idempotency_key field](json.get(value=`prepared`, key="idempotency_key"))**.
We retain **`tspec` = [Read the spec field](json.get(value=`prepared`, key="spec"))**.
We retain **`effect` = [Read the effect field](json.get(value=`tspec`, key="effect"))**.

1. `needs`
  We retain **`led` = [Effects prepare](agent_effects_prepare(operation_id=`opid`, idempotency_key=`idem`, args_digest=`digest`, effect_class=`effect`, logical_run_id=`logical_run_id`, step_id=`step_id`, ledger_dir=`ledger_dir`))**.
  We retain **`rec` = [Read the record field](json.get(value=`led`, key="record"))**.
  We retain **`st0` = [Read the state field](json.get(value=`rec`, key="state"))**.
  1. `st0` == "committed"
    We retain **`receipt` = [Read the result_ref field](json.get(value=`rec`, key="result_ref"))**.
    1. not `receipt`
      We retain **receipt = "effect:" + `opid`**.
    Return *[Tools wrap result](agent_tools_wrap_result(ok=True, status="succeeded", summary="deduped committed effect; skipped re-exec", structured_content=`rec`, effect_receipt_ref=`receipt`))*.
  2. *
    1. `st0` == "reconciled_committed"
      We retain **`receipt` = [Read the result_ref field](json.get(value=`rec`, key="result_ref"))**.
      1. not `receipt`
        We retain **receipt = "effect:" + `opid`**.
      Return *[Tools wrap result](agent_tools_wrap_result(ok=True, status="succeeded", summary="deduped reconciled effect; skipped re-exec", structured_content=`rec`, effect_receipt_ref=`receipt`))*.
    2. *
      1. `st0` == "unknown"
        We retain **receipt = "effect:" + `opid`**.
        Return *[Tools wrap result](agent_tools_wrap_result(ok=False, status="unknown_effect", summary="prior effect unknown; reconcile before retry", structured_content=`rec`, effect_receipt_ref=`receipt`))*.
      2. *
        1. `st0` == "started"
          We **[Effects transition](agent_effects_transition(operation_id=`opid`, state="unknown", note="retry saw started without commit; mark unknown", ledger_dir=`ledger_dir`))**.
          We retain **receipt = "effect:" + `opid`**.
          Return *[Tools wrap result](agent_tools_wrap_result(ok=False, status="unknown_effect", summary="interrupted after started; reconcile before retry", structured_content=`rec`, effect_receipt_ref=`receipt`))*.
        2. *
          1. `st0` == "needs_input"
            We retain **receipt = "effect:" + `opid`**.
            Return *[Tools wrap result](agent_tools_wrap_result(ok=False, status="needs_input", summary="effect needs_input; will not re-exec", structured_content=`rec`, effect_receipt_ref=`receipt`))*.
          2. *
            We **[Effects transition](agent_effects_transition(operation_id=`opid`, state="started", ledger_dir=`ledger_dir`))**.
            **_ = 1**
2. *
  **_ = 1**

We retain **`id` = [Spawn](subtask.spawn(fn=`name`))**.
We retain **`raw` = [Wait](subtask.wait(id=`id`))**.
1. `needs`
  We retain **`raw_s` = [Encode the structured value as JSON text](json.stringify(value=`raw`))**.
  1. not `raw`
    We **[Effects transition](agent_effects_transition(operation_id=`opid`, state="unknown", note="empty tool outcome; reconcile required", ledger_dir=`ledger_dir`))**.
    We retain **receipt = "effect:" + `opid`**.
    Return *[Tools wrap result](agent_tools_wrap_result(ok=False, status="unknown_effect", summary="tool outcome empty after started; ledger=unknown", structured_content=`raw`, effect_receipt_ref=`receipt`))*.
  2. *
    We retain **`is_err` = [Starts with](text.starts_with(text=`raw_s`, prefix="Tool not allowed"))**.
    1. `is_err`
      We **[Effects transition](agent_effects_transition(operation_id=`opid`, state="failed_no_effect", note="rejected before effect", ledger_dir=`ledger_dir`))**.
      We retain **receipt = "effect:" + `opid`**.
      Return *[Tools wrap result](agent_tools_wrap_result(ok=False, status="failed", summary=`raw_s`, effect_receipt_ref=`receipt`))*.
    2. *
      We retain **receipt = "effect:" + `opid`**.
      We **[Effects transition](agent_effects_transition(operation_id=`opid`, state="committed", result_ref=`receipt`, ledger_dir=`ledger_dir`))**.
      Return *[Tools wrap result](agent_tools_wrap_result(ok=True, status="succeeded", summary="tool executed; effect committed", structured_content=`raw`, effect_receipt_ref=`receipt`))*.
2. *
  Return *raw*.

## tools_register

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| spec | any | None | One structured tool or operation specification |
| specs | any | None | List of structured specifications |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Register ToolSpecV1 document(s). Unsupported schema keywords are refused. |

Use `spec`=None, `specs`=None as the inputs for this operation.

Register ToolSpecV1 document(s). Unsupported schema keywords are refused.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools register](agent_tools_register(spec=`spec`, specs=`specs`))*.

## tools_clear

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Clear the in-process ToolSpec registry (tests / reseeding). |

Clear the in-process ToolSpec registry (tests / reseeding).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools clear](agent_tools_clear())*.

## tools_search

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| query | any | "" | Text used to search the bounded catalog or tool registry |
| tools | any | None | Explicit tool whitelist or tool-specification records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Search authorized registered tools (summaries only). |

Use `query`="", `tools`=None as the inputs for this operation.

Search authorized registered tools (summaries only).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools search](agent_tools_search(query=`query`, tools=`tools`))*.

## tools_describe

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| tools | any | None | Explicit tool whitelist or tool-specification records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Return full ToolSpecV1 for an authorized tool id. |

Use `id`, `tools`=None as the inputs for this operation.

Return full ToolSpecV1 for an authorized tool id.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools describe](agent_tools_describe(id=`id`, tools=`tools`))*.

## tools_register_eng_defaults

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Seed the five engineering tool specs (`tool_find` … `tool_summarize`). |

Seed the five engineering tool specs (`tool_find` … `tool_summarize`).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools register eng defaults](agent_tools_register_eng_defaults())*.

## tools_prepare_call

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| tool_id | any | required | Registered tool identifier, not an arbitrary function name |
| arguments | any | None | Named argument record for the selected registered tool |
| tools | any | None | Explicit tool whitelist or tool-specification records |
| call_id | any | None | Identifier of this tool invocation |
| operation_id | any | None | Stable idempotent operation identifier |
| logical_run_id | any | None | Stable logical execution identifier shared by retries |
| step_id | any | None | Identifier of the current workbook step |
| idempotency_key | any | None | Key identifying the same logical effect across retries |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Validate schema + authorize; returns ToolCallV1 or ToolResultV1 error (no side effects). Also returns `args_digest` / `idempotency_key` / `needs_ledger` for EffectLedger wiring. |

Use `tool_id`, `arguments`=None, `tools`=None, `call_id`=None, `operation_id`=None, `logical_run_id`=None, `step_id`=None, `idempotency_key`=None as the inputs for this operation.

Validate schema + authorize; returns ToolCallV1 or ToolResultV1 error (no side effects).
Also returns `args_digest` / `idempotency_key` / `needs_ledger` for EffectLedger wiring.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Tools prepare call](agent_tools_prepare_call(tool_id=`tool_id`, arguments=`arguments`, tools=`tools`, call_id=`call_id`, operation_id=`operation_id`, logical_run_id=`logical_run_id`, step_id=`step_id`, idempotency_key=`idempotency_key`))*.

## output_build

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| status | any | "succeeded" | Domain status recorded in the structured result |
| text | any | "" | Text supplied to the transformation or adapter |
| format | any | "markdown" | Output representation selected by the caller |
| answer | any | None | Final answer text to project into the output |
| artifacts | any | None | Structured artifact references accompanying the result |
| citations | any | None | Source references accompanying the result |
| validation | any | None | Explicit validation evidence; not a self-reported success flag |
| finish_reason | any | None | Reason the operation stopped |
| recovery_options | any | None | Allowed recovery choices for the caller |
| channels | any | None | Structured output-channel records |
| allow_passed_without_report | any | False | Whether policy permits a passed status without a validation report |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Build an `AgentOutputV1` document (M-A4). `validation.status=passed` requires `report_ref`. |

Use `status`="succeeded", `text`="", `format`="markdown", `answer`=None, `artifacts`=None, `citations`=None, `validation`=None, `finish_reason`=None, `recovery_options`=None, `channels`=None, `allow_passed_without_report`=False as the inputs for this operation.

Build an `AgentOutputV1` document (M-A4). `validation.status=passed` requires `report_ref`.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Output build](agent_output_build(status=`status`, text=`text`, format=`format`, answer=`answer`, artifacts=`artifacts`, citations=`citations`, validation=`validation`, finish_reason=`finish_reason`, recovery_options=`recovery_options`, channels=`channels`, allow_passed_without_report=`allow_passed_without_report`))*.

## output_apply_policy

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| status | any | "succeeded" | Domain status recorded in the structured result |
| text | any | "" | Text supplied to the transformation or adapter |
| format | any | "markdown" | Output representation selected by the caller |
| answer | any | None | Final answer text to project into the output |
| artifacts | any | None | Structured artifact references accompanying the result |
| citations | any | None | Source references accompanying the result |
| validation | any | None | Explicit validation evidence; not a self-reported success flag |
| policy | any | None | Policy data governing this operation |
| schema | any | None | Structured schema used to validate or project output |
| schema_version | any | None | Version of the structured schema |
| finish_reason | any | None | Reason the operation stopped |
| recovery_options | any | None | Allowed recovery choices for the caller |
| channels | any | None | Structured output-channel records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply `ResponsePolicy` (detail / max_chars / max_output_tokens / artifact_allowed) to an AgentOutputV1 or raw text. |

Use `status`="succeeded", `text`="", `format`="markdown", `answer`=None, `artifacts`=None, `citations`=None, `validation`=None, `policy`=None, `schema`=None, `schema_version`=None, `finish_reason`=None, `recovery_options`=None, `channels`=None as the inputs for this operation.

Apply `ResponsePolicy` (detail / max_chars / max_output_tokens / artifact_allowed) to an AgentOutputV1 or raw text.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Output apply policy](agent_output_apply_policy(status=`status`, text=`text`, format=`format`, answer=`answer`, artifacts=`artifacts`, citations=`citations`, validation=`validation`, policy=`policy`, schema=`schema`, schema_version=`schema_version`, finish_reason=`finish_reason`, recovery_options=`recovery_options`, channels=`channels`))*.

## output_channel_event

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| channel | any | required | Output channel identifier |
| delta | any | "" | Incremental text or record update |
| resource_id | any | None | Referenced resource identifier |
| meta | any | None | Metadata associated with the structured item |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Split a stream delta by channel. Only `answer` sets `append_to_chat=true`. |

Use `channel`, `delta`="", `resource_id`=None, `meta`=None as the inputs for this operation.

Split a stream delta by channel. Only `answer` sets `append_to_chat=true`.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Output channel event](agent_output_channel_event(channel=`channel`, delta=`delta`, resource_id=`resource_id`, meta=`meta`))*.

## output_project_conclusion

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| document | any | "" | Document record being packed or projected |
| text | any | "" | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Deterministic chat projection from `# 结论` / `# conclusion` (F01 transition). Falls back to the first body paragraph. |

Use `document`="", `text`="" as the inputs for this operation.

Deterministic chat projection from `# 结论` / `# conclusion` (F01 transition). Falls back to the first body paragraph.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Output project conclusion](agent_output_project_conclusion(document=`document`, text=`text`))*.

## context_pack_build

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| items | any | None | Ordered context or output items |
| budget_tokens | any | 8000 | Maximum context token budget |
| output_reserve_pct | any | 20 | Percentage of the context budget reserved for output |
| compact_threshold_pct | any | 70 | Budget usage threshold triggering compaction |
| goal | any | None | Human task goal represented by this operation |
| constraints | any | None | Explicit task constraints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Assemble `ContextPackV2` with origin/trust/priority. Web/tool items cannot be `trust=trusted`. |

Use `items`=None, `budget_tokens`=8000, `output_reserve_pct`=20, `compact_threshold_pct`=70, `goal`=None, `constraints`=None as the inputs for this operation.

Assemble `ContextPackV2` with origin/trust/priority. Web/tool items cannot be `trust=trusted`.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Context pack build](agent_context_pack_build(items=`items`, budget_tokens=`budget_tokens`, output_reserve_pct=`output_reserve_pct`, compact_threshold_pct=`compact_threshold_pct`, goal=`goal`, constraints=`constraints`))*.

## context_pack_compact

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| items | any | None | Ordered context or output items |
| budget_tokens | any | 8000 | Maximum context token budget |
| output_reserve_pct | any | 20 | Percentage of the context budget reserved for output |
| compact_threshold_pct | any | 70 | Budget usage threshold triggering compaction |
| goal | any | None | Human task goal represented by this operation |
| constraints | any | None | Explicit task constraints |
| schema | any | None | Structured schema used to validate or project output |
| schema_version | any | None | Version of the structured schema |
| available_tokens | any | None | Token budget available to the next turn |
| used_tokens | any | None | Observed token usage, not a guessed count |
| source_event_range | any | None | Event range represented by a compacted context item |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Compact over-threshold packs; retain goal/constraints/pending_write/plan/trusted anchors (F19). |

Use `items`=None, `budget_tokens`=8000, `output_reserve_pct`=20, `compact_threshold_pct`=70, `goal`=None, `constraints`=None, `schema`=None, `schema_version`=None, `available_tokens`=None, `used_tokens`=None, `source_event_range`=None as the inputs for this operation.

Compact over-threshold packs; retain goal/constraints/pending_write/plan/trusted anchors (F19).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Context pack compact](agent_context_pack_compact(items=`items`, budget_tokens=`budget_tokens`, output_reserve_pct=`output_reserve_pct`, compact_threshold_pct=`compact_threshold_pct`, goal=`goal`, constraints=`constraints`, schema=`schema`, schema_version=`schema_version`, available_tokens=`available_tokens`, used_tokens=`used_tokens`, source_event_range=`source_event_range`))*.

## render_workbook_skeleton

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| goal | any | required | Human task goal represented by this operation |
| skeleton | any | "single" | Workbook source template to instantiate |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Build a runnable workbook. `skeleton=single` (default): one worker `step`. `skeleton=dual`: research then write agents. On success the parent chooses DONE; runtime `agent_workbook_solidify` freezes the answer. |

Use `goal`, `skeleton`="single" as the inputs for this operation.

Build a runnable workbook. `skeleton=single` (default): one worker `step`. `skeleton=dual`: research then write agents. On success the parent chooses DONE; runtime `agent_workbook_solidify` freezes the answer.

This function produces source text, not a running worker. The caller owns choosing a destination, writing that text, starting the child, and inspecting the child's exit status and returned value. Generation alone neither proves the task solved nor freezes a reusable skill. The templates are data in this function: their marker characters must remain literal until a later Marqdo process parses the generated workbook.

The common header names the libraries the generated program imports. Its Goal section explains where the parent goal is embedded, while its Solidify section explains who may freeze the answer. The entry begins by loading environment configuration. This makes the generated program usable by the existing workbook runner without adding a second template interpreter or hidden workflow format.

The single-worker branch creates one model and an empty tools list, then asks one worker to solve the embedded task. The empty whitelist is intentional: mentioning a capability in the task does not register a callable tool. A worker must answer directly unless the author later adds an actual tool definition and explicitly supplies it. The returned child value is the result field from the worker's structured turn.

The dual-worker branch uses the same model configuration for two roles. A research worker first returns notes. Those notes are serialized into the writing worker's task, and the writing worker returns the final text. The roles describe sequencing; they do not imply independent evidence collection, browser access, or any permission beyond the configured whitelist and runtime grants.

The goal is JSON-quoted before it is inserted into either source template. Quotes and line breaks in a user's goal therefore stay data rather than becoming workbook statements. The surrounding fragments retain the exact original newlines, imports and return markers. When changing this generator, review both complete outputs, parse them as ordinary source, and test goals containing quotes, backticks and newlines.

Environment variables are looked up when the child runs. No credentials are fetched or written by this generator. Missing model configuration consequently belongs to the execution result, not to template construction. A parent should await that result before deciding RUN, CONTINUE or DONE. Successful execution may then be solidified by the existing runtime mechanism; this function does not silently promote its generated text into trusted knowledge.

We retain **`q` = [Quote the JSON text value](json.quote(text=`goal`))**.
Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_h_lines` =

| line |
|---|
| "---" |
| "title: agent workbook" |
| "import:" |
| "  llm: ext/ai/llm.mq.md" |
| "  agent: ext/ai/agent.mq.md" |
| "  json: lib/json.mq.md" |
| "  sys: lib/sys.mq.md" |
| "  table: lib/table.mq.md" |
| "  wb: lib/writeback.mq.md" |
| "  subtask: lib/subtask.mq.md" |
| "---" |
| "" |
| "# Goal" |
| "" |
| "Parent goal is embedded as JSON in # main." |
| "" |
| "## Solidify" |
| "" |
| "When the child returns a good value, parent DECISION: DONE — runtime agent_workbook_solidify freezes the answer as **return** (do not invent long FIND/REPLACE)." |
| "" |
| "# main" |
| "" |
| "> llm.load_env" |
| "" |
| "" |

Join exactly those lines into **_literal_esc_h_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_h_lines`, sep="\n"))**.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_s1_lines` =

| line |
|---|
| "**\x60env_model\x60 = > sys.env_get name=\"OPENAI_MODEL\"**" |
| "**\x60env_base\x60 = > sys.env_get name=\"OPENAI_BASE_URL\"**" |
| "**\x60env_key\x60 = > sys.env_get name=\"OPENAI_API_KEY\"**" |
| "**\x60model\x60 = > llm.create model=\x60env_model\x60 base_url=\x60env_base\x60 api_key=\x60env_key\x60 backend=\"openai-compatible\" name=\"workbook\"**" |
| "" |

Join exactly those lines into **_literal_esc_s1_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_s1_lines`, sep="\n"))**.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_s3_lines` =

| line |
|---|
| "**\x60worker\x60 = > agent.agent model=\x60model\x60 tools=\x60tools\x60 standing=\"You are a workbook worker. tools=[] means never CALL. Finish the task with a direct final answer; do not invent tools.\"**" |
| "" |

Join exactly those lines into **_literal_esc_s3_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_s3_lines`, sep="\n"))**.

The following record names each literal field without a JSON construction chain.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "d10" | "**\x60wt\x60 = \x60wt\x60 + \x60ns\x60**\n" |
| 2 | "d11" | "**\x60out\x60 = > \x60writer\x60.step task=\x60wt\x60 stream=True**\n" |
| 3 | "d11b" | "**\x60text\x60 = > json.get value=\x60out\x60 key=\"result\"**\n" |
| 4 | "d12" | "*\x60text\x60*\n" |
| 5 | "d3" | "**\x60research\x60 = > agent.agent model=\x60model\x60 tools=\x60tools\x60 standing=\"You gather facts for the goal. Be concise. Do not invent tools.\"**\n" |
| 6 | "d3b" | "**\x60writer\x60 = > agent.agent model=\x60model\x60 tools=\x60tools\x60 standing=\"You write the final answer from research notes. Do not invent tools.\"**\n" |
| 7 | "d4" | "**\x60wrap\x60 = > json.parse text={\"task\":" |
| 8 | "d5" | "}**\n" |
| 9 | "d6" | "**\x60task\x60 = > json.get value=\x60wrap\x60 key=\"task\"**\n" |
| 10 | "d7" | "**\x60notes\x60 = > \x60research\x60.step task=\x60task\x60 stream=True**\n" |
| 11 | "d8" | "**\x60ns\x60 = > json.stringify value=\x60notes\x60**\n" |
| 12 | "d9" | "**\x60wt\x60 = \"Write the final answer. Research notes:\"**\n" |
| 13 | "h" | `_literal_esc_h_text` |
| 14 | "s1" | `_literal_esc_s1_text` |
| 15 | "s2" | "**\x60tools\x60 = > table.empty_list**\n" |
| 16 | "s3" | `_literal_esc_s3_text` |
| 17 | "s4" | "**\x60wrap\x60 = > json.parse text={\"task\":" |
| 18 | "s5" | "}**\n" |
| 19 | "s6" | "**\x60task\x60 = > json.get value=\x60wrap\x60 key=\"task\"**\n" |
| 20 | "s7" | "**\x60out\x60 = > \x60worker\x60.step task=\x60task\x60 stream=True**\n" |
| 21 | "s7b" | "**\x60text\x60 = > json.get value=\x60out\x60 key=\"result\"**\n" |
| 22 | "s8" | "*\x60text\x60*\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`h` = [Read the h field](json.get(value=`esc`, key="h"))**.
We retain **`s1` = [Read the s1 field](json.get(value=`esc`, key="s1"))**.
We retain **`s2` = [Read the s2 field](json.get(value=`esc`, key="s2"))**.

1. `skeleton` == dual
  We retain **`s3` = [Read the d3 field](json.get(value=`esc`, key="d3"))**.
  We retain **`s3b` = [Read the d3b field](json.get(value=`esc`, key="d3b"))**.
  We retain **`s4` = [Read the d4 field](json.get(value=`esc`, key="d4"))**.
  We retain **`s5` = [Read the d5 field](json.get(value=`esc`, key="d5"))**.
  We retain **`s6` = [Read the d6 field](json.get(value=`esc`, key="d6"))**.
  We retain **`s7` = [Read the d7 field](json.get(value=`esc`, key="d7"))**.
  We retain **`s8` = [Read the d8 field](json.get(value=`esc`, key="d8"))**.
  We retain **`s9` = [Read the d9 field](json.get(value=`esc`, key="d9"))**.
  We retain **`s10` = [Read the d10 field](json.get(value=`esc`, key="d10"))**.
  We retain **`s11` = [Read the d11 field](json.get(value=`esc`, key="d11"))**.
  We retain **`s11b` = [Read the d11b field](json.get(value=`esc`, key="d11b"))**.
  We retain **`s12` = [Read the d12 field](json.get(value=`esc`, key="d12"))**.
  Return *h + s1 + s2 + s3 + s3b + s4 + q + s5 + s6 + s7 + s8 + s9 + s10 + s11 + s11b + s12*.
2. *
  We retain **`s3` = [Read the s3 field](json.get(value=`esc`, key="s3"))**.
  We retain **`s4` = [Read the s4 field](json.get(value=`esc`, key="s4"))**.
  We retain **`s5` = [Read the s5 field](json.get(value=`esc`, key="s5"))**.
  We retain **`s6` = [Read the s6 field](json.get(value=`esc`, key="s6"))**.
  We retain **`s7` = [Read the s7 field](json.get(value=`esc`, key="s7"))**.
  We retain **`s7b` = [Read the s7b field](json.get(value=`esc`, key="s7b"))**.
  We retain **`s8` = [Read the s8 field](json.get(value=`esc`, key="s8"))**.
  Return *h + s1 + s2 + s3 + s4 + q + s5 + s6 + s7 + s7b + s8*.

---

## inspect_workbook

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| exit_code | any | None | Child process or tool exit status |
| value | any | None | Value supplied to this operation; immutable transformations return a new value |
| stdout | any | None | Captured standard output |
| stderr | any | None | Captured standard error |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Structured observation for the parent developer-agent: source, named writeback slots, exit code, optional child return value, and quiet-captured I/O. |

Use `path`, `exit_code`=None, `value`=None, `stdout`=None, `stderr`=None as the inputs for this operation.

Structured observation for the parent developer-agent: source, named writeback slots, exit code, optional child return value, and quiet-captured I/O.

We retain **`source` = [Read UTF-8 text from the selected file](fs.read_text(path=`path`))**.
We retain **`slots` = [Scan path](wb.scan_path(path=`path`))**.

`obs` =

| path | source | slots | exit_code |
|------|--------|-------|-----------|
| `path` | `source` | `slots` | `exit_code` |

1. `value`
  We retain **`obs` = [Set the value field in the new record](table.put(in=`obs`, at="value", value=`value`))**.
2. *
  **_ = 1**
1. `stdout`
  We retain **`obs` = [Set the stdout field in the new record](table.put(in=`obs`, at="stdout", value=`stdout`))**.
2. *
  **_ = 1**
1. `stderr`
  We retain **`obs` = [Set the stderr field in the new record](table.put(in=`obs`, at="stderr", value=`stderr`))**.
2. *
  **_ = 1**

We retain **last_ok = None**.
We retain **last_error = None**.
- [slot](遍历 `slots` 为 `slot`)
  We retain **`k` = [Read the key field](json.get(value=`slot`, key="key"))**.
  We retain **`body` = [Read the body field](json.get(value=`slot`, key="body"))**.
  1. `k` == "ok"
    We retain **last_ok = body**.
  2. `k` == "error"
    We retain **last_error = body**.
  3. *
    **_ = 1**

1. `last_ok`
  We retain **`obs` = [Set the last_ok field in the new record](table.put(in=`obs`, at="last_ok", value=`last_ok`))**.
2. *
  **_ = 1**

1. `last_error`
  We retain **`obs` = [Set the last_error field in the new record](table.put(in=`obs`, at="last_error", value=`last_error`))**.
2. *
  **_ = 1**

Return *obs*.

---

## trial

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| source | any | None | Source text inspected or prepared by this operation |
| root | any | None | Explicit root directory for discovery or isolated execution |
| write_path | any | None | Explicit output path or None |
| text | any | None | Text supplied to the transformation or adapter |
| commit | any | True | Whether to promote trial changes after successful validation |
| profile | any | "default" | Capability profile selected for the trial |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Opt-in CoW trial (O1): open → optional upper write → optional run → diff → commit on success else discard. Host tree unchanged until a successful commit. Return map is a GFM table (link-index readable). |

Use `path`=None, `source`=None, `root`=None, `write_path`=None, `text`=None, `commit`=True, `profile`="default" as the inputs for this operation.

Opt-in CoW trial (O1): open → optional upper write → optional run → diff → commit on success else discard.
Host tree unchanged until a successful commit. Return map is a GFM table (link-index readable).

We retain **`sb` = [Open](sandbox.open(root=`root`, profile=`profile`))**.
We retain **`id` = [Read the id field](json.get(value=`sb`, key="id"))**.
We retain **`prof` = [Read the profile field](json.get(value=`sb`, key="profile"))**.
We retain **ok = True**.
We retain **stdout = ""**.
We retain **stderr = ""**.

1. `write_path`
  1. `text`
    We **[Write](sandbox.write(id=`id`, path=`write_path`, text=`text`))**.
  2. *
    **_ = 1**
2. *
  **_ = 1**

1. `source`
  We retain **`ret` = [Run](sandbox.run(id=`id`, source=`source`))**.
  We retain **`ok` = [Read the ok field](json.get(value=`ret`, key="ok"))**.
  We retain **`stdout` = [Read the stdout field](json.get(value=`ret`, key="stdout"))**.
  We retain **`stderr` = [Read the stderr field](json.get(value=`ret`, key="stderr"))**.
2. `path`
  We retain **`ret` = [Run](sandbox.run(id=`id`, path=`path`))**.
  We retain **`ok` = [Read the ok field](json.get(value=`ret`, key="ok"))**.
  We retain **`stdout` = [Read the stdout field](json.get(value=`ret`, key="stdout"))**.
  We retain **`stderr` = [Read the stderr field](json.get(value=`ret`, key="stderr"))**.
3. *
  **_ = 1**

We retain **`pending` = [Diff](sandbox.diff(id=`id`))**.
We retain **committed = False**.

1. `ok`
  1. `commit`
    We **[Commit](sandbox.commit(id=`id`))**.
    We retain **committed = True**.
  2. *
    We **[Discard](sandbox.discard(id=`id`))**.
2. *
  We **[Discard](sandbox.discard(id=`id`))**.

**out = **

| ok | committed | diff | stdout | stderr | id | profile |
|----|-----------|------|--------|--------|-----|---------|
| `ok` | `committed` | `pending` | `stdout` | `stderr` | `id` | `prof` |

Return *out*.

## step_route

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| kind | any | "default" | Operation or factorization kind listed in this function's behavior |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | O3: map step class (`lookup` &#124; `patch` &#124; `sandbox_run` &#124; `net` &#124; `default`) → Adaptive Routing backend. Authority is the GFM table in `ext/ai/policies/step-route.mq.md` (mirrored here for offline reads). |

Use `kind`="default" as the inputs for this operation.

O3: map step class (`lookup` | `patch` | `sandbox_run` | `net` | `default`) → Adaptive Routing backend.
Authority is the GFM table in `ext/ai/policies/step-route.mq.md` (mirrored here for offline reads).

1. `kind` == "lookup"
  We retain **backend = "marqdo"**.
2. `kind` == "patch"
  We retain **backend = "small-llm"**.
3. `kind` == "sandbox_run"
  We retain **backend = "marqdo"**.
4. `kind` == "net"
  We retain **backend = "llm"**.
5. *
  We retain **backend = "auto"**.

We retain **`out` = [Set the kind field in the new record](table.put(in=None, at="kind", value=`kind`))**.
We retain **`out` = [Set the backend field in the new record](table.put(in=`out`, at="backend", value=`backend`))**.
Return *out*.

## await_workbook

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| sandbox | any | False | Whether to use the CoW trial path |
| sandbox_root | any | None | Root directory of the CoW trial |
| sandbox_profile | any | "default" | Capability profile for CoW execution |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Spawn a workbook file (quiet by default), wait, and inspect. With `sandbox=True`, run via `## trial` first (commit on ok); host unchanged until that commit. |

Use `path`, `sandbox`=False, `sandbox_root`=None, `sandbox_profile`="default" as the inputs for this operation.

Spawn a workbook file (quiet by default), wait, and inspect.
With `sandbox=True`, run via `## trial` first (commit on ok); host unchanged until that commit.

1. `sandbox`
  We retain **`tr` = [Trial](trial(path=`path`, root=`sandbox_root`, profile=`sandbox_profile`, commit=True))**.
  We retain **`ok` = [Read the ok field](json.get(value=`tr`, key="ok"))**.
  We retain **`stdout` = [Read the stdout field](json.get(value=`tr`, key="stdout"))**.
  We retain **`stderr` = [Read the stderr field](json.get(value=`tr`, key="stderr"))**.
  1. `ok`
    We retain **code = 0**.
  2. *
    We retain **code = 1**.
  We retain **value = stdout**.
  We retain **`obs` = [Inspect workbook](inspect_workbook(path=`path`, exit_code=`code`, value=`value`, stdout=`stdout`, stderr=`stderr`))**.
  **out = **

  | code | value | observation | sandbox |
  |------|-------|-------------|---------|
  | `code` | `value` | `obs` | `tr` |

  Return *out*.
2. *
  We retain **`id` = [Spawn](subtask.spawn(path=`path`))**.
  We retain **`waited` = [Wait](subtask.wait(id=`id`))**.
  We retain **`code` = [Read the code field](json.get(value=`waited`, key="code"))**.
  We retain **`value` = [Read the value field](json.get(value=`waited`, key="value"))**.
  We retain **`stdout` = [Read the stdout field](json.get(value=`waited`, key="stdout"))**.
  We retain **`stderr` = [Read the stderr field](json.get(value=`waited`, key="stderr"))**.
  We retain **`obs` = [Inspect workbook](inspect_workbook(path=`path`, exit_code=`code`, value=`value`, stdout=`stdout`, stderr=`stderr`))**.
  **out = **

  | code | value | observation |
  |------|-------|-------------|
  | `code` | `value` | `obs` |

  Return *out*.

---

## text_head_lines

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | any | required | Text supplied to the transformation or adapter |
| max_lines | any | 80 | Maximum retained lines in the text excerpt |
| max_chars | any | 4000 | Maximum retained characters in the text excerpt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Keep at most the requested leading lines of text when preparing a bounded context excerpt. |

Use `text`, `max_lines`=80, `max_chars`=4000 as the inputs for this operation.

Keep at most the requested leading lines of text when preparing a bounded context excerpt.

Keep the literal separator as **esc = [Keep the exact separator](table.put(in=None, at="nl", value="\n"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`raw` = [Str](str(value=`text`))**.
We retain **`lines` = [Split](split(value=`raw`, sep=`nl`))**.
We retain **`acc` = [Empty list](table.empty_list())**.
We retain **chars = 0**.
We retain **count = 0**.
- [line](遍历 `lines` 为 `line`)
  1. `count` < `max_lines`
    We retain **`ln` = [Len](len(value=`line`))**.
    We retain **next = chars + ln**.
    1. `next` > `max_chars`
      We retain **count = max_lines**.
    2. *
      We retain **`acc` = [Append this item to the new list](table.append(list=`acc`, item=`line`))**.
      We retain **chars = next**.
      We retain **count = count + 1**.
  2. *
    **_ = 1**
Return *[Join](join(value=`acc`, sep=`nl`))*.

---

## text_tail_lines

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | any | required | Text supplied to the transformation or adapter |
| max_lines | any | 40 | Maximum retained lines in the text excerpt |
| max_chars | any | 2000 | Maximum retained characters in the text excerpt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Keep at most the requested trailing lines of text when preparing a bounded context excerpt. |

Use `text`, `max_lines`=40, `max_chars`=2000 as the inputs for this operation.

Keep at most the requested trailing lines of text when preparing a bounded context excerpt.

Keep the literal separator as **esc = [Keep the exact separator](table.put(in=None, at="nl", value="\n"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`raw` = [Str](str(value=`text`))**.
We retain **`lines` = [Split](split(value=`raw`, sep=`nl`))**.
We retain **`n` = [Len](len(value=`lines`))**.
We retain **start = n - max_lines**.
1. `start` < 0
  We retain **start = 0**.
2. *
  **_ = 1**
We retain **`acc` = [Empty list](table.empty_list())**.
We retain **i = 0**.
We retain **chars = 0**.
- [line](遍历 `lines` 为 `line`)
  1. `i` >= `start`
    We retain **`ln` = [Len](len(value=`line`))**.
    We retain **next = chars + ln**.
    1. `next` > `max_chars`
      We retain **i = n**.
    2. *
      We retain **`acc` = [Append this item to the new list](table.append(list=`acc`, item=`line`))**.
      We retain **chars = next**.
  2. *
    **_ = 1**
  We retain **i = i + 1**.
Return *[Join](join(value=`acc`, sep=`nl`))*.

---

## strip_html_comments

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | any | required | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Drop `<!-- … -->` blocks (including marqdo-out bodies) so parent excerpts stay structural. |

Use `text` as the inputs for this operation.

Drop `<!-- … -->` blocks (including marqdo-out bodies) so parent excerpts stay structural.

This is a line-oriented removal helper for workbook inspection. It tracks whether a comment span is open across lines, so hidden author notes do not inflate the excerpt. It is not an HTML parser and does not sanitize rendered pages.

Keep the literal separator as **esc = [Keep the exact separator](table.put(in=None, at="nl", value="\n"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`raw` = [Str](str(value=`text`))**.
We retain **`lines` = [Split](split(value=`raw`, sep=`nl`))**.
We retain **`acc` = [Empty list](table.empty_list())**.
We retain **skip = None**.
- [line](遍历 `lines` 为 `line`)
  1. `skip`
    We retain **`end_parts` = [Split](split(value=`line`, sep="-->"))**.
    We retain **`en` = [Len](len(value=`end_parts`))**.
    1. `en` > 1
      We retain **skip = None**.
    2. *
      **_ = 1**
  2. *
    We retain **`start_parts` = [Split](split(value=`line`, sep="<!--"))**.
    We retain **`sn` = [Len](len(value=`start_parts`))**.
    1. `sn` > 1
      We retain **`before` = [At](at(value=`start_parts`, index=0))**.
      We retain **`before` = [Trim](trim(value=`before`))**.
      1. `before`
        We retain **`acc` = [Append this item to the new list](table.append(list=`acc`, item=`before`))**.
      2. *
        **_ = 1**
      We retain **`tail` = [At](at(value=`start_parts`, index=1))**.
      We retain **`end_parts` = [Split](split(value=`tail`, sep="-->"))**.
      We retain **`en` = [Len](len(value=`end_parts`))**.
      1. `en` > 1
        We retain **skip = None**.
      2. *
        We retain **skip = 1**.
    2. *
      We retain **`acc` = [Append this item to the new list](table.append(list=`acc`, item=`line`))**.
Return *[Join](join(value=`acc`, sep=`nl`))*.

---

## workbook_source_excerpt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| source | any | required | Source text inspected or prepared by this operation |
| max_lines | any | 80 | Maximum retained lines in the text excerpt |
| max_chars | any | 4000 | Maximum retained characters in the text excerpt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Select a bounded excerpt from workbook source for the next planning observation. |

Use `source`, `max_lines`=80, `max_chars`=4000 as the inputs for this operation.

Select a bounded excerpt from workbook source for the next planning observation.

We retain **`clean` = [Strip html comments](strip_html_comments(text=`source`))**.
Return *[Text head lines](text_head_lines(text=`clean`, max_lines=`max_lines`, max_chars=`max_chars`))*.

---

## workbook_excerpt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| max_lines | any | 80 | Maximum retained lines in the text excerpt |
| max_chars | any | 4000 | Maximum retained characters in the text excerpt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read a workbook excerpt using explicit depth and length budgets. |

Use `path`, `max_lines`=80, `max_chars`=4000 as the inputs for this operation.

Read a workbook excerpt using explicit depth and length budgets.

We retain **`src` = [Read UTF-8 text from the selected file](fs.read_text(path=`path`))**.
Return *[Workbook source excerpt](workbook_source_excerpt(source=`src`, max_lines=`max_lines`, max_chars=`max_chars`))*.

---

## workbook_read

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| depth | any | "default" | Requested context or discovery depth |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read a workbook by path and expose the budgeted view required by the planner. |

Use `path`, `depth`="default" as the inputs for this operation.

We retain **`src` = [Read UTF-8 text from the selected file](fs.read_text(path=`path`))**.
1. `depth` == deep
  Return *[Workbook source excerpt](workbook_source_excerpt(source=`src`, max_lines=200, max_chars=12000))*.
2. *
  Return *[Workbook source excerpt](workbook_source_excerpt(source=`src`, max_lines=80, max_chars=4000))*.

---

## lib_catalog

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Callable dotted names (whitelist for `CALL:lib…`) plus static module file list for workbook imports. Parent also exposes evidence tools `corpus_search` / `mcp_list_tools` / `mcp_call` (not authority). |

Callable dotted names (whitelist for `CALL:lib…`) plus static module file list for workbook imports. Parent also exposes evidence tools `corpus_search` / `mcp_list_tools` / `mcp_call` (not authority).

The import files are a list of source modules. They are not all enabled as direct parent tools.

The two lists answer different questions: files can be imported into a workbook, while callable records name the limited actions understood by the parent dispatcher. This is not a complete catalog of the standard library. New source functions remain discoverable through the normal source catalog and MLSP; adding a file here does not grant it execution permission.

`_literal_files` =

| file |
|---|
| "fs.mq.md" |
| "json.mq.md" |
| "subtask.mq.md" |
| "sys.mq.md" |
| "writeback.mq.md" |
| "text.mq.md" |
| "time.mq.md" |
| "math.mq.md" |
| "net.mq.md" |
| "table.mq.md" |
| "plugin.mq.md" |
| "foreign.mq.md" |

Only the following names belong to this parent dispatcher. The description explains the action;
a name still passes through the normal allowlist and its effect boundary at execution.

`_literal_callable` =

| @ | desc | name |
|---|---|---|
| 1 | "Read UTF-8 text file" | "lib.fs.read_text" |
| 2 | "Path exists?" | "lib.fs.exists" |
| 3 | "List directory entries" | "lib.fs.list_dir" |
| 4 | "Parse JSON text" | "lib.json.parse" |
| 5 | "Serialize JSON" | "lib.json.stringify" |
| 6 | "Process working directory" | "lib.sys.cwd" |
| 7 | "Local markdown/text keyword search (evidence)" | "corpus_search" |
| 8 | "List tools from MCP JSON fixture" | "mcp_list_tools" |
| 9 | "Call a tool from MCP JSON fixture" | "mcp_call" |

The note is returned with both lists, so a caller can distinguish an importable file from an
allowed CALL name. Evidence adapters read fixtures here; they do not establish workbook authority.

`out` =

| callable | files | note |
|---|---|---|
| `_literal_callable` | `_literal_files` | "Direct CALL:lib.<mod>.<fn> + optional ARGS:{json} for whitelist only. Or import via frontmatter > lib/<file>.mq.md then PATCH / scratch_tool_write. Evidence tools: CALL:corpus_search \| mcp_list_tools \| mcp_call (fixture RAG/MCP) — workbook .mq.md remains authority." |
Return *out*.

---

## corpus_search

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| query | any | required | Text used to search the bounded catalog or tool registry |
| root | any | ".marqdo/agent-corpus" | Explicit root directory for discovery or isolated execution |
| limit | any | 6 | Maximum returned entries or candidates; this is not a success threshold |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Local keyword search over `.md` / `.mq.md` / `.txt` under `root`. Returns ranked excerpts. Evidence only — runnable workbook / OKF skill stays authority (`authority=workbook`). Ensures the native agent plugin is loaded so plain `marqdo run` (without constructing `# agent`) can call this. |

Use `query`, `root`=".marqdo/agent-corpus", `limit`=6 as the inputs for this operation.

Local keyword search over `.md` / `.mq.md` / `.txt` under `root`. Returns ranked excerpts. Evidence only — runnable workbook / OKF skill stays authority (`authority=workbook`).

Ensures the native agent plugin is loaded so plain `marqdo run` (without constructing `# agent`) can call this.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Corpus search](agent_corpus_search(query=`query`, root=`root`, limit=`limit`))*.

---

## mcp_list_tools

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| fixture | any | required | Deterministic fixture data used for offline behavior |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List tools from an offline MCP-shaped JSON fixture (`tools` array). Evidence only. |

Use `fixture` as the inputs for this operation.

List tools from an offline MCP-shaped JSON fixture (`tools` array). Evidence only.

Return *[Mcp fixture](agent_mcp_fixture(action="list", fixture=`fixture`))*.

---

## mcp_call

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| fixture | any | required | Deterministic fixture data used for offline behavior |
| arguments | any | None | Named argument record for the selected registered tool |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Call a named tool from a MCP-shaped fixture (`results` / `calls` map). Optional `arguments` echoed; fixture results are keyed by name. Evidence only. |

Use `name`, `fixture`, `arguments`=None as the inputs for this operation.

Call a named tool from a MCP-shaped fixture (`results` / `calls` map). Optional `arguments` echoed; fixture results are keyed by name. Evidence only.

1. `arguments`
  Return *[Mcp fixture](agent_mcp_fixture(action="call", fixture=`fixture`, name=`name`, arguments=`arguments`))*.
2. *
  Return *[Mcp fixture](agent_mcp_fixture(action="call", fixture=`fixture`, name=`name`))*.

---

## mcp_connect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| command | any | required | Configured external command |
| args | any | None | Arguments passed to the selected command or operation |
| session | any | "default" | Existing session handle returned by the corresponding connector |
| name | any | "mcp" | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Open a stdio MCP client session (spawn `command` + `args`, JSON-RPC initialize). Evidence only — workbook remains authority. |

Use `command`, `args`=None, `session`="default", `name`="mcp" as the inputs for this operation.

Open a stdio MCP client session (spawn `command` + `args`, JSON-RPC initialize). Evidence only — workbook remains authority.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

1. `args`
  Return *[Mcp client](agent_mcp_client(action="connect", command=`command`, args=`args`, session=`session`, name=`name`))*.
2. *
  Return *[Mcp client](agent_mcp_client(action="connect", command=`command`, session=`session`, name=`name`))*.

## mcp_client_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| session | any | "default" | Existing session handle returned by the corresponding connector |
| command | any | None | Configured external command |
| args | any | None | Arguments passed to the selected command or operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List tools from an open MCP session, or one-shot when `command=` is set (no session retained). |

Use `session`="default", `command`=None, `args`=None as the inputs for this operation.

List tools from an open MCP session, or one-shot when `command=` is set (no session retained).

1. `command`
  1. `args`
    Return *[Mcp client](agent_mcp_client(action="list", command=`command`, args=`args`))*.
  2. *
    Return *[Mcp client](agent_mcp_client(action="list", command=`command`))*.
2. *
  Return *[Mcp client](agent_mcp_client(action="list", session=`session`))*.

## mcp_client_call

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | None | Name registered or looked up by this operation |
| session | any | "default" | Existing session handle returned by the corresponding connector |
| arguments | any | None | Named argument record for the selected registered tool |
| command | any | None | Configured external command |
| args | any | None | Arguments passed to the selected command or operation |
| tool | any | None | Registered tool or tool name selected by policy |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Call a tool on an open MCP session, or one-shot when `command=` is set. Prefer `tool=` for oneshot when `name` would collide with server name. |

Use `name`=None, `session`="default", `arguments`=None, `command`=None, `args`=None, `tool`=None as the inputs for this operation.

Call a tool on an open MCP session, or one-shot when `command=` is set. Prefer `tool=` for oneshot when `name` would collide with server name.

1. `command`
  1. `tool`
    We retain **tn = tool**.
  2. *
    We retain **tn = name**.
  1. `args`
    Return *[Mcp client](agent_mcp_client(action="call", command=`command`, args=`args`, tool=`tn`, arguments=`arguments`))*.
  2. *
    Return *[Mcp client](agent_mcp_client(action="call", command=`command`, tool=`tn`, arguments=`arguments`))*.
2. *
  Return *[Mcp client](agent_mcp_client(action="call", session=`session`, name=`name`, arguments=`arguments`))*.

## mcp_disconnect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| session | any | "default" | Existing session handle returned by the corresponding connector |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Close a stdio MCP client session. |

Use `session`="default" as the inputs for this operation.

Close a stdio MCP client session.

Return *[Mcp client](agent_mcp_client(action="close", session=`session`))*.

---

## resume_save

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| goal | any | None | Human task goal represented by this operation |
| task | any | None | Structured task or text supplied to the agent |
| status | any | "running" | Domain status recorded in the structured result |
| round | any | 0 | Current planning round number |
| workbook | any | None | Workbook path or structured workbook reference |
| events | any | None | Ordered semantic or provider stream-event records |
| route | any | None | Selected route or route-policy data |
| result | any | None | Structured result from the preceding operation |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |
| checkpoint_v2 | any | False | Versioned continuation checkpoint record |
| attempt_id | any | None | Identifier for this execution attempt |
| effect_ledger_watermark | any | None | Last durable effect-ledger position included by the checkpoint |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Persist a plan/run checkpoint as `.marqdo/agent-resume/<id>.json` (atomic write). Pass `checkpoint_v2=True` for CheckpointV2 schema. `resume_dir` must be host-relative. |

Use `id`, `goal`=None, `task`=None, `status`="running", `round`=0, `workbook`=None, `events`=None, `route`=None, `result`=None, `resume_dir`=".marqdo/agent-resume", `checkpoint_v2`=False, `attempt_id`=None, `effect_ledger_watermark`=None as the inputs for this operation.

Persist a plan/run checkpoint as `.marqdo/agent-resume/<id>.json` (atomic write).
Pass `checkpoint_v2=True` for CheckpointV2 schema. `resume_dir` must be host-relative.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Resume save](agent_resume_save(id=`id`, goal=`goal`, task=`task`, status=`status`, round=`round`, workbook=`workbook`, events=`events`, route=`route`, result=`result`, resume_dir=`resume_dir`, checkpoint_v2=`checkpoint_v2`, attempt_id=`attempt_id`, effect_ledger_watermark=`effect_ledger_watermark`))*.

## resume_load

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load a resume checkpoint (or `{ok:false,found:false}`). Legacy / missing effect ledger → `recovery_mode=manual_review`, `write_auto_resume=false`. |

Use `id`, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Load a resume checkpoint (or `{ok:false,found:false}`).
Legacy / missing effect ledger → `recovery_mode=manual_review`, `write_auto_resume=false`.

Return *[Resume load](agent_resume_load(id=`id`, resume_dir=`resume_dir`))*.

## error_make

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | any | required | Stable failure category or bound code value, as stated by this function |
| message | any | None | Outgoing WebSocket or application message |
| category | any | None | Failure category used by recovery policy |
| stage | any | "agent" | Execution stage where the failure occurred |
| retryable | any | None | Whether policy considers this failure eligible for retry |
| effect_state | any | "none" | Observed state of the logical effect |
| recovery_action | any | None | Selected recovery operation |
| call_id | any | None | Identifier of this tool invocation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Build an `AgentErrorV1` document (M-A1). |

Use `code`, `message`=None, `category`=None, `stage`="agent", `retryable`=None, `effect_state`="none", `recovery_action`=None, `call_id`=None as the inputs for this operation.

Build an `AgentErrorV1` document (M-A1).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Error make](agent_error_make(code=`code`, message=`message`, category=`category`, stage=`stage`, retryable=`retryable`, effect_state=`effect_state`, recovery_action=`recovery_action`, call_id=`call_id`))*.

## error_decide_retry

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| error | any | required | Structured failure information supplied by the caller |
| attempts | any | 0 | Retry or exploration attempts already performed |
| effect_class | any | "read" | Declared kind of effect for this operation |
| cancelled | any | False | Whether cancellation has already been requested |
| fingerprint_repeats | any | 0 | Number of repeated identical failure fingerprints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Bounded recovery decision. Write / unknown effect → `manual_review` (no auto redo). |

Use `error`, `attempts`=0, `effect_class`="read", `cancelled`=False, `fingerprint_repeats`=0 as the inputs for this operation.

Bounded recovery decision. Write / unknown effect → `manual_review` (no auto redo).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Error decide retry](agent_error_decide_retry(error=`error`, attempts=`attempts`, effect_class=`effect_class`, cancelled=`cancelled`, fingerprint_repeats=`fingerprint_repeats`))*.

## resume_clear

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delete a resume checkpoint. |

Use `id`, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Delete a resume checkpoint.

Return *[Resume clear](agent_resume_clear(id=`id`, resume_dir=`resume_dir`))*.

## resume_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List checkpoint ids under `resume_dir`. |

Use `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

List checkpoint ids under `resume_dir`.

Return *[Resume list](agent_resume_list(resume_dir=`resume_dir`))*.

## resume

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Author entry: load checkpoint for continuation. Returns the checkpoint map (`found`/`round`/`workbook`/…). Callers continue with `plan`/`run` using saved fields; clear when done. |

Use `id`, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Author entry: load checkpoint for continuation. Returns the checkpoint map (`found`/`round`/`workbook`/…). Callers continue with `plan`/`run` using saved fields; clear when done.

Return *[Resume load](resume_load(id=`id`, resume_dir=`resume_dir`))*.

## effects_prepare

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation_id | any | required | Stable idempotent operation identifier |
| idempotency_key | any | required | Key identifying the same logical effect across retries |
| args_digest | any | required | Digest of the exact operation arguments |
| effect_class | any | "write" | Declared kind of effect for this operation |
| resource_scope | any | None | Resource boundary to which the operation applies |
| expected_revision | any | None | Revision the controlled update expects |
| result_ref | any | None | Durable reference to an already-produced result |
| logical_run_id | any | None | Stable logical execution identifier shared by retries |
| step_id | any | None | Identifier of the current workbook step |
| attempt_id | any | None | Identifier for this execution attempt |
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Create (or dedupe) an `EffectLedgerRecordV1` in `prepared`. Same idempotency_key + args_digest returns the existing record; same key with different digest is rejected. |

Use `operation_id`, `idempotency_key`, `args_digest`, `effect_class`="write", `resource_scope`=None, `expected_revision`=None, `result_ref`=None, `logical_run_id`=None, `step_id`=None, `attempt_id`=None, `ledger_dir`=".marqdo/agent-effects" as the inputs for this operation.

Create (or dedupe) an `EffectLedgerRecordV1` in `prepared`. Same idempotency_key + args_digest returns the existing record; same key with different digest is rejected.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects prepare](agent_effects_prepare(operation_id=`operation_id`, idempotency_key=`idempotency_key`, args_digest=`args_digest`, effect_class=`effect_class`, resource_scope=`resource_scope`, expected_revision=`expected_revision`, result_ref=`result_ref`, logical_run_id=`logical_run_id`, step_id=`step_id`, attempt_id=`attempt_id`, ledger_dir=`ledger_dir`))*.

## effects_transition

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation_id | any | required | Stable idempotent operation identifier |
| state | any | required | Structured current state |
| result_ref | any | None | Durable reference to an already-produced result |
| note | any | None | Human-readable context associated with the record |
| reconcile_hint | any | None | Hint for reconciling an uncertain effect, not authorization to repeat it |
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Advance ledger state (`prepared→started→committed&#124;failed_no_effect&#124;unknown→reconciled_*`). |

Use `operation_id`, `state`, `result_ref`=None, `note`=None, `reconcile_hint`=None, `ledger_dir`=".marqdo/agent-effects" as the inputs for this operation.

Advance ledger state (`prepared→started→committed|failed_no_effect|unknown→reconciled_*`).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects transition](agent_effects_transition(operation_id=`operation_id`, state=`state`, result_ref=`result_ref`, note=`note`, reconcile_hint=`reconcile_hint`, ledger_dir=`ledger_dir`))*.

## effects_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation_id | any | required | Stable idempotent operation identifier |
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load one ledger record. |

Use `operation_id`, `ledger_dir`=".marqdo/agent-effects" as the inputs for this operation.

Load one ledger record.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects get](agent_effects_get(operation_id=`operation_id`, ledger_dir=`ledger_dir`))*.

## effects_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |
| logical_run_id | any | None | Stable logical execution identifier shared by retries |
| state | any | None | Structured current state |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List ledger records (optional filters). |

Use `ledger_dir`=".marqdo/agent-effects", `logical_run_id`=None, `state`=None as the inputs for this operation.

List ledger records (optional filters).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects list](agent_effects_list(ledger_dir=`ledger_dir`, logical_run_id=`logical_run_id`, state=`state`))*.

## effects_watermark

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |
| logical_run_id | any | None | Stable logical execution identifier shared by retries |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Digest committed/open effects for CheckpointV2. `write_auto_resume_safe` is false while unknown or pending writes remain. |

Use `ledger_dir`=".marqdo/agent-effects", `logical_run_id`=None as the inputs for this operation.

Digest committed/open effects for CheckpointV2. `write_auto_resume_safe` is false while unknown or pending writes remain.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects watermark](agent_effects_watermark(ledger_dir=`ledger_dir`, logical_run_id=`logical_run_id`))*.

## effects_reconcile

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| operation_id | any | required | Stable idempotent operation identifier |
| outcome | any | required | Observed domain result |
| result_ref | any | None | Durable reference to an already-produced result |
| note | any | None | Human-readable context associated with the record |
| ledger_dir | any | ".marqdo/agent-effects" | Directory holding the durable effect ledger |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Reconcile an `unknown` record: `outcome` = committed &#124; no_effect &#124; needs_input. |

Use `operation_id`, `outcome`, `result_ref`=None, `note`=None, `ledger_dir`=".marqdo/agent-effects" as the inputs for this operation.

Reconcile an `unknown` record: `outcome` = committed | no_effect | needs_input.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

Return *[Effects reconcile](agent_effects_reconcile(operation_id=`operation_id`, outcome=`outcome`, result_ref=`result_ref`, note=`note`, ledger_dir=`ledger_dir`))*.

---

## extract_call_args

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | None | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse `ARGS:{…}` JSON line, or a bare `{…}` line after CALL. |

Use `reply`=None as the inputs for this operation.

Parse `ARGS:{…}` JSON line, or a bare `{…}` line after CALL.

The following record names each literal field without a JSON construction chain.

Only the explicit ARGS line is parsed as JSON. Other reply prose remains inert, and absent arguments stay absent. This extraction does not validate the selected tool's schema or execute the reply; the dispatcher performs those later checks.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "brace" | "{" |
| 2 | "mark" | "ARGS:" |
| 3 | "nl" | "\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`mark` = [Read the mark field](json.get(value=`esc`, key="mark"))**.
We retain **`brace` = [Read the brace field](json.get(value=`esc`, key="brace"))**.
We retain **`empty` = [Empty map](table.empty_map())**.
1. `reply`
  We retain **`text` = [Trim](trim(value=`reply`))**.
  We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
  We retain **found = None**.
  - [line](遍历 `lines` 为 `line`)
    1. `found`
      **_ = 1**
    2. *
      We retain **`t` = [Trim](trim(value=`line`))**.
      We retain **`chunks` = [Split](split(value=`t`, sep=`mark`))**.
      We retain **`cn` = [Len](len(value=`chunks`))**.
      1. `cn` > 1
        We retain **`rest` = [At](at(value=`chunks`, index=1))**.
        We retain **`found` = [Trim](trim(value=`rest`))**.
      2. *
        We retain **`chunks` = [Split](split(value=`t`, sep=`brace`))**.
        We retain **`cn` = [Len](len(value=`chunks`))**.
        We retain **`prefix` = [At](at(value=`chunks`, index=0))**.
        1. `cn` > 1
          1. `prefix`
            **_ = 1**
          2. *
            We retain **found = t**.
        2. *
          **_ = 1**
  1. `found`
    Return *[Parse the JSON transport text](json.parse(text=`found`))*.
  2. *
    Return *empty*.
2. *
  Return *empty*.

---

## tool_result_brief

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | None | Value supplied to this operation; immutable transformations return a new value |
| max_chars | any | 1024 | Maximum retained characters in the text excerpt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Truncate a tool result for `tool_end` events (~1KiB). |

Use `value`=None, `max_chars`=1024 as the inputs for this operation.

Truncate a tool result for `tool_end` events (~1KiB).

1. `value`
  We retain **`s` = [Encode the structured value as JSON text](json.stringify(value=`value`))**.
  Return *[Text head lines](text_head_lines(text=`s`, max_lines=40, max_chars=`max_chars`))*.
2. *
  Return *None*.

---

## plan_append_tool

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |
| type | any | required | Structured item or event type |
| name | any | required | Name registered or looked up by this operation |
| kind | any | required | Operation or factorization kind listed in this function's behavior |
| result | any | None | Structured result from the preceding operation |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Always append `tool_start` / `tool_end` to `events` for audit / view process cards. SSE publish only when `stream=True`. |

Use `events`, `type`, `name`, `kind`, `result`=None, `stream`=False as the inputs for this operation.

Always append `tool_start` / `tool_end` to `events` for audit / view process cards. SSE publish only when `stream=True`.

`ev` =

| type | name | kind |
|------|------|------|
| `type` | `name` | `kind` |

1. `result`
  We retain **`ev` = [Set the result field in the new record](table.put(in=`ev`, at="result", value=`result`))**.
2. *
  **_ = 1**
We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
1. `stream`
  We **[Stream publish](sys.stream_publish(event=`ev`))**.
2. *
  **_ = 1**

Return *events*.

---

## run_lib_call

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| reply | any | None | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Whitelist `CALL:lib.<mod>.<fn>` (+ ARGS). Rejects net/exec/write/delete. |

Use `name`, `reply`=None as the inputs for this operation.

Whitelist `CALL:lib.<mod>.<fn>` (+ ARGS). Rejects net/exec/write/delete.

We retain **`args` = [Extract call args](extract_call_args(reply=`reply`))**.
Keep the literal separator as **errs = [Keep the exact separator](table.put(in=None, at="deny", value="Lib CALL not allowed: "))**.

1. `name` == lib.fs.exists
  We retain **`path` = [Read the path field](json.get(value=`args`, key="path"))**.
  Return *[Exists](fs.exists(path=`path`))*.
2. `name` == lib.fs.read_text
  We retain **`path` = [Read the path field](json.get(value=`args`, key="path"))**.
  Return *[Read UTF-8 text from the selected file](fs.read_text(path=`path`))*.
3. `name` == lib.fs.list_dir
  We retain **`path` = [Read the path field](json.get(value=`args`, key="path"))**.
  Return *[List dir](fs.list_dir(path=`path`))*.
4. `name` == lib.json.parse
  We retain **`text` = [Read the text field](json.get(value=`args`, key="text"))**.
  Return *[Parse the JSON transport text](json.parse(text=`text`))*.
5. `name` == lib.json.stringify
  We retain **`value` = [Read the value field](json.get(value=`args`, key="value"))**.
  Return *[Encode the structured value as JSON text](json.stringify(value=`value`))*.
6. `name` == lib.sys.cwd
  Return *[Cwd](sys.cwd())*.
7. *
  We retain **`deny` = [Read the deny field](json.get(value=`errs`, key="deny"))**.
  Return *deny + name*.

---

## scratch_tool_write

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| text | any | required | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Write a scratch tool workbook under `.marqdo/agent-runs/tools/<name>.mq.md`. |

Use `name`, `text` as the inputs for this operation.

Write a scratch tool workbook under `.marqdo/agent-runs/tools/<name>.mq.md`.

The following record names each literal field without a JSON construction chain.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "a" | ".marqdo/agent-runs/tools/" |
| 2 | "b" | ".mq.md" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`a` = [Read the a field](json.get(value=`esc`, key="a"))**.
We retain **`b` = [Read the b field](json.get(value=`esc`, key="b"))**.
We retain **dir = ".marqdo/agent-runs/tools"**.
We **[Make dir](fs.make_dir(path=`dir`))**.
We retain **path = a + name + b**.
We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`text`))**.

`out` =

| ok | path |
|----|------|
| True | `path` |

Return *out*.

---

## skill_core_card

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Protected v2.1 Occlusion card always prepended to skill_brief (M05). Must not be truncated by budget. |

Protected v2.1 Occlusion card always prepended to skill_brief (M05). Must not be truncated by budget.

The card is kept as ordered lines so changes to any syntax instruction remain visible in review.

These rules precede the budgeted skill excerpt so essential marker meanings remain available when a long skill is shortened. The card describes the runtime surface, including legacy explicit parameters; new library authors use narrative parameters as required by the development guide. Keep both explanations consistent when evolving the language.

`_literal_core_lines` =

| line |
|---|
| "## CORE SYNTAX (v2.1 Occlusion)" |
| "1. Print with > print / > 打印 — never use bold markers to print." |
| "2. Bold markers = statement; italic markers = return." |
| "3. [desc](payload) = occlusion; brackets never execute." |
| "4. Read with access(coll, key) — never Classic [key](map) index." |
| "5. Foreach: - [desc](for coll as item). import + one language per file." |
| "6. # type · ## function · + params · numbered branches · - loops." |
| "" |

We preserve **_literal_core_text = [Join the protected instructions with their exact newlines](text.str_join(xs=`_literal_core_lines`, sep="\n"))**
and retain **esc = [Name the complete protected card](table.put(in=None, at="card", value=`_literal_core_text`))**.
Return *[Read the card field](json.get(value=`esc`, key="card"))*.

---

## skill_brief

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| max_chars | any | 1200 | Maximum retained characters in the text excerpt |
| depth | any | "default" | Requested context or discovery depth |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Budgeted marqdo skill text for prompts. Core Occlusion card is always included; remaining budget truncates the rest. `depth=deep` raises the char budget. |

Use `max_chars`=1200, `depth`="default" as the inputs for this operation.

Budgeted marqdo skill text for prompts. Core Occlusion card is always included; remaining budget truncates the rest. `depth=deep` raises the char budget.

1. `depth` == deep
  We retain **budget = 4000**.
2. *
  We retain **budget = max_chars**.
We retain **`core` = [Skill core card](skill_core_card())**.
We retain **`core_len` = [Len](len(value=`core`))**.
We retain **rest_budget = budget - core_len - 2**.
1. `rest_budget` < 200
  We retain **rest_budget = 200**.
2. *
  **_ = 1**
We retain **`skill` = [Marqdo skill](agent_marqdo_skill())**.
We retain **`full` = [Len](len(value=`skill`))**.
We retain **`rest` = [Text head lines](text_head_lines(text=`skill`, max_lines=80, max_chars=`rest_budget`))**.
Keep the literal separator as **esc = [Keep the exact separator](table.put(in=None, at="nl", value="\n\n"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **ex = core + nl + rest**.
We retain **`used` = [Len](len(value=`ex`))**.
1. `used` < `full` + `core_len`
  The following record names each literal field without a JSON construction chain.

  `_literal_esc2_fields` =

  | @ | key | value |
  |---|---|---|
  | 1 | "a" | "\n…[truncated skill_len=" |
  | 2 | "b" | " budget_chars=" |
  | 3 | "c" | "; emit READ:skill to deepen]" |

  We build **esc2 = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc2_fields`))**.


  We retain **`a` = [Read the a field](json.get(value=`esc2`, key="a"))**.
  We retain **`b` = [Read the b field](json.get(value=`esc2`, key="b"))**.
  We retain **`c` = [Read the c field](json.get(value=`esc2`, key="c"))**.
  Return *ex + a + full + b + budget + c*.
2. *
  Return *ex*.

---

## source_brief

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| depth | any | "default" | Requested context or discovery depth |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Budgeted current-module `.mq.md` source for `step` prompts. Default ~80 lines / 4KiB; `depth=deep` ~200 / 12KiB. |

Use `depth`="default" as the inputs for this operation.

Budgeted current-module `.mq.md` source for `step` prompts. Default ~80 lines / 4KiB; `depth=deep` ~200 / 12KiB.

We retain **`src` = [Module source](agent_module_source())**.
We retain **`full` = [Len](len(value=`src`))**.
1. `depth` == deep
  We retain **`ex` = [Workbook source excerpt](workbook_source_excerpt(source=`src`, max_lines=200, max_chars=12000))**.
  We retain **budget = 12000**.
2. *
  We retain **`ex` = [Workbook source excerpt](workbook_source_excerpt(source=`src`, max_lines=80, max_chars=4000))**.
  We retain **budget = 4000**.
We retain **`used` = [Len](len(value=`ex`))**.
1. `used` < `full`
  The following record names each literal field without a JSON construction chain.

  `_literal_esc_fields` =

  | @ | key | value |
  |---|---|---|
  | 1 | "a" | "\n…[truncated source_len=" |
  | 2 | "b" | " budget_chars=" |
  | 3 | "c" | "; emit READ:source to deepen]" |

  We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


  We retain **`a` = [Read the a field](json.get(value=`esc`, key="a"))**.
  We retain **`b` = [Read the b field](json.get(value=`esc`, key="b"))**.
  We retain **`c` = [Read the c field](json.get(value=`esc`, key="c"))**.
  Return *ex + a + full + b + budget + c*.
2. *
  Return *ex*.

---

## extract_plan_read

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse READ:source&#124;stderr&#124;stdout&#124;slots&#124;skill (Chinese: 读取：…). |

Use `reply` as the inputs for this operation.

Parse READ:source|stderr|stdout|slots|skill (Chinese: 读取：…).

The following record names each literal field without a JSON construction chain.

A READ request asks the orchestration loop for a bounded excerpt, rather than reading a file directly. The parser extracts the named context kind from a reply line. The caller still chooses which kinds it supports and how many additional turns it permits.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "fw" | "：" |
| 2 | "nl" | "\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`fw` = [Read the fw field](json.get(value=`esc`, key="fw"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **found = None**.
- [line](遍历 `lines` 为 `line`)
  We retain **`t` = [Trim](trim(value=`line`))**.
  We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
  We retain **`n` = [Len](len(value=`parts`))**.
  1. `n` > 1
    We retain **`head` = [At](at(value=`parts`, index=0))**.
    We retain **`head` = [Trim](trim(value=`head`))**.
    We retain **`rest` = [At](at(value=`parts`, index=1))**.
    We retain **`rest` = [Trim](trim(value=`rest`))**.
    1. `head` == READ
      We retain **found = rest**.
    2. `head` == 读取
      We retain **found = rest**.
    3. *
      **_ = 1**
  2. *
    We retain **`parts` = [Split](split(value=`t`, sep=`fw`))**.
    We retain **`n` = [Len](len(value=`parts`))**.
    1. `n` > 1
      We retain **`head` = [At](at(value=`parts`, index=0))**.
      We retain **`head` = [Trim](trim(value=`head`))**.
      We retain **`rest` = [At](at(value=`parts`, index=1))**.
      We retain **`rest` = [Trim](trim(value=`rest`))**.
      1. `head` == 读取
        We retain **found = rest**.
      2. `head` == READ
        We retain **found = rest**.
      3. *
        **_ = 1**
    2. *
      **_ = 1**
Return *found*.

---

## extract_plan_act

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Priority: CALL → READ → DECISION. Returns `{kind,name}`. |

Use `reply` as the inputs for this operation.

Priority: CALL → READ → DECISION. Returns `{kind,name}`.

We retain **`tool` = [Extract tool name](extract_tool_name(reply=`reply`))**.
1. `tool`

  `out` =

  | kind | name |
  |------|------|
  | call | `tool` |

  Return *out*.
2. *
  **_ = 1**
We retain **`rk` = [Extract plan read](extract_plan_read(reply=`reply`))**.
1. `rk`

  `out` =

  | kind | name |
  |------|------|
  | read | `rk` |

  Return *out*.
2. *
  **_ = 1**
We retain **`dec` = [Extract plan decision](extract_plan_decision(reply=`reply`))**.
1. `dec`

  `out` =

  | kind | name |
  |------|------|
  | decision | `dec` |

  Return *out*.
2. *

  `out` =

  | kind |
  |------|
  | unknown |

  Return *out*.

---

## extract_scratch_tool

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse NAME: id plus a triple-angle body after CALL:scratch_tool_write. |

Use `reply` as the inputs for this operation.

Parse NAME: id plus a triple-angle body after CALL:scratch_tool_write.

The following record names each literal field without a JSON construction chain.

The tool name and source body are returned as data. The body starts only at the explicit triple-angle opening line and ends at its matching closing line; surrounding commentary is excluded. This function does not write or run that source. The scratch writer remains responsible for destination checks, and the dispatcher still decides whether the requested tool is allowed.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "close" | ">>>" |
| 2 | "nl" | "\n" |
| 3 | "open" | "<<<" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`open` = [Read the open field](json.get(value=`esc`, key="open"))**.
We retain **`close` = [Read the close field](json.get(value=`esc`, key="close"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **name = None**.
We retain **`body_lines` = [Empty list](table.empty_list())**.
We retain **in_body = None**.
- [line](遍历 `lines` 为 `line`)
  1. `in_body`
    We retain **`t` = [Trim](trim(value=`line`))**.
    1. `t` == `close`
      We retain **in_body = None**.
    2. *
      We retain **`body_lines` = [Append this item to the new list](table.append(list=`body_lines`, item=`line`))**.
  2. *
    We retain **`t` = [Trim](trim(value=`line`))**.
    We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
    We retain **`n` = [Len](len(value=`parts`))**.
    1. `n` > 1
      We retain **`head` = [At](at(value=`parts`, index=0))**.
      We retain **`head` = [Trim](trim(value=`head`))**.
      We retain **`rest` = [At](at(value=`parts`, index=1))**.
      We retain **`rest` = [Trim](trim(value=`rest`))**.
      1. `head` == NAME
        We retain **name = rest**.
      2. `head` == 名称
        We retain **name = rest**.
      3. *
        **_ = 1**
    2. *
      **_ = 1**
    1. `t` == `open`
      We retain **in_body = 1**.
    2. *
      **_ = 1**
We retain **`body` = [Join](join(value=`body_lines`, sep=`nl`))**.

`out` =

| name | text |
|------|------|
| `name` | `body` |

Return *out*.

---

## run_parent_tool

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| observation | any | None | Tool observation or workbook result retained for the next turn |
| reply | any | None | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parent Plan-and-Move tools (helpers + whitelist `CALL:lib…`). |

Use `name`, `path`, `observation`=None, `reply`=None as the inputs for this operation.

Parent Plan-and-Move tools (helpers + whitelist `CALL:lib…`).

1. `name` == workbook_read
This dispatcher has a fixed set of parent operations. Workbook readers operate on the selected workbook path; catalog lookup returns data; scratch creation first extracts a name and body. The corpus and MCP-shaped evidence branches parse explicit ARGS records and report missing required fields before calling their adapters. Those fixture adapters are not a live MCP transport.

Arguments are deliberately read only after the value has been recognized as a record. Optional root, limit and argument fields are passed only when present, allowing the target helper to apply its own defaults. An unknown name reaches the restricted library dispatcher; it does not become an arbitrary function call. Editing this branch list therefore changes the parent tool surface and needs an allowlist regression, in addition to a prose update.

  Return *[Workbook read](workbook_read(path=`path`, depth="deep"))*.
2. `name` == workbook_excerpt
  Return *[Workbook excerpt](workbook_excerpt(path=`path`))*.
3. `name` == lib_catalog
  Return *[Lib catalog](lib_catalog())*.
4. `name` == scratch_tool_write
  We retain **`parsed` = [Extract scratch tool](extract_scratch_tool(reply=`reply`))**.
  We retain **`parsed_s` = [Encode the structured value as JSON text](json.stringify(value=`parsed`))**.
  We retain **`parsed_map` = [Starts with](text.starts_with(text=`parsed_s`, prefix="{"))**.
  We retain **tn = None**.
  We retain **tt = None**.
  1. `parsed_map`
    We retain **`tn` = [Read the name field](json.get(value=`parsed`, key="name"))**.
    We retain **`tt` = [Read the text field](json.get(value=`parsed`, key="text"))**.
  2. *
    **_ = 1**
  The following record names each literal field without a JSON construction chain.

  `_literal_errs_fields` =

  | @ | key | value |
  |---|---|---|
  | 1 | "a" | "scratch_tool_write needs NAME line and fenced body" |
  | 2 | "b" | "scratch_tool_write needs NAME id" |

  We build **errs = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_errs_fields`))**.


  1. `tn`
    1. `tt`
      Return *[Scratch tool write](scratch_tool_write(name=`tn`, text=`tt`))*.
    2. *
      Return *[Read the a field](json.get(value=`errs`, key="a"))*.
  2. *
    Return *[Read the b field](json.get(value=`errs`, key="b"))*.
5. `name` == corpus_search
  We retain **`args` = [Extract call args](extract_call_args(reply=`reply`))**.
  We retain **`args_s` = [Encode the structured value as JSON text](json.stringify(value=`args`))**.
  We retain **`args_map` = [Starts with](text.starts_with(text=`args_s`, prefix="{"))**.
  We retain **q = None**.
  We retain **root = None**.
  We retain **limit = None**.
  1. `args_map`
    We retain **`q` = [Read the query field](json.get(value=`args`, key="query"))**.
    We retain **`root` = [Read the root field](json.get(value=`args`, key="root"))**.
    We retain **`limit` = [Read the limit field](json.get(value=`args`, key="limit"))**.
  2. *
    **_ = 1**
  Keep the literal separator as **cerrs = [Keep the exact separator](table.put(in=None, at="q", value="corpus_search needs ARGS.query"))**.
  1. not `q`
    Return *[Read the q field](json.get(value=`cerrs`, key="q"))*.
  2. `root`
    1. `limit`
      Return *[Corpus search](corpus_search(query=`q`, root=`root`, limit=`limit`))*.
    2. *
      Return *[Corpus search](corpus_search(query=`q`, root=`root`))*.
  3. `limit`
    Return *[Corpus search](corpus_search(query=`q`, limit=`limit`))*.
  4. *
    Return *[Corpus search](corpus_search(query=`q`))*.
6. `name` == mcp_list_tools
  We retain **`args` = [Extract call args](extract_call_args(reply=`reply`))**.
  We retain **`args_s` = [Encode the structured value as JSON text](json.stringify(value=`args`))**.
  We retain **`args_map` = [Starts with](text.starts_with(text=`args_s`, prefix="{"))**.
  We retain **fx = None**.
  1. `args_map`
    We retain **`fx` = [Read the fixture field](json.get(value=`args`, key="fixture"))**.
  2. *
    **_ = 1**
  Keep the literal separator as **merrs = [Keep the exact separator](table.put(in=None, at="f", value="mcp_list_tools needs ARGS.fixture"))**.
  1. `fx`
    Return *[Mcp list tools](mcp_list_tools(fixture=`fx`))*.
  2. *
    Return *[Read the f field](json.get(value=`merrs`, key="f"))*.
7. `name` == mcp_call
  We retain **`args` = [Extract call args](extract_call_args(reply=`reply`))**.
  We retain **`args_s` = [Encode the structured value as JSON text](json.stringify(value=`args`))**.
  We retain **`args_map` = [Starts with](text.starts_with(text=`args_s`, prefix="{"))**.
  We retain **fx = None**.
  We retain **tn = None**.
  We retain **targ = None**.
  1. `args_map`
    We retain **`fx` = [Read the fixture field](json.get(value=`args`, key="fixture"))**.
    We retain **`tn` = [Read the name field](json.get(value=`args`, key="name"))**.
    We retain **`targ` = [Read the arguments field](json.get(value=`args`, key="arguments"))**.
  2. *
    **_ = 1**
  The following record names each literal field without a JSON construction chain.

  `_literal_xerrs_fields` =

  | @ | key | value |
  |---|---|---|
  | 1 | "f" | "mcp_call needs ARGS.fixture" |
  | 2 | "n" | "mcp_call needs ARGS.name" |

  We build **xerrs = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_xerrs_fields`))**.


  1. `fx`
    1. `tn`
      1. `targ`
        Return *[Mcp call](mcp_call(name=`tn`, fixture=`fx`, arguments=`targ`))*.
      2. *
        Return *[Mcp call](mcp_call(name=`tn`, fixture=`fx`))*.
    2. *
      Return *[Read the n field](json.get(value=`xerrs`, key="n"))*.
  2. *
    Return *[Read the f field](json.get(value=`xerrs`, key="f"))*.
8. *
  We retain **`dots` = [Split](split(value=`name`, sep=".*"))**.
  We retain **`head` = [At](at(value=`dots`, index=0))**.
  1. `head` == lib
    Return *[Run lib call](run_lib_call(name=`name`, reply=`reply`))*.
  2. *
    We retain **msg = "Parent tool not allowed:"**.
    Return *msg + name*.

---

## plan_read_deepen

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| observation | any | required | Tool observation or workbook result retained for the next turn |
| kind | any | required | Operation or factorization kind listed in this function's behavior |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply the requested source or skill depth increase before preparing another planning turn. |

Use `observation`, `kind`, `path` as the inputs for this operation.

We retain **obs = observation**.
1. `kind` == source
  We retain **`ex` = [Workbook read](workbook_read(path=`path`, depth="deep"))**.
  We retain **`obs` = [Set the source_excerpt field in the new record](table.put(in=`obs`, at="source_excerpt", value=`ex`))**.
  We retain **`obs` = [Set the read_source field in the new record](table.put(in=`obs`, at="read_source", value=True))**.
2. `kind` == stderr
  We retain **`err` = [Read the stderr field](json.get(value=`obs`, key="stderr"))**.
  We retain **`tail` = [Text tail lines](text_tail_lines(text=`err`, max_lines=80, max_chars=6000))**.
  We retain **`obs` = [Set the stderr_tail field in the new record](table.put(in=`obs`, at="stderr_tail", value=`tail`))**.
  We retain **`obs` = [Set the read_stderr field in the new record](table.put(in=`obs`, at="read_stderr", value=True))**.
3. `kind` == stdout
  We retain **`out` = [Read the stdout field](json.get(value=`obs`, key="stdout"))**.
  We retain **`tail` = [Text tail lines](text_tail_lines(text=`out`, max_lines=80, max_chars=6000))**.
  We retain **`obs` = [Set the stdout_tail field in the new record](table.put(in=`obs`, at="stdout_tail", value=`tail`))**.
  We retain **`obs` = [Set the read_stdout field in the new record](table.put(in=`obs`, at="read_stdout", value=True))**.
4. `kind` == slots
  We retain **`slots` = [Read the slots field](json.get(value=`obs`, key="slots"))**.
  We retain **`obs` = [Set the slots_detail field in the new record](table.put(in=`obs`, at="slots_detail", value=`slots`))**.
  We retain **`obs` = [Set the read_slots field in the new record](table.put(in=`obs`, at="read_slots", value=True))**.
5. `kind` == skill
  We retain **`brief` = [Skill brief](skill_brief(depth="deep"))**.
  We retain **`obs` = [Set the skill_brief field in the new record](table.put(in=`obs`, at="skill_brief", value=`brief`))**.
  We retain **`obs` = [Set the read_skill field in the new record](table.put(in=`obs`, at="read_skill", value=True))**.
6. *
  **_ = 1**
Return *obs*.

---

## compact_plan_observation

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| observation | any | required | Tool observation or workbook result retained for the next turn |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Bounded observation for the parent prompt (actionable, short). Tolerates missing/non-map observation (empty compact) instead of `json get needs map`. Map check via stringify prefix `{` — never hard-get on None / bare text / list. |

Use `observation` as the inputs for this operation.

Bounded observation for the parent prompt (actionable, short).
Tolerates missing/non-map observation (empty compact) instead of `json get needs map`.
Map check via stringify prefix `{` — never hard-get on None / bare text / list.

A child observation can contain a full workbook, process output and slot values. Sending all of that on every parent turn wastes context and can obscure the next decision. This projection retains the execution path and exit status, records source size, and carries selected excerpts or tails for repair. It returns a new record; it neither edits the workbook nor changes the original observation.

The source marker check is a structural hint: it looks for worker.step in the source text. That hint is not evidence that a model ran or that its answer is correct. Likewise, an exit status of zero is useful execution evidence, but a parent still needs a returned value and task-specific verification before DONE. The compact record's note communicates that decision protocol without granting the helper authority to declare success.

When an observation already supplies a source excerpt or an output tail, that bounded view is reused. Otherwise the existing excerpt and tail helpers derive it from the full field. This keeps the same repair window available without transporting every log line. Missing source, output or slots simply leave the corresponding optional information absent.

The initial record test protects None, ordinary text and lists from map-only access. A compact observation is therefore usable even after a child fails before producing its usual record. Consumers should read the available status and diagnostic fields instead of assuming that absence means an empty successful result. To investigate beyond these bounds, request an explicit deeper read through the parent protocol.

For example, a failed child may provide stderr and source but no value. Keep the diagnostic tail for repair and leave result absent; replacing the missing value with a successful placeholder would destroy that evidence.

We retain **obs_path = None**.
We retain **obs_code = None**.
We retain **obs_val = None**.
We retain **obs_slots = None**.
We retain **obs_src = None**.
We retain **obs_stdout = None**.
We retain **obs_stderr = None**.
We retain **`obs_s` = [Encode the structured value as JSON text](json.stringify(value=`observation`))**.
We retain **`obs_map` = [Starts with](text.starts_with(text=`obs_s`, prefix="{"))**.
1. `obs_map`
  We retain **`obs_path` = [Read the path field](json.get(value=`observation`, key="path"))**.
  We retain **`obs_code` = [Read the exit_code field](json.get(value=`observation`, key="exit_code"))**.
  We retain **`obs_val` = [Read the value field](json.get(value=`observation`, key="value"))**.
  We retain **`obs_slots` = [Read the slots field](json.get(value=`observation`, key="slots"))**.
  We retain **`obs_src` = [Read the source field](json.get(value=`observation`, key="source"))**.
  We retain **`obs_stdout` = [Read the stdout field](json.get(value=`observation`, key="stdout"))**.
  We retain **`obs_stderr` = [Read the stderr field](json.get(value=`observation`, key="stderr"))**.
2. *
  **_ = 1**
We retain **src_len = 0**.
We retain **has_step = False**.
1. `obs_src`
  We retain **`src_len` = [Len](len(value=`obs_src`))**.
  We retain **`step_parts` = [Split](split(value=`obs_src`, sep="worker.step"))**.
  We retain **`n_step` = [Len](len(value=`step_parts`))**.
  1. `n_step` > 1
    We retain **has_step = True**.
  2. *
    **_ = 1**
2. *
  **_ = 1**

`compact` =

| note | path | exit_code | source_len | has_worker_step | solidify_on_done |
|------|------|-----------|------------|-----------------|------------------|
| Plan-and-Move: CALL/READ then DECISION. Success (exit 0 + has_value) → DONE. solidify_on_done. No long prose. | `obs_path` | `obs_code` | `src_len` | `has_step` | True |

1. `obs_map`
  1. `obs_src`
    We retain **`ex` = [Read the source_excerpt field](json.get(value=`observation`, key="source_excerpt"))**.
    1. `ex`
      We retain **`compact` = [Set the source_excerpt field in the new record](table.put(in=`compact`, at="source_excerpt", value=`ex`))**.
    2. *
      We retain **`ex` = [Workbook source excerpt](workbook_source_excerpt(source=`obs_src`))**.
      We retain **`compact` = [Set the source_excerpt field in the new record](table.put(in=`compact`, at="source_excerpt", value=`ex`))**.
  2. *
    **_ = 1**

  1. `obs_stderr`
    We retain **`tail` = [Read the stderr_tail field](json.get(value=`observation`, key="stderr_tail"))**.
    1. `tail`
      We retain **`compact` = [Set the stderr_tail field in the new record](table.put(in=`compact`, at="stderr_tail", value=`tail`))**.
    2. *
      We retain **`tail` = [Text tail lines](text_tail_lines(text=`obs_stderr`))**.
      We retain **`compact` = [Set the stderr_tail field in the new record](table.put(in=`compact`, at="stderr_tail", value=`tail`))**.
  2. *
    **_ = 1**
  1. `obs_stdout`
    We retain **`tail` = [Read the stdout_tail field](json.get(value=`observation`, key="stdout_tail"))**.
    1. `tail`
      We retain **`compact` = [Set the stdout_tail field in the new record](table.put(in=`compact`, at="stdout_tail", value=`tail`))**.
    2. *
      We retain **`tail` = [Text tail lines](text_tail_lines(text=`obs_stdout`))**.
      We retain **`compact` = [Set the stdout_tail field in the new record](table.put(in=`compact`, at="stdout_tail", value=`tail`))**.
  2. *
    **_ = 1**

  We retain **`val_txt` = [Str](str(value=`obs_val`))**.
  We retain **`val_len` = [Len](len(value=`val_txt`))**.
  1. `obs_val`
    We retain **`compact` = [Set the has_value field in the new record](table.put(in=`compact`, at="has_value", value=True))**.
    We retain **`compact` = [Set the value_len field in the new record](table.put(in=`compact`, at="value_len", value=`val_len`))**.
    1. `val_len` > 200
      We retain **`preview` = [Text head lines](text_head_lines(text=`val_txt`, max_lines=4, max_chars=200))**.
      We retain **`compact` = [Set the value_preview field in the new record](table.put(in=`compact`, at="value_preview", value=`preview`))**.
    2. *
      We retain **`compact` = [Set the value field in the new record](table.put(in=`compact`, at="value", value=`obs_val`))**.
  2. *
    We retain **`compact` = [Set the has_value field in the new record](table.put(in=`compact`, at="has_value", value=False))**.

  We retain **`slots_brief` = [Empty list](table.empty_list())**.
  1. `obs_slots`
    - [slot](遍历 `obs_slots` 为 `slot`)
      We retain **`sk` = [Read the key field](json.get(value=`slot`, key="key"))**.
      We retain **`sl` = [Read the line field](json.get(value=`slot`, key="line"))**.
      We retain **`sb` = [Read the body field](json.get(value=`slot`, key="body"))**.
      We retain **`sb_txt` = [Str](str(value=`sb`))**.
      We retain **`sb_len` = [Len](len(value=`sb_txt`))**.

      `brief` =

      | key | line | body_len |
      |-----|------|----------|
      | `sk` | `sl` | `sb_len` |

      1. `sk` == error
        We retain **`prev` = [Text head lines](text_head_lines(text=`sb_txt`, max_lines=8, max_chars=400))**.
        We retain **`brief` = [Set the body_preview field in the new record](table.put(in=`brief`, at="body_preview", value=`prev`))**.
      2. *
        **_ = 1**
      We retain **`slots_brief` = [Append this item to the new list](table.append(list=`slots_brief`, item=`brief`))**.
  2. *
    **_ = 1**
  We retain **`compact` = [Set the slots field in the new record](table.put(in=`compact`, at="slots", value=`slots_brief`))**.
  We retain **`detail` = [Read the slots_detail field](json.get(value=`observation`, key="slots_detail"))**.
  1. `detail`
    We retain **`compact` = [Set the slots_detail field in the new record](table.put(in=`compact`, at="slots_detail", value=`detail`))**.
  2. *
    **_ = 1**

  We retain **`tool_result` = [Read the tool_result field](json.get(value=`observation`, key="tool_result"))**.
  1. `tool_result`
    We retain **`tr` = [Str](str(value=`tool_result`))**.
    We retain **`tr` = [Text head lines](text_head_lines(text=`tr`, max_lines=40, max_chars=3000))**.
    We retain **`compact` = [Set the tool_result field in the new record](table.put(in=`compact`, at="tool_result", value=`tr`))**.
  2. *
    **_ = 1**

  We retain **`last_error` = [Read the last_error field](json.get(value=`observation`, key="last_error"))**.
  1. `last_error`
    We retain **`err_txt` = [Str](str(value=`last_error`))**.
    We retain **`err_len` = [Len](len(value=`err_txt`))**.
    1. `err_len` > 200
      We retain **`prev` = [Text head lines](text_head_lines(text=`err_txt`, max_lines=8, max_chars=400))**.
      We retain **`compact` = [Set the last_error_preview field in the new record](table.put(in=`compact`, at="last_error_preview", value=`prev`))**.
      We retain **`compact` = [Set the last_error_len field in the new record](table.put(in=`compact`, at="last_error_len", value=`err_len`))**.
    2. *
      We retain **`compact` = [Set the last_error field in the new record](table.put(in=`compact`, at="last_error", value=`last_error`))**.
  2. *
    **_ = 1**
2. *
  We retain **`compact` = [Set the has_value field in the new record](table.put(in=`compact`, at="has_value", value=False))**.
  We retain **`empty_slots` = [Empty list](table.empty_list())**.
  We retain **`compact` = [Set the slots field in the new record](table.put(in=`compact`, at="slots", value=`empty_slots`))**.

Return *compact*.

---

## extract_plan_decision

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse `DECISION: DONE` / `CONTINUE` / `RUN` (Chinese: `决定：完成` / `继续` / `运行`). First matching DECISION / 决定 line wins (ignore later conflicting decisions in the same reply). |

Use `reply` as the inputs for this operation.

Parse `DECISION: DONE` / `CONTINUE` / `RUN` (Chinese: `决定：完成` / `继续` / `运行`).
First matching DECISION / 决定 line wins (ignore later conflicting decisions in the same reply).

The following record names each literal field without a JSON construction chain.

A decision is extracted from the model's protocol text and returned to the orchestration loop. It is not executed here. The loop must still distinguish RUN, CONTINUE and DONE and verify whether the workbook has a usable result.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "fw" | "：" |
| 2 | "nl" | "\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`fw` = [Read the fw field](json.get(value=`esc`, key="fw"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **found = None**.

- [line](遍历 `lines` 为 `line`)
  1. `found`
    **_ = 1**
  2. *
    We retain **`t` = [Trim](trim(value=`line`))**.
    We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
    We retain **`n` = [Len](len(value=`parts`))**.
    1. `n` > 1
      We retain **`head` = [At](at(value=`parts`, index=0))**.
      We retain **`head` = [Trim](trim(value=`head`))**.
      We retain **`rest` = [At](at(value=`parts`, index=1))**.
      We retain **`rest` = [Trim](trim(value=`rest`))**.
      1. `head` == DECISION
        We retain **found = rest**.
      2. `head` == 决定
        We retain **found = rest**.
      3. *
        **_ = 1**
    2. *
      We retain **`parts` = [Split](split(value=`t`, sep=`fw`))**.
      We retain **`n` = [Len](len(value=`parts`))**.
      1. `n` > 1
        We retain **`head` = [At](at(value=`parts`, index=0))**.
        We retain **`head` = [Trim](trim(value=`head`))**.
        We retain **`rest` = [At](at(value=`parts`, index=1))**.
        We retain **`rest` = [Trim](trim(value=`rest`))**.
        1. `head` == 决定
          We retain **found = rest**.
        2. `head` == DECISION
          We retain **found = rest**.
        3. *
          **_ = 1**
      2. *
        **_ = 1**

1. `found`
  Take the first token only (`DECISION: RUN please` → RUN).
  We retain **`toks` = [Split](split(value=`found`, sep=" "))**.
  We retain **`found` = [At](at(value=`toks`, index=0))**.
  We retain **`found` = [Trim](trim(value=`found`))**.
2. *
  **_ = 1**

1. `found` == DONE
  Return *"DONE"*.
2. `found` == CONTINUE
  Return *"CONTINUE"*.
3. `found` == RUN
  Return *"RUN"*.
4. `found` == REUSE
  Return *"REUSE"*.
5. `found` == NEW
  Return *"NEW"*.
6. `found` == 完成
  Return *"DONE"*.
7. `found` == 继续
  Return *"CONTINUE"*.
8. `found` == 运行
  Return *"RUN"*.
9. `found` == 复用
  Return *"REUSE"*.
10. `found` == 新建
  Return *"NEW"*.
11. `found` == PATCH
  Models sometimes invent `DECISION: PATCH` — treat as CONTINUE.
  Return *"CONTINUE"*.
12. *
  If the reply carries a FIND/REPLACE block, treat unknown labels as CONTINUE.
  We retain **`has_find` = [Split](split(value=`text`, sep="<<<"))**.
  We retain **`n_find` = [Len](len(value=`has_find`))**.
  1. `n_find` > 1
    Return *"CONTINUE"*.
  2. *
    Return *found*.

---

## extract_soft_match_slug

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse `SLUG: …` / `标识：…` after a soft-match REUSE decision. |

Use `reply` as the inputs for this operation.

Parse `SLUG: …` / `标识：…` after a soft-match REUSE decision.

The following record names each literal field without a JSON construction chain.

The returned slug identifies a candidate from the proposed knowledge entries. It is a lookup result rather than a filesystem authorization. The later route-application stage must resolve it against the knowledge directory before any workbook is executed.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "fw" | "：" |
| 2 | "nl" | "\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`fw` = [Read the fw field](json.get(value=`esc`, key="fw"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **found = None**.

- [line](遍历 `lines` 为 `line`)
  We retain **`t` = [Trim](trim(value=`line`))**.
  We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
  We retain **`n` = [Len](len(value=`parts`))**.
  1. `n` > 1
    We retain **`head` = [At](at(value=`parts`, index=0))**.
    We retain **`head` = [Trim](trim(value=`head`))**.
    We retain **`rest` = [At](at(value=`parts`, index=1))**.
    We retain **`rest` = [Trim](trim(value=`rest`))**.
    1. `head` == SLUG
      We retain **found = rest**.
    2. `head` == 标识
      We retain **found = rest**.
    3. *
      **_ = 1**
  2. *
    We retain **`parts` = [Split](split(value=`t`, sep=`fw`))**.
    We retain **`n` = [Len](len(value=`parts`))**.
    1. `n` > 1
      We retain **`head` = [At](at(value=`parts`, index=0))**.
      We retain **`head` = [Trim](trim(value=`head`))**.
      We retain **`rest` = [At](at(value=`parts`, index=1))**.
      We retain **`rest` = [Trim](trim(value=`rest`))**.
      1. `head` == 标识
        We retain **found = rest**.
      2. `head` == SLUG
        We retain **found = rest**.
      3. *
        **_ = 1**
    2. *
      **_ = 1**

Return *found*.

---

## build_soft_match_prompt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| goal | any | required | Human task goal represented by this operation |
| tasks | any | required | Ordered task records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Short parent prompt: REUSE+SLUG or NEW only. Candidates may include lexical `score` (prefer higher). `list_tasks` rows may also carry `description` / `status` / `llm_free`. |

Use `goal`, `tasks` as the inputs for this operation.

Short parent prompt: REUSE+SLUG or NEW only. Candidates may include lexical `score` (prefer higher). `list_tasks` rows may also carry `description` / `status` / `llm_free`.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

Candidate rows are evidence for a reuse choice, not executable instructions. Their slugs identify existing knowledge; titles and optional metadata help the parent compare task fit. A lexical score measures local text similarity, while llm_free describes whether the selected skill can run without a parent model. Neither alone establishes that its answer satisfies this new goal.

The output protocol deliberately permits only REUSE with a listed slug or NEW. The caller parses that decision and applies the selected resource through the ordinary route path. Building this prompt does not execute a candidate, alter its status, or add model-generated names to the tool whitelist.

`_literal_esc_c_lines` =

| line |
|---|
| "" |
| "" |
| "--- how to act ---" |
| "Exact reuse miss; candidates are ranked by local n-gram score when present. Prefer higher scores and llm_free=true skills. Reply with ONE protocol only:" |
| "DECISION: REUSE" |
| "SLUG: <exact-slug-from-list>" |
| "or" |
| "DECISION: NEW" |
| "No other prose." |
| "" |

Join exactly those lines into **_literal_esc_c_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_c_lines`, sep="\n"))**.

The following record names each literal field without a JSON construction chain.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "a" | "\n\n--- goal ---\n" |
| 2 | "b" | "\n\n--- candidate tasks (slug \| title \| meta) ---\n" |
| 3 | "c" | `_literal_esc_c_text` |
| 4 | "lf" | "llm_free=" |
| 5 | "nl" | "\n" |
| 6 | "sep" | " \| " |
| 7 | "sp" | "score=" |
| 8 | "st" | "status=" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`a` = [Read the a field](json.get(value=`esc`, key="a"))**.
We retain **`b` = [Read the b field](json.get(value=`esc`, key="b"))**.
We retain **`c` = [Read the c field](json.get(value=`esc`, key="c"))**.
We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`sep` = [Read the sep field](json.get(value=`esc`, key="sep"))**.
We retain **`sp` = [Read the sp field](json.get(value=`esc`, key="sp"))**.
We retain **`st` = [Read the st field](json.get(value=`esc`, key="st"))**.
We retain **`lf` = [Read the lf field](json.get(value=`esc`, key="lf"))**.
We retain **`lines` = [Empty list](table.empty_list())**.
- [t](遍历 `tasks` 为 `t`)
  We retain **`slug` = [Read the slug field](json.get(value=`t`, key="slug"))**.
  We retain **`title` = [Read the title field](json.get(value=`t`, key="title"))**.
  We retain **row = slug + sep + title**.
  We retain **`score` = [Read the score field](json.get(value=`t`, key="score"))**.
  1. `score`
    We retain **row = row + sep + sp + score**.
  2. *
    **_ = 1**
  We retain **`status` = [Read the status field](json.get(value=`t`, key="status"))**.
  1. `status`
    We retain **row = row + sep + st + status**.
  2. *
    **_ = 1**
  We retain **`free` = [Read the llm_free field](json.get(value=`t`, key="llm_free"))**.
  1. `free` == True
    We retain **row = row + sep + lf + "true"**.
  2. `free` == true
    We retain **row = row + sep + lf + "true"**.
  3. `free` == False
    We retain **row = row + sep + lf + "false"**.
  4. `free` == false
    We retain **row = row + sep + lf + "false"**.
  5. *
    **_ = 1**
  We retain **`desc` = [Read the description field](json.get(value=`t`, key="description"))**.
  1. `desc`
    We retain **row = row + sep + desc**.
  2. *
    **_ = 1**
  We retain **`lines` = [Append this item to the new list](table.append(list=`lines`, item=`row`))**.
We retain **`catalog` = [Join](join(value=`lines`, sep=`nl`))**.
We retain **p = "Soft-match: same task intent?"**.
Return *p + a + goal + b + catalog + c*.

---

## extract_plan_summary

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | First `SUMMARY:` / `汇总:` / `汇总：` line body, else trimmed reply. |

Use `reply` as the inputs for this operation.

First `SUMMARY:` / `汇总:` / `汇总：` line body, else trimmed reply.

The following record names each literal field without a JSON construction chain.

The summary is the short human-facing explanation attached to the plan result. It is kept separate from the decision so a sentence that happens to mention DONE does not itself finish a run. Source patches and tool requests are handled by their own extractors.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "fw" | "：" |
| 2 | "nl" | "\n" |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`nl` = [Read the nl field](json.get(value=`esc`, key="nl"))**.
We retain **`fw` = [Read the fw field](json.get(value=`esc`, key="fw"))**.
We retain **`text` = [Trim](trim(value=`reply`))**.
We retain **`lines` = [Split](split(value=`text`, sep=`nl`))**.
We retain **found = None**.

- [line](遍历 `lines` 为 `line`)
  We retain **`t` = [Trim](trim(value=`line`))**.
  We retain **`parts` = [Split](split(value=`t`, sep=":"))**.
  We retain **`n` = [Len](len(value=`parts`))**.
  1. `n` > 1
    We retain **`head` = [At](at(value=`parts`, index=0))**.
    We retain **`head` = [Trim](trim(value=`head`))**.
    We retain **`rest` = [At](at(value=`parts`, index=1))**.
    We retain **`rest` = [Trim](trim(value=`rest`))**.
    1. `head` == SUMMARY
      We retain **found = rest**.
    2. `head` == 汇总
      We retain **found = rest**.
    3. *
      **_ = 1**
  2. *
    We retain **`parts` = [Split](split(value=`t`, sep=`fw`))**.
    We retain **`n` = [Len](len(value=`parts`))**.
    1. `n` > 1
      We retain **`head` = [At](at(value=`parts`, index=0))**.
      We retain **`head` = [Trim](trim(value=`head`))**.
      We retain **`rest` = [At](at(value=`parts`, index=1))**.
      We retain **`rest` = [Trim](trim(value=`rest`))**.
      1. `head` == 汇总
        We retain **found = rest**.
      2. `head` == SUMMARY
        We retain **found = rest**.
      3. *
        **_ = 1**
    2. *
      **_ = 1**

1. `found`
  Return *found*.
2. *
  Return *text*.

---

## build_plan_context

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| agent | any | required | Agent configuration or receiver created by this package |
| goal | any | required | Human task goal represented by this operation |
| observation | any | required | Tool observation or workbook result retained for the next turn |
| explore_attempt | any | None | Current exploration attempt number |
| explore_n | any | None | Maximum configured exploration attempts |
| phase | any | "revise" | Named execution phase |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parent Plan-and-Move prompt. Short protocol only — no long monologues. |

Use `agent`, `goal`, `observation`, `explore_attempt`=None, `explore_n`=None, `phase`="revise" as the inputs for this operation.

Parent Plan-and-Move prompt. Short protocol only — no long monologues.

The parent receives a decision surface rather than the whole child process. Its goal is serialized as data, standing instructions supply the agent role, and a compact observation conveys the last execution outcome. If the observation carries a skill brief, that already selected context is reused; otherwise the normal budgeted skill brief is loaded. Source remains authoritative even when an evidence adapter contributes useful facts.

There are two instruction branches. Before the first run, the decompose phase checks whether the generated skeleton can be executed as written. It may request evidence, propose a short patch, or choose RUN. During revision, the parent sees the child's outcome and chooses whether to repair or finish. A successful child with a usable value can lead to DONE; the mere presence of a worker call is not a reason to keep rewriting the workbook.

The protocol text is stored in visible line tables. Literal examples of CALL, READ, DECISION and patch delimiters are instructions for a later model reply, not commands executed while assembling the prompt. Their exact line breaks matter to the corresponding extractors. Do not replace this representation with a second workflow language, or hide a new tool behind a descriptive label without wiring its implementation.

The parent-tools section explains the restricted workbook, source and evidence actions. An explicit library CALL uses the dispatcher's whitelist and argument parsing. READ requests deepen context through the caller's budget. Scratch tool creation returns to the ordinary source-and-tool path; a sentence in the prompt cannot bypass filesystem roots, runtime effects or ToolSpec preparation.

Exploration metadata tells the parent which attempt it is inspecting and how many variants the surrounding planner configured. It does not create a loop here. This function only returns text; the plan loop owns round accounting, child execution, patch application and writeback. When debugging a wrong decision, inspect the returned sections alongside the actual observation and call site before changing model instructions.

To extend a parent's capabilities, first implement a real dispatcher branch with its input contract and tests. Then update the visible tool instructions. Changing this prompt alone only changes what the model sees and cannot establish a callable implementation.

We retain **`standing` = [Read the standing field](json.get(value=`agent`, key="standing"))**.
1. `standing`
  We retain **up = standing**.
2. *
  We retain **up = None**.

We retain **skill_from_obs = None**.
We retain **`obs_probe` = [Encode the structured value as JSON text](json.stringify(value=`observation`))**.
We retain **`obs_is_map` = [Starts with](text.starts_with(text=`obs_probe`, prefix="{"))**.
1. `obs_is_map`
  We retain **`skill_from_obs` = [Read the skill_brief field](json.get(value=`observation`, key="skill_brief"))**.
2. *
  **_ = 1**
1. `skill_from_obs`
  We retain **skill = skill_from_obs**.
2. *
  We retain **`skill` = [Skill brief](skill_brief())**.
We retain **`compact` = [Compact plan observation](compact_plan_observation(observation=`observation`))**.
We retain **`obs_s` = [Encode the structured value as JSON text](json.stringify(value=`compact`))**.
We retain **`goal_s` = [Encode the structured value as JSON text](json.stringify(value=`goal`))**.
Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_e_dec_lines` =

| line |
|---|
| "" |
| "" |
| "--- how to act ---" |
| "PRE-RUN decompose. Reply with ONE protocol only (no long reasoning). Exactly one DECISION line." |
| "1) Skeleton OK → DECISION: RUN (preferred). Do not rewrite imports; they are already nested import: (llm/agent/…)." |
| "2) Need evidence → READ:source or READ:skill or CALL:workbook_read / lib_catalog / corpus_search / mcp_call." |
| "3) Reshape body/logic only → DECISION: CONTINUE + short PATCH (<20 lines)." |
| "4) Fixed answer without LLM → PATCH to return, then RUN or DONE." |
| "PATCH must use <<< FIND === REPLACE >>> only (no \x60\x60\x60find / Begin Patch)." |
| "Protocols:" |
| "CALL:name" |
| "READ:kind" |
| "DECISION: RUN" |
| "DECISION: DONE" |
| "SUMMARY: one line" |
| "DECISION: CONTINUE" |
| "" |

Join exactly those lines into **_literal_esc_e_dec_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_e_dec_lines`, sep="\n"))**.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_e_rev_lines` =

| line |
|---|
| "" |
| "" |
| "--- how to act ---" |
| "Plan-and-Move parent. Reply with ONE protocol only (no long reasoning). Exactly one DECISION line." |
| "1) exit_code=0 and has_value → DECISION: DONE + one-line SUMMARY. solidify_on_done. Do not CONTINUE just because has_worker_step." |
| "2) Need more evidence → READ:kind or CALL:tool (then you will be re-prompted)." |
| "3) Failure / wrong value → DECISION: CONTINUE + short PATCH (<20 lines). Never paste user prose into REPLACE." |
| "4) Do NOT CONTINUE only to rewrite frontmatter imports; skeleton already uses nested import: map (llm/agent/json…)." |
| "PATCH must use triple-angle blocks only:" |
| "<<<" |
| "FIND" |
| "<exact old>" |
| "===" |
| "REPLACE" |
| "<new>" |
| ">>>" |
| "Do not use \x60\x60\x60find, \x60\x60\x60replace, or *** Begin Patch ***." |
| "Protocols:" |
| "CALL:name" |
| "READ:kind" |
| "DECISION: DONE" |
| "SUMMARY: one line" |
| "DECISION: CONTINUE" |
| "" |

Join exactly those lines into **_literal_esc_e_rev_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_e_rev_lines`, sep="\n"))**.

Keep this text as ordered lines so its instructions and blank-line boundaries can be read directly.

`_literal_esc_tools_lines` =

| line |
|---|
| "" |
| "" |
| "--- parent tools ---" |
| "CALL:workbook_read \| workbook_excerpt \| lib_catalog \| scratch_tool_write" |
| "CALL:corpus_search \| mcp_list_tools \| mcp_call  (ARGS:{…}; evidence only — workbook authority)" |
| "CALL:lib.fs.read_text\|exists\|list_dir \| lib.json.parse\|stringify \| lib.sys.cwd (+ ARGS:{json})" |
| "READ:source \| stderr \| stdout \| slots \| skill" |
| "Create tools: CONTINUE PATCH add ##, or CALL:scratch_tool_write with NAME line and fenced body." |
| "" |

Join exactly those lines into **_literal_esc_tools_text = [Retain the original newline boundaries](text.str_join(xs=`_literal_esc_tools_lines`, sep="\n"))**.

The following record names each literal field without a JSON construction chain.

`_literal_esc_fields` =

| @ | key | value |
|---|---|---|
| 1 | "a" | "\n\n--- standing ---\n" |
| 2 | "b" | "\n\n--- goal ---\n" |
| 3 | "c" | "\n\n--- observation ---\n" |
| 4 | "d" | "\n\n--- skill brief ---\n" |
| 5 | "e_dec" | `_literal_esc_e_dec_text` |
| 6 | "e_rev" | `_literal_esc_e_rev_text` |
| 7 | "f" | "\n\n--- explore ---\n" |
| 8 | "tools" | `_literal_esc_tools_text` |

We build **esc = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_esc_fields`))**.


We retain **`a` = [Read the a field](json.get(value=`esc`, key="a"))**.
We retain **`b` = [Read the b field](json.get(value=`esc`, key="b"))**.
We retain **`c` = [Read the c field](json.get(value=`esc`, key="c"))**.
We retain **`d` = [Read the d field](json.get(value=`esc`, key="d"))**.
We retain **`tools` = [Read the tools field](json.get(value=`esc`, key="tools"))**.
We retain **`f` = [Read the f field](json.get(value=`esc`, key="f"))**.
1. `phase` == decompose
  We retain **`e` = [Read the e_dec field](json.get(value=`esc`, key="e_dec"))**.
  We retain **p = "Plan-and-Move: decompose before first run."**.
2. *
  We retain **`e` = [Read the e_rev field](json.get(value=`esc`, key="e_rev"))**.
  We retain **p = "Plan-and-Move: revise after child run."**.
We retain **p = p + a + up + b + goal_s + c + obs_s + tools + d + skill + e**.
1. `explore_attempt`
  We retain **p = p + f**.
  We retain **p = p + "Attempt"**.
  We retain **p = p + explore_attempt**.
  We retain **p = p + "of"**.
  We retain **p = p + explore_n**.
  We retain **p = p + ". Different path; prefer code when fixed."**.
2. *
  **_ = 1**
Return *p*.

---

## plan_llm_act_loop

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| agent | any | required | Agent configuration or receiver created by this package |
| goal | any | required | Human task goal represented by this operation |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| observation | any | required | Tool observation or workbook result retained for the next turn |
| phase | any | "revise" | Named execution phase |
| explore_attempt | any | None | Current exploration attempt number |
| explore_n | any | None | Maximum configured exploration attempts |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| events | any | required | Ordered semantic or provider stream-event records |
| max_acts | any | 6 | Maximum model/tool actions for this planning loop |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | LLM Plan → CALL/READ/DECISION loop. Returns `{decision,reply,observation,events,llm_calls,tokens}`. |

Use `agent`, `goal`, `path`, `observation`, `phase`="revise", `explore_attempt`=None, `explore_n`=None, `stream`=False, `echo`=False, `events`, `max_acts`=6 as the inputs for this operation.

LLM Plan → CALL/READ/DECISION loop. Returns `{decision,reply,observation,events,llm_calls,tokens}`.

Each model turn may answer directly, request context, or ask for one allowed parent action. The loop records the resulting observation and rebuilds context before asking again. Its bounded iteration prevents an endless READ or CALL exchange; it does not automatically authorize a tool mentioned by the model. Transport errors and exhausted budgets remain visible to the enclosing planner.

We retain **`model` = [Read the model field](json.get(value=`agent`, key="model"))**.
We retain **last_obs = observation**.
We retain **left = max_acts**.
We retain **decision = None**.
We retain **last_reply = None**.
We retain **evs_all = events**.
We retain **llm_calls = 0**.
We retain **tokens = 0**.

We retain **transport_err = None**.

- `left` > 0
  1. `decision`
    We retain **left = 0**.
  2. `transport_err`
    We retain **left = 0**.
  3. *
    We retain **`ctx` = [Build plan context](build_plan_context(agent=`agent`, goal=`goal`, observation=`last_obs`, explore_attempt=`explore_attempt`, explore_n=`explore_n`, phase=`phase`))**.
    Plan protocol streams as `stream_kind=plan` (process trail). Answer tokens
    still come from workbook `worker.step stream=True` (`stream_kind=delta`).
    1. `stream`
      We retain **`turn` = [Model turn](model_turn(model=`model`, prompt=`ctx`, stream=True, echo=`echo`, stream_kind="plan"))**.
    2. *
      We retain **`turn` = [Model turn](model_turn(model=`model`, prompt=`ctx`, stream=False, echo=`echo`))**.
    We retain **`turn_s` = [Encode the structured value as JSON text](json.stringify(value=`turn`))**.
    We retain **`turn_map` = [Starts with](text.starts_with(text=`turn_s`, prefix="{"))**.
    We retain **terr = None**.
    We retain **last_reply = ""**.
    1. not `turn_map`
      We retain **transport_err = "model_turn_not_map"**.
      We retain **left = 0**.
    2. *
      We retain **`terr` = [Read the error field](json.get(value=`turn`, key="error"))**.
      1. `terr`
        We retain **transport_err = `terr`**.
        We retain **left = 0**.
      2. *
        We retain **`last_reply` = [Read the text field](json.get(value=`turn`, key="text"))**.
        We retain **`c` = [Read the llm_calls field](json.get(value=`turn`, key="llm_calls"))**.
        We retain **`t` = [Read the tokens field](json.get(value=`turn`, key="tokens"))**.
        1. not `c`
          We retain **c = 0**.
        2. *
          **_ = 1**
        1. not `t`
          We retain **t = 0**.
        2. *
          **_ = 1**
        We retain **llm_calls = llm_calls + c**.
        We retain **tokens = tokens + t**.
    1. `transport_err`
      We retain **decision = "TRANSPORT_ERROR"**.
      We retain **last_reply = `transport_err`**.
      We retain **left = 0**.
    2. *
      1. not `last_reply`
        We retain **last_reply = ""**.
      2. *
        We retain **`last_reply` = [Trim](trim(value=`last_reply`))**.
      Plan replies often embed CALL: inside PATCH bodies — prefer an explicit
      DECISION line when present so protocol exit is not stolen by a false CALL.
      We retain **`dec_line` = [Extract plan decision](extract_plan_decision(reply=`last_reply`))**.
      We retain **kind = "unknown"**.
      We retain **name = None**.
      1. `dec_line`
        We retain **kind = "decision"**.
        We retain **name = `dec_line`**.
      2. *
        We retain **`act` = [Extract plan act](extract_plan_act(reply=`last_reply`))**.
        We retain **`act_s` = [Encode the structured value as JSON text](json.stringify(value=`act`))**.
        We retain **`act_map` = [Starts with](text.starts_with(text=`act_s`, prefix="{"))**.
        1. `act_map`
          We retain **`kind` = [Read the kind field](json.get(value=`act`, key="kind"))**.
          We retain **`name` = [Read the name field](json.get(value=`act`, key="name"))**.
        2. *
          **_ = 1**
      Live act marker so the UI never goes silent between plan tokens and the next step.
      1. `stream`

        `act_ev` =

        | type | kind | name |
        |------|------|------|
        | plan_act | `kind` | `name` |

        We **[Stream publish](sys.stream_publish(event=`act_ev`))**.
      2. *
        **_ = 1**
      1. `kind` == call
        We retain **`evs_all` = [Plan append tool](plan_append_tool(events=`evs_all`, type="tool_start", name=`name`, kind="call", stream=`stream`))**.
        We retain **`tool_out` = [Run parent tool](run_parent_tool(name=`name`, path=`path`, observation=`last_obs`, reply=`last_reply`))**.
        We retain **`brief` = [Tool result brief](tool_result_brief(value=`tool_out`))**.
        We retain **`evs_all` = [Plan append tool](plan_append_tool(events=`evs_all`, type="tool_end", name=`name`, kind="call", result=`brief`, stream=`stream`))**.
        We retain **`obs_s` = [Encode the structured value as JSON text](json.stringify(value=`last_obs`))**.
        We retain **`obs_map` = [Starts with](text.starts_with(text=`obs_s`, prefix="{"))**.
        1. `obs_map`
          We retain **`last_obs` = [Set the tool_result field in the new record](table.put(in=`last_obs`, at="tool_result", value=`tool_out`))**.
        2. *
          **_ = 1**
        We retain **left = left - 1**.
      2. `kind` == read
        We retain **`evs_all` = [Plan append tool](plan_append_tool(events=`evs_all`, type="tool_start", name=`name`, kind="read", stream=`stream`))**.
        We retain **`last_obs` = [Plan read deepen](plan_read_deepen(observation=`last_obs`, kind=`name`, path=`path`))**.
        We retain **`brief` = [Tool result brief](tool_result_brief(value=`last_obs`))**.
        We retain **`evs_all` = [Plan append tool](plan_append_tool(events=`evs_all`, type="tool_end", name=`name`, kind="read", result=`brief`, stream=`stream`))**.
        We retain **left = left - 1**.
      3. `kind` == decision
        We retain **decision = name**.
        We retain **`evs_all` = [Plan append decision](plan_append_decision(events=`evs_all`, decision=`decision`, stream=`stream`))**.
        We retain **left = 0**.
      4. *
        We retain **decision = "unknown"**.
        We retain **left = 0**.

`out` =

| decision | reply | observation | events | llm_calls | tokens |
|----------|-------|-------------|--------|-----------|--------|
| `decision` | `last_reply` | `last_obs` | `evs_all` | `llm_calls` | `tokens` |

1. `transport_err`
  We retain **`out` = [Set the error field in the new record](table.put(in=`out`, at="error", value=`transport_err`))**.
2. *
  **_ = 1**
Return *out*.

---

## plan_echo_decompose

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| workbook | any | required | Workbook path or structured workbook reference |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Pre-run parent decompose marker. With `stream=True`, publish `{type:decompose}` so view is not silent before the first child. |

Use `workbook`, `stream`=False, `echo`=False as the inputs for this operation.

Pre-run parent decompose marker. With `stream=True`, publish `{type:decompose}` so view is not silent before the first child.

1. `stream`

  `ev` =

  | type | workbook |
  |------|----------|
  | decompose | `workbook` |

  We **[Stream publish](sys.stream_publish(event=`ev`))**.
  1. `echo`
    We **[Print the prepared message](print(text="plan:decompose"))**.
  2. *
    **_ = 1**
  Return *ev*.
2. *
  Return *None*.

## plan_echo_await

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| round | any | required | Current planning round number |
| workbook | any | required | Workbook path or structured workbook reference |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Marker before `await_workbook`. With `stream=True`, publish `{type:await,round,workbook}` so the Stream panel shows progress while the quiet child runs (TTY `echo` optional). |

Use `round`, `workbook`, `stream`=False, `echo`=False as the inputs for this operation.

Marker before `await_workbook`. With `stream=True`, publish `{type:await,round,workbook}` so the Stream panel shows progress while the quiet child runs (TTY `echo` optional).

1. `stream`

  `ev` =

  | type | round | workbook |
  |------|-------|----------|
  | await | `round` | `workbook` |

  We **[Stream publish](sys.stream_publish(event=`ev`))**.
  1. `echo`
    We **[Print the prepared message](print(text="plan:await"))**.
    We **[Print the prepared message](print(text=`round`))**.
  2. *
    **_ = 1**
  Return *ev*.
2. *
  Return *None*.

## workbook_has_worker_step

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | True when source still contains `worker.step` or `.单步`. |

Use `path` as the inputs for this operation.

True when source still contains `worker.step` or `.单步`.

We retain **`src` = [Read UTF-8 text from the selected file](fs.read_text(path=`path`))**.
We retain **`a` = [Split](split(value=`src`, sep="worker.step"))**.
We retain **`na` = [Len](len(value=`a`))**.
We retain **`b` = [Split](split(value=`src`, sep=".单步"))**.
We retain **`nb` = [Len](len(value=`b`))**.
1. `na` > 1
  Return *1*.
2. `nb` > 1
  Return *1*.
3. *
  Return *0*.

## plan_append_round

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |
| round | any | required | Current planning round number |
| workbook | any | required | Workbook path or structured workbook reference |
| exit_code | any | required | Child process or tool exit status |
| result | any | None | Structured result from the preceding operation |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Always append a `round` event (child workbook finished) for audit / view. Optional `result` is the child `# main` return. SSE + optional TTY `plan:round` only when `stream` / `echo`. |

Use `events`, `round`, `workbook`, `exit_code`, `result`=None, `stream`=False, `echo`=False as the inputs for this operation.

Always append a `round` event (child workbook finished) for audit / view. Optional `result` is the child `# main` return. SSE + optional TTY `plan:round` only when `stream` / `echo`.

`ev` =

| type | round | workbook | exit_code |
|------|-------|----------|-----------|
| round | `round` | `workbook` | `exit_code` |

1. `result`
  We retain **`ev` = [Set the result field in the new record](table.put(in=`ev`, at="result", value=`result`))**.
2. *
  **_ = 1**
We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
1. `stream`
  We **[Stream publish](sys.stream_publish(event=`ev`))**.
2. *
  **_ = 1**
1. `echo`
  We **[Print the prepared message](print(text="plan:round"))**.
2. *
  **_ = 1**

Return *events*.

## plan_append_decision

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |
| decision | any | required | Structured planning decision |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| summary | any | None | Summary text or record from the preceding phase |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Always append a `decision` event. SSE publish only when `stream=True`. |

Use `events`, `decision`, `stream`=False, `summary`=None as the inputs for this operation.

Always append a `decision` event. SSE publish only when `stream=True`.

`ev` =

| type | decision |
|------|----------|
| decision | `decision` |

1. `summary`
  We retain **`ev` = [Set the summary field in the new record](table.put(in=`ev`, at="summary", value=`summary`))**.
2. *
  **_ = 1**
We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
1. `stream`
  We **[Stream publish](sys.stream_publish(event=`ev`))**.
2. *
  **_ = 1**

Return *events*.

## plan_merge_deltas

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |
| from | any | required | Source value of the transformation |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Bubble parent `complete stream=True` reasoning/deltas (and errors) into the plan event list. Skip nested `done` so plan owns the final `done`. Always merge into `events` when present; `stream` reserved for callers that also SSE-publish upstream. |

Use `events`, `from`, `stream`=False as the inputs for this operation.

Bubble parent `complete stream=True` reasoning/deltas (and errors) into the plan event list. Skip nested `done` so plan owns the final `done`. Always merge into `events` when present; `stream` reserved for callers that also SSE-publish upstream.

- [ev](遍历 `from` 为 `ev`)
  We retain **`t` = [Read the type field](json.get(value=`ev`, key="type"))**.
  1. `t` == delta
    We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
  2. `t` == reasoning
    We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
  3. `t` == error
    We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
  4. *
    **_ = 1**

Return *events*.

## plan_finish_stream

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| out | any | required | Structured output accumulated by the operation |
| events | any | required | Ordered semantic or provider stream-event records |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| trace | any | False | Whether to retain the documented execution trace |
| result | any | None | Structured result from the preceding operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Always append `done` and attach `events` on the result map (process audit even when not streaming). SSE publish of `done` only when `stream=True`. Optional `trace=True` writes the event list to writeback slot `trace`. |

Use `out`, `events`, `stream`=False, `trace`=False, `result`=None as the inputs for this operation.

Always append `done` and attach `events` on the result map (process audit even when not streaming). SSE publish of `done` only when `stream=True`. Optional `trace=True` writes the event list to writeback slot `trace`.

We retain **`ev` = [Set the type field in the new record](table.put(in=None, at="type", value="done"))**.
1. `result`
  We retain **`ev` = [Set the result field in the new record](table.put(in=`ev`, at="result", value=`result`))**.
2. *
  **_ = 1**
1. `events`
  We retain **`events` = [Append this item to the new list](table.append(list=`events`, item=`ev`))**.
2. *
  We retain **`events` = [Append this item to the new list](table.append(list=None, item=`ev`))**.
1. `stream`
  We **[Stream publish](sys.stream_publish(event=`ev`))**.
2. *
  **_ = 1**
1. not `out`
  We retain **`out` = [Empty map](table.empty_map())**.
2. *
  **_ = 1**
We retain **`out` = [Set the events field in the new record](table.put(in=`out`, at="events", value=`events`))**.

1. `trace`
  We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`events`))**.
  We **[Record](wb.record(value=`body`, key="trace"))**.
2. *
  **_ = 1**

Return *out*.

---

# agent

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | required | Model resource or provider model identifier, as required by this operation |
| tools | any | required | Explicit tool whitelist or tool-specification records |
| standing | any | required | Standing instructions supplied by the author |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load ABI v2 agent plugin once (session bag + context queries). |

Use `model`, `tools`, `standing` as the inputs for this operation.

Load ABI v2 agent plugin once (session bag + context queries).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

We retain **`id` = [Alloc](agent_alloc())**.

`h` =

| _type | id | model | tools |
|-------|-----|-------|-------|
| agent | `id` | `model` | `tools` |

1. `standing`
  We retain **`h` = [Set the standing field in the new record](table.put(in=`h`, at="standing", value=`standing`))**.
2. *
  **_ = 1**

Return *h*.

## clear_history

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Clear the plugin session bag for this agent id. |

Clear the plugin session bag for this agent id.

We retain **`id` = [Read the id field](json.get(value=`self`, key="id"))**.
We **[History clear](agent_history_clear(id=`id`))**.
Return *None*.

## resolve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| near_match | any | True | Whether to consider near-match candidates |
| near_threshold | any | 0.78 | Minimum similarity score for near-match candidates |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Skill resolver (v2): exact → alias → canonical → near. Returns `mode` / `resource` / `llm_free` / `confidence`. Plugin ABI — no core host_* calls. |

Use `task`, `kb_dir`=".marqdo/agent-kb", `near_match`=True, `near_threshold`=0.78 as the inputs for this operation.

Skill resolver (v2): exact → alias → canonical → near. Returns `mode` / `resource` / `llm_free` / `confidence`. Plugin ABI — no core host_* calls.

We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.
Return *[Resolve](agent_resolve(task=`task`, kb_dir=`kb_dir`, near_match=`near_match`, near_threshold=`near_threshold`, tools=`tools`))*.

## route

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| backend | any | "auto" | Configured execution or provider backend |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| threshold | any | 0.78 | Policy threshold required by this operation |
| step_kind | any | None | Class of workbook step |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Adaptive Routing (Phase 4): abstract router — `marqdo` &#124; `small-llm` &#124; `llm` &#124; `auto`. `jev` is accepted as a **compat alias** of `small-llm` (not a first-class backend). `auto` cascades policy → kb → small-llm request. May return `needs_llm=True` + `prompt` for the caller to complete with `router_model` (any llm handle), then `route_apply`. Optional `step_kind=` (O3) selects backend via module `## step_route` before the ABI call. |

Use `task`, `backend`="auto", `kb_dir`=".marqdo/agent-kb", `memory_dir`=".marqdo/agent-memory", `threshold`=0.78, `step_kind`=None as the inputs for this operation.

Adaptive Routing (Phase 4): abstract router — `marqdo` | `small-llm` | `llm` | `auto`. `jev` is accepted as a **compat alias** of `small-llm` (not a first-class backend). `auto` cascades policy → kb → small-llm request. May return `needs_llm=True` + `prompt` for the caller to complete with `router_model` (any llm handle), then `route_apply`.
Optional `step_kind=` (O3) selects backend via module `## step_route` before the ABI call.

1. `step_kind`
  We retain **`sr` = [Step route](step_route(kind=`step_kind`))**.
  We retain **`backend` = [Read the backend field](json.get(value=`sr`, key="backend"))**.
2. *
  **_ = 1**

We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.
Return *[Route](agent_route(task=`task`, backend=`backend`, kb_dir=`kb_dir`, memory_dir=`memory_dir`, threshold=`threshold`, tools=`tools`))*.

## route_apply

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| reply | any | required | Model reply to parse as data; it is not permission to execute arbitrary code |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| backend | any | "small-llm" | Configured execution or provider backend |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply a router model line (`SKILL:<slug>` / `EXPLORE`) to a kb skill hit. |

Use `task`, `reply`, `kb_dir`=".marqdo/agent-kb", `backend`="small-llm" as the inputs for this operation.

Apply a router model line (`SKILL:<slug>` / `EXPLORE`) to a kb skill hit.

Return *[Route apply](agent_route_apply(task=`task`, reply=`reply`, kb_dir=`kb_dir`, backend=`backend`))*.

## compile_policy

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| min_evidence | any | 3 | Minimum evidence count required before promotion |
| force | any | False | Explicit override supported by this operation; it does not bypass host authorization |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Policy Compilation: stable episode skill selections → `policies/router.mq.md` (deterministic Marqdo rules). |

Use `memory_dir`=".marqdo/agent-memory", `min_evidence`=3, `force`=False as the inputs for this operation.

Policy Compilation: stable episode skill selections → `policies/router.mq.md` (deterministic Marqdo rules).

Return *[Compile policy](agent_compile_policy(memory_dir=`memory_dir`, min_evidence=`min_evidence`, force=`force`))*.

## maybe_compile_policy

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| min_evidence | any | 3 | Minimum evidence count required before promotion |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Learn gate for the Adaptive Router policy layer. |

Use `memory_dir`=".marqdo/agent-memory", `min_evidence`=3 as the inputs for this operation.

Learn gate for the Adaptive Router policy layer.

Return *[Maybe compile policy](agent_maybe_compile_policy(memory_dir=`memory_dir`, min_evidence=`min_evidence`))*.

## record_episode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| status | any | "ok" | Domain status recorded in the structured result |
| result | any | None | Structured result from the preceding operation |
| mode | any | "explore" | Operation mode listed in the function behavior |
| skill | any | "none" | Skill source or identifier selected for inspection |
| observation | any | None | Tool observation or workbook result retained for the next turn |
| workbook | any | None | Workbook path or structured workbook reference |
| cache | any | None | Whether to use the documented result cache |
| match | any | None | Resolved match or match policy |
| llm_calls | any | 0 | Observed model-call count |
| tool_calls | any | 0 | Observed tool-call count |
| error | any | None | Structured failure information supplied by the caller |
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Persist one run as `.mq.md` under `memory_dir/episodes/YYYY/MM/` (v2 Phase 1). Memory is executable documentation, not a vector bag. |

Use `task`, `status`="ok", `result`=None, `mode`="explore", `skill`="none", `observation`=None, `workbook`=None, `cache`=None, `match`=None, `llm_calls`=0, `tool_calls`=0, `error`=None, `memory_dir`=".marqdo/agent-memory" as the inputs for this operation.

Persist one run as `.mq.md` under `memory_dir/episodes/YYYY/MM/` (v2 Phase 1). Memory is executable documentation, not a vector bag.

Return *[Record episode](agent_record_episode(task=`task`, status=`status`, result=`result`, mode=`mode`, skill=`skill`, observation=`observation`, workbook=`workbook`, cache=`cache`, match=`match`, llm_calls=`llm_calls`, tool_calls=`tool_calls`, error=`error`, memory_dir=`memory_dir`))*.

## quarantine

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| slug | any | required | Knowledge entry identifier within the configured knowledge directory |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| reason | any | "sandbox_or_harness_failed" | Human-readable explanation for the recorded decision |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | O5: mark OKF skill `status: quarantine` and force `llm_free: false`. Regression fixtures stay with the slug. |

Use `slug`, `kb_dir`=".marqdo/agent-kb", `reason`="sandbox_or_harness_failed" as the inputs for this operation.

O5: mark OKF skill `status: quarantine` and force `llm_free: false`. Regression fixtures stay with the slug.

Return *[Kb quarantine](agent_kb_quarantine(kb_dir=`kb_dir`, slug=`slug`, reason=`reason`))*.

## maybe_learn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| improve_every | any | 3 | Number of observations between improvement attempts |
| promote | any | True | Whether successful evidence may be promoted |
| candidate_only | any | False | Whether to retain a candidate without publishing it |
| product_gate_ok | any | True | Result of the external product gate |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Offline learn gate: when ≥ `improve_every` successful episodes share a task hash, generalize → synthesize → compile → optional OKF promote (`llm_free`). Also attempts Policy Compilation for Adaptive Routing. O2: promote path requires sandbox proof unless `require_proof=false`. M04: `candidate_only=True` or `product_gate_ok=False` forces promote off via eng_publish_gate. |

Use `task`, `kb_dir`=".marqdo/agent-kb", `memory_dir`=".marqdo/agent-memory", `improve_every`=3, `promote`=True, `candidate_only`=False, `product_gate_ok`=True as the inputs for this operation.

Offline learn gate: when ≥ `improve_every` successful episodes share a task hash, generalize → synthesize → compile → optional OKF promote (`llm_free`). Also attempts Policy Compilation for Adaptive Routing.
O2: promote path requires sandbox proof unless `require_proof=false`.
M04: `candidate_only=True` or `product_gate_ok=False` forces promote off via eng_publish_gate.

We retain **`gate` = [Eng publish gate](> `self`.eng_publish_gate project="." candidate_only=`candidate_only` product_gate_ok=`product_gate_ok` )**.
We retain **`may_prom` = [Read the may_promote field](json.get(value=`gate`, key="may_promote"))**.
1. not `may_prom`
  We retain **promote = False**.
2. *
  **_ = 1**
We retain **`skill_learn` = [Maybe learn](agent_maybe_learn(task=`task`, kb_dir=`kb_dir`, memory_dir=`memory_dir`, improve_every=`improve_every`, promote=`promote`))**.
We retain **`policy_learn` = [Maybe compile policy](agent_maybe_compile_policy(memory_dir=`memory_dir`, min_evidence=`improve_every`))**.
We retain **`sl` = [Read the learned field](json.get(value=`skill_learn`, key="learned"))**.

`out` =

| learned | skill | policy | promote_allowed | gate |
|---------|-------|--------|-----------------|------|
| `sl` | `skill_learn` | `policy_learn` | `may_prom` | `gate` |

Return *out*.

## compile

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| min_evidence | any | 3 | Minimum evidence count required before promotion |
| promote | any | True | Whether successful evidence may be promoted |
| force | any | False | Explicit override supported by this operation; it does not bypass host authorization |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Compile a candidate skill into versioned `skills/<slug>/vN.mq.md` and optionally promote into agent-kb. |

Use `task`, `kb_dir`=".marqdo/agent-kb", `memory_dir`=".marqdo/agent-memory", `min_evidence`=3, `promote`=True, `force`=False as the inputs for this operation.

Compile a candidate skill into versioned `skills/<slug>/vN.mq.md` and optionally promote into agent-kb.

Return *[Compile skill](agent_compile_skill(task=`task`, kb_dir=`kb_dir`, memory_dir=`memory_dir`, min_evidence=`min_evidence`, promote=`promote`, force=`force`))*.

## metrics

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Reasoning amortization stats over recorded episodes (`avg_llm_calls`, `compiled_runs`, …). |

Use `memory_dir`=".marqdo/agent-memory" as the inputs for this operation.

Reasoning amortization stats over recorded episodes (`avg_llm_calls`, `compiled_runs`, …).

Return *[Metrics](agent_metrics(memory_dir=`memory_dir`))*.

## run

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| memory_dir | any | ".marqdo/agent-memory" | Directory holding learned episodes or policy memory |
| learn | any | True | Whether to record or learn from this operation |
| verify | any | True | Whether to run the documented verifier |
| writeback | any | True | Whether to write the observation back to its author document |
| reuse | any | True | Whether to search for reusable knowledge before new work |
| near_match | any | True | Whether to consider near-match candidates |
| near_threshold | any | 0.78 | Minimum similarity score for near-match candidates |
| promote | any | True | Whether successful evidence may be promoted |
| improve_every | any | 3 | Number of observations between improvement attempts |
| explore_n | any | 3 | Maximum configured exploration attempts |
| max_rounds | any | 4 | Maximum workbook planning rounds |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| trace | any | False | Whether to retain the documented execution trace |
| force | any | False | Explicit override supported by this operation; it does not bypass host authorization |
| router | any | "auto" | Router configuration or route-policy resource |
| router_model | any | None | Model resource used for routing |
| resume_id | any | None | Identifier of the stored continuation checkpoint |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |
| sandbox | any | False | Whether to use the CoW trial path |
| sandbox_profile | any | "default" | Capability profile for CoW execution |
| efi | any | True | Structured effect and failure information |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Primary author entry (v2 + P4): Adaptive `route` → execute (policy/kb skill) or `plan` → record episode → maybe_learn (skill + policy). `router` = `auto`&#124;`marqdo`&#124;`small-llm`&#124;`llm` (`jev`→`small-llm` alias). Optional `router_model` (llm handle) answers `needs_llm` prompts — small models welcome. |

Use `task`, `kb_dir`=".marqdo/agent-kb", `memory_dir`=".marqdo/agent-memory", `learn`=True, `verify`=True, `writeback`=True, `reuse`=True, `near_match`=True, `near_threshold`=0.78, `promote`=True, `improve_every`=3, `explore_n`=3, `max_rounds`=4, `stream`=False, `echo`=False, `trace`=False, `force`=False, `router`="auto", `router_model`=None, `resume_id`=None, `resume_dir`=".marqdo/agent-resume", `sandbox`=False, `sandbox_profile`="default", `efi`=True as the inputs for this operation.

Primary author entry (v2 + P4): Adaptive `route` → execute (policy/kb skill) or `plan` → record episode → maybe_learn (skill + policy). `router` = `auto`|`marqdo`|`small-llm`|`llm` (`jev`→`small-llm` alias). Optional `router_model` (llm handle) answers `needs_llm` prompts — small models welcome.
`sandbox=True` (O1 opt-in): workbook paths run via CoW trial before host commit. `efi=True` (O3): EKC preflight before explore; REUSE short-circuits. Pass `efi=False` to keep legacy golds that skip eng knowledge.

Use this entry when the application wants the full reuse-and-learning lifecycle. A single conversational turn belongs in step; run first tries existing executable knowledge and can fall back to a workbook plan. The output records which execution mode actually supplied the result, allowing callers to distinguish compiled reuse, router-assisted reuse and exploration instead of inferring that distinction from answer text.

Routing precedes exploration. A matched resource is awaited as a workbook; a failed child does not count as a reusable success. When a route needs a model, the configured routing model, or the agent's normal model, produces the restricted selection reply. Applying that reply resolves it to an actual knowledge resource. Model words alone never stand in for the child's returned value.

Engineering preflight supplies an additional reuse gate. A REUSE result can bind and execute an existing engineering capability; a refusal to create is returned as an error rather than ignored. Missing engineering knowledge is distinguished from a positive refusal. The lower resolver then checks reusable skills, including whether an entry is marked llm_free, before a new plan is requested. Preserve this order when extending the method so new capabilities do not quietly skip existing knowledge.

The workbook await helper owns the execution boundary. With sandbox enabled it uses the configured CoW path; attaching a profile name to an answer is insufficient. Native plugin loading and explicit runtime grants are still required by the selected resource. A successful reused child can therefore avoid parent model calls, while an exploratory result carries the usage reported by its plan.

After execution, the method extracts task status, result, skill and observation for the episode record. Verification here is a recorded execution-status check; it is not an independent domain proof of the answer. Applications with stronger requirements must validate the returned value using their own task-specific assertions. Do not interpret a status string as evidence that files, network writes or generated source were reviewed.

Episodes persist the observed outcome for later reuse decisions. Learning is attempted only for successful outcomes when enabled, and promotion follows the existing evidence path. The result attaches execution details, model usage, routing information and the episode reference. An optional resume identifier also saves a checkpoint and effect-ledger watermark. These records describe what happened; they do not make an uncertain external effect safe to repeat.

For a reproducible investigation, retain the returned workbook reference and observation together. The same goal text can resolve differently after knowledge promotion, so a later rerun of the request is not a substitute for examining the recorded resource.

We retain **efi_pf = None**.
1. `efi`
  We retain **`efi_pf` = [Preflight](> `self`.preflight task=`task` )**.
2. *
  **_ = 1**

We retain **`routed` = [Route](> `self`.route task=`task` backend=`router` kb_dir=`kb_dir` memory_dir=`memory_dir` threshold=`near_threshold` )**.
We retain **`route_matched` = [Read the matched field](json.get(value=`routed`, key="matched"))**.
We retain **`needs_llm` = [Read the needs_llm field](json.get(value=`routed`, key="needs_llm"))**.
We retain **exec_mode = "explore"**.
We retain **llm_calls = 0**.
We retain **tokens = 0**.
We retain **token_cost = 0**.
We retain **out = None**.
We retain **`route_backend` = [Read the backend field](json.get(value=`routed`, key="backend"))**.

1. `route_matched`
  We retain **`path` = [Read the resource field](json.get(value=`routed`, key="resource"))**.
  1. `path`
    We retain **`aw` = [Await workbook](await_workbook(path=`path`, sandbox=`sandbox`, sandbox_profile=`sandbox_profile`))**.
    We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
    We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
    We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
    1. `code` == 0
      We **[Kb record hit](agent_kb_record_hit(kb_dir=`kb_dir`, goal=`task`, improve_every=`improve_every`))**.
      We retain **`lf` = [Read the llm_free field](json.get(value=`routed`, key="llm_free"))**.
      1. `lf`
        We retain **exec_mode = "compiled"**.
      2. *
        We retain **exec_mode = "routed"**.
      We retain **`rmode` = [Read the mode field](json.get(value=`routed`, key="mode"))**.
      We retain **`sk` = [Read the skill field](json.get(value=`routed`, key="skill"))**.
      **out = **

      | status | goal | result | workbook | cache | match | skill | observation | summary | route |
      |--------|------|--------|----------|-------|-------|-------|-------------|---------|-------|
      | ok | `task` | `child_val` | `path` | hit | `rmode` | `sk` | `last_obs` | adaptive route hit | `routed` |

      We retain **llm_calls = 0**.
      1. `stream`
        We retain **`_ev` = [Empty list](table.empty_list())**.
        We retain **`_ev` = [Plan append decision](plan_append_decision(events=`_ev`, decision="REUSE", stream=True, summary="adaptive route hit"))**.
        We retain **`_ev` = [Plan append round](plan_append_round(events=`_ev`, round=1, workbook=`path`, exit_code=`code`, result=`child_val`, stream=True))**.
      2. *
        **_ = 1**
    2. *
      We retain **route_matched = False**.
  2. *
    We retain **route_matched = False**.
2. *
  **_ = 1**

1. `route_matched`
  **_ = 1**
2. `needs_llm`
  1. `router_model`
    We retain **`prompt` = [Read the prompt field](json.get(value=`routed`, key="prompt"))**.
    We retain **`turn` = [Model turn](model_turn(model=`router_model`, prompt=`prompt`, stream=`stream`, echo=`echo`))**.
    We retain **`reply` = [Read the text field](json.get(value=`turn`, key="text"))**.
    We retain **`llm_calls` = [Read the llm_calls field](json.get(value=`turn`, key="llm_calls"))**.
    We retain **`tokens` = [Read the tokens field](json.get(value=`turn`, key="tokens"))**.
    We retain **`applied` = [Route apply](> `self`.route_apply task=`task` reply=`reply` kb_dir=`kb_dir` backend=`router` )**.
    We retain **`am` = [Read the matched field](json.get(value=`applied`, key="matched"))**.
    1. `am`
      We retain **`path` = [Read the resource field](json.get(value=`applied`, key="resource"))**.
      We retain **`aw` = [Await workbook](await_workbook(path=`path`, sandbox=`sandbox`, sandbox_profile=`sandbox_profile`))**.
      We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
      We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
      We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
      1. `code` == 0
        We retain **exec_mode = "routed"**.
        We retain **`sk` = [Read the skill field](json.get(value=`applied`, key="skill"))**.
        **out = **

        | status | goal | result | workbook | cache | match | skill | observation | summary | route |
        |--------|------|--------|----------|-------|-------|-------|-------------|---------|-------|
        | ok | `task` | `child_val` | `path` | soft-hit | routed | `sk` | `last_obs` | router model selected skill | `applied` |

        We retain **route_matched = True**.
      2. *
        We retain **route_matched = False**.
    2. *
      We retain **route_matched = False**.
  2. *
    We retain **route_matched = False**.
3. *
  **_ = 1**

1. `route_matched`
  **_ = 1**
2. *
  1. `efi_pf`
    We retain **`efi_st` = [Read the status field](json.get(value=`efi_pf`, key="status"))**.
    We retain **`efi_dec` = [Read the decision field](json.get(value=`efi_pf`, key="decision"))**.
    We retain **`efi_allow` = [Read the create_allowed field](json.get(value=`efi_pf`, key="create_allowed"))**.
    1. `efi_st` == "missing_knowledge"
      **_ = 1**
    2. `efi_dec` == "REUSE"
      We retain **`efi_exec` = [Eng bind execute](agent_eng_bind_execute(project=".", query=`task`, policy="pure", candidate_only=False))**.
      We retain **`efi_ok` = [Read the ok field](json.get(value=`efi_exec`, key="ok"))**.
      1. `efi_ok`
        We retain **exec_mode = "efi_bound"**.
        **out = **

        | status | goal | result | summary | efi | reuse_trace |
        |--------|------|--------|---------|-----|-------------|
        | ok | `task` | `efi_exec` | EFI REUSE bound+executed | `efi_pf` | `efi_exec` |

      2. *
        **_ = 1**
    3. `efi_dec` == "ADAPT"
      **_ = 1**
    4. `efi_allow`
      **_ = 1**
    5. *
      **out = **

      | status | goal | error | summary | efi |
      |--------|------|-------|---------|-----|
      | error | `task` | efi_create_denied | EKC preflight denied CREATE | `efi_pf` |

      We retain **exec_mode = "efi"**.
  2. *
    **_ = 1**

  1. `out`
    **_ = 1**
  2. *
  We retain **`resolved` = [Resolve](> `self`.resolve task=`task` kb_dir=`kb_dir` near_match=`near_match` near_threshold=`near_threshold` )**.
  We retain **`mode` = [Read the mode field](json.get(value=`resolved`, key="mode"))**.
  We retain **`llm_free` = [Read the llm_free field](json.get(value=`resolved`, key="llm_free"))**.
  1. not `llm_free`
    We retain **llm_free = False**.
  1. `llm_free`
    We retain **`path` = [Read the resource field](json.get(value=`resolved`, key="resource"))**.
    We retain **`aw` = [Await workbook](await_workbook(path=`path`, sandbox=`sandbox`, sandbox_profile=`sandbox_profile`))**.
    We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
    We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
    We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
    1. `code` == 0
      We **[Kb record hit](agent_kb_record_hit(kb_dir=`kb_dir`, goal=`task`, improve_every=`improve_every`))**.
      We retain **exec_mode = "compiled"**.
      We retain **`sk` = [Read the skill field](json.get(value=`resolved`, key="skill"))**.
      **out = **

      | status | goal | result | workbook | cache | match | skill | observation | summary |
      |--------|------|--------|----------|-------|-------|-------|-------------|--------|
      | ok | `task` | `child_val` | `path` | hit | `mode` | `sk` | `last_obs` | compiled skill (llm_free) |

      We retain **llm_calls = 0**.
    2. *
      We retain **`out` = [Plan](> `self`.plan goal=`task` kb_dir=`kb_dir` writeback=`writeback` reuse=`reuse` near_match=`near_match` near_threshold=`near_threshold` promote=`promote` improve_every=`improve_every` explore_n=`explore_n` max_rounds=`max_rounds` stream=`stream` echo=`echo` trace=`trace` force=`force` )**.
      We retain **exec_mode = "explore"**.
      We retain **`llm_calls` = [Read the llm_calls field](json.get(value=`out`, key="llm_calls"))**.
      We retain **`tokens` = [Read the tokens field](json.get(value=`out`, key="tokens"))**.
      1. `llm_calls` == None
        We retain **llm_calls = 1**.
      1. `tokens` == None
        We retain **tokens = 0**.
  2. *
    We retain **`out` = [Plan](> `self`.plan goal=`task` kb_dir=`kb_dir` writeback=`writeback` reuse=`reuse` near_match=`near_match` near_threshold=`near_threshold` promote=`promote` improve_every=`improve_every` explore_n=`explore_n` max_rounds=`max_rounds` stream=`stream` echo=`echo` trace=`trace` force=`force` )**.
    We retain **`cache` = [Read the cache field](json.get(value=`out`, key="cache"))**.
    1. `cache` == "hit"
      We retain **exec_mode = "compiled"**.
      We retain **llm_calls = 0**.
      We retain **tokens = 0**.
    2. `cache` == "soft-hit"
      We retain **exec_mode = "near"**.
      We retain **llm_calls = 0**.
      We retain **tokens = 0**.
    3. *
      We retain **exec_mode = "explore"**.
      We retain **`llm_calls` = [Read the llm_calls field](json.get(value=`out`, key="llm_calls"))**.
      We retain **`tokens` = [Read the tokens field](json.get(value=`out`, key="tokens"))**.
      1. `llm_calls` == None
        We retain **llm_calls = 1**.
      1. `tokens` == None
        We retain **tokens = 0**.

We retain **`st` = [Read the status field](json.get(value=`out`, key="status"))**.
We retain **`res` = [Read the result field](json.get(value=`out`, key="result"))**.
We retain **`wb_path` = [Read the workbook field](json.get(value=`out`, key="workbook"))**.
We retain **`obs` = [Read the observation field](json.get(value=`out`, key="observation"))**.
We retain **`sk2` = [Read the skill field](json.get(value=`out`, key="skill"))**.
We retain **`cache2` = [Read the cache field](json.get(value=`out`, key="cache"))**.
We retain **`match2` = [Read the match field](json.get(value=`out`, key="match"))**.
We retain **`err` = [Read the error field](json.get(value=`out`, key="error"))**.

1. `verify`
  1. `st` == "ok"
    We retain **verification = "status: passed"**.
  2. *
    We retain **verification = "status: failed"**.
2. *
  We retain **verification = "status: skipped"**.

We retain **`ep` = [Record episode](> `self`.record_episode task=`task` status=`st` result=`res` mode=`exec_mode` skill=`sk2` observation=`obs` workbook=`wb_path` cache=`cache2` match=`match2` llm_calls=`llm_calls` tokens=`tokens` error=`err` memory_dir=`memory_dir` )**.

We retain **learning = None**.
1. `learn`
  1. `st` == "ok"
    We retain **`learning` = [Maybe learn](> `self`.maybe_learn task=`task` kb_dir=`kb_dir` memory_dir=`memory_dir` improve_every=`improve_every` promote=`promote` )**.
  2. *
    We retain **learning = None**.
2. *
  **_ = 1**

We retain **token_cost = tokens**.
**usage = **

| llm_calls | tokens | token_cost |
|-----------|--------|------------|
| `llm_calls` | `tokens` | `token_cost` |

**execution = **

| mode | llm_calls | tokens | token_cost | skill | router |
|------|-----------|--------|------------|-------|--------|
| `exec_mode` | `llm_calls` | `tokens` | `token_cost` | `sk2` | `route_backend` |

We retain **`out` = [Set the execution field in the new record](table.put(in=`out`, at="execution", value=`execution`))**.
We retain **`out` = [Set the episode field in the new record](table.put(in=`out`, at="episode", value=`ep`))**.
We retain **`out` = [Set the route field in the new record](table.put(in=`out`, at="route", value=`routed`))**.
We retain **`out` = [Set the usage field in the new record](table.put(in=`out`, at="usage", value=`usage`))**.
We retain **`out` = [Set the token_cost field in the new record](table.put(in=`out`, at="token_cost", value=`token_cost`))**.
1. `learning`
  We retain **`out` = [Set the learning field in the new record](table.put(in=`out`, at="learning", value=`learning`))**.
2. *
  **_ = 1**

1. `efi_pf`
  We retain **`out` = [Set the efi field in the new record](table.put(in=`out`, at="efi", value=`efi_pf`))**.
2. *
  **_ = 1**

1. `resume_id`
  We retain **`_wm` = [Effects watermark](effects_watermark(ledger_dir=".marqdo/agent-effects", logical_run_id=`resume_id`))**.
  We retain **`_ckpt` = [Resume save](resume_save(id=`resume_id`, goal=`task`, task=`task`, status=`st`, round=1, workbook=`wb_path`, result=`res`, route=`routed`, resume_dir=`resume_dir`, checkpoint_v2=True, attempt_id="attempt_1", effect_ledger_watermark=`_wm`))**.
  We retain **`out` = [Set the resume field in the new record](table.put(in=`out`, at="resume", value=`_ckpt`))**.
  We retain **`out` = [Set the effect_ledger_watermark field in the new record](table.put(in=`out`, at="effect_ledger_watermark", value=`_wm`))**.
2. *
  **_ = 1**

Return *out*.

## step

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| writeback | any | True | Whether to write the observation back to its author document |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| max_reads | any | 2 | Maximum context-deepening reads |
| sandbox | any | False | Whether to use the CoW trial path |
| sandbox_profile | any | "default" | Capability profile for CoW execution |
| resume_id | any | None | Identifier of the stored continuation checkpoint |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | One atomic turn: budgeted context → LLM → optional `READ:` deepen → optional tool via subtask. Returns a *map* (`status`, `task`, `decision`, and on success `result` plus optional `tool` / `tool_result`; on failure `error`). By default (`writeback=True`) persists that map under named slots `ok` / `error` at the call site; pass `writeback=False` to skip. Optional `resume_id` scopes EffectLedger idempotency for mutating tools. With `stream=True`, the model call uses SSE; `echo=True` prints delta text to stdout as it arrives. The returned map is unchanged (final `result` string). Context source/skill start at default budget; up to `max_reads` of `READ:source` / `READ:skill` deepen before CALL/answer. |

Use `task`, `writeback`=True, `stream`=False, `echo`=False, `max_reads`=2, `sandbox`=False, `sandbox_profile`="default", `resume_id`=None as the inputs for this operation.

One atomic turn: budgeted context → LLM → optional `READ:` deepen → optional tool via subtask. Returns a *map* (`status`, `task`, `decision`, and on success `result` plus optional `tool` / `tool_result`; on failure `error`). By default (`writeback=True`) persists that map under named slots `ok` / `error` at the call site; pass `writeback=False` to skip.
Optional `resume_id` scopes EffectLedger idempotency for mutating tools.

With `stream=True`, the model call uses SSE; `echo=True` prints delta text to stdout as it arrives. The returned map is unchanged (final `result` string). Context source/skill start at default budget; up to `max_reads` of `READ:source` / `READ:skill` deepen before CALL/answer.
`sandbox=True` (O1 opt-in): attach the sandbox profile row on the result map (trial via `## trial` / `run sandbox=`).

We retain **`id` = [Read the id field](json.get(value=`self`, key="id"))**.
We retain **`model` = [Read the model field](json.get(value=`self`, key="model"))**.
We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.

`user_turn` =

| role | content |
|------|---------|
| user | `task` |

We **[Append this item to the new list](agent_history_append(id=`id`, item=`user_turn`))**.

We retain **source_depth = "default"**.
We retain **skill_depth = "default"**.
We retain **reads_left = max_reads**.
We retain **reply = None**.
We retain **decision = None**.
We retain **llm_calls = 0**.
We retain **tokens = 0**.

We retain **transport_err = None**.

- `reads_left` > 0
  1. `decision`
    We retain **reads_left = 0**.
  2. `transport_err`
    We retain **reads_left = 0**.
  3. *
    We retain **`ctx` = [Build step context](build_step_context(agent=`self`, task=`task`, source_depth=`source_depth`, skill_depth=`skill_depth`))**.
    We retain **`turn` = [Model turn](model_turn(model=`model`, prompt=`ctx`, stream=`stream`, echo=`echo`))**.
    We retain **`terr` = [Read the error field](json.get(value=`turn`, key="error"))**.
    1. `terr`
      We retain **transport_err = `terr`**.
      We retain **reads_left = 0**.
    2. *
      We retain **`reply` = [Read the text field](json.get(value=`turn`, key="text"))**.
      We retain **`c` = [Read the llm_calls field](json.get(value=`turn`, key="llm_calls"))**.
      We retain **`t` = [Read the tokens field](json.get(value=`turn`, key="tokens"))**.
      1. not `c`
        We retain **c = 0**.
      2. *
        **_ = 1**
      1. not `t`
        We retain **t = 0**.
      2. *
        **_ = 1**
      We retain **llm_calls = llm_calls + c**.
      We retain **tokens = tokens + t**.
      We retain **`reply` = [Trim](trim(value=`reply`))**.
      We retain **`rk` = [Extract plan read](extract_plan_read(reply=`reply`))**.
      1. `rk` == source
        We retain **source_depth = "deep"**.
        We retain **reads_left = reads_left - 1**.
      2. `rk` == skill
        We retain **skill_depth = "deep"**.
        We retain **reads_left = reads_left - 1**.
      3. *
        We retain **decision = reply**.
        We retain **reads_left = 0**.

1. `transport_err`

  `out` =

  | status | task | error | llm_calls | tokens |
  |--------|------|-------|-----------|--------|
  | error | `task` | `transport_err` | `llm_calls` | `tokens` |

  1. `writeback`
    We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
    We **[Record](wb.record(value=`body`, key="error"))**.
  2. *
    **_ = 1**
  Return *out*.
2. *
  **_ = 1**

We retain **`reply` = [Trim](trim(value=`reply`))**.
We retain **decision = reply**.
We retain **`tool_name` = [Extract tool name](extract_tool_name(reply=`reply`))**.

`out` =

| status | task | decision |
|--------|------|----------|
| ok | `task` | `decision` |

1. `tool_name`
  We retain **`tool_out` = [Run tool](run_tool(tools=`tools`, name=`tool_name`, logical_run_id=`resume_id`))**.
  We retain **`tool_s` = [Encode the structured value as JSON text](json.stringify(value=`tool_out`))**.
  We retain **`deny` = [Split](split(value=`tool_s`, sep="Tool not allowed"))**.
  We retain **`deny_n` = [Len](len(value=`deny`))**.
  1. `deny_n` > 1
    We retain **`task_s` = [Encode the structured value as JSON text](json.stringify(value=`task`))**.
    We retain **fp = "Tool CALL was rejected (not on the tools whitelist). Do NOT emit CALL. Answer the user task directly with the final answer only.\n\nUser task:\n"**.
    We retain **fp = fp + task_s**.
    We retain **`turn2` = [Model turn](model_turn(model=`model`, prompt=`fp`, stream=`stream`, echo=`echo`))**.
    We retain **`reply` = [Read the text field](json.get(value=`turn2`, key="text"))**.
    We retain **`c` = [Read the llm_calls field](json.get(value=`turn2`, key="llm_calls"))**.
    We retain **`t` = [Read the tokens field](json.get(value=`turn2`, key="tokens"))**.
    1. not `c`
      We retain **c = 0**.
    2. *
      **_ = 1**
    1. not `t`
      We retain **t = 0**.
    2. *
      **_ = 1**
    We retain **llm_calls = llm_calls + c**.
    We retain **tokens = tokens + t**.
    We retain **`out` = [Set the denied_tool field in the new record](table.put(in=`out`, at="denied_tool", value=`tool_name`))**.
    We retain **`out` = [Set the result field in the new record](table.put(in=`out`, at="result", value=`reply`))**.
  2. *
    We retain **`task_s` = [Encode the structured value as JSON text](json.stringify(value=`task`))**.
    We retain **fp = "The user asked:"**.
    We retain **fp = fp + task_s**.
    We retain **fp = fp + ". Tool"**.
    We retain **fp = fp + tool_name**.
    We retain **fp = fp + "ran via subtask and returned:"**.
    We retain **fp = fp + tool_s**.
    We retain **fp = fp + ". Reply to the user briefly."**.
    We retain **`turn2` = [Model turn](model_turn(model=`model`, prompt=`fp`, stream=`stream`, echo=`echo`))**.
    We retain **`reply` = [Read the text field](json.get(value=`turn2`, key="text"))**.
    We retain **`c` = [Read the llm_calls field](json.get(value=`turn2`, key="llm_calls"))**.
    We retain **`t` = [Read the tokens field](json.get(value=`turn2`, key="tokens"))**.
    1. not `c`
      We retain **c = 0**.
    2. *
      **_ = 1**
    1. not `t`
      We retain **t = 0**.
    2. *
      **_ = 1**
    We retain **llm_calls = llm_calls + c**.
    We retain **tokens = tokens + t**.
    We retain **`out` = [Set the tool field in the new record](table.put(in=`out`, at="tool", value=`tool_name`))**.
    We retain **`out` = [Set the tool_result field in the new record](table.put(in=`out`, at="tool_result", value=`tool_s`))**.
    We retain **`out` = [Set the result field in the new record](table.put(in=`out`, at="result", value=`reply`))**.
2. *
  We retain **`out` = [Set the result field in the new record](table.put(in=`out`, at="result", value=`reply`))**.

We retain **`out` = [Set the llm_calls field in the new record](table.put(in=`out`, at="llm_calls", value=`llm_calls`))**.
We retain **`out` = [Set the tokens field in the new record](table.put(in=`out`, at="tokens", value=`tokens`))**.

1. `sandbox`
  We retain **`prof` = [Profile](sandbox.profile(name=`sandbox_profile`))**.
  We retain **`out` = [Set the sandbox_profile field in the new record](table.put(in=`out`, at="sandbox_profile", value=`prof`))**.
2. *
  **_ = 1**

`as_turn` =

| role | content |
|------|---------|
| assistant | `reply` |

We **[Append this item to the new list](agent_history_append(id=`id`, item=`as_turn`))**.

1. `writeback`
  We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
  We retain **`st` = [Read the status field](json.get(value=`out`, key="status"))**.
  1. `st` == "ok"
    We **[Record](wb.record(value=`body`, key="ok"))**.
  2. *
    We **[Record](wb.record(value=`body`, key="error"))**.
2. *
  **_ = 1**

Return *out*.

## preflight

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| root | any | "." | Explicit root directory for discovery or isolated execution |
| marqdo_dir | any | None | Marqdo project directory used for engineering discovery |
| min_candidates | any | 3 | Minimum reusable candidates to seek before adaptation |
| allow_create_after | any | 2 | Number of bounded attempts before new work may be proposed |
| reuse_threshold | any | 0.72 | Score required to choose reuse |
| adapt_threshold | any | 0.45 | Score required to choose adaptation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Engineering First Information gate (008 / EKC). Reads compiled `.marqdo` knowledge. Returns `{status,decision,create_allowed,existing,constraints,decisions,failures,recommended,context_path}`. Hard rule: generate only when `create_allowed` is true (or decision is REUSE/ADAPT). |

Use `task`, `root`=".", `marqdo_dir`=None, `min_candidates`=3, `allow_create_after`=2, `reuse_threshold`=0.72, `adapt_threshold`=0.45 as the inputs for this operation.

Engineering First Information gate (008 / EKC). Reads compiled `.marqdo` knowledge.
Returns `{status,decision,create_allowed,existing,constraints,decisions,failures,recommended,context_path}`.
Hard rule: generate only when `create_allowed` is true (or decision is REUSE/ADAPT).

1. `marqdo_dir`
  We retain **`pf` = [Eng preflight](agent_eng_preflight(task=`task`, root=`root`, marqdo_dir=`marqdo_dir`, min_candidates=`min_candidates`, allow_create_after=`allow_create_after`, reuse_threshold=`reuse_threshold`, adapt_threshold=`adapt_threshold`))**.
2. *
  We retain **`pf` = [Eng preflight](agent_eng_preflight(task=`task`, root=`root`, min_candidates=`min_candidates`, allow_create_after=`allow_create_after`, reuse_threshold=`reuse_threshold`, adapt_threshold=`adapt_threshold`))**.

Return *pf*.

## eng_reuse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| task | any | required | Structured task or text supplied to the agent |
| root | any | "." | Explicit root directory for discovery or isolated execution |
| marqdo_dir | any | None | Marqdo project directory used for engineering discovery |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Alias of `preflight` focused on REUSE / ADAPT / CREATE payload (discovery_only). |

Use `task`, `root`=".", `marqdo_dir`=None as the inputs for this operation.

Alias of `preflight` focused on REUSE / ADAPT / CREATE payload (discovery_only).

1. `marqdo_dir`
  Return *[Eng reuse](agent_eng_reuse(task=`task`, root=`root`, marqdo_dir=`marqdo_dir`))*.
2. *
  Return *[Eng reuse](agent_eng_reuse(task=`task`, root=`root`))*.

---

## eng_bind_execute

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| project | any | "." | Project root or project metadata |
| query | any | required | Text used to search the bounded catalog or tool registry |
| policy | any | "pure" | Policy data governing this operation |
| inputs | any | None | Structured input record for the bound engineering capability |
| candidate_only | any | False | Whether to retain a candidate without publishing it |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | M04: discover → bind → execute → record. Prefer over treating eng_reuse scores as execution. |

Use `project`=".", `query`, `policy`="pure", `inputs`=None, `candidate_only`=False as the inputs for this operation.

M04: discover → bind → execute → record. Prefer over treating eng_reuse scores as execution.

1. `inputs`
  Return *[Eng bind execute](agent_eng_bind_execute(project=`project`, query=`query`, policy=`policy`, inputs=`inputs`, candidate_only=`candidate_only`))*.
2. *
  Return *[Eng bind execute](agent_eng_bind_execute(project=`project`, query=`query`, policy=`policy`, candidate_only=`candidate_only`))*.

---

## eng_publish_gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| project | any | "." | Project root or project metadata |
| artifact_id | any | None | Resolved artifact identifier from the compiler/catalog |
| candidate_only | any | False | Whether to retain a candidate without publishing it |
| product_gate_ok | any | True | Result of the external product gate |
| policy | any | "business" | Policy data governing this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Refuse promote when candidate_only or product/verification gate fails. |

Use `project`=".", `artifact_id`=None, `candidate_only`=False, `product_gate_ok`=True, `policy`="business" as the inputs for this operation.

Refuse promote when candidate_only or product/verification gate fails.

Return *[Eng publish gate](agent_eng_publish_gate(project=`project`, artifact_id=`artifact_id`, candidate_only=`candidate_only`, product_gate_ok=`product_gate_ok`, policy=`policy`))*.

## eng_record

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| decision | any | required | Structured planning decision |
| root | any | "." | Explicit root directory for discovery or isolated execution |
| marqdo_dir | any | None | Marqdo project directory used for engineering discovery |
| duplicated | any | False | Whether a duplicate capability was detected |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Record REUSE/ADAPT/CREATE into `.marqdo/agent/episodes/reuse_metrics.json`. |

Use `decision`, `root`=".", `marqdo_dir`=None, `duplicated`=False as the inputs for this operation.

Record REUSE/ADAPT/CREATE into `.marqdo/agent/episodes/reuse_metrics.json`.

1. `marqdo_dir`
  Return *[Eng record](agent_eng_record(decision=`decision`, root=`root`, marqdo_dir=`marqdo_dir`, duplicated=`duplicated`))*.
2. *
  Return *[Eng record](agent_eng_record(decision=`decision`, root=`root`, duplicated=`duplicated`))*.

## plan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| goal | any | required | Human task goal represented by this operation |
| workbook_dir | any | None | Directory in which the planner owns its workbook files |
| workbook | any | None | Workbook path or structured workbook reference |
| max_rounds | any | 4 | Maximum workbook planning rounds |
| confirm | any | False | Whether decomposition confirmation is requested |
| writeback | any | True | Whether to write the observation back to its author document |
| skeleton | any | "single" | Workbook source template to instantiate |
| reuse | any | True | Whether to search for reusable knowledge before new work |
| optimize | any | False | Whether to run the documented improvement path |
| force | any | False | Explicit override supported by this operation; it does not bypass host authorization |
| soft_match | any | False | Whether bounded soft matching is enabled |
| near_match | any | True | Whether to consider near-match candidates |
| near_threshold | any | 0.78 | Minimum similarity score for near-match candidates |
| promote | any | True | Whether successful evidence may be promoted |
| kb_dir | any | ".marqdo/agent-kb" | Knowledge-library directory used for discovery and writeback |
| improve_every | any | 3 | Number of observations between improvement attempts |
| explore_n | any | 3 | Maximum configured exploration attempts |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| trace | any | False | Whether to retain the documented execution trace |
| resume_id | any | None | Identifier of the stored continuation checkpoint |
| resume_dir | any | ".marqdo/agent-resume" | Directory holding continuation checkpoints |
| eng_preflight | any | False | Whether to run engineering reuse preflight |
| marqdo_dir | any | ".marqdo" | Marqdo project directory used for engineering discovery |
| allow_create_after | any | 2 | Number of bounded attempts before new work may be proposed |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Multi-step with OKF agent-kb. Default workbook is `kb_dir/resources/<slug>.mq.md`. While task file count `< explore_n` and skill is not llm_free, force a new explore variant under `kb_dir/explore/<slug>/`. Code-first: llm_free hits skip parent LLM. File children return via `# main`; `plan` exposes that as `result`. Engineering Knowledge preflight when `eng_preflight=True`: REUSE/ADAPT preferred; CREATE only if `create_allowed`. Production agents should enable this; tests that only exercise agent-kb may leave it false. Reuse lookup: exact → alias → canonicalize → optional local n-gram `near` when `near_match=True` and score ≥ `near_threshold`. Non-hit path: optional `soft_match=True` parent REUSE/NEW over ranked `agent_kb_near_match` candidates; else decompose before first child spawn (`DECISION: RUN` / `CONTINUE`+patch / solidified `DONE`). Then `await` → revise loop. Process events (`decision` / `round` / `done` / tools) are always attached on the result map for audit and view process cards; `stream=True` additionally SSE-publishes them (and parent `delta`). `echo=True` prints `plan:decompose` / `plan:await` / deltas. `trace=True` writes events to writeback slot `trace`. Quiet child subtasks stay quiet. |

Use `goal`, `workbook_dir`=None, `workbook`=None, `max_rounds`=4, `confirm`=False, `writeback`=True, `skeleton`="single", `reuse`=True, `optimize`=False, `force`=False, `soft_match`=False, `near_match`=True, `near_threshold`=0.78, `promote`=True, `kb_dir`=".marqdo/agent-kb", `improve_every`=3, `explore_n`=3, `stream`=False, `echo`=False, `trace`=False, `resume_id`=None, `resume_dir`=".marqdo/agent-resume", `eng_preflight`=False, `marqdo_dir`=".marqdo", `allow_create_after`=2 as the inputs for this operation.

Multi-step with OKF agent-kb. Default workbook is `kb_dir/resources/<slug>.mq.md`. While task file count `< explore_n` and skill is not llm_free, force a new explore variant under `kb_dir/explore/<slug>/`. Code-first: llm_free hits skip parent LLM. File children return via `# main`; `plan` exposes that as `result`.

Engineering Knowledge preflight when `eng_preflight=True`: REUSE/ADAPT preferred; CREATE only if `create_allowed`. Production agents should enable this; tests that only exercise agent-kb may leave it false.

Reuse lookup: exact → alias → canonicalize → optional local n-gram `near` when `near_match=True` and score ≥ `near_threshold`. Non-hit path: optional `soft_match=True` parent REUSE/NEW over ranked `agent_kb_near_match` candidates; else decompose before first child spawn (`DECISION: RUN` / `CONTINUE`+patch / solidified `DONE`). Then `await` → revise loop. Process events (`decision` / `round` / `done` / tools) are always attached on the result map for audit and view process cards; `stream=True` additionally SSE-publishes them (and parent `delta`). `echo=True` prints `plan:decompose` / `plan:await` / deltas. `trace=True` writes events to writeback slot `trace`. Quiet child subtasks stay quiet.

This method coordinates a workbook lifecycle. The workbook is an ordinary executable document with a main entry, not a private planner data format. The parent proposes decisions and short repairs; the runtime writes or awaits source, captures an observation and returns the child's actual value. Choose this method when reuse and a bounded revise loop are appropriate. Choose step for a single model turn with an optional tool.

The first gate can consult engineering knowledge. REUSE and ADAPT are recorded as such; creating new work requires the configured create decision. A missing knowledge store is a separate state from an explicit denial. This gate does not manufacture a candidate merely to satisfy a minimum count. Callers should retain its diagnostics when deciding whether their project has enough reusable context.

Knowledge lookup then establishes the resource and cache state. Exact, alias and canonical matches are tried before optional near matches. A similarity threshold filters candidates but does not prove semantic equivalence. An llm_free resource can execute through the existing code path without a parent model. The returned cache, match and resource fields let the application explain which path was chosen and inspect the source that ran.

Exploration uses owned workbook files. With no explicit workbook path, the configured knowledge directory supplies the resource location; bounded variants live under its exploration subtree. The exploration count controls attempts to establish useful reusable evidence. It should not be confused with the maximum revision rounds of one workbook. Keep both directory ownership and attempt accounting intact when changing the planner.

An optional soft-match turn compares existing candidates and produces either REUSE with a concrete slug or NEW. The reply is parsed and resolved before execution. It cannot authorize an invented slug or silently replace a failed child with model prose. When reuse does not yield a result, the chosen single-worker or dual-worker template provides a starting document, with the goal embedded as quoted data.

Before the first child execution, decomposition asks whether that starting document is sufficient. RUN accepts the structure. CONTINUE supplies a bounded source patch, and evidence requests go through the restricted parent tool dispatcher. DONE belongs to the existing solidification path and must have a usable value. Rewriting already valid imports is not itself progress toward the user's goal.

Child execution is awaited before revision. The observation contains process status, returned value, slots and bounded output; the parent prompt uses a compact projection so large logs do not drown out the next repair. If the child fails or its value is unsuitable, a short patch can be applied and another round attempted. Exhausting the round budget is reported by the result path, rather than hidden behind a fabricated successful answer.

The confirm option concerns the decomposition stage; it is not a substitute for host permissions. File creation, source patches, subprocess execution and model transport retain their own effect boundaries. Quiet subtasks remain quiet. Turning on echo changes incremental terminal output, while stream publishes process events. Neither option changes the requirement to inspect the final structured result.

Events are retained for audit even when streaming is disabled. Decisions, rounds and completion events describe the work that actually ran. Trace writeback is separately controlled, and normal writeback stores the result at its source call site. Consumers should use these fields to diagnose a plan instead of scraping console messages or assuming every emitted event represents a completed child.

Successful outcomes may be promoted through the knowledge mechanism when enabled. Promotion preserves the workbook as inspectable source and uses the configured evidence process; generating a file alone is not compilation into a trusted skill. Optimization and force options select existing improvement behavior, while the regular reuse gates remain visible in the outcome. A later author can open the resulting resource and reproduce its invocation.

With a resume identifier, checkpoint records connect the goal, round, workbook and result to a logical run. The checkpoint is continuation data, not a replay authorization for uncertain external writes. Tools with effects still use their own ledger and reconciliation rules. Preserve these references when integrating a scheduler or another application so continuation does not duplicate work that already committed.

We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.
We retain **cache = "miss"**.
We retain **path = None**.
We retain **improve = None**.
We retain **explore = None**.
We retain **explore_attempt = None**.
We retain **skel_kind = skeleton**.
We retain **`events` = [Empty list](table.empty_list())**.
We retain **eng = None**.

1. `eng_preflight`
  We retain **`eng` = [Eng preflight](agent_eng_preflight(task=`goal`, marqdo_dir=`marqdo_dir`, allow_create_after=`allow_create_after`))**.
  We retain **`eng_status` = [Read the status field](json.get(value=`eng`, key="status"))**.
  We retain **`eng_decision` = [Read the decision field](json.get(value=`eng`, key="decision"))**.
  We retain **`eng_allowed` = [Read the create_allowed field](json.get(value=`eng`, key="create_allowed"))**.
  1. `eng_status` == missing_knowledge
    **_ = 1**
  2. `eng_decision` == REUSE
    We **[Eng record](agent_eng_record(decision="REUSE", marqdo_dir=`marqdo_dir`))**.
    **_ = 1**
  3. `eng_decision` == ADAPT
    We **[Eng record](agent_eng_record(decision="ADAPT", marqdo_dir=`marqdo_dir`))**.
    **_ = 1**
  4. `eng_allowed`
    We **[Eng record](agent_eng_record(decision="CREATE", marqdo_dir=`marqdo_dir`))**.
    **_ = 1**
  5. *

    `out` =

    | status | cache | eng | decision | preflight | events |
    |--------|-------|-----|----------|-----------|--------|
    | blocked | eng-block | true | BLOCKED | `eng` | `events` |

    Return *out*.
2. *
  **_ = 1**

We retain **`tf` = [Kb task files](agent_kb_task_files(kb_dir=`kb_dir`, goal=`goal`, tools=`tools`))**.
We retain **`nfiles` = [Read the count field](json.get(value=`tf`, key="count"))**.

1. `force`
  **_ = 1**
2. `optimize`
  **_ = 1**
3. `reuse`
  We retain **`hit` = [Kb lookup](agent_kb_lookup(kb_dir=`kb_dir`, goal=`goal`, tools=`tools`, near_match=`near_match`, near_threshold=`near_threshold`))**.
  1. `hit`
    We retain **`match_kind` = [Read the match field](json.get(value=`hit`, key="match"))**.
    1. `match_kind` == alias
      We retain **cache_label = "soft-hit"**.
    2. `match_kind` == canonical
      We retain **cache_label = "soft-hit"**.
      We retain **`hit_slug` = [Read the slug field](json.get(value=`hit`, key="slug"))**.
      We **[Kb add alias](agent_kb_add_alias(kb_dir=`kb_dir`, slug=`hit_slug`, alias=`goal`))**.
    3. `match_kind` == near
      We retain **cache_label = "soft-hit"**.
      We retain **`hit_slug` = [Read the slug field](json.get(value=`hit`, key="slug"))**.
      We **[Kb add alias](agent_kb_add_alias(kb_dir=`kb_dir`, slug=`hit_slug`, alias=`goal`))**.
    4. `match_kind` == soft
      We retain **cache_label = "soft-hit"**.
    5. *
      We retain **cache_label = "hit"**.
    We retain **`lf` = [Read the llm_free field](json.get(value=`hit`, key="llm_free"))**.
    1. `lf`
      We retain **`path` = [Read the resource field](json.get(value=`hit`, key="resource"))**.
      We retain **`aw` = [Await workbook](await_workbook(path=`path`))**.
      We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
      We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
      We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
      1. `code` == 0
        We **[Kb record hit](agent_kb_record_hit(kb_dir=`kb_dir`, goal=`goal`, tools=`tools`, improve_every=`improve_every`))**.
        We retain **cache = cache_label**.
        We retain **`hit_score` = [Read the score field](json.get(value=`hit`, key="score"))**.
        We retain **`sk` = [Read the skill field](json.get(value=`hit`, key="skill"))**.
        We retain **`st` = [Read the status field](json.get(value=`hit`, key="status"))**.
        The following record names each literal field without a JSON construction chain.

        `_literal_sum_fields` =

        | @ | key | value |
        |---|---|---|
        | 1 | "a" | "OKF llm_free skill hit (" |
        | 2 | "b" | "); spawned resource" |

        We build **sum = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_sum_fields`))**.


        We retain **`suma` = [Read the a field](json.get(value=`sum`, key="a"))**.
        We retain **`sumb` = [Read the b field](json.get(value=`sum`, key="b"))**.
        We retain **sum = suma + match_kind + sumb**.

        `out` =

        | status | goal | workbook | rounds | cache | match | skill | skill_status | summary | observation | result |
        |--------|------|----------|--------|-------|-------|-------|--------------|---------|-------------|--------|
        | ok | `goal` | `path` | 1 | `cache_label` | `match_kind` | `sk` | `st` | `sum` | `last_obs` | `child_val` |

        1. `hit_score`
          We retain **`out` = [Set the score field in the new record](table.put(in=`out`, at="score", value=`hit_score`))**.
        2. *
          **_ = 1**
        We retain **`events` = [Plan append decision](plan_append_decision(events=`events`, decision="REUSE", stream=`stream`, summary=`sum`))**.
        We retain **`events` = [Plan append round](plan_append_round(events=`events`, round=1, workbook=`path`, exit_code=`code`, result=`child_val`, stream=`stream`, echo=`echo`))**.
        1. `writeback`
          We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
          We **[Record](wb.record(value=`body`, key="ok"))**.
        2. *
          **_ = 1**
        We retain **`out` = [Plan finish stream](plan_finish_stream(out=`out`, events=`events`, stream=`stream`, trace=`trace`, result=`child_val`))**.
        Return *out*.
      2. *
        We retain **path = None**.
    2. `nfiles` < `explore_n`
      We retain **explore = 1**.
      We retain **cache = "explore"**.
      We retain **path = None**.
      We retain **explore_attempt = nfiles + 1**.
      1. `explore_attempt` == 2
        We retain **skel_kind = "dual"**.
      2. *
        We retain **skel_kind = skeleton**.
    3. *
      We retain **`path` = [Read the resource field](json.get(value=`hit`, key="resource"))**.
      We retain **`aw` = [Await workbook](await_workbook(path=`path`))**.
      We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
      We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
      We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
      1. `code` == 0
        We retain **`rec` = [Kb record hit](agent_kb_record_hit(kb_dir=`kb_dir`, goal=`goal`, tools=`tools`, improve_every=`improve_every`))**.
        We retain **`due` = [Read the improve_due field](json.get(value=`rec`, key="improve_due"))**.
        1. `due`
          We retain **improve = 1**.
          We retain **cache = "improve"**.
        2. *
          We retain **cache = cache_label**.
          We retain **`hit_score` = [Read the score field](json.get(value=`hit`, key="score"))**.
          We retain **`sk` = [Read the skill field](json.get(value=`hit`, key="skill"))**.
          We retain **`st` = [Read the status field](json.get(value=`hit`, key="status"))**.
          The following record names each literal field without a JSON construction chain.

          `_literal_sum_fields` =

          | @ | key | value |
          |---|---|---|
          | 1 | "a" | "OKF skill hit (" |
          | 2 | "b" | "); spawned resource" |

          We build **sum = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_sum_fields`))**.


          We retain **`suma` = [Read the a field](json.get(value=`sum`, key="a"))**.
          We retain **`sumb` = [Read the b field](json.get(value=`sum`, key="b"))**.
          We retain **sum = suma + match_kind + sumb**.

          `out` =

          | status | goal | workbook | rounds | cache | match | skill | skill_status | summary | observation | result |
          |--------|------|----------|--------|-------|-------|-------|--------------|---------|-------------|--------|
          | ok | `goal` | `path` | 1 | `cache_label` | `match_kind` | `sk` | `st` | `sum` | `last_obs` | `child_val` |

          1. `hit_score`
            We retain **`out` = [Set the score field in the new record](table.put(in=`out`, at="score", value=`hit_score`))**.
          2. *
            **_ = 1**
          We retain **`events` = [Plan append decision](plan_append_decision(events=`events`, decision="REUSE", stream=`stream`, summary=`sum`))**.
          We retain **`events` = [Plan append round](plan_append_round(events=`events`, round=1, workbook=`path`, exit_code=`code`, result=`child_val`, stream=`stream`, echo=`echo`))**.
          1. `writeback`
            We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
            We **[Record](wb.record(value=`body`, key="ok"))**.
          2. *
            **_ = 1**
          We retain **`out` = [Plan finish stream](plan_finish_stream(out=`out`, events=`events`, stream=`stream`, trace=`trace`, result=`child_val`))**.
          Return *out*.
      2. *
        We retain **path = None**.
  2. *
    1. `soft_match`
      We retain **`near` = [Kb near match](agent_kb_near_match(kb_dir=`kb_dir`, goal=`goal`))**.
      We retain **`tasks` = [Read the candidates field](json.get(value=`near`, key="candidates"))**.
      We retain **`tn` = [Len](len(value=`tasks`))**.
      1. `tn` < 1
        We retain **`listed` = [Kb list tasks](agent_kb_list_tasks(kb_dir=`kb_dir`))**.
        We retain **`tasks` = [Read the tasks field](json.get(value=`listed`, key="tasks"))**.
        We retain **`tn` = [Len](len(value=`tasks`))**.
      2. *
        **_ = 1**
      1. `tn` > 0
        We retain **`soft_prompt` = [Build soft match prompt](build_soft_match_prompt(goal=`goal`, tasks=`tasks`))**.
        We retain **`model` = [Read the model field](json.get(value=`self`, key="model"))**.
        1. `stream`
          We retain **`evs` = [Stream](model.stream(prompt=`soft_prompt`, echo=`echo`))**.
          We retain **`soft_reply` = [Collect](llm.collect(events=`evs`))**.
          We retain **`events` = [Plan merge deltas](plan_merge_deltas(events=`events`, from=`evs`, stream=`stream`))**.
        2. *
          We retain **`soft_turn` = [Model turn](model_turn(model=`model`, prompt=`soft_prompt`, stream=False, echo=`echo`))**.
          We retain **`soft_reply` = [Read the text field](json.get(value=`soft_turn`, key="text"))**.
        We retain **`soft_reply` = [Trim](trim(value=`soft_reply`))**.
        We retain **`soft_dec` = [Extract plan decision](extract_plan_decision(reply=`soft_reply`))**.
        We retain **`events` = [Plan append decision](plan_append_decision(events=`events`, decision=`soft_dec`, stream=`stream`))**.
        1. `soft_dec` == REUSE
          We retain **`reuse_slug` = [Extract soft match slug](extract_soft_match_slug(reply=`soft_reply`))**.
          1. `reuse_slug`
            We retain **`hit` = [Kb lookup](agent_kb_lookup(kb_dir=`kb_dir`, slug=`reuse_slug`, goal=`goal`, tools=`tools`))**.
            1. `hit`
              We **[Kb add alias](agent_kb_add_alias(kb_dir=`kb_dir`, slug=`reuse_slug`, alias=`goal`))**.
              We retain **cache_label = "soft-hit"**.
              We retain **`path` = [Read the resource field](json.get(value=`hit`, key="resource"))**.
              We retain **`aw` = [Await workbook](await_workbook(path=`path`))**.
              We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
              We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
              We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
              1. `code` == 0
                We **[Kb record hit](agent_kb_record_hit(kb_dir=`kb_dir`, goal=`goal`, tools=`tools`, improve_every=`improve_every`))**.
                We retain **cache = "soft-hit"**.
                We retain **`sk` = [Read the skill field](json.get(value=`hit`, key="skill"))**.
                We retain **`st` = [Read the status field](json.get(value=`hit`, key="status"))**.
                We retain **sum = "OKF soft_match REUSE; spawned resource"**.

                `out` =

                | status | goal | workbook | rounds | cache | match | skill | skill_status | summary | observation | result |
                |--------|------|----------|--------|-------|-------|-------|--------------|---------|-------------|--------|
                | ok | `goal` | `path` | 1 | soft-hit | soft | `sk` | `st` | `sum` | `last_obs` | `child_val` |
                We retain **`events` = [Plan append decision](plan_append_decision(events=`events`, decision="REUSE", stream=`stream`, summary=`sum`))**.
                We retain **`events` = [Plan append round](plan_append_round(events=`events`, round=1, workbook=`path`, exit_code=`code`, result=`child_val`, stream=`stream`, echo=`echo`))**.
                1. `writeback`
                  We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
                  We **[Record](wb.record(value=`body`, key="ok"))**.
                2. *
                  **_ = 1**
                We retain **`out` = [Plan finish stream](plan_finish_stream(out=`out`, events=`events`, stream=`stream`, trace=`trace`, result=`child_val`))**.
                Return *out*.
              2. *
                We retain **path = None**.
            2. *
              **_ = 1**
          2. *
            **_ = 1**
        2. *
          **_ = 1**
      2. *
        **_ = 1**
    2. *
      **_ = 1**
    1. `nfiles` < `explore_n`
      We retain **explore = 1**.
      We retain **cache = "explore"**.
      We retain **explore_attempt = nfiles + 1**.
      1. `explore_attempt` == 2
        We retain **skel_kind = "dual"**.
      2. *
        We retain **skel_kind = skeleton**.
    2. *
      **_ = 1**
4. *
  1. `nfiles` < `explore_n`
    We retain **explore = 1**.
    We retain **cache = "explore"**.
    We retain **explore_attempt = nfiles + 1**.
  2. *
    **_ = 1**

We retain **`skel` = [Render workbook skeleton](render_workbook_skeleton(goal=`goal`, skeleton=`skel_kind`))**.

1. `workbook`
  We retain **path = workbook**.
  We retain **`ex` = [Exists](fs.exists(path=`path`))**.
  1. `ex`
    **_ = 1**
  2. *
    We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`skel`))**.
2. *
  1. `path`
    **_ = 1**
  2. *
    We retain **`slug` = [Goal slug](agent_goal_slug(goal=`goal`))**.
    1. `workbook_dir`
      We **[Make dir](fs.make_dir(path=`workbook_dir`))**.
      We retain **`ts` = [Now ms](time.now_ms())**.
      The following record names each literal field without a JSON construction chain.

      `_literal_parts_fields` =

      | @ | key | value |
      |---|---|---|
      | 1 | "a" | "/workbook-" |
      | 2 | "b" | "-" |
      | 3 | "c" | ".mq.md" |

      We build **parts = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_parts_fields`))**.


      We retain **`a` = [Read the a field](json.get(value=`parts`, key="a"))**.
      We retain **`b` = [Read the b field](json.get(value=`parts`, key="b"))**.
      We retain **`c` = [Read the c field](json.get(value=`parts`, key="c"))**.
      We retain **path = workbook_dir + a + slug + b + ts + c**.
      We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`skel`))**.
    2. `explore`
      The following record names each literal field without a JSON construction chain.

      `_literal_parts_fields` =

      | @ | key | value |
      |---|---|---|
      | 1 | "a" | "/explore/" |
      | 2 | "b" | "/" |
      | 3 | "c" | ".mq.md" |

      We build **parts = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_parts_fields`))**.


      We retain **`a` = [Read the a field](json.get(value=`parts`, key="a"))**.
      We retain **`b` = [Read the b field](json.get(value=`parts`, key="b"))**.
      We retain **`c` = [Read the c field](json.get(value=`parts`, key="c"))**.
      We retain **path = kb_dir + a + slug + b + explore_attempt + c**.
      We retain **`ex` = [Exists](fs.exists(path=`path`))**.
      1. `ex`
        **_ = 1**
      2. *
        We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`skel`))**.
    3. *
      The following record names each literal field without a JSON construction chain.

      `_literal_parts_fields` =

      | @ | key | value |
      |---|---|---|
      | 1 | "a" | "/resources/" |
      | 2 | "b" | ".mq.md" |

      We build **parts = [Collect these named fields into one record](table.from_pairs(pairs=`_literal_parts_fields`))**.


      We retain **`a` = [Read the a field](json.get(value=`parts`, key="a"))**.
      We retain **`b` = [Read the b field](json.get(value=`parts`, key="b"))**.
      We retain **path = kb_dir + a + slug + b**.
      We retain **`ex` = [Exists](fs.exists(path=`path`))**.
      1. `ex`
        **_ = 1**
      2. *
        We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`skel`))**.

1. `confirm`

  `out` =

  | status | goal | workbook | summary | cache |
  |--------|------|----------|---------|-------|
  | pending | `goal` | `path` | workbook created; confirm to run | bypass |
  1. `writeback`
    We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
    We **[Record](wb.record(value=`body`, key="ok"))**.
  2. *
    **_ = 1**
  We retain **`out` = [Plan finish stream](plan_finish_stream(out=`out`, events=`events`, stream=`stream`, trace=`trace`))**.
  Return *out*.
2. *
  **_ = 1**

We retain **`model` = [Read the model field](json.get(value=`self`, key="model"))**.
We retain **round = 0**.
We retain **last_obs = None**.
We retain **last_reply = None**.
We retain **child_val = None**.
We retain **done = None**.
We retain **summary = None**.
We retain **status = "error"**.
We retain **err = "max_rounds exhausted"**.
We retain **skip_loop = None**.
We retain **plan_llm_calls = 0**.
We retain **plan_tokens = 0**.

We **[Plan echo decompose](plan_echo_decompose(workbook=`path`, stream=`stream`, echo=`echo`))**.
We retain **`last_obs` = [Inspect workbook](inspect_workbook(path=`path`))**.
We retain **`turn` = [Plan llm act loop](plan_llm_act_loop(agent=`self`, goal=`goal`, path=`path`, observation=`last_obs`, phase="decompose", explore_attempt=`explore_attempt`, explore_n=`explore_n`, stream=`stream`, echo=`echo`, events=`events`))**.
We retain **`turn_s` = [Encode the structured value as JSON text](json.stringify(value=`turn`))**.
We retain **`turn_map` = [Starts with](text.starts_with(text=`turn_s`, prefix="{"))**.
We retain **dec = None**.
We retain **last_reply = None**.
1. `turn_map`
  We retain **`dec` = [Read the decision field](json.get(value=`turn`, key="decision"))**.
  We retain **`last_reply` = [Read the reply field](json.get(value=`turn`, key="reply"))**.
  We retain **`last_obs` = [Read the observation field](json.get(value=`turn`, key="observation"))**.
  We retain **`events` = [Read the events field](json.get(value=`turn`, key="events"))**.
  We retain **`_c` = [Read the llm_calls field](json.get(value=`turn`, key="llm_calls"))**.
  We retain **`_t` = [Read the tokens field](json.get(value=`turn`, key="tokens"))**.
  1. `_c` != None
    We retain **plan_llm_calls = plan_llm_calls + _c**.
  1. `_t` != None
    We retain **plan_tokens = plan_tokens + _t**.
2. *
  We retain **dec = "TRANSPORT_ERROR"**.
  We retain **last_reply = "plan_llm_act_loop_not_map"**.
  We retain **err = `last_reply`**.

1. `dec` == "TRANSPORT_ERROR"
  We retain **done = 1**.
  We retain **status = "error"**.
  We retain **err = `last_reply`**.
  1. `turn_map`
    We retain **`e2` = [Read the error field](json.get(value=`turn`, key="error"))**.
    1. `e2`
      We retain **err = `e2`**.
  We retain **summary = "llm transport error"**.
  We retain **skip_loop = 1**.
2. `dec` == CONTINUE
  We retain **`n` = [Apply patch blocks](fs.apply_patch_blocks(path=`path`, text=`last_reply`, soft=True))**.
  1. `n`
    **_ = 1**
  2. *
    We retain **done = 1**.
    We retain **status = "error"**.
    We retain **err = "no patches applied"**.
    We retain **`summary` = [Extract plan summary](extract_plan_summary(reply=`last_reply`))**.
    We retain **skip_loop = 1**.
3. `dec` == RUN
  **_ = 1**
4. `dec` == DONE
  We retain **`has_step` = [Workbook has worker step](workbook_has_worker_step(path=`path`))**.
  1. `has_step`
    **_ = 1**
  2. *
    We **[Plan echo await](plan_echo_await(round=1, workbook=`path`, stream=`stream`, echo=`echo`))**.
    We retain **`aw` = [Await workbook](await_workbook(path=`path`))**.
    We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
    We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
    We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
    We retain **round = 1**.
    We retain **`events` = [Plan append round](plan_append_round(events=`events`, round=1, workbook=`path`, exit_code=`code`, result=`child_val`, stream=`stream`, echo=`echo`))**.
    1. `code` == 0
      We retain **done = 1**.
      We retain **status = "ok"**.
      We retain **`summary` = [Extract plan summary](extract_plan_summary(reply=`last_reply`))**.
      We retain **err = None**.
      We retain **skip_loop = 1**.
    2. *
      **_ = 1**
5. *
  Free-form parent reply with no DECISION/CALL/READ (dec=unknown): accept as DONE
  when there is prose — chat soft-recover depends on result/summary surviving.
  1. `dec` == "unknown"
    1. `last_reply`
      We retain **done = 1**.
      We retain **status = "ok"**.
      We retain **err = None**.
      We retain **summary = `last_reply`**.
      We retain **skip_loop = 1**.
    2. *
      We retain **done = 1**.
      We retain **status = "error"**.
      We retain **err = "unrecognized plan decision"**.
      We retain **summary = last_reply**.
      We retain **skip_loop = 1**.
  2. *
    We retain **done = 1**.
    We retain **status = "error"**.
    We retain **err = "unrecognized plan decision"**.
    We retain **summary = last_reply**.
    We retain **skip_loop = 1**.

1. `skip_loop`
  We retain **left = 0**.
2. `improve`
  We retain **left = 1**.
3. *
  We retain **left = max_rounds**.

- `left` > 0
  1. `done`
    We retain **left = 0**.
  2. *
    We retain **round = round + 1**.
    We **[Plan echo await](plan_echo_await(round=`round`, workbook=`path`, stream=`stream`, echo=`echo`))**.
    We retain **`aw` = [Await workbook](await_workbook(path=`path`))**.
    We retain **`code` = [Read the code field](json.get(value=`aw`, key="code"))**.
    We retain **`child_val` = [Read the value field](json.get(value=`aw`, key="value"))**.
    We retain **`last_obs` = [Read the observation field](json.get(value=`aw`, key="observation"))**.
    We retain **`events` = [Plan append round](plan_append_round(events=`events`, round=`round`, workbook=`path`, exit_code=`code`, result=`child_val`, stream=`stream`, echo=`echo`))**.

Deterministic success stop (loop engineering): when the child already returned a value, do not ask the parent LLM to invent solidify patches — DONE + runtime solidify.

    1. `code` == 0
      1. `child_val`
        1. `improve`
          We retain **auto_done = None**.
        2. *
          We retain **auto_done = 1**.
      2. *
        We retain **auto_done = None**.
    2. *
      We retain **auto_done = None**.

    1. `auto_done`
      We **[Workbook solidify](agent_workbook_solidify(path=`path`, observation=`last_obs`))**.
      We retain **done = 1**.
      We retain **status = "ok"**.
      We retain **summary = "child returned value; solidified"**.
      We retain **err = None**.
      We retain **dec = "DONE"**.
      We retain **`events` = [Plan append decision](plan_append_decision(events=`events`, decision=`dec`, stream=`stream`, summary=`summary`))**.
      We retain **left = 0**.
    2. *
      We retain **`turn` = [Plan llm act loop](plan_llm_act_loop(agent=`self`, goal=`goal`, path=`path`, observation=`last_obs`, phase="revise", explore_attempt=`explore_attempt`, explore_n=`explore_n`, stream=`stream`, echo=`echo`, events=`events`))**.
      We retain **`dec` = [Read the decision field](json.get(value=`turn`, key="decision"))**.
      We retain **`last_reply` = [Read the reply field](json.get(value=`turn`, key="reply"))**.
      We retain **`last_obs` = [Read the observation field](json.get(value=`turn`, key="observation"))**.
      We retain **`events` = [Read the events field](json.get(value=`turn`, key="events"))**.
      We retain **`_c` = [Read the llm_calls field](json.get(value=`turn`, key="llm_calls"))**.
      We retain **`_t` = [Read the tokens field](json.get(value=`turn`, key="tokens"))**.
      1. `_c` != None
        We retain **plan_llm_calls = plan_llm_calls + _c**.
      1. `_t` != None
        We retain **plan_tokens = plan_tokens + _t**.
      1. `dec` == DONE
        We **[Workbook solidify](agent_workbook_solidify(path=`path`, observation=`last_obs`))**.
        We retain **done = 1**.
        We retain **status = "ok"**.
        We retain **`summary` = [Extract plan summary](extract_plan_summary(reply=`last_reply`))**.
        We retain **err = None**.
        We retain **left = 0**.
      2. `dec` == CONTINUE
        We retain **`n` = [Apply patch blocks](fs.apply_patch_blocks(path=`path`, text=`last_reply`, soft=True))**.
        1. `n`
          We retain **left = left - 1**.
        2. *
          We retain **done = 1**.
          We retain **status = "error"**.
          We retain **err = "no patches applied"**.
          We retain **`summary` = [Extract plan summary](extract_plan_summary(reply=`last_reply`))**.
          We retain **left = 0**.
      3. `dec` == RUN
        We retain **left = left - 1**.
      4. *
        1. `dec` == "unknown"
          1. `last_reply`
            We retain **done = 1**.
            We retain **status = "ok"**.
            We retain **err = None**.
            We retain **summary = `last_reply`**.
            We retain **left = 0**.
          2. *
            We retain **done = 1**.
            We retain **status = "error"**.
            We retain **err = "unrecognized plan decision"**.
            We retain **summary = last_reply**.
            We retain **left = 0**.
        2. *
          We retain **done = 1**.
          We retain **status = "error"**.
          We retain **err = "unrecognized plan decision"**.
          We retain **summary = last_reply**.
          We retain **left = 0**.

1. `status` == "ok"
  1. `promote`
    We retain **`prom` = [Kb promote](agent_kb_promote(kb_dir=`kb_dir`, goal=`goal`, workbook=`path`, tools=`tools`))**.
    We retain **cache = "refreshed"**.
  2. *
    We retain **cache = "miss"**.
2. *
  **_ = 1**

`out` =

| status | goal | workbook | rounds | cache | llm_calls | tokens |
|--------|------|----------|--------|-------|-----------|--------|
| `status` | `goal` | `path` | `round` | `cache` | `plan_llm_calls` | `plan_tokens` |

1. `summary`
  We retain **`out` = [Set the summary field in the new record](table.put(in=`out`, at="summary", value=`summary`))**.
2. *
  **_ = 1**

1. `err`
  We retain **`out` = [Set the error field in the new record](table.put(in=`out`, at="error", value=`err`))**.
2. *
  **_ = 1**

1. `last_obs`
  We retain **`out` = [Set the observation field in the new record](table.put(in=`out`, at="observation", value=`last_obs`))**.
2. *
  **_ = 1**

1. `child_val`
  We retain **`out` = [Set the result field in the new record](table.put(in=`out`, at="result", value=`child_val`))**.
  We retain **plan_result = child_val**.
2. `summary`
  We retain **`out` = [Set the result field in the new record](table.put(in=`out`, at="result", value=`summary`))**.
  We retain **plan_result = summary**.
3. *
  We retain **plan_result = None**.

1. `writeback`
  We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`out`))**.
  1. `status` == "ok"
    We **[Record](wb.record(value=`body`, key="ok"))**.
  2. *
    We **[Record](wb.record(value=`body`, key="error"))**.
2. *
  **_ = 1**

We retain **`out` = [Plan finish stream](plan_finish_stream(out=`out`, events=`events`, stream=`stream`, trace=`trace`, result=`plan_result`))**.
Return *out*.

---

# mcp_server

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | "marqdo" | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Minimal MCP Server host (stdio JSON-RPC / Streamable HTTP). Tools point at entry-module `lib.member` functions (same bridge as `ext/web` invoke). Evidence/results may carry `authority=workbook`. |

Use `name`="marqdo" as the inputs for this operation.

Minimal MCP Server host (stdio JSON-RPC / Streamable HTTP). Tools point at entry-module `lib.member` functions (same bridge as `ext/web` invoke). Evidence/results may carry `authority=workbook`.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/agent: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

We retain **`tools0` = [Empty list](table.empty_list())**.

`h` =

| _type | tools | name |
|-------|-------|------|
| mcp_server | `tools0` | `name` |

Return *h*.

## tool

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| fn | any | required | Registered function target |
| description | any | "" | Human-readable tool description |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Register one tool. `fn` must be `lib.member` imported on the entry workbook. |

Use `name`, `fn`, `description`="" as the inputs for this operation.

Register one tool. `fn` must be `lib.member` imported on the entry workbook.

We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.
1. not `tools`
  We retain **`tools` = [Empty list](table.empty_list())**.

`row` =

| name | fn | description |
|------|-----|-------------|
| `name` | `fn` | `description` |

We retain **`tools` = [Append this item to the new list](table.append(list=`tools`, item=`row`))**.
We retain **`h` = [Set the tools field in the new record](table.put(in=`self`, at="tools", value=`tools`))**.
Return *h*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| transport | any | "stdio" | Connector transport type |
| host | any | "127.0.0.1" | Explicit bind or remote host |
| port | any | 7434 | Explicit server port |
| path | any | "/mcp" | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Block on MCP: `stdio` (line JSON-RPC) or `http` / `streamable-http` (Streamable HTTP, POST JSON-RPC + optional GET SSE). Spec 2025-03-26. |

Use `transport`="stdio", `host`="127.0.0.1", `port`=7434, `path`="/mcp" as the inputs for this operation.

Block on MCP: `stdio` (line JSON-RPC) or `http` / `streamable-http` (Streamable HTTP, POST JSON-RPC + optional GET SSE). Spec 2025-03-26.

We retain **`name` = [Read the name field](json.get(value=`self`, key="name"))**.
We retain **`tools` = [Read the tools field](json.get(value=`self`, key="tools"))**.
Return *[Mcp serve](agent_mcp_serve(name=`name`, tools=`tools`, transport=`transport`, host=`host`, port=`port`, path=`path`))*.
