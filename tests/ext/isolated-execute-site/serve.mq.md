---
markup: occlusion
title: isolated execute MR-01 site
description: Dual-stream execute + short models for ADR 0009 gold sample.
import:
  web: ext/web/web.mq.md
  site: ./site.mq.md
---

# main

**pg = > web.page title="isolated-execute"**
**app = > web.app page=`pg` host="127.0.0.1" port=18220**
**app = > web.route app=`app` path="/v1/chat/completions" method="POST" fn="site.chat" request="json" response="sse-live" class="execute"**
**app = > web.route app=`app` path="/v1/models" method="GET" fn="site.models" request="query" response="json" class="short"**
> `app`.listen
