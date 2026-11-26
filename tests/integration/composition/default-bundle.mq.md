---
import:
  io: "std:io"
  format: "std:format"
  json: "std:data/json"
  text_lib: "std:text"
  utf8: "std:text/utf8"
  path: "std:path"
  datetime: "std:datetime"
---

# @exec bundle

## main

**内存 = memory.new(1, 4)**

**长度 = text_lib.utf8编码("é", 内存, 0)**

**解码 = utf8.解码(内存, 0, 长度)**

**对象 = json.解析("[1, 2]")**

**时间 = datetime.格式化(1795248000123)**

> **io.写值("default-bundle")**

`return [长度, 内存[0], 内存[1], 解码.value, format.显示(["ok", 3]), json.格式化(对象.value), path.连接(["a", "b", "c"]), 时间, format.字节长度("é")]`
