---
markup: occlusion
title: net multipart_parse
import net:lib/net.mq.md
---

# main

`boundary` = ----WebKitFormBoundary7MA4YWxkTrZu0gW

`body` = "--`boundary`\nContent-Disposition: form-data; name=\"title\"\n\nHello\n--`boundary`\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\nContent-Type: text/plain\n\nfile body\n--`boundary`--\n"

**parts = > net.multipart_parse body=`body` boundary=`boundary`**

> print text=[name](access([1](access(`parts`, "1")), "name"))
> print text=[value](access([1](access(`parts`, "1")), "value"))
> print text=[name](access([2](access(`parts`, "2")), "name"))
> print text=[filename](access([2](access(`parts`, "2")), "filename"))
> print text=[content_type](access([2](access(`parts`, "2")), "content_type"))
> print text=[value](access([2](access(`parts`, "2")), "value"))
