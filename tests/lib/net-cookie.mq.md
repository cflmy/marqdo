---
title: net cookie_parse
import net:lib/net.mq.md
---

# main

*req = > net.cookie_parse text="session=abc123; theme=dark"*

*resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; Secure; SameSite=Lax, theme=light; Max-Age=3600" is_response=True*

> print text=`req`[^1][^name]
> print text=`req`[^1][^value]
> print text=`resp`[^1][^http_only]
> print text=`resp`[^1][^secure]
> print text=`resp`[^1][^same_site]
> print text=`resp`[^2][^max_age]
