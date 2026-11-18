---
markup: occlusion
title: Executable theme migration evidence
import:
  document: lib/document.mq.md
  table: lib/table.mq.md
  hash: lib/hash.mq.md
  result: lib/result.mq.md
  theme: styles/index.mq.md
  original: styles/shell.mq.md
---

This verification reads actual source declarations and compiles both the retained original and the View projection. It neither certifies the whole site nor replaces HTTP and browser tests. Call theme_values with plugin,fs.read and --fs-root examples/web-site; use an owned copy when validating other site operations.

## theme_values

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing verified original row count and real CSS identities |

Each of the six original areas is read through the native Markdown table reader. Compare every projected record to its original after removing only selector: values, media, properties, ordering and row count must match. Tokens and keyframe selectors themselves must remain unchanged. A mismatch returns an actual failed Result before claiming preservation.

The original and projected CSS hashes identify actual compiled outputs; a different View hash is expected because its semantic addresses and wrapper geometry changed. They are not a visual acceptance certificate. Browser tests separately inspect the real palette, title typography, grid, form flow and current records. This tool makes the preserved declarations reviewable without hiding the implementation in another language.

`areas` =

| name |
|---|
| vars |
| motion |
| layout |
| responsive |
| chrome |
| widgets |

We **count = 0**.
- [Compare each actual original area to its projection](for areas as name)
  We **columns = [Read the source declaration leaf](document.read_table(path="styles/shell.tables.md", title=`name`))** and **rows = [Retain original row order](table.from_columns(columns=`columns`))** and **projected = [Run the actual selector projection](theme._table(name=`name`))**.
  1. len(`rows`) != len(`projected`)
    Return *[Keep a dropped declaration visible](result.error(code="site.theme_rows", message=`name`, operation="theme_values"))*.
  We **position = 0**.
  - [Compare original property, value and media for every row](for rows as row)
    We **actual = [Read the same projected position](table.at(xs=`projected`, i=`position`))** and **before = [Set aside only the original selector](table.delete(map=`row`, key="selector"))** and **after = [Set aside only the projected selector](table.delete(map=`actual`, key="selector"))**.
    1. `before` != `after`
      Return *[Keep any changed declaration payload visible](result.error(code="site.theme_value", message=`name`, operation="theme_values"))*.
    1. `name` == "vars" or `name` == "motion"
      1. access(`row`, "selector") != access(`actual`, "selector")
        Return *[Keep global token/keyframe addresses unchanged](result.error(code="site.theme_global", message=`name`, operation="theme_values"))*.
    We **position = position + 1** and **count = count + 1**.
We **old_css = [Compile the retained original theme](original.css())** and **view_css = [Compile the actual View theme](theme.css())** and **original_sha256 = [Identify real original CSS bytes](hash.sha256(text=`old_css`))** and **view_sha256 = [Identify real projected CSS bytes](hash.sha256(text=`view_css`))**.

`evidence` =

| preserved_rows | original_sha256 | view_sha256 |
|---|---|---|
| `count` | `original_sha256` | `view_sha256` |

Return *[Expose real preservation and compilation evidence](result.ok(value=`evidence`))*.
