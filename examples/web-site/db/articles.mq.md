## schema

`fields` =

| name | type | nullable |
|------|------|----------|
| id | integer | false |
| title | text | false |
| slug | text | false |
| summary | text | true |
| body | text | true |

*`fields`*

## seed

Longer demo rows — cards show summary; /post/{slug} shows full body (Markdown).

`rows` =

| id | title | slug | summary | body |
|----|-------|------|---------|------|
| 1 | Hello Marqdo | hello-marqdo | A runnable Markdown file is both the site docs and the deployed app — tables declare routes, forms, and styles. | "Marqdo treats an .mq.md file as an executable document. The narrative you read in the editor is the same artifact the server runs: GFM tables become schema, forms, navigation, and CSS. This demo shows that contract end-to-end — open New, trigger a rule-table validation error, then publish a row that appears on Home and Admin without any author JavaScript." |
| 2 | Tables as the full stack | tables-full-stack | Schema, forms, routes, admin pages, and theme tokens share one tabular dialect instead of a JS/CSS split. | "Most stacks split concerns across languages: SQL or an ORM for data, a form library for input, a router for URLs, and a stylesheet for look. Marqdo collapses those surfaces into GFM tables with the same geometry authors already use for documentation. Field tables, rule tables, bind tables, and style tables are not metaphors — they are the runtime. The result is a site you can skim as prose and execute as a program." |
| 3 | Zero author JS and CSS | zero-author-js-css | Business logic stays in Markdown; appearance is style tables plus the host shell — no hand-written business JS or author .css. | "Author business JavaScript is zero by construction: validation runs from rule tables on the server, and list/detail pages are bind tables rendered by the host. Author .css files are also unnecessary when theme tokens live in property/value tables assembled by make_style. The demo turns shell_css=off so only those declared styles paint the page — proving the claim for a conference audience." |
| 4 | Card to detail in one bind | card-to-detail | List cards bind href to slug; a /post/{slug} route plus detail=True opens the full article. | "Click any card on the home page. The list bind exposes href from articles.slug, and link_prefix prefixes /post/. The detail page reuses the same table with a query slug = {slug} and page.detail so the first row renders as a full article — title and Markdown body — instead of a card grid. That is the same pattern used by larger Marqdo sites such as 求道量子." |
| 5 | Style tables not stylesheets | style-tables | Named style tables (kicker, intro_title, card_title) bind through the third column of every layout table. | "Intro, cards, and chrome all use the bind shape front/back/style. The style column points at a named property table in styles/shell.mq.md. When compose_intro or compose_main runs, those tables become CSS classes on the page. Authors never open a .css file to tweak the demo look — they edit readable Markdown tables that stay next to the copy they style." |
| 6 | Forms from field and rule tables | forms-from-tables | Empty title on New fails from the rule table; a valid submit inserts a row shared with Admin. | "The New and Admin pages mount the same form object: a field table declares labels and types, a rule table declares required and max constraints with messages. Submit with an empty title and the error is server-side, not a client script. Fill a title and slug, submit again, and the row shows up in the card grid and under Admin — one schema, two surfaces, zero author JS." |
| 7 | Seed data is documentation | seed-is-docs | These paragraphs were written as a GFM seed table — the demo content is part of the program. | "Seed rows live beside the schema in db/articles.mq.md. On first boot the host inserts them when the articles table is empty. That means the demo narrative is versioned with the code: longer summaries for cards, fuller bodies for detail pages, and slugs that make deep links stable. Reviewers can open the seed table and see exactly what the venue site will show." |
| 8 | Why this Demo matters | why-demo-matters | WWW Demo needs a hands-on artifact reviewers can run, click, and compare to source in minutes. | "A Demo track asks for an implemented system, not a slide-only pitch. This sample is intentionally small: one SQLite table, a handful of routes, and a literary shell — yet it covers the paper claim (tables as the full stack) with a three-step script. Clone the repo, run the web-site sample, click a card, break a form, then open index.mq.md beside the browser. That loop is the publication." |

*`rows`*
