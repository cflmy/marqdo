---
title: net cookie_parse
import net:lib/net.mq.md
---

# main

**req = > net.cookie_parse text="session=abc123; theme=dark"**

**resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; Secure; SameSite=Lax, theme=light; Max-Age=3600" is_response=True**

> print text=[name]([1](`req`))
> print text=[value]([1](`req`))
> print text=[http_only]([1](`resp`))
> print text=[secure]([1](`resp`))
> print text=[same_site]([1](`resp`))
> print text=[max_age]([2](`resp`))
