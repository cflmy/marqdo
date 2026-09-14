---
title: lib/html
description: HTML escape / unescape (Mid2 M8). XSS floor for string assembly.
---

HTML escape helpers. Use before stitching untrusted text into markup.

## escape
    + `text`

Escape text for HTML.

Caller: [html.escape] text=`s`

*> host_html_escape text=`text`*

## unescape
    + `text`

Unescape HTML entities in text.

*> host_html_unescape text=`text`*
