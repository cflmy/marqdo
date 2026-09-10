---
title: web G12 API key smoke (W-G12)
description: Offline web_api_key_check via Go libweb.so.
import plugin:lib/plugin.mq.md
---

# main

*p = "../../plugins/web/build/libweb.so"*
> plugin.load path=`p`

*pepper = "smoke-pepper"*

`keys` =

| hash | scopes |
|------|--------|
| 0ae9134cca7eff9d756fbf52402ee28ae0c2be22f01e53653e4fe86021479703 | read,api |

*ok = > web_api_key_check key="smoke-secret" authorization="" pepper=`pepper` keys=`keys`*
1. ok[^ok]
  > print text=api-key-ok
2. *
  > print text=api-key-fail

*bad = > web_api_key_check key="wrong" authorization="" pepper=`pepper` keys=`keys`*
1. bad[^ok]
  > print text=api-key-bad-ok
2. *
  > print text=api-key-bad-fail
