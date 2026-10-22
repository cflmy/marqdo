---
markup: occlusion
title: net cookie_parse
import net:lib/net.mq.md
---

# main

**req = > net.cookie_parse text="session=abc123; theme=dark"**

**resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; Secure; SameSite=Lax, theme=light; Max-Age=3600" is_response=True**

> print text=[name](access([1](access(`req`, "1")), "name"))
> print text=[value](access([1](access(`req`, "1")), "value"))
> print text=[http_only](access([1](access(`resp`, "1")), "http_only"))
> print text=[secure](access([1](access(`resp`, "1")), "secure"))
> print text=[same_site](access([1](access(`resp`, "1")), "same_site"))
> print text=[max_age](access([2](access(`resp`, "2")), "max_age"))
