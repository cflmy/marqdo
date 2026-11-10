---
markup: occlusion
type: web
view: main
route: /tools
title: Network parsing tools
import:
  net: lib/net.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
  component: ext/web/component/index.mq.md
  layout: ../components/index.mq.md
---

This Document executes the standard network parsers against explicit local fixtures. It makes no external HTTP request. Cookie headers remain text, and the multipart fixture is terminal protocol data in components/content/multipart.txt. The page shows actual returned records as plain preformatted text instead of concatenating raw HTML.

## _panel

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| title | text | required | Author-controlled heading describing one observed parser value |
| value | any | required | Actual native parser result to display literally |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Markdown heading followed by an escaped native preformatted value |

Use `title` and `value` to keep each observed value beside its purpose. A parser record is data, not an HTML fragment; convert it to its native display text and let the pre element preserve characters. The author supplies headings, while protocol content never becomes a heading or executable expression. This pure helper performs no IO or parsing itself.

We **heading = "## " + title** and **label = [Describe the observation in Markdown](component.markdown(text=`heading`))** and **text = [Retain the actual structured parser value](str(value))** and **data = [Show that value literally](component.element(kind="pre", value=`text`))**.
Return *[Keep meaning and actual evidence together](table.concat(a=`label`, b=`data`))*.

## _cookies

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Actual Cookie request and Set-Cookie response observations |

The first parser call reads a request Cookie header containing session and theme pairs. The second deliberately enables response-header mode, where Path, HttpOnly and SameSite describe the id cookie rather than becoming independent request cookies. Both results come from the maintained standard library. These are deterministic text fixtures, not real credentials or a browser session lookup.

Keeping both modes visible preserves the original network-tools capability and explains why their output shapes differ. No cookie is written, accepted for authentication or sent to a remote service. The output panes expose all native fields, so a reader can inspect names, values and flags without decoding a generated HTML string.

We **request = [Parse the explicit request-cookie fixture](net.cookie_parse(text="session=abc123; theme=dark"))** and **response = [Parse one actual response-cookie fixture](net.cookie_parse(text="id=42; Path=/; HttpOnly; SameSite=Lax", is_response=True))**.
We **request_panel = [Show actual request pairs](_panel(title="Cookie request", value=`request`))** and **response_panel = [Show actual response attributes](_panel(title="Set-Cookie response", value=`response`))**.
Return *[Keep the two parser modes adjacent](table.concat(a=`request_panel`, b=`response_panel`))*.

## _multipart

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Actual multipart field and file-part observations |

Read the protocol fixture with fs.read from the site root, then parse it using net-boundary, the delimiter declared by that fixture. The title part contains Hello. The cover part names pic.png with image/png content type; its value is the literal demonstration text <bytes>, not an uploaded binary image. This distinction keeps the example's claims aligned with the bytes actually parsed.

The result is a native part list, so the field and file panes select its first and second records through collection helpers. A missing or malformed fixture remains a parser or filesystem diagnostic rather than a hand-authored expected result. No upload, file extraction or network access occurs, and the fixture is not interpreted as Markdown code.

We **body = [Read the exact local multipart protocol bytes as text](fs.read_text(path="components/content/multipart.txt"))** and **parts = [Parse the real boundary and disposition fields](net.multipart_parse(body=`body`, boundary="net-boundary"))**.
We **field = [Select the first parsed part](table.first(list=`parts`))** and **file = [Select the second parsed part](table.at(xs=`parts`, i=1))** and **field_panel = [Show the actual form field](_panel(title="Multipart field", value=`field`))** and **file_panel = [Show actual file metadata](_panel(title="Multipart file", value=`file`))**.
Return *[Keep both actual parser observations](table.concat(a=`field_panel`, b=`file_panel`))*.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Shared site frame around all four native parser observations |

This startup assembly requires fs.read only for the terminal multipart fixture and the shared Markdown layout leaves. The standard parser calls themselves are pure. The four panes preserve request cookies, response cookies, a multipart field and multipart file metadata. None represents a request to an external site or a user upload.

Changing a fixture should change the displayed result because the page runs the real parser, not a copied expected answer. Keep the explanatory heading and the observed values together, and retain the shared navigation to Publish, administration and the live widget.

We **intro = [Explain the deterministic tools page](component.markdown(text="# Net tools\n\nParsing demonstrations powered by the actual standard network library."))** and **cookies = [Observe both cookie modes](_cookies())** and **parts = [Observe both multipart part types](_multipart())**.
We **body = [Keep the explanation before observations](table.concat(a=`intro`, b=`cookies`))** and **body = [Retain multipart observations after cookies](table.concat(a=`body`, b=`parts`))**.
Return *[Place all observations in the shared site frame](layout.frame(body=`body`))*.
