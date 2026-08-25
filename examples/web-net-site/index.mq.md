---
title: web-net-site
description: Home + /about + /new (form) + /tools (net lib) + /admin (auth) + /live (WebSocket).
import web:ext/web/web.mq.md
import net:lib/net.mq.md
import text:lib/text.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

`home` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side_panel` |
| foot.`foot` | |

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.body | shell.`card_body` |

Home intro also embeds the live WebSocket widget (HTML is served raw).

*store = > db.open*
*page = > web.page title="Marqdo Net Site" intro="<h1>Net Site</h1><p>Articles from SQLite, a live WebSocket widget, login-gated admin, and net-library parsing tools.</p><h2 id=\"live\">Live WebSocket</h2><p id=\"live-status\">connecting…</p><input id=\"live-msg\" placeholder=\"type a message…\"><button id=\"live-send\" disabled>send</button><div id=\"live-out\"></div><script src=\"/static/live.js\"></script>"*
*page = > page.compose_components components=home*
*page = > page.compose_main main=index*

About page: same shell.

*about = > web.page title="About" intro="<h1>About</h1><p>Demonstrates the network-extended web library: sessions, auth, WebSocket, multipart/cookie parsing.</p>"*
*about = > about.compose_components components=home*

<!-- Tools page (lib/net parsing) -->

Parse a `Cookie` request header with the standard library.

*parsed_req = > net.cookie_parse text="session=abc123; theme=dark"*
*req_cookie = parsed_req[^1]*

Parse a `Set-Cookie` response header.

*parsed_resp = > net.cookie_parse text="id=42; Path=/; HttpOnly; SameSite=Lax" is_response=True*
*resp_cookie = parsed_resp[^1]*

Parse a `multipart/form-data` body.

`_b` = net-boundary

`_body` = "--`_b`\nContent-Disposition: form-data; name=\"title\"\n\nHello\n--`_b`\nContent-Disposition: form-data; name=\"cover\"; filename=\"pic.png\"\nContent-Type: image/png\n\n<bytes>\n--`_b`--\n"

*mp = > net.multipart_parse body=`_body` boundary=`_b`*
*mp_title = mp[^1]*
*mp_cover = mp[^2]*

Build the tools page as one HTML intro (intro is served raw).

`tools_parts` =

| 段 |
|----|
| <h1>Net tools</h1><p>Parsing demos powered by lib/net: cookie_parse and multipart_parse.</p><section class="content cards"> |
| <article><h2>Cookie request</h2><p> |
| `req_cookie`[^name]=`req_cookie`[^value] |
| </p></article><article><h2>Set-Cookie response</h2><p> |
| `resp_cookie`[^name]=`resp_cookie`[^value] · HttpOnly=`resp_cookie`[^http_only] |
| </p></article><article><h2>Multipart field</h2><p> |
| `mp_title`[^name]=`mp_title`[^value] |
| </p></article><article><h2>Multipart file</h2><p> |
| `mp_cover`[^name] · `mp_cover`[^filename] · `mp_cover`[^content_type] |
| </p></article></section> |

*tools_intro = > text.str_join xs=`tools_parts` sep=""*
*tools = > web.page title="Tools" intro=`tools_intro`*
*tools = > tools.compose_components components=home*

Article create form: field table + rules table.

`article_fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | Title | text | true | |
| body | Body | textarea | false | |

`article_rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | Title is required |
| title | max:120 | Title is too long |
| body | max:8000 | Body is too long |

*article_form = > web.form table="articles" action="insert"*
*article_form = > article_form.fields fields=article_fields*
*article_form = > article_form.rules rules=article_rules*

New article page: form embedded in the main slot.

*new = > web.page title="Publish" intro="<h1>Publish</h1><p>Form inserts into articles.</p>"*
*new = > new.compose_components components=home*
*new = > new.compose_form id="article" form=article_form*

Admin users table (english + chinese column headers both work).

`admins` =

| 行 | username | password |
|----|----------|----------|
| 1 | admin | secret |

*app = > web.app page=page db=store admin=True host="127.0.0.1" port=18083*
*app = > app.route path="/about" page=about*
*app = > app.route path="/new" page=new*
*app = > app.route path="/tools" page=tools*
*app = > app.static dir="public" mount="/static"*
*app = > app.route_ws path="/live" echo=True*
*app = > app.auth users=`admins` session_ttl=3600*
> `app`.listen