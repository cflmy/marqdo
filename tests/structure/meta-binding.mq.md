---
title: meta-binding
model: ${env.MARQDO_META_MODEL ?? "default-model"}
greeting: "Hello ${env.MARQDO_META_USER ?? "World"}"
tokens: ${int(env.MARQDO_META_TOKENS ?? "16")}
api_key: ${secret.MARQDO_META_SECRET ?? "sk-test"}
import sys:lib/sys.mq.md
---

# main

**m = > sys.meta_get key="model"**
> print text=`m`

**g = > sys.meta_get key="greeting"**
> print text=`g`

**t = > sys.meta_get key="tokens"**
> print text=`t`

**k = > sys.meta_get key="api_key"**
> print text=`k`

**ov = > sys.meta_get key="extra"**
1. `ov`
  > print text=`ov`
2. *
  > print text=no-extra
