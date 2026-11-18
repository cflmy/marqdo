---
markup: occlusion
title: Literary theme entry for recursive View Documents
import:
  document: lib/document.mq.md
  web: ext/web/web.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

Read this entry before changing presentation. shell.tables.md retains every original declaration and shell.mq.md can still compile the original CSS. This entry projects only selectors onto the explicit View tree, restores the named property tables formerly attached by compose, and adds the bounded wrapper geometry in view.tables.md. Colors, property values, media conditions, particles and keyframe bodies remain the original data.

## _selector

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| selector | text | required | One original author-controlled selector |
| mappings | list | required | Explicit old/new structural addresses read from view.tables.md |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Equivalent selector scoped to the explicit recursive site frame |

Use `selector` and `mappings` from the actual retained theme and migration leaf. The old body-level sidebar and desk classes now belong to site-layout, while the main region is a section. The mappings preserve Markdown's actual heading, category, badge list and reading steps rather than restoring a private intro-binding dialect.

Global document backgrounds, root tokens, selection styling, resets and keyframes stay global. Local selectors gain site scope so basic View defaults cannot override the deliberate theme. This is the fixed migration of this known site tree, not a generic CSS parser or visitor-controlled rewrite. The caller reads mappings once per area and passes those rows here, keeping this helper pure and avoiding repeated filesystem reads.

- [Apply this site's explicit structural address mappings](for mappings as mapping)
  We **selector = [Replace only the known selector address](text.replace(text=`selector`, old=access(mapping, "old"), new=access(mapping, "new")))**.
1. `selector` == "*" or `selector` == "::selection" or text.starts_with(text=`selector`, prefix="body") or text.starts_with(text=`selector`, prefix="html") or text.starts_with(text=`selector`, prefix=":root") or text.starts_with(text=`selector`, prefix="@")
  Return *selector* because document-level rules and keyframes keep their existing scope.
1. text.starts_with(text=`selector`, prefix=".site-layout")
  Return *[Scope the layout-state selector once](".mq-document " + selector)*.
Return *[Keep region selectors within this site's own frame](".mq-document .site-layout " + selector)*.


## _table

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Original vars, motion, layout, responsive, chrome or widgets section |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Original declaration rows with only their selector field projected |

Use `name` with fs.read under the site root. Native Markdown reading preserves the original table and typed cells; from_columns converts its aligned columns to row records. Every row keeps the same property, value, optional media and original position. Only selector changes, and immutable updates preserve the retained source table.

This function does not compile or write CSS. Keeping the projected rows callable makes migration evidence inspectable: compare them to the original native table, including keyframe values and media, before looking at browser output. A missing section, ragged column or unsupported literal remains a real diagnostic rather than silently dropping a style. The mapping applies to this known site tree, not arbitrary CSS supplied by a visitor.

We **mappings = [Read the fixed structural migration table](document.read_table(path="styles/view.tables.md", title="selectors"))**.
We **columns = [Read the unchanged declaration leaf](document.read_table(path="styles/shell.tables.md", title=`name`))** and **rows = [Keep each original declaration as a record](table.from_columns(columns=`columns`))** and **projected = [Start the selector projection](table.empty_list())**.
- [Retain every declaration in its original position](for rows as row)
  We **selector = [Project only this row's structural address](_selector(selector=access(row, "selector"), mappings=`mappings`))** and **record = [Keep every other declaration field unchanged](table.put(in=`row`, at="selector", value=`selector`))** and **projected = [Retain this original property and value](table.append(list=`projected`, item=`record`))**.
Return *projected* for actual compilation or independent comparison.

## _property

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Original named property/value section |
| selector | text | required | Known semantic target in the new View tree, without leading dot |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Compiled original properties at their explicit new target |

Use `name` and `selector` with fs.read and plugin. Removed compose operations used to attach these named properties to generated fields. The same declarations now target the native navigation, side/footer, current card classes and terminal Markdown elements. Their values do not change, and they are not inferred from user content.

The selector is authored by the theme entry and begins with mq-document so the native style compiler supplies its leading class dot. The explicit target makes old property tables useful without restoring an obsolete page-binding dialect. Compilation returns text only; the serving workflow owns the selected generated path and native write.

We **properties = [Read the original named property declaration](document.read_table(path="styles/shell.tables.md", title=`name`))**.
Return *[Compile the known semantic target](web.make_style(name=`selector`, table=`properties`))*.

## css

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Full View theme with original values, explicit selector projection and wrapper geometry |

Call with fs.read and plugin on the selected site root. Tokens and keyframes keep their original order, followed by layout, responsive, chrome and widget rules. Named property declarations restore the typography and region surfaces that the old compose calls attached separately. The generated CSS contains no replacement palette or abbreviated theme.

The final geometry table only adapts the existing runtime wrappers: it removes the basic preview width/font defaults, lets native navigation resources participate in the shared grid, and preserves the explicit sidebar/desk class relationship. Markdown feature bullets become the original flex badge group, and the first explanatory paragraph uses the original lede properties. This keeps the content leaf readable without a hidden HTML template.

Compilation neither writes the stylesheet nor claims browser acceptance. The serving workflow writes the result once before discovery. Verify the real computed colors, type sizes, wide sidebar, narrow stacked order, current cards and gated desk. If a selector no longer reaches its intended node, repair that mapping rather than deleting the original visual capability.

We **areas = [Read the preserved theme area order](document.read_table(path="styles/view.tables.md", title="areas"))**.

We **parts = [Start the actual ordered theme](table.empty_list())**.
- [Compile every original theme area](for areas as name)
  We **rows = [Retain all original values and media](_table(name=`name`))** and **compiled = [Compile the actual projected declarations](web.make_style(name=`name`, table=`rows`))** and **parts = [Keep the original cascade order](table.append(list=`parts`, item=`compiled`))**.

We **properties = [Read the actual named-property targets](document.read_table(path="styles/view.tables.md", title="properties"))**.

- [Restore each originally bound property group](for properties as property)
  We **compiled = [Compile its explicit semantic target](_property(name=access(property, "name"), selector=access(property, "selector")))** and **parts = [Retain that real property group](table.append(list=`parts`, item=`compiled`))**.
We **geometry = [Read the bounded View-wrapper adaptation](document.read_table(path="styles/view.tables.md", title="geometry"))** and **compiled = [Compile native geometry adaptation](web.make_style(name="view", table=`geometry`))** and **parts = [Finish with the explicit wrapper rules](table.append(list=`parts`, item=`compiled`))**.
Return *[Keep actual compiled sections in order](text.str_join(xs=`parts`, sep=""))*.
