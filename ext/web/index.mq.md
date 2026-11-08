---
markup: occlusion
title: ext/web
description: Executable web handbook: decompose layout down to Markdown leaves.
import:
  layout: ext/web/layout/index.mq.md
  web: ext/web/web.mq.md
---

Web handbook — split page structure first, author content second, then preview,
serve and verify. Ordinary .mq.md functions are the tools; no separate Skill
compiler or manifest is needed. Install with marqdo ext add web. In this source
repo build with bash scripts/build-web-plugin.sh and optionally select its
absolute path with MARQDO_WEB_PLUGIN. Keep installation outside source ext.

Start with executable evidence

Run these from the repository root; installed users replace FILE with its actual
installed path. Preserve the inner text quotes and protect dollar references
from shell expansion. Inspect process status and ok before consuming result.

```sh
marqdo call ext/web/index.mq.md layout_rules --json
marqdo call ext/web/index.mq.md preview_markdown 'body="# Readable page"' 'title="Preview"' --effects plugin --json
marqdo call ext/web/index.mq.md preview_nodes 'nodes=[{"type":"row","value":[{"type":"markdown","value":"## Left"},{"type":"markdown","value":"## Right"}]}]' --effects plugin --json
```

The engineering method: recursively decompose to Markdown leaves

Split home into navigation, main and footer. Split a two-region footer into
brand and links. If links still contains two columns, split it again into docs
and community. Continue while a block needs independent internal layout. Stop
at one simple Markdown responsibility, not at one file per paragraph.

```text
site/
  index.mq.md                 Document selects the layout root
  components/index.mq.md      stack navigation, main, footer
    nav/index.mq.md
    nav/content.md            ordinary Markdown links
    main/index.mq.md
    main/content.md           ordinary Markdown prose
    footer/index.mq.md        columns: brand, links
      brand/index.mq.md
      brand/content.md        brand and copyright
      links/index.mq.md       columns: docs, community
        docs/index.mq.md
        docs/content.md
        community/index.mq.md
        community/content.md
  pages/about.mq.md           another declared page
  api/ping.mq.md              Endpoint
  preview.mq.md               assemble and render offline
  serve.mq.md                 process launcher, no page content
```

A parent index imports children, combines their rows and declares one layout
relationship. A terminal content.md is plain Markdown. Split again if a parent
contains long content, multiple layout responsibilities or HTML tags. A leaf
does not decide its external placement. Cutting a large HTML string into files
still hides the structure in strings. Forms, data sources and interaction need
their own responsibilities; they do not justify restoring an HTML page shell.

Follow the real project

The [English reference](../../examples/web-layout-handbook/index.mq.md) implements
this tree. Follow its [component root](../../examples/web-layout-handbook/components/index.mq.md)
to footer grandchildren. The [Chinese reference](../../examples/web-layout-handbook-zh/index.mq.md)
uses the same representation and renderer. Preview from the repo root, then
change into the site directory to start the process:

```text
marqdo run examples/web-layout-handbook/preview.mq.md
cd examples/web-layout-handbook
marqdo run serve.mq.md
```

Check /, /about and /api/ping. The first is a genuinely nested layout, the second
an ordinary Document, and the third JSON. Wide layouts show columns; narrow
layouts collapse without authored HTML or CSS.

One View representation for both parents and leaves

Public columns remain type, slot, value, attrs, style, with Chinese aliases
类型、槽、值、属性、样式. Use @ record tables for lists of node maps. A row/column
container holds child rows in value. nav/header/footer/section/aside supply
semantic regions. A markdown value is content text. Merging defines sibling
order, not geometry: use columns to establish side-by-side placement. A style
cell references class names; it neither defines CSS nor silently compiles a theme.
Runtime defaults supply the basic responsive geometry.

Short leaves can pass Markdown directly; longer leaves read a content.md as in
the reference. Parents retain structural values rather than rendering children
before composition. This small executable program demonstrates the relationship;
real sites put each responsibility into the files shown above.

```marqdo
---
markup: occlusion
import:
  component: ext/web/component/index.mq.md
  layout: ext/web/layout/index.mq.md
  web: ext/web/web.mq.md
---

# main

**left = component.markdown text="### Brand" slot="brand"**
**right = component.markdown text="[About](/about)" slot="links"**
**siblings = component.merge a=`left` b=`right`**
**footer = layout.columns children=`siblings` slot="footer"**
**html = web.render nodes=`footer` title="Column preview"**
> print text=`html`
```

Document, Endpoint and lifecycle boundaries

Ordinary Documents declare type: web, route and title around Markdown body,
retaining the existing Markdown/data-source path. A composed Document also
declares view: main: its # main returns View rows. serve executes that assembly
once at startup and uses the same renderer as offline preview. Restart after
content changes. Automatic data_source binding uses ordinary Documents and
cannot be mixed with startup assembly. Assembly returns structure; it must not render again or launch
a server. Request-time interaction belongs in Endpoints with type, path, method,
request and response metadata. Startup assembly is not an automatic request-time
template/data-binding system.

serve is a blocking lifecycle operation for a launcher run with marqdo run.
call is for pure construction, previews and inspection; --effects plugin does
not grant network or filesystem writes, so call serve is not a launch workflow.
inspect with root scans disk declarations without listening; an empty app in a
fresh process cannot inspect another process's private state.

```sh
marqdo call ext/web/web.mq.md inspect 'root="examples/web-layout-handbook"' --effects plugin --json
```

Open the module for the next responsibility

[layout](layout/index.mq.md) supplies decomposition, stack and columns;
[component](component/index.mq.md) supplies Markdown leaves and semantic wrappers;
[page](page/index.mq.md) supplies handles and offline render;
[route](route/index.mq.md) is for explicit advanced registration, while whole
sites normally use Metadata. [dom](dom/index.mq.md) converts content, retaining
raw HTML only at a compatibility boundary. [client](client/index.mq.md) loads
the official Marqdo WASM bridge; business effects stay in lib/browser.
[_capability](_capability/index.mq.md) diagnoses loading. The [facade](web.mq.md)
keeps serve/render/inspect and the app API. Data, auth and HTTP clients belong
to ext/data, ext/security and ext/net. [Chinese handbook](网页/索引.mq.md).

Develop and accept a change

Compose Markdown, stack and columns before inventing a primitive. Only genuinely
new structural capability needs plugins/web/internal/artifact and ABI changes;
ordinary pages need no new host primitive. Keep bilingual helpers and the five
columns aligned. Put parameter contracts after headings and return contracts
before execution. ## is a callable heading, not ordinary document sectioning.
Update src/ext_cli.rs if adding package files. Preserve ordinary Documents and
Endpoints when changing startup assembly.

```text
cargo test --locked --test web_handbook
cargo test --locked --test call_cli web_
cargo test --locked --test gold ext_web -- --test-threads=1
```

Handbook tests execute the bilingual sh fences and author examples directly,
then preview and serve the real reference tree to check structure, routes and
API behavior. Narration ratio is a minimum, not proof of usability. A checklist
cannot truthfully accept a component without inspecting its implementation.
This package does not automatically decompose arbitrary pages or implement a
full CSS compiler through the style cell; explicit design remains the author's
responsibility, made followable and testable by these indexes.

## site_playbook

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Development workflow and stopping rule |

Return the workflow and stopping rule without creating files or claiming to
accept a particular site. Define the region tree, author parents and Markdown
leaves, then start and test the declared HTTP surface.

`workflow` =

| key | value |
|-----|-------|
| goal | build a site as executable documents |
| step_A | draw region tree before content |
| step_B | recursively split components to Markdown leaves |
| step_C | declare home Document and view main for composed structure |
| step_D | declare more Documents with their own routes |
| step_E | declare Endpoint handlers for request-time work |
| step_F | run serve.mq.md from the site directory |
| step_G | compare preview and live output; test routes and browser geometry |
| stopping_rule | one Markdown responsibility and no internal layout |

*workflow*

## layout_rules

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Development workflow and stopping rule |

Reuse the layout module convention without a second rule catalog. No
parameters or plugin. Actual acceptance still inspects leaf responsibilities,
parent relationships and the running result.

*> layout.rules*

## preview_markdown

| 参数 | 类型 | 说明 |
|------|------|------|
| body | text | Markdown content |
| title | text | Document title |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | HTML document |

Render Markdown `body`="" with default `title`="Preview" .
Return HTML text without writing files or registering routes. For a complex
page split the layout first; raw HTML strings are not the content strategy.
This tool needs the native plugin.

*> web.render body=`body` title=`title`*

## preview_nodes

| 参数 | 类型 | 说明 |
|------|------|------|
| nodes | list | Nested View rows |
| title | text | Document title |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | HTML document |

Render decomposed `nodes` with default `title`="Preview" .
Parent values retain child rows, while leaf values hold Markdown. Returning
HTML starts no server. The same renderer serves composed Documents, so this
preview checks that column relationships survive assembly.

*> web.render nodes=`nodes` title=`title`*
