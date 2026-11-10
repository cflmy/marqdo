---
markup: occlusion
title: Shared article form and server validation
import:
  form: ext/data/form.mq.md
  document: lib/document.mq.md
---

New and Admin place the same article form reference. The serving entry mounts this resource once at article; rendering, validation and actual database submission therefore share field definitions and rules. The public creation path remains public as in the original demonstration, while direct access to the custom desk is role-gated.

## article

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Insert form with title, slug, summary, Markdown body and original validation limits |

Grant plugin and fs.read on the site root before mounting. The adjacent literal leaf keeps field/rule data readable without turning the workflow into a data dump. Title and slug are required, with original maxima of 120 and 80 characters. Summary and body remain optional, limited to 500 and 8000 characters. These are server rules even if a caller bypasses browser constraints. Constructing the resource writes no rows and grants no authentication.

The field types remain text and textarea, retaining the original schema behavior. A rejected submission returns field messages and current CSRF for retry; success returns to the home list, where the current-query resource observes the new row. Keep every updated resource value before mounting; using an older value would lose attached fields or rules.

We **fields = [Read the actual literal field contract](document.read_table(path="forms.tables.md", title="fields"))** and **rules = [Read the actual literal validation rules](document.read_table(path="forms.tables.md", title="rules"))**.

We **selected = [Create the actual article insertion resource](form.form(table="articles"))** and **selected = [Attach the readable field records](> `selected`.fields fields=`fields`)** and **selected = [Attach the actual server rules](> `selected`.rules rules=`rules`)**.
Return *[Keep submission copy and its successful return target](> `selected`.labels submit="Publish" cancel="Cancel" cancel_href="/")*.
