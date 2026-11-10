---
markup: occlusion
title: Browser example recursive page assembly
import:
  navigation: navigation.mq.md
  home: home/index.mq.md
  about: about.mq.md
  network: network/index.mq.md
  controls: controls.mq.md
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

Read this root to see the whole page: introduction, path, navigation, three panels, feedback and runtime. Follow one import at a time until a child is a plain Markdown leaf or native control. Business values never become a page HTML string. The official runtime starts only after the server has staged the current built assets and the actual client source. Controls stay disabled until boot succeeds; missing resources therefore remain visible as a loading failure. The readiness target and enable selectors are typed resource metadata, not executable snippets. Styles reference the generated stylesheet produced by serve. Server discovery reads the terminal content leaves under the granted root; the browser receives the same ordered region tree.

We **source = [Read the plain introduction](fs.read_text(path="content/intro.md"))** and **intro = [Retain its Markdown](component.markdown(text=`source`, slot="intro"))** and **path = [Describe the initial pathname](controls.text(id="path", value="path: /"))** and **nav = [Assemble the authored destinations](navigation.nodes())** and **home = [Assemble list and timer](home.nodes())** and **about = [Read the About leaf](about.nodes())** and **network = [Assemble independent network controls](network.nodes())** and **log = [Describe literal runtime feedback](controls.text(id="log", value="loading…", kind="pre"))** and **style = [Reference the compiled declarative theme](component.asset(url="/static/theme.css", kind="stylesheet"))**.

`runtime_attrs` =

| type | data-mq-wasm | data-mq-source-url | data-mq-ready | data-mq-enable |
|---|---|---|---|---|
| module | "/static/marqdo_wasm.wasm" | "/static/app.mq.md" | "#log" | "#draft,#add,#save,#load,#start-tick,#stop-tick,#fetch-all,#ws-open,#ws-msg,#ws-send,#ws-close,#copy-log,#dl-list" |

We **runtime = [Declare the official module and readiness targets](component.asset(url="/static/marqdo-bridge.js", attrs=`runtime_attrs`, slot="runtime"))**.

`groups` =

| rows |
|---|
| `intro` |
| `path` |
| `nav` |
| `home` |
| `about` |
| `network` |
| `log` |
| `style` |
| `runtime` |

We **children = [Start the complete ordered page](table.empty_list())**.
- [Retain each independently inspectable region](for groups as group)
  We **children = [Keep typed children in reading order](table.concat(a=`children`, b=`group`))**.
Return *[Wrap the recursive page once](component.wrap(children=`children`, slot="browser-app", style="browser-app"))*.
