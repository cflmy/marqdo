---
title: ext/llm
description: Official OpenAI-compatible LLM object (English). Not stdlib — import ext/llm.mq.md.
> lib/sys.mq.md
> lib/net.mq.md
> lib/json.mq.md
---

## load_env

Load `.env` from cwd (optional named arg `path=`). Does not override existing variables.

**> load_dotenv**

# llm

Construct an LLM handle from `OPENAI_*` / `MARQDO_LLM_*` (default base OpenAI v1, model `gpt-4o-mini`).

*`d` = > parse text={"base":"https://api.openai.com/v1","model":"gpt-4o-mini","suffix":"/chat/completions","bearer":"Bearer ","p1":"{\"model\":","p2":",\"messages\":[{\"role\":\"user\",\"content\":","p3":"}]}","h1":"{\"Authorization\":","h2":"}","a1":"{\"api_key\":","a2":",\"base_url\":","a3":",\"model\":","a4":",\"suffix\":","a5":",\"bearer\":","a6":",\"p1\":","a7":",\"p2\":","a8":",\"p3\":","a9":",\"h1\":","a10":",\"h2\":","a11":"}"} *

*`api_key` = > env_get name=OPENAI_API_KEY *
+ `api_key`
  *`_k` = 1*
+ *
  *`api_key` = > env_get name=MARQDO_LLM_API_KEY *
+ `api_key`
  *`_k` = 1*
+ *
  > print text=ext/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY
  > exit code=1

*`base_url` = > env_get name=OPENAI_BASE_URL *
+ `base_url`
  *`_k` = 1*
+ *
  *`base_url` = > env_get name=MARQDO_LLM_BASE_URL *
+ `base_url`
  *`_k` = 1*
+ *
  *`base_url` = > get value=`d` key=base *

*`model` = > env_get name=OPENAI_MODEL *
+ `model`
  *`_k` = 1*
+ *
  *`model` = > env_get name=MARQDO_LLM_MODEL *
+ `model`
  *`_k` = 1*
+ *
  *`model` = > get value=`d` key=model *

*`q_key` = > quote text=`api_key` *
*`q_base` = > quote text=`base_url` *
*`q_model` = > quote text=`model` *
*`suffix` = > get value=`d` key=suffix *
*`bearer` = > get value=`d` key=bearer *
*`p1` = > get value=`d` key=p1 *
*`p2` = > get value=`d` key=p2 *
*`p3` = > get value=`d` key=p3 *
*`h1` = > get value=`d` key=h1 *
*`h2` = > get value=`d` key=h2 *
*`q_suf` = > quote text=`suffix` *
*`q_br` = > quote text=`bearer` *
*`q_p1` = > quote text=`p1` *
*`q_p2` = > quote text=`p2` *
*`q_p3` = > quote text=`p3` *
*`q_h1` = > quote text=`h1` *
*`q_h2` = > quote text=`h2` *
*`a1` = > get value=`d` key=a1 *
*`a2` = > get value=`d` key=a2 *
*`a3` = > get value=`d` key=a3 *
*`a4` = > get value=`d` key=a4 *
*`a5` = > get value=`d` key=a5 *
*`a6` = > get value=`d` key=a6 *
*`a7` = > get value=`d` key=a7 *
*`a8` = > get value=`d` key=a8 *
*`a9` = > get value=`d` key=a9 *
*`a10` = > get value=`d` key=a10 *
*`a11` = > get value=`d` key=a11 *
*`raw` = `a1` + `q_key` + `a2` + `q_base` + `a3` + `q_model` + `a4` + `q_suf` + `a5` + `q_br` + `a6` + `q_p1` + `a7` + `q_p2` + `a8` + `q_p3` + `a9` + `q_h1` + `a10` + `q_h2` + `a11` *
**> parse text=`raw`**

## complete
    - prompt

Chat completion using `self` / `自` handle fields.

*`api_key` = > get value=`self` key=api_key *
*`base_url` = > get value=`self` key=base_url *
*`model` = > get value=`self` key=model *
*`suffix` = > get value=`self` key=suffix *
*`bearer` = > get value=`self` key=bearer *
*`p1` = > get value=`self` key=p1 *
*`p2` = > get value=`self` key=p2 *
*`p3` = > get value=`self` key=p3 *
*`h1` = > get value=`self` key=h1 *
*`h2` = > get value=`self` key=h2 *
*`url` = `base_url` + `suffix` *
*`auth` = `bearer` + `api_key` *
*`q_auth` = > quote text=`auth` *
*`hdr_raw` = `h1` + `q_auth` + `h2` *
*`headers` = > parse text=`hdr_raw` *
*`q_model` = > quote text=`model` *
*`q_prompt` = > quote text=`prompt` *
*`body` = `p1` + `q_model` + `p2` + `q_prompt` + `p3` *
*`resp` = > http_post url=`url` body=`body` headers=`headers` *
*`status` = > get value=`resp` key=status *
*`raw` = > get value=`resp` key=body *

+ `status` == 200
  *`data` = > parse text=`raw` *
  *`choices` = > get value=`data` key=choices *
  *`first` = > at value=`choices` index=0 *
  *`message` = > get value=`first` key=message *
  *`content` = > get value=`message` key=content *
  **`content`**
+ *
  > print text=ext/llm: HTTP error
  > print text=`status`
  > print text=`raw`
  > exit code=1

---

## chat
    - prompt

Alias for `complete`.

**> `self`.complete prompt=`prompt`**
