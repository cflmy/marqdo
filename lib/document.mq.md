---
markup: occlusion
---

Inert document operations share the renderer Markdown parser. Every function is pure.

Inspect and transform Markdown without executing its contents.

Behavior and limits: Shares the Markdown parser with rendering and recognizes YAML frontmatter. Spans are original UTF-8 byte offsets. Duplicate section names fail. Replacement preserves surrounding source; persist via fs.write_checked.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/document.rs](../src/host/document.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## sections

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | List headings with original UTF-8 byte spans and body text; nested sections end at the next same-or-higher heading |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/document.mq.md sections 'text="# Notes"' --effects none --json
```

List headings with original UTF-8 byte spans and body text; nested sections end at the next same-or-higher heading. No execution.

Return *[List headings with original UTF-8 byte spans and body text; nested sections end at the next same-or-higher heading](> host_document_sections text=`text`)*.

## section

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |
| title | text | required | Exact, unique Markdown heading title |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Read the body under one exact heading; missing or duplicate headings fail |

Use `text`, `title` as the inputs for this operation.

Effects: pure.

Read the body under one exact heading; missing or duplicate headings fail.

Return *[Read the body under one exact heading; missing or duplicate headings fail](> host_document_section text=`text` title=`title`)*.

## replace_section

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |
| title | text | required | Exact, unique Markdown heading title |
| body | text | required | Request body, or replacement text for document operations |
| expected_sha256 | any | None | Digest of the exact prior text; None disables the check where allowed |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Return Result with replaced section body, preserving its heading and surrounding source |

Use `text`, `title`, `body`, `expected_sha256`=None as the inputs for this operation.

Effects: pure.

Return Result with replaced section body, preserving its heading and surrounding source. Optional digest checks the whole source. Pure transform; persist via fs.write_checked.

Return *[Return Result with replaced section body, preserving its heading and surrounding source](> host_document_replace_section text=`text` title=`title` body=`body` expected_sha256=`expected_sha256`)*.

## tables

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Extract inert GFM headers and cell text, plus original byte ranges; cells are never evaluated |

Use `text` as the inputs for this operation.

Effects: pure.

Extract inert GFM headers and cell text, plus original byte ranges; cells are never evaluated.

Return *[Extract inert GFM headers and cell text, plus original byte ranges; cells are never evaluated](> host_document_tables text=`text`)*.

## replace_table

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |
| index | number | required | Zero-based table index in source order |
| headers | list | required | Ordered list of table header text |
| rows | list | required | List of cell-text lists; each row must match the header width |
| expected_sha256 | any | None | Digest of the exact prior text; None disables the check where allowed |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Replace the zero-based table from text headers and rows, preserving all surrounding source; return Result |

Use `text`, `index`, `headers`, `rows`, `expected_sha256`=None as the inputs for this operation.

Effects: pure.

Replace the zero-based table from text headers and rows, preserving all surrounding source; return Result.

Return *[Replace the zero-based table from text headers and rows, preserving all surrounding source; return Result](> host_document_replace_table text=`text` index=`index` headers=`headers` rows=`rows` expected_sha256=`expected_sha256`)*.

## links

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | List Markdown link labels and targets with byte spans; never follow or execute them |

Use `text` as the inputs for this operation.

Effects: pure.

List Markdown link labels and targets with byte spans; never follow or execute them.

Return *[List Markdown link labels and targets with byte spans; never follow or execute them](> host_document_links text=`text`)*.

## code_blocks

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | List code fence language and literal source with byte spans; never run foreign or Marqdo code |

Use `text` as the inputs for this operation.

Effects: pure.

List code fence language and literal source with byte spans; never run foreign or Marqdo code.

Return *[List code fence language and literal source with byte spans; never run foreign or Marqdo code](> host_document_code_blocks text=`text`)*.
