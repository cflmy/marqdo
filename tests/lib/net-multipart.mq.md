---
title: net multipart_parse
import net:lib/net.mq.md
---

# main

`boundary` = ----WebKitFormBoundary7MA4YWxkTrZu0gW

`body` = "--`boundary`\nContent-Disposition: form-data; name=\"title\"\n\nHello\n--`boundary`\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\nContent-Type: text/plain\n\nfile body\n--`boundary`--\n"

**parts = > net.multipart_parse body=`body` boundary=`boundary`**

> print text=[name]([1](`parts`))
> print text=[value]([1](`parts`))
> print text=[name]([2](`parts`))
> print text=[filename]([2](`parts`))
> print text=[content_type]([2](`parts`))
> print text=[value]([2](`parts`))
