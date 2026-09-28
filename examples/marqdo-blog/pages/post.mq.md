---
type: web
title: Post
route: /post/{slug}
method: GET
data_source: posts
data_where: slug={slug}
description: Dynamic Document route — body is the article shell; data from Resource.
---

# Post

Article page for `{slug}`. List and detail data are provided by `ext/data` at serve time when `data_source` is set on the Artifact.
