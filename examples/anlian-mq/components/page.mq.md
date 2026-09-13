---
title: components/page
description: 用原站 chrome 包一层完整 HTML 引言（bare 布局）。
import text:lib/text.mq.md
import chrome:chrome.mq.md
---

## 包装
    + `正文`
    + `体类`=""

导航 + 正文 + 页脚；体类如 `page-home`。

**顶 = > chrome.导航HTML**
**底 = > chrome.页脚HTML**
**鉴权脚本 = "<script>(function(){var c=document.cookie.indexOf('marqdo_sid=')>=0;document.querySelectorAll('.nav-when-guest').forEach(function(el){el.style.display=c?'none':'';});document.querySelectorAll('.nav-when-auth').forEach(function(el){el.style.display=c?'':'none';});})();</script>"**
**鉴权样式 = "<style>.nav-when-auth{display:none}</style>"**
*鉴权样式 + 顶 + "<main class=\"anlian-main " + 体类 + "\">" + 正文 + "</main>" + 底 + 鉴权脚本*
