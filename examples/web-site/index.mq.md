---
title: web-site
description: >-
  WWW Demo sample — GFM tables for shell, schema, forms, styles, and admin.
  Author business JavaScript: none. Author hand-written .css: none.
import web:ext/web/web.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import desk:components/desk.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

This Markdown file is both the **site documentation** and the **running app**.
Routes, forms, validation, admin pages, and look-and-feel are **GFM tables** —
no hand-written business JavaScript and no author .css file.

`home` =

| src | style |
|-----|-------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side_panel` |
| foot.`foot` | shell.`footer` |

Public shell (nav + side + foot). Desk shell drops the public side rail.

`desk_shell` =

| src | style |
|-----|-------|
| desk.`nav` | shell.`topnav` |
| desk.`foot` | shell.`footer` |

List cards: summary + slug link (click through to detail).

`index` =

| front | back | style |
|-------|------|-------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.summary | shell.`card_body` |
| href | articles.`articles`.slug | |

Detail bind for /post/{slug} (full Markdown body).

`detail` =

| front | back | style |
|-------|------|-------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.body | |

`by_slug` =

| field | op | value |
|-------|-----|-------|
| slug | = | {slug} |

Home intro — same bind shape as main (front / back / style).

`home_intro` =

| front | back | style |
|-------|------|-------|
| kicker | WWW demo · tables as the full stack | shell.`kicker` |
| title | Marqdo | shell.`intro_title` |
| lede | "One executable Markdown artifact declares routes, forms, admin pages, and styles — no hand-written business JavaScript, no author .css. Click a card for the full article, then try the three steps below." | shell.`lede` |
| claim | 0 author JS | shell.`claim` |
| claim | 0 author .css | shell.`claim` |
| claim | GFM tables | shell.`claim` |
| step | "Click any card — /post/{slug} opens the detail page from the same articles table." | shell.`step` |
| step | "Open [New](/new) and submit with an empty title (validation from a rule table)." | shell.`step` |
| step | "Open top-nav **Login** (admin / demo), then **Admin** — gated desk with create form + list." | shell.`step` |

`about_intro` =

| front | back | style |
|-------|------|-------|
| kicker | about | shell.`kicker` |
| title | About | shell.`intro_title` |
| lede | "Documentation and the deployed site stay one .mq.md artifact. Layout and theme are GFM style tables in styles/shell.mq.md — structure without hand-written CSS or business JavaScript." | shell.`lede` |
| claim | docs = program | shell.`claim` |
| claim | server-side rules | shell.`claim` |
| claim | hands-on | shell.`claim` |

`new_intro` =

| front | back | style |
|-------|------|-------|
| kicker | form · try validation | shell.`kicker` |
| title | New article | shell.`intro_title` |
| lede | "Leave Title empty and submit — the error comes from the rule table, not client JavaScript. Then fill title, slug, and summary to create a row shared with Admin and the card grid." | shell.`lede` |

`admin_intro` =

| front | back | style |
|-------|------|-------|
| kicker | desk · you are in admin | shell.`kicker` |
| title | Admin | shell.`intro_title` |
| lede | "This is the management desk (login required): same articles schema as the public site, assembled as list + create form — not a separate SPA. Use Site in the topnav to return, or Logout to end the session." | shell.`lede` |
| claim | login-gated | shell.`claim` |
| claim | table-declared | shell.`claim` |
| claim | "create + list" | shell.`claim` |

`post_intro` =

| front | back | style |
|-------|------|-------|
| kicker | article · table-bound detail | shell.`kicker` |

Article create form: field table + rules table (server-side validation).

`article_fields` =

| name | label | type | required | default |
|------|-------|------|----------|---------|
| title | Title | text | true | |
| slug | Slug | text | true | |
| summary | Summary | textarea | false | |
| body | Body | textarea | false | |

`article_rules` =

| field | rule | message |
|-------|------|---------|
| title | required | Title is required |
| title | max:120 | Title is too long |
| slug | required | Slug is required |
| slug | max:80 | Slug is too long |
| summary | max:500 | Summary is too long |
| body | max:8000 | Body is too long |

SEO / Open Graph for the home page (still a table).

`home_meta` =

| key | value |
|-----|-------|
| description | Tables as the full stack — Marqdo Web demo with zero author business JavaScript. |
| og:type | website |
| og:title | Marqdo Web Site |

Neat CJK sans for body; Great Vibes for English display.

`fonts` =

| rel | href | crossorigin |
|-----|------|-------------|
| preconnect | https://fonts.googleapis.com | |
| preconnect | https://fonts.gstatic.com | anonymous |
| stylesheet | https://fonts.googleapis.com/css2?family=Great+Vibes&family=Noto+Sans+SC:wght@300;400;500;700&family=Noto+Serif+SC:wght@400;600&display=swap | |

**store = > db.open**
**site_css = > shell.css**

**page = > web.page title="Marqdo Web Site" shell_css="off"**
**page = > page.compose_intro intro=home_intro**
**page = > page.meta meta=home_meta**
**page = > page.head table=fonts**
**page = > page.css css=site_css**
**page = > page.compose_components components=home**
**page = > page.compose_main main=index**
**page = > page.link_prefix prefix="/post/"**

**about = > web.page title="About · Marqdo Web" shell_css="off"**
**about = > about.compose_intro intro=about_intro**
**about = > about.head table=fonts**
**about = > about.css css=site_css**
**about = > about.compose_components components=home**

**article_form = > web.form table="articles" action="insert"**
**article_form = > article_form.fields fields=article_fields**
**article_form = > article_form.rules rules=article_rules**

**new = > web.page title="New article · Marqdo Web" shell_css="off"**
**new = > new.compose_intro intro=new_intro**
**new = > new.head table=fonts**
**new = > new.css css=site_css**
**new = > new.compose_components components=home**
**new = > new.compose_form id="article" form=article_form**

**desk = > web.page title="Admin · Marqdo Web" shell_css="off"**
**desk = > desk.chrome body_class="is-desk"**
**desk = > desk.compose_intro intro=admin_intro**
**desk = > desk.head table=fonts**
**desk = > desk.css css=site_css**
**desk = > desk.compose_components components=desk_shell**
**desk = > desk.compose_form id="article" form=article_form**
**desk = > desk.compose_main main=index**
**desk = > desk.link_prefix prefix="/post/"**

**post = > web.page title="Article · Marqdo Web" shell_css="off"**
**post = > post.compose_intro intro=post_intro**
**post = > post.head table=fonts**
**post = > post.css css=site_css**
**post = > post.compose_components components=home**
**post = > post.compose_main main=detail**
**post = > post.query query=by_slug**
**post = > post.detail detail=True**

Demo credentials (table-declared users — host session auth, no author JS).

`admins` =

| username | password | role |
|----------|----------|------|
| admin | demo | admin |

**app = > web.app page=page db=store admin=False host="127.0.0.1" port=18081 shell_css="off" login_redirect="/admin" logout_redirect="/"**
**app = > app.route path="/about" page=about**
**app = > app.route path="/new" page=new**
**app = > app.route path="/admin" page=desk**
**app = > app.route path="/post/{slug}" page=post**
**app = > app.auth users=admins session_ttl=3600 login_path="/login" login_redirect="/admin" logout_redirect="/"**
**app = > app.gate path="/admin" roles="admin" match="prefix" on_deny="redirect" exclude="/login"**
> `app`.listen
