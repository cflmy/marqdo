---
markup: occlusion
title: ext/net/url
description: URL helpers (stdlib wrappers). ADR 0007.
import:
  net: lib/net.mq.md
---

Use this leaf to encode data for a URL component, not to fetch or validate a remote service. The [network entry](index.mq.md) provides a directly runnable encoding example. Whole URLs, path components and query values have different escaping responsibilities; pass the intended component and let the caller assemble the final URL. Prefer composing lib/url.mq.md when adding pure transformations.

## encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | any | required | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Percent-encode the supplied text using the public standard library URL helper. |

Use `text` as the inputs for this operation.

Percent-encode the supplied text using the public standard library URL helper.

Return *[Url encode](net.url_encode(text=`text`))*.
