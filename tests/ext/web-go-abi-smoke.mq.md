---
title: web Go ABI smoke (W-G0)
description: Load Go libweb.so; probe web_go_ready + web_page_new.
import plugin:lib/plugin.mq.md
---

# main

*p = "../../plugins/web/build/libweb.so"*
> plugin.load path=`p`

*r = > web_go_ready*
> print text=`r`

*page = > web_page_new title="Go Web" intro="W-G0" shell_css="" layout="" asset_version=""*
> print text=`page`
