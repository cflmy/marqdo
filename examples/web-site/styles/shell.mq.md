---
markup: occlusion
title: Retained literary theme declarations
import:
  document: lib/document.mq.md
  web: ext/web/web.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

This file retains original declaration readers and original CSS compilation for migration comparison. The current site uses styles/index.mq.md for the actual View projection. Do not mistake an unchanged original hash for proof that new selectors reach their intended nodes. Each literal section remains independently callable.

## vars

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Root palette, typefaces, spacing and elevation tokens.

Compile these declarations before rules that reference their custom properties. The selection-color rows remain global alongside :root; neither the View migration nor a caller should scope those tokens inside one card. Editing a value changes the shared theme, so compare real compiled CSS and browser colors rather than renaming a property locally.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="vars"))*.

## motion

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Original entrance, floating, particle and glow keyframes.

Keyframe selectors and their property values remain global and in their original order. Animation timing belongs to the consuming rule. Reading this data does not schedule an animation; the browser applies the compiled stylesheet. The View projection must not prefix these addresses with a component class.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="motion"))*.

## layout

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Document background, typography and original region-grid declarations.

The original body layout classes describe the old page shell. Current View use projects those known addresses to the explicit site-layout region through styles/index; their property values remain unchanged. Keep document-level backgrounds and particle layers global, and compare actual wide/narrow geometry after any edit.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="layout"))*.

## responsive

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Original media-conditioned layout and reduced-motion rows.

These rows retain media text as a separate column. Width conditions collapse the public grid and change padding; prefers-reduced-motion disables selected animation without deleting its keyframes. Preserve row ordering and media alongside the value, rather than checking only that a selector name still appears in CSS.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="responsive"))*.

## topnav

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value declarations for the top navigation surface.

The original sticky positioning, gradient, blur, spacing and border are still applied to the native topnav class. Links and their current-session conditions belong to the navigation resource. A property table cannot authenticate a visitor or make a route exist, and compiling it does not mount a stylesheet.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="topnav"))*.

## side_panel

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value declarations for the public side rail.

The side rail's background, padding and separator distinguish task links from article content. Its external grid position belongs to the parent frame, and the desk intentionally omits it. Keep this visual responsibility separate from the link table and from the native session policy.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="side_panel"))*.

## footer

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value declarations for the closing region.

These values retain the original footer color, type size, spacing, border and background. The Markdown footer owns actual destinations; this table neither creates links nor starts logout. The same surface properties can style public and desk footers while their terminal content stays separate.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="footer"))*.

## card_title

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value typography for article-card headings.

The current-query renderer escapes article titles and retains its native h2 structure. The View theme targets that structure with these original properties, rather than returning to front/back page bindings. This reader does not query articles, truncate titles or modify their stored text.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="card_title"))*.

## card_body

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value typography for card summaries.

Summary data belongs to the current article query, and full Markdown body remains reserved for detail. These properties affect only appearance; they do not choose or replace content. After changing the selector target, verify that both escaped summaries and full article bodies keep their separate behavior.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="card_body"))*.

## chrome

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Original introduction panels, badge container, reading steps and full-article rules.

Current introductions are terminal Markdown, so the View mapping addresses real headings, bullets and ordered lists. Article selectors continue to match the native detail renderer. Keep those mappings explicit and retain the original values; do not hide layout inside authored HTML to recover an old selector.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="chrome"))*.

## kicker

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value typography for the introduction category.

The original letter spacing, color, small type and floating animation target the Markdown blockquote used for the category. Its content remains an ordinary readable line in the leaf. Compiling these properties does not create that content or make the browser fetch a font.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="kicker"))*.

## intro_title

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value typography for the primary introduction heading.

These declarations retain the display font, responsive size, gradient text and glow animation. The View theme applies them to the actual Markdown h1, avoiding an extra heading-size multiplier on a wrapper. Browser validation checks the computed font family and size; reading this table alone cannot prove rendering.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="intro_title"))*.

## lede

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value typography for supporting introduction prose.

The original width, line height, color, spacing and typeface target the leaf's explanatory paragraph. It remains separate from claim bullets and numbered steps. Keep the selector on the real paragraph rather than styling a wrapper and relying on browser default margins to approximate the old appearance.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="lede"))*.

## claim

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Named property/value declarations for each short feature badge.

The Markdown bullet list is the original flex badge container, while each item receives these original inline badge properties. The container's gap and wrapping belong to the chrome section. This distinction preserves both readable list content and the actual visual grouping without another binding-table language.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="claim"))*.

## step

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Original margin declaration for one guided-reading step.

This section has a single property row, so its native result contains scalar cells rather than aligned lists. The same style compiler accepts that shape. The ordered Markdown list owns step numbering and links; this value only supplies the original per-item spacing and does not reorder the reading sequence.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="step"))*.

## widgets

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Original selector/property/value and optional media table, retaining column or single-row scalar shape |

Original card, form, flash and administration state declarations.

Hover and focus rules, field widths, action rows and desk-specific styles remain in their original order. The native form resource owns validation and submission. Theme projection changes only structural addresses; a visual form error is not proof that a server rejected data, so keep real failed and successful submissions in acceptance.

Grant fs.read on the site root. The exact section must be unique; missing or unsupported content remains a native diagnostic. The result is data, not a written stylesheet, running animation or business query.

Return *[Read this section's actual original literal table](document.read_table(path="styles/shell.tables.md", title="widgets"))*.


## _compile

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Exact original section in shell.tables.md |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | One theme section compiled at its original selectors |

Use `name` with fs.read and plugin on the site root. Native reading preserves original values and geometry; the style compiler handles selectors, media and keyframes. This retained helper intentionally keeps old-shell addresses for migration comparison. Current serving uses styles/index for explicit View targets. A missing section remains diagnostic, and returned text neither writes a file nor proves the new structure is styled.

We **declarations = [Read this section's actual original declarations](document.read_table(path="styles/shell.tables.md", title=`name`))**.
Return *[Compile original addresses for retained comparison](web.make_style(name=`name`, table=`declarations`))*.

## css

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Baseline CSS preserving the six original areas and cascade |

Grant fs.read and plugin on the site root. Compile tokens, keyframes, layout, responsive, chrome and widgets in their original order and concatenate them exactly as before. Named properties remain independently readable but are not added to this baseline, which would change its original cascade; the View entry restores them at actual semantic targets.

The current site loads styles/index output rather than this old-shell baseline. Equal hashes prove retained compilation, not selector reach, font delivery or mobile geometry. Pair verify's actual row comparison with browser acceptance, and let the serving workflow explicitly write the actual projected theme.

`names` =

| name |
|---|
| vars |
| motion |
| layout |
| responsive |
| chrome |
| widgets |

We **parts = [Start the ordered compiled sections](table.empty_list())**.
- [Compile each area in the declared cascade order](for names as name)
  We **css = [Compile the selected theme area](_compile(name=`name`))** and **parts = [Preserve its original position](table.append(list=`parts`, item=`css`))**.
Return *[Join the compiled sections without changing separators](text.str_join(xs=`parts`, sep=""))*.
