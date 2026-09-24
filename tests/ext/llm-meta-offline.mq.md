---
title: llm meta binding offline
description: Create handle from Artifact Metadata (model / api_key); no network.
type: llm
model: ${env.MARQDO_LLM_META_MODEL ?? "meta-model"}
api_key: ${secret.OPENAI_API_KEY ?? "sk-test"}
import llm:ext/ai/llm.mq.md
---

# main

**h = > llm.llm**
> print text=[model](`h`)
> print text=[api_key](`h`)
