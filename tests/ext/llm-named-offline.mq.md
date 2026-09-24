---
title: llm named handles + prompt_load offline
description: fast/reasoning names, ollama backend tag, and type:prompt body load; no network.
import llm:ext/ai/llm.mq.md
import text:lib/text.mq.md
---

# main

**fast = > llm.fast model="qwen-fast-test"**
> print text=[name](`fast`)
> print text=[model](`fast`)

**reason = > llm.reasoning model="gpt-reason-test"**
> print text=[name](`reason`)
> print text=[model](`reason`)

**ollama = > llm.create backend="ollama" model="llama3.2"**
> print text=[backend](`ollama`)
> print text=[model](`ollama`)

**prompt = > llm.prompt_load path="fixtures/prompt-marqdo.md"**
**has = > text.contains text=`prompt` sub="Explain Marqdo"**
**no_fm = > text.contains text=`prompt` sub="type: prompt"**
> print text=`has`
> print text=`no_fm`
