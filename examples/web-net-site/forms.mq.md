---
markup: occlusion
title: Article form resource
import:
  form: ext/data/form.mq.md
---

The same form resource supplies the page controls and validates the system submission route. The serving entry mounts it as article. The page only places a reference, so fields, limits and errors cannot drift between two separate form implementations.

## article

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native insert form with title/body fields and original validation rules |

Call with plugin before listening. Title is required and limited to 120 characters; body remains optional with an 8000-character limit. These are the original site's server-side rules. Rendering neither inserts a row nor authenticates a user; submission uses the application's actual articles database and current CSRF context.

The field and rule tables stay immediately visible beside their explanation. No raw form HTML or client validation program is authored. After a valid submission the resource returns to the article list; rejected values retain field errors for correction. This public demonstration form is distinct from the login-gated native administration surface.

`fields` =

| name | label | type | required |
|---|---|---|---|
| title | Title | text | True |
| body | Body | textarea | False |

`rules` =

| field | rule | message |
|---|---|---|
| title | required | Title is required |
| title | max:120 | Title is too long |
| body | max:8000 | Body is too long |

We **selected = [Create the actual article insert resource](form.form(table="articles", action="insert"))** and **selected = [Attach the readable field definitions](> `selected`.fields fields=`fields`)** and **selected = [Attach the server validation rules](> `selected`.rules rules=`rules`)**.
Return *[Keep submission and cancellation copy with this resource](> `selected`.labels submit="Publish" cancel="Cancel" cancel_href="/")*.
