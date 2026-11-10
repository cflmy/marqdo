# View geometry adaptation

These are literal declarations for the explicit View wrappers. The original palette, motion, typography and component properties remain in shell.tables.md. The callable theme entry explains how these small geometry rules connect the recursive frame to that preserved data.

## geometry

The basic preview wrapper should inherit this site's own fonts and full width. Resource placeholders remain structural values but their native semantic children participate in the original named grid. Lists use Markdown rather than a private intro binding table.

| selector | property | value |
|---|---|---|
| .mq-document .mq-view | max-width | none |
| .mq-document .mq-view | margin | 0 |
| .mq-document .mq-view | padding | 0 |
| .mq-document .mq-view | line-height | inherit |
| .mq-document .mq-view | font-family | inherit |
| .mq-document .site-layout | display | grid |
| .mq-document .site-layout | grid-template-rows | auto 1fr auto |
| .mq-document .site-layout | min-height | 100vh |
| .mq-document .site-layout > .slot-nav | display | contents |
| .mq-document .site-layout > .slot-sidebar | display | contents |
| .mq-document .site-layout .main-intro ul | padding | 0 |
| .mq-document .site-layout .main-intro ul | list-style | none |
| .mq-document .site-layout .main-intro li > p | margin | 0 |
| .mq-document .site-layout .main-intro blockquote > p | margin | 0 |

## selectors

Only these structural addresses change. The original property, value, media and order remain untouched; this is a site migration table, not a new page language.

| @ | old | new |
|---|---|---|
| 1 | body.has-sidebar | .site-layout.has-sidebar |
| 2 | body.no-sidebar | .site-layout.no-sidebar |
| 3 | body.layout-stacked | .site-layout.layout-stacked |
| 4 | body.is-desk | .site-layout.is-desk |
| 5 | main.main | section.main |
| 6 | .main-intro > .claim | .main-intro ul |
| 7 | .main-intro > .steps | .main-intro ol |
| 8 | .intro_title | .main-intro h1 |
| 9 | .kicker | .main-intro blockquote |

## areas

Compile these original sections in the same cascade order.

| name |
|---|
| vars |
| motion |
| layout |
| responsive |
| chrome |
| widgets |


## properties

Restore every originally attached named property table at its actual semantic target. The native style compiler adds the leading class dot to each selected name.

| @ | name | selector |
|---|---|---|
| 1 | topnav | mq-document .site-layout .topnav |
| 2 | side_panel | mq-document .site-layout .side |
| 3 | footer | mq-document .site-layout .foot |
| 4 | card_title | mq-document .site-layout .content.cards h2 |
| 5 | card_body | mq-document .site-layout .content.cards p |
| 6 | kicker | mq-document .site-layout .main-intro blockquote |
| 7 | intro_title | mq-document .site-layout .main-intro h1 |
| 8 | lede | mq-document .site-layout .main-intro .slot-intro-content > p |
| 9 | claim | mq-document .site-layout .main-intro ul > li |
| 10 | step | mq-document .site-layout .main-intro ol > li |
