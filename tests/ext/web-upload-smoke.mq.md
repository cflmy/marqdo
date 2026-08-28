---
title: web upload smoke (W5)
description: Offline validate + save into file: storage.
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

`types` =

| 类型 | 扩展名 |
|------|--------|
| image/png | png |
| text/plain | txt |

*m = > web.media*
*ok = > `m`.validate filename="a.png" content_type="image/png" size=100 max_bytes=1000 types=`types`*
*ook = ok[^ok]*
1. `ook`
  > print text=validate-ok
2. *
  > print text=validate-fail

*bad = > `m`.validate filename="a.exe" content_type="application/octet-stream" size=100 max_bytes=1000 types=`types`*
*bok = bad[^ok]*
1. `bok`
  > print text=reject-type-fail
2. *
  > print text=reject-type-ok

*big = > `m`.validate filename="a.png" content_type="image/png" size=9000 max_bytes=1000 types=`types`*
*gok = big[^ok]*
1. `gok`
  > print text=reject-size-fail
2. *
  > print text=reject-size-ok

*blob = > web.storage url="file:tests/ext/web-fixtures/data/upload-blobs"*
*m = > web.media storage=`blob`*
*saved = > `m`.save path="tests/ext/web-fixtures/upload/sample.txt" content_type="text/plain" prefix="smoke/"*
*sok = saved[^ok]*
*skey = saved[^key]*
1. `sok`
  > print text=save-ok
2. *
  > print text=save-fail

*got = > `blob`.get key=`skey`*
*body = got[^body]*
1. `body` == "hello-upload"
  > print text=save-roundtrip-ok
2. *
  > print text=save-roundtrip-fail

<!-- form file field renders with multipart enctype -->

`fields` =

| name | label | type |
|------|-------|------|
| title | Title | text |
| file | File | file |

*frm = > web.form table="x" fields=`fields`*
*html = > `frm`.render id="up"*
1. `html`
  > print text=form-file-ok
2. *
  > print text=form-file-fail
