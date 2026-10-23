---
markup: occlusion
title: llm named handles + prompt_load offline
description: fast/reasoning names, ollama backend tag, and type:prompt body load; no network.
import:
  llm: ext/ai/llm.mq.md
  text: lib/text.mq.md
---

# main

**fast = > llm.fast model="qwen-fast-test"**
> print text=[name](access(`fast`, "name"))
> print text=[model](access(`fast`, "model"))

**reason = > llm.reasoning model="gpt-reason-test"**
> print text=[name](access(`reason`, "name"))
> print text=[model](access(`reason`, "model"))

**ollama = > llm.create backend="ollama" model="llama3.2"**
> print text=[backend](access(`ollama`, "backend"))
> print text=[model](access(`ollama`, "model"))

**prompt = > llm.prompt_load path="fixtures/prompt-marqdo.md"**
**has = > text.contains text=`prompt` sub="Explain Marqdo"**
**no_fm = > text.contains text=`prompt` sub="type: prompt"**
> print text=`has`
> print text=`no_fm`
