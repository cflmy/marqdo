---
title: net multipart_parse
import net:lib/net.mq.md
---

# main

`boundary` = ----WebKitFormBoundary7MA4YWxkTrZu0gW

`body` = "--`boundary`\nContent-Disposition: form-data; name=\"title\"\n\nHello\n--`boundary`\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\nContent-Type: text/plain\n\nfile body\n--`boundary`--\n"

*parts = > net.multipart_parse body=`body` boundary=`boundary`*

> print text=`parts`[^1][^name]
> print text=`parts`[^1][^value]
> print text=`parts`[^2][^name]
> print text=`parts`[^2][^filename]
> print text=`parts`[^2][^content_type]
> print text=`parts`[^2][^value]
