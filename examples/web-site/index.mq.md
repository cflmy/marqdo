---
markup: occlusion
type: web
view: main
route: /
title: Marqdo Web Site
import:
  layout: components/index.mq.md
  intro: components/intro.mq.md
  articles: components/articles.mq.md
  database: db/index.mq.md
  forms: forms.mq.md
  theme: styles/index.mq.md
  web: ext/web/web.mq.md
  form: ext/data/form.mq.md
  auth: ext/security/auth.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
---

This entry is the executable site handbook. Follow components/index for public and administration geometry, then Markdown leaves, current article resources, the form and database. Five declared Documents retain Home, About, New, Admin and slug detail. Session navigation and server access policy remain separate; the custom desk does not enable the unrelated native administration landing page.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Public frame containing the readable introduction and current article cards |

Startup assembly reads terminal Markdown and constructs a typed query, without querying or initializing a database. The retained application resource supplies current articles at request time. Shared layout preserves navigation, side rail, footer, SEO, fonts and the compiled literary theme. Follow each imported responsibility rather than rebuilding the complete page here.

We **introduction = [Read the terminal home introduction](intro.nodes(name="home"))** and **cards = [Describe the current article cards](articles.cards())** and **body = [Keep the introduction before the records](table.concat(a=`introduction`, b=`cards`))**.
Return *[Place home inside the shared public frame](layout.frame(body=`body`, homepage=True))*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-site" | Explicit site discovery directory from the checkout |
| port | number | 18081 | Loopback demonstration port |
| url | any | None | None uses retained site-root SQLite; tests supply an owned URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener after real data, form, authentication and desk policy bindings |

Use `root`="examples/web-site", `port`=18081 and `url`=None from the checkout with --fs-root examples/web-site and plugin,fs.write,network.write. Database initialization preserves existing rows and seeds only an empty table. The original declaration leaves remain unchanged; styles/index projects their selectors onto the recursive View layout and restores the named style properties previously attached by removed compose APIs.

Theme output is explicitly generated under public/generated before discovery. Preparation reads declared Documents but opens no listener. Mount the same public article form used by New and the desk, attach local fixture authentication, and protect the actual /admin path before listening. Keep every returned application. The admin/demo credentials are local demonstration data; replace the user store and network policy for a real deployment. Shutdown belongs to the supervising caller, not a hidden background process.

We **store = [Initialize the retained article database](database.open(url=`url`))** and **css = [Compile the actual View theme projection](theme.css())** and **[Create only the explicit generated resource directory](fs.make_dirs(path="public/generated"))** and **[Write the compiled theme through native IO](fs.atomic_write(path="public/generated/theme.css", text=`css`))**.
We **app = [Discover the five declared site Documents](web.prepare(root=`root`, host="127.0.0.1", port=`port`, db=`store`, static_dir="public"))** and **article = [Construct one shared server-validation resource](forms.article())** and **app = [Mount the public article submission explicitly](form.mount(app=`app`, id="article", form=`article`))**.

`users` =

| @ | username | password | role |
|---|---|---|---|
| 1 | admin | demo | admin |

We **app = [Attach native login sessions](auth.attach(app=`app`, users=`users`, login_path="/login", login_redirect="/admin", logout_redirect="/"))** and **app = [Protect direct requests to the custom desk](auth.gate(app=`app`, path="/admin", roles="admin", match="prefix", on_deny="redirect", exclude="/login"))**.
Return *[Listen after actual resources and policy are ready](web.listen(app=`app`))*.
