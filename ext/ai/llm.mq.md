---
title: ext/ai/llm
description: OpenAI-compatible LLM object (English). Import ext/ai/llm.mq.md.
> lib/sys.mq.md
> lib/net.mq.md
> lib/json.mq.md
---

## load_env
    + `path`=None

Load `.env` from cwd (optional named arg `path=`). Does not override existing variables.

**> sys.load_dotenv path=`path`**

## stream_result
    + `events`

Reduce a `complete stream=True` event list to the final `done.result` text (or `error.message`).

*`answer` = "" *

- [`ev`](`events`)
  *`t` = > json.get value=`ev` key=type *
  1. `t` == "done"
    *`answer` = > json.get value=`ev` key=result *
  2. `t` == "error"
    *`answer` = > json.get value=`ev` key=message *
  3. *
    *`_` = 1*

**`answer`**

# llm

Construct an LLM handle from `OPENAI_*` / `MARQDO_LLM_*` (default base OpenAI v1, model `gpt-4o-mini`).

*`d` = > json.parse text={"base":"https://api.openai.com/v1","model":"gpt-4o-mini","suffix":"/chat/completions","bearer":"Bearer ","p1":"{\"model\":","p2":",\"messages\":[{\"role\":\"user\",\"content\":","p3":"}]}","h1":"{\"Authorization\":","h2":"}","a1":"{\"api_key\":","a2":",\"base_url\":","a3":",\"model\":","a4":",\"suffix\":","a5":",\"bearer\":","a6":",\"p1\":","a7":",\"p2\":","a8":",\"p3\":","a9":",\"h1\":","a10":",\"h2\":","a11":"}"} *

*`api_key` = > sys.env_get name=OPENAI_API_KEY *
1. `api_key`
  *`_k` = 1*
2. *
  *`api_key` = > sys.env_get name=MARQDO_LLM_API_KEY *
3. `api_key`
  *`_k` = 1*
4. *
  > print text=ext/ai/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY
  > sys.exit code=1

*`base_url` = > sys.env_get name=OPENAI_BASE_URL *
1. `base_url`
  *`_k` = 1*
2. *
  *`base_url` = > sys.env_get name=MARQDO_LLM_BASE_URL *
3. `base_url`
  *`_k` = 1*
4. *
  *`base_url` = > json.get value=`d` key=base *

*`model` = > sys.env_get name=OPENAI_MODEL *
1. `model`
  *`_k` = 1*
2. *
  *`model` = > sys.env_get name=MARQDO_LLM_MODEL *
3. `model`
  *`_k` = 1*
4. *
  *`model` = > json.get value=`d` key=model *

*`q_key` = > json.quote text=`api_key` *
*`q_base` = > json.quote text=`base_url` *
*`q_model` = > json.quote text=`model` *
*`suffix` = > json.get value=`d` key=suffix *
*`bearer` = > json.get value=`d` key=bearer *
*`p1` = > json.get value=`d` key=p1 *
*`p2` = > json.get value=`d` key=p2 *
*`p3` = > json.get value=`d` key=p3 *
*`h1` = > json.get value=`d` key=h1 *
*`h2` = > json.get value=`d` key=h2 *
*`q_suf` = > json.quote text=`suffix` *
*`q_br` = > json.quote text=`bearer` *
*`q_p1` = > json.quote text=`p1` *
*`q_p2` = > json.quote text=`p2` *
*`q_p3` = > json.quote text=`p3` *
*`q_h1` = > json.quote text=`h1` *
*`q_h2` = > json.quote text=`h2` *
*`a1` = > json.get value=`d` key=a1 *
*`a2` = > json.get value=`d` key=a2 *
*`a3` = > json.get value=`d` key=a3 *
*`a4` = > json.get value=`d` key=a4 *
*`a5` = > json.get value=`d` key=a5 *
*`a6` = > json.get value=`d` key=a6 *
*`a7` = > json.get value=`d` key=a7 *
*`a8` = > json.get value=`d` key=a8 *
*`a9` = > json.get value=`d` key=a9 *
*`a10` = > json.get value=`d` key=a10 *
*`a11` = > json.get value=`d` key=a11 *
*`raw` = `a1` + `q_key` + `a2` + `q_base` + `a3` + `q_model` + `a4` + `q_suf` + `a5` + `q_br` + `a6` + `q_p1` + `a7` + `q_p2` + `a8` + `q_p3` + `a9` + `q_h1` + `a10` + `q_h2` + `a11` *
**> json.parse text=`raw`**

## complete
    + `prompt`
    + `stream`=False
    + `echo`=False

Chat completion using `self` / `自` handle fields.

With `stream=True`, returns a list of event maps (`delta` / `done` / `error`) for foreach; default remains a single answer string. Optional `echo=True` prints deltas to stdout while the SSE body is read.

*`api_key` = > json.get value=`self` key=api_key *
*`base_url` = > json.get value=`self` key=base_url *
*`model` = > json.get value=`self` key=model *
*`suffix` = > json.get value=`self` key=suffix *
*`bearer` = > json.get value=`self` key=bearer *
*`p1` = > json.get value=`self` key=p1 *
*`p2` = > json.get value=`self` key=p2 *
*`p3` = > json.get value=`self` key=p3 *
*`h1` = > json.get value=`self` key=h1 *
*`h2` = > json.get value=`self` key=h2 *
*`url` = `base_url` + `suffix` *
*`auth` = `bearer` + `api_key` *
*`q_auth` = > json.quote text=`auth` *
*`hdr_raw` = `h1` + `q_auth` + `h2` *
*`headers` = > json.parse text=`hdr_raw` *
*`q_model` = > json.quote text=`model` *
*`q_prompt` = > json.quote text=`prompt` *

1. `stream`
  *`p3s` = "}],\"stream\":true}" *
  *`body` = `p1` + `q_model` + `p2` + `q_prompt` + `p3s` *
  *`resp` = > net.http_post_sse url=`url` body=`body` headers=`headers` echo=`echo` *
  *`status` = > json.get value=`resp` key=status *
  *`events` = > json.get value=`resp` key=events *
  1. `status` == 200
    **`events`**
  2. *
    > print text=ext/ai/llm: HTTP error (stream)
    > print text=`status`
    > sys.exit code=1
2. *
  *`body` = `p1` + `q_model` + `p2` + `q_prompt` + `p3` *
  *`resp` = > net.http_post url=`url` body=`body` headers=`headers` *
  *`status` = > json.get value=`resp` key=status *
  *`raw` = > json.get value=`resp` key=body *
  1. `status` == 200
    *`data` = > json.parse text=`raw` *
    *`choices` = > json.get value=`data` key=choices *
    *`first` = > at value=`choices` index=0 *
    *`message` = > json.get value=`first` key=message *
    *`content` = > json.get value=`message` key=content *
    **`content`**
  2. *
    > print text=ext/ai/llm: HTTP error
    > print text=`status`
    > print text=`raw`
    > sys.exit code=1

---

## chat
    + `prompt`
    + `stream`=False
    + `echo`=False

Alias for `complete`.

**> `self`.complete prompt=`prompt` stream=`stream` echo=`echo`**
