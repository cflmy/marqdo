---
markup: occlusion
title: ext/ai/llm
description: >-
  Marqdo LLM intelligence primitive — ask / stream / collect / named handles.
  OpenAI-compatible and Ollama HTTP stay behind the semantic surface.
import:
  sys: lib/sys.mq.md
  json: lib/json.mq.md
  table: lib/table.mq.md
  fs: lib/fs.mq.md
  text: lib/text.mq.md
  openai: ext/ai/llm/openai.mq.md
---

Choose the public model resource here for ask, stream and collected response behavior. [The AI entry](index.mq.md) provides an offline event workflow; [the adapter entry](llm/index.mq.md) owns provider configuration and transport shapes. Agents and workbook orchestration belong in agent.mq.md. A created handle is configuration, not proof that authentication or a model request succeeded. Keep transport errors, semantic events and usage counters visible when extending this file.

## load_env

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load `.env` from cwd (optional `path=`). Does not override existing variables. |

Use `path`=None as the inputs for this operation.

Load `.env` from cwd (optional `path=`). Does not override existing variables.

Return *[Load dotenv](sys.load_dotenv(path=`path`))*.

## collect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Collapse a stream event list to final answer text. Accepts semantic events (`delta` / `text` / `done.result` / `error.message`) and native-plugin raw SSE frames (`type=data` with OpenAI chunk JSON in `data`). Alias: `stream_result`. |

Use `events` as the inputs for this operation.

Collapse a stream event list to final answer text.
Accepts semantic events (`delta` / `text` / `done.result` / `error.message`) and
native-plugin raw SSE frames (`type=data` with OpenAI chunk JSON in `data`).
Alias: `stream_result`.

We retain **answer = ""**.

- [ev](遍历 events 为 ev)
  We retain **`t` = [Read the type field](json.get(value=`ev`, key="type"))**.
  1. `t` == "done"
    We retain **`done_r` = [Read the result field](json.get(value=`ev`, key="result"))**.
    1. `done_r`
      We retain **answer = `done_r`**.
    2. *
      **_ = 1**
  2. `t` == "error"
    We retain **`em` = [Read the message field](json.get(value=`ev`, key="message"))**.
    1. `em`
      We retain **answer = `em`**.
    2. *
      **_ = 1**
  3. `t` == "delta"
  4. `t` == "text"
    We retain **`chunk` = [Read the text field](json.get(value=`ev`, key="text"))**.
    1. `chunk`
      We retain **answer = answer + chunk**.
    2. *
      **_ = 1**
  5. `t` == "data"
    We retain **`raw` = [Read the data field](json.get(value=`ev`, key="data"))**.
    1. `raw`
      1. `raw` == "[DONE]"
        **_ = 1**
      2. *
        We retain **`parsed` = [Parse the JSON transport text](json.parse(text=`raw`))**.
        1. `parsed`
          We retain **`choices` = [Read the choices field](json.get(value=`parsed`, key="choices"))**.
          1. `choices`
            We retain **`c0` = [Read 1 field](access(`choices`, "1"))**.
            We retain **`delta` = [Read the delta field](json.get(value=`c0`, key="delta"))**.
            1. `delta`
              We retain **`piece` = [Read the content field](json.get(value=`delta`, key="content"))**.
              1. `piece`
                We retain **answer = answer + piece**.
              2. *
                **_ = 1**
            2. *
              We retain **`msg` = [Read the message field](json.get(value=`c0`, key="message"))**.
              1. `msg`
                We retain **`piece` = [Read the content field](json.get(value=`msg`, key="content"))**.
                1. `piece`
                  We retain **answer = answer + piece**.
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
  6. *
    **_ = 1**

1. not `answer`
  We retain **answer = ""**.
Return *answer*.

## collect_usage

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Best-effort `total_tokens` from stream frames (requires `stream_options.include_usage`). |

Use `events` as the inputs for this operation.

Best-effort `total_tokens` from stream frames (requires `stream_options.include_usage`).

We retain **tokens = None**.
1. `events`
  - [ev](遍历 events 为 ev)
    We retain **`t` = [Read the type field](json.get(value=`ev`, key="type"))**.
    We retain **raw = None**.
    1. `t` == "data"
      We retain **`raw` = [Read the data field](json.get(value=`ev`, key="data"))**.
    2. *
      We retain **`u0` = [Read the usage field](json.get(value=`ev`, key="usage"))**.
      1. `u0`
        We retain **`tt0` = [Read the total_tokens field](json.get(value=`u0`, key="total_tokens"))**.
        1. `tt0` != None
          We retain **tokens = `tt0`**.
    1. `raw`
      1. not `raw` == "[DONE]"
        We retain **`parsed` = [Parse the JSON transport text](json.parse(text=`raw`))**.
        1. `parsed`
          We retain **`u` = [Read the usage field](json.get(value=`parsed`, key="usage"))**.
          1. `u`
            We retain **`tt` = [Read the total_tokens field](json.get(value=`u`, key="total_tokens"))**.
            1. `tt` != None
              We retain **tokens = `tt`**.
Return *tokens*.

## stream_result

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | any | required | Ordered semantic or provider stream-event records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Compatibility alias for `collect`. |

Use `events` as the inputs for this operation.

Compatibility alias for `collect`.

Return *[Collect](collect(events=`events`))*.

## usage_from

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map (or empty zeros). |

Use `data` as the inputs for this operation.

Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map (or empty zeros).

Return *[Usage from](openai.usage_from(data=`data`))*.

## create

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |
| backend | any | "openai-compatible" | Configured execution or provider backend |
| name | any | None | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Factory for an LLM handle (same as constructing `# llm`). Env fallbacks apply inside the constructor. Optional `name=` tags the handle (`fast` / `reasoning`). |

Use `model`=None, `base_url`=None, `api_key`=None, `backend`="openai-compatible", `name`=None as the inputs for this operation.

Factory for an LLM handle (same as constructing `# llm`). Env fallbacks apply inside the constructor. Optional `name=` tags the handle (`fast` / `reasoning`).

We retain **`h` = [Llm](llm(model=`model`, base_url=`base_url`, api_key=`api_key`, backend=`backend`, name=`name`))**.
Return *h*.

## fast

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Named small/fast handle. Model from arg, else `MARQDO_LLM_FAST_MODEL`, else default create model. |

Use `model`=None, `base_url`=None, `api_key`=None as the inputs for this operation.

Named small/fast handle. Model from arg, else `MARQDO_LLM_FAST_MODEL`, else default create model.

1. `model`
  We retain **mdl = `model`**.
2. *
  We retain **`mdl` = [Env get](sys.env_get(name="MARQDO_LLM_FAST_MODEL"))**.

Return *[Create](create(model=`mdl`, base_url=`base_url`, api_key=`api_key`, name="fast", backend="openai-compatible"))*.

## reasoning

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Named strong/reasoning handle. Model from arg, else `MARQDO_LLM_REASONING_MODEL`, else default create model. |

Use `model`=None, `base_url`=None, `api_key`=None as the inputs for this operation.

Named strong/reasoning handle. Model from arg, else `MARQDO_LLM_REASONING_MODEL`, else default create model.

1. `model`
  We retain **mdl = `model`**.
2. *
  We retain **`mdl` = [Env get](sys.env_get(name="MARQDO_LLM_REASONING_MODEL"))**.

Return *[Create](create(model=`mdl`, base_url=`base_url`, api_key=`api_key`, name="reasoning", backend="openai-compatible"))*.

## prompt_body

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| src | any | required | Prompt source or source-reference value |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | If `src` starts with YAML frontmatter declaring `type: prompt` / `类型: 提示`, return the body after the closing `---`; otherwise return `src` unchanged. |

Use `src` as the inputs for this operation.

If `src` starts with YAML frontmatter declaring `type: prompt` / `类型: 提示`, return the body after the closing `---`; otherwise return `src` unchanged.

We retain **`sw` = [Starts with](text.starts_with(text=`src`, prefix="---"))**.
1. not `sw`
  Return *src*.
2. *
  We retain **`parts` = [Split](split(value=`src`, sep="\n---\n"))**.
  We retain **`n` = [Len](len(value=`parts`))**.
  1. `n` < 2
    Return *src*.
  2. *
    We retain **`fm` = [At](at(value=`parts`, index=0))**.
    We retain **`is_p` = [Contains](text.contains(text=`fm`, sub="type: prompt"))**.
    1. not `is_p`
      We retain **`is_p` = [Contains](text.contains(text=`fm`, sub="类型: 提示"))**.
    2. *
      **_ = 1**
    1. `is_p`
      We retain **`body` = [At](at(value=`parts`, index=1))**.
      Return *[Str trim](text.str_trim(s=`body`))*.
    2. *
      Return *src*.

## prompt_load

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load a prompt artifact (`.mq.md` / `.md`) from disk for `ask`. |

Use `path` as the inputs for this operation.

Load a prompt artifact (`.mq.md` / `.md`) from disk for `ask`.

We retain **`raw` = [Read UTF-8 text from the selected file](fs.read_text(path=`path`))**.
Return *[Prompt body](prompt_body(src=`raw`))*.

## prompt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| text | any | None | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Prompt artifact helper: `path=` loads a file; `text=` strips `type: prompt` frontmatter from an in-memory string. |

Use `path`=None, `text`=None as the inputs for this operation.

Prompt artifact helper: `path=` loads a file; `text=` strips `type: prompt` frontmatter from an in-memory string.

1. `path`
  Return *[Prompt load](prompt_load(path=`path`))*.
2. `text`
  Return *[Prompt body](prompt_body(src=`text`))*.
3. *
  We **[Print the prepared message](print(text="ext/ai/llm.prompt: pass", path="or", text=""))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

## ask

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | None | Prompt text or structured prompt expected by this adapter |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Module convenience: create a default handle and return **answer text**. Pass `path=` to load a prompt document, `prompt=` text, or omit both to use the entry `.mq.md` as a prompt artifact (`type: prompt` body via `sys.module_source`). |

Use `prompt`=None, `path`=None, `model`=None, `base_url`=None, `api_key`=None as the inputs for this operation.

Module convenience: create a default handle and return **answer text**. Pass `path=` to load a prompt document, `prompt=` text, or omit both to use the entry `.mq.md` as a prompt artifact (`type: prompt` body via `sys.module_source`).

We retain **p = `prompt`**.
1. `path`
  We retain **`p` = [Prompt load](prompt_load(path=`path`))**.
2. `prompt`
  We retain **p = `prompt`**.
3. *
  We retain **`raw` = [Module source](sys.module_source())**.
  1. not `raw`
    We **[Print the prepared message](print(text="ext/ai/llm.ask: pass", prompt="or", path="(or run a prompt document as entry)"))**.
    We **[End this run with the selected exit status](sys.exit(code=1))**.
  2. *
    We retain **`p` = [Prompt body](prompt_body(src=`raw`))**.

We retain **`m` = [Create](create(model=`model`, base_url=`base_url`, api_key=`api_key`))**.
We retain **`r` = [Ask](m.ask(prompt=`p`))**.
We retain **`_ret` = [Read the text field](json.get(value=`r`, key="text"))**.
Return *_ret*.

# llm

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |
| backend | any | "openai-compatible" | Configured execution or provider backend |
| name | any | None | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | LLM handle — intelligence primitive, not an HTTP client. Resolution: explicit args → entry-scope / Artifact Metadata (`sys.meta_get`) → `sys.env_get` → defaults. Prefer declaring `model` / `api_key` / `base_url` in frontmatter Binding, then use the lifted names. |

Use `model`=None, `base_url`=None, `api_key`=None, `backend`="openai-compatible", `name`=None as the inputs for this operation.

LLM handle — intelligence primitive, not an HTTP client.

Resolution: explicit args → entry-scope / Artifact Metadata (`sys.meta_get`) → `sys.env_get` → defaults.
Prefer declaring `model` / `api_key` / `base_url` in frontmatter Binding, then use the lifted names.

1. `api_key`
  We retain **key = `api_key`**.
2. *
  We retain **`key` = [Meta get](sys.meta_get(key="api_key"))**.
  1. not `key`
    We retain **`key` = [Env get](sys.env_get(name="OPENAI_API_KEY"))**.
    1. not `key`
      We retain **`key` = [Env get](sys.env_get(name="MARQDO_LLM_API_KEY"))**.
    2. *
      **_ = 1**
  2. *
    **_ = 1**

1. `backend` == ollama
  1. not `key`
    We retain **key = "ollama"**.
  2. *
    **_ = 1**
2. not `key`
  We **[Print the prepared message](print(text="ext/ai/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY (or pass api_key= / metadata api_key)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
3. *
  **_ = 1**

1. `base_url`
  We retain **url = `base_url`**.
2. *
  We retain **`url` = [Meta get](sys.meta_get(key="base_url"))**.
  1. `url`
    **_ = 1**
  2. `backend` == ollama
    We retain **`url` = [Env get](sys.env_get(name="OLLAMA_HOST"))**.
    1. not `url`
      We retain **url = "http://127.0.0.1:11434/v1"**.
    2. *
      **_ = 1**
  3. *
    We retain **`url` = [Env get](sys.env_get(name="OPENAI_BASE_URL"))**.
    1. not `url`
      We retain **`url` = [Env get](sys.env_get(name="MARQDO_LLM_BASE_URL"))**.
    2. *
      **_ = 1**
    1. not `url`
      We retain **url = "https://api.openai.com/v1"**.
    2. *
      **_ = 1**

1. `model`
  We retain **mdl = `model`**.
2. *
  We retain **`mdl` = [Meta get](sys.meta_get(key="model"))**.
  1. `mdl`
    **_ = 1**
  2. *
    We retain **`mdl` = [Env get](sys.env_get(name="OPENAI_MODEL"))**.
    1. not `mdl`
      We retain **`mdl` = [Env get](sys.env_get(name="MARQDO_LLM_MODEL"))**.
    2. *
      **_ = 1**
    1. not `mdl`
      1. `backend` == ollama
        We retain **mdl = "llama3.2"**.
      2. *
        We retain **mdl = "gpt-4o-mini"**.
    2. *
      **_ = 1**

1. `name`
  We retain **nm = `name`**.
2. *
  We retain **nm = None**.

`h` =

| _type | backend | api_key | base_url | model | suffix | bearer | name |
|-------|---------|---------|----------|-------|--------|--------|------|
| llm | `backend` | `key` | `url` | `mdl` | /chat/completions | "Bearer " | `nm` |

Return *h*.

## ask

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | None | Prompt text or structured prompt expected by this adapter |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| messages | any | None | Ordered chat message records |
| selection | any | None | Explicit runtime/model selection |
| strict_selection | any | False | Whether an unresolved selection is an error |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Semantic ask → **LLMResult** (`text`, `model`, `finish`, `backend`, `usage`, `name`, `tool_calls`). Prefer `[text](result)` for the answer string. Omit `prompt=` / `path=` to use the entry document (`sys.module_source` + `prompt_body`). Pass `messages=` for multi-turn history (M01). `selection=` + `strict_selection=True` freeze request identity without env fallback. |

Use `prompt`=None, `path`=None, `messages`=None, `selection`=None, `strict_selection`=False as the inputs for this operation.

Semantic ask → **LLMResult** (`text`, `model`, `finish`, `backend`, `usage`, `name`, `tool_calls`). Prefer `[text](result)` for the answer string. Omit `prompt=` / `path=` to use the entry document (`sys.module_source` + `prompt_body`). Pass `messages=` for multi-turn history (M01). `selection=` + `strict_selection=True` freeze request identity without env fallback.

We retain **p = `prompt`**.
1. `messages`
  We retain **p = None**.
2. `path`
  We retain **`p` = [Prompt load](prompt_load(path=`path`))**.
3. `prompt`
  We retain **p = `prompt`**.
4. *
  We retain **`raw` = [Module source](sys.module_source())**.
  1. not `raw`
    We retain **p = ""**.
  2. *
    We retain **`p` = [Prompt body](prompt_body(src=`raw`))**.
We retain **`pack` = [Complete result](self.complete_result(prompt=`p`, messages=`messages`, selection=`selection`, strict_selection=`strict_selection`, stream=False, echo=False))**.
We retain **`text` = [Read the text field](json.get(value=`pack`, key="text"))**.
We retain **`usage` = [Read the usage field](json.get(value=`pack`, key="usage"))**.
We retain **`finish` = [Read the finish field](json.get(value=`pack`, key="finish"))**.
We retain **`err` = [Read the error field](json.get(value=`pack`, key="error"))**.
We retain **`tc` = [Read the tool_calls field](json.get(value=`pack`, key="tool_calls"))**.
1. not `tc`
  We retain **`tc` = [Empty list](table.empty_list())**.
2. *
  **_ = 1**
1. not `text`
  We retain **text = ""**.
2. *
  **_ = 1**
1. not `finish`
  1. `err`
    We retain **finish = "error"**.
  2. *
    We retain **finish = "stop"**.
2. *
  **_ = 1**

Hoist receiver fields before the result table — table cells can shadow `self` /
column names (`model`, `usage`), so `[model](self)` / `[total_tokens](usage)`
inside the table is unsafe.

We retain **`mdl` = [Read the model field](json.get(value=`self`, key="model"))**.
We retain **`b` = [Read the backend field](json.get(value=`self`, key="backend"))**.
We retain **`nm` = [Read the name field](json.get(value=`self`, key="name"))**.
1. `usage`
  We retain **`tt` = [Read the total_tokens field](json.get(value=`usage`, key="total_tokens"))**.
  We retain **`ustate` = [Read the usage_state field](json.get(value=`usage`, key="usage_state"))**.
2. *
  We retain **tt = None**.
  We retain **ustate = "unknown"**.
1. `ustate` == "unknown"
  Keep null — never coerce missing usage to zero tokens (M02 / F04).
  We retain **tt = None**.
2. not `tt`
  We retain **tt = None**.
3. *
  **_ = 1**
1. `err`
  We retain **calls = 0**.
2. *
  We retain **calls = 1**.

`result` =

| text | model | finish | backend | usage | name | llm_calls | tokens | tool_calls | error | selection |
|------|-------|--------|---------|-------|------|-----------|--------|------------|-------|-----------|
| `text` | `mdl` | `finish` | `b` | `usage` | `nm` | `calls` | `tt` | `tc` | `err` | `selection` |

Return *result*.

## stream

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | None | Prompt text or structured prompt expected by this adapter |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Semantic stream: return the event list (use `llm.collect` to reduce to text). Omit `prompt=` / `path=` to use the entry document. |

Use `prompt`=None, `path`=None, `echo`=False, `stream_kind`="delta" as the inputs for this operation.

Semantic stream: return the event list (use `llm.collect` to reduce to text). Omit `prompt=` / `path=` to use the entry document.
`stream_kind=plan` tags EventBus frames for the parent plan protocol (process trail).

1. `path`
  We retain **`p` = [Prompt load](prompt_load(path=`path`))**.
2. `prompt`
  We retain **p = `prompt`**.
3. *
  We retain **`raw` = [Module source](sys.module_source())**.
  1. not `raw`
    We retain **p = ""**.
  2. *
    We retain **`p` = [Prompt body](prompt_body(src=`raw`))**.

Return *[Complete](self.complete(prompt=`p`, stream=True, echo=`echo`, stream_kind=`stream_kind`))*.

## complete_result

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | None | Prompt text or structured prompt expected by this adapter |
| messages | any | None | Ordered chat message records |
| selection | any | None | Explicit runtime/model selection |
| strict_selection | any | False | Whether an unresolved selection is an error |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Transport pack via `ext/ai/llm/openai` (Ollama uses the same OpenAI-compatible path). Non-stream → `{text,usage,finish}`; stream → `{events}`. Prefer `messages=` for multi-turn; optional `selection=` (ModelSelectionV1) with `strict_selection=True` for request-scoped identity (M01). |

Use `prompt`=None, `messages`=None, `selection`=None, `strict_selection`=False, `stream`=False, `echo`=False, `stream_kind`="delta" as the inputs for this operation.

Transport pack via `ext/ai/llm/openai` (Ollama uses the same OpenAI-compatible path). Non-stream → `{text,usage,finish}`; stream → `{events}`. Prefer `messages=` for multi-turn; optional `selection=` (ModelSelectionV1) with `strict_selection=True` for request-scoped identity (M01).

Return *[Chat completions](openai.chat_completions(base_url=[base_url](access(`self`, "base_url")), api_key=[api_key](access(`self`, "api_key")), model=[model](access(`self`, "model")), prompt=`prompt`, messages=`messages`, selection=`selection`, strict_selection=`strict_selection`, stream=`stream`, echo=`echo`, stream_kind=`stream_kind`, suffix=[suffix](access(`self`, "suffix")), bearer=[bearer](access(`self`, "bearer"))))*.

## complete

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | required | Prompt text or structured prompt expected by this adapter |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Internal / compatibility primitive. Prefer `ask` / `stream` in new code. |

Use `prompt`, `stream`=False, `echo`=False, `stream_kind`="delta" as the inputs for this operation.

Internal / compatibility primitive. Prefer `ask` / `stream` in new code.

1. `stream`
  We retain **`pack` = [Complete result](self.complete_result(prompt=`prompt`, stream=True, echo=`echo`, stream_kind=`stream_kind`))**.
  We retain **`_ret` = [Read the events field](json.get(value=`pack`, key="events"))**.
  Return *_ret*.
2. *
  We retain **`pack` = [Complete result](self.complete_result(prompt=`prompt`, stream=False, echo=`echo`, stream_kind=`stream_kind`))**.
  We retain **`_ret` = [Read the text field](json.get(value=`pack`, key="text"))**.
  Return *_ret*.

## chat

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | any | required | Prompt text or structured prompt expected by this adapter |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Alias for `complete` (compat). |

Use `prompt`, `stream`=False, `echo`=False, `stream_kind`="delta" as the inputs for this operation.

Alias for `complete` (compat).

Return *[Complete](self.complete(prompt=`prompt`, stream=`stream`, echo=`echo`, stream_kind=`stream_kind`))*.
