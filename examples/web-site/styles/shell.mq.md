---
title: styles/shell
description: >-
  Literary dark theme for examples/web-site — ink / wine / gold after the
  anlian.cyou Zhang Ailing topic. GFM tables only; no author .css.
import web:ext/web/web.mq.md
import text:lib/text.mq.md
---

Authors style the site with **GFM tables** only. App pages use `shell_css=off`
so host defaults do not fight this theme. Palette mirrors
[海上花开](https://www.anlian.cyou/topics/author-zhangailing/): night ink,
wine, cold gold, serif display.

## vars

`vars` =

| selector | property | value |
|----------|----------|-------|
| :root | --ink | #0a0809 |
| :root | --ink-lift | #141012 |
| :root | --wine | #6b1e3a |
| :root | --wine-soft | rgba(107,30,58,0.35) |
| :root | --gold | #d4af37 |
| :root | --gold-dim | rgba(212,175,55,0.45) |
| :root | --gold-soft | rgba(212,175,55,0.12) |
| :root | --paper | rgba(255,248,240,0.92) |
| :root | --paper-dim | rgba(255,248,240,0.72) |
| :root | --paper-faint | rgba(255,248,240,0.48) |
| :root | --glass | rgba(20,12,18,0.72) |
| :root | --glass-strong | rgba(20,12,18,0.88) |
| :root | --border | rgba(212,175,55,0.22) |
| :root | --border-soft | rgba(212,175,55,0.12) |
| :root | --danger | #e8a0a8 |
| :root | --radius | 4px |
| :root | --radius-sm | 2px |
| :root | --space-2 | 8px |
| :root | --space-3 | 12px |
| :root | --space-4 | 16px |
| :root | --space-5 | 20px |
| :root | --space-6 | 24px |
| :root | --space-8 | 32px |
| :root | --text | 16px |
| :root | --leading | "1.75" |
| :root | --shadow | "0 24px 80px rgba(0,0,0,.55), inset 0 1px 0 rgba(255,255,255,.06)" |
| :root | --shadow-card | "0 12px 40px rgba(107,30,58,.35)" |
| :root | --ring | "0 0 0 2px rgba(212,175,55,.35)" |
| :root | --serif | "'Noto Serif SC', 'Source Serif 4', Georgia, serif" |
| :root | --sans | "'Noto Sans SC', 'IBM Plex Sans', system-ui, sans-serif" |
| :root | --display | "'Great Vibes', 'Allura', cursive" |
| :root | --mono | "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace" |
| ::selection | background | rgba(212,175,55,0.35) |
| ::selection | color | var(--paper) |

*`vars`*

## motion

`motion` =

| selector | property | value |
|----------|----------|-------|
| @keyframes rise 0% | opacity | "0" |
| @keyframes rise 0% | transform | translateY(14px) |
| @keyframes rise 100% | opacity | "1" |
| @keyframes rise 100% | transform | translateY(0) |
| @keyframes float 0% | transform | translateY(0) |
| @keyframes float 0% | opacity | "0.7" |
| @keyframes float 50% | transform | translateY(6px) |
| @keyframes float 50% | opacity | "1" |
| @keyframes float 100% | transform | translateY(0) |
| @keyframes float 100% | opacity | "0.7" |
| @keyframes goldPulse 0% | text-shadow | "0 0 24px rgba(212,175,55,.2)" |
| @keyframes goldPulse 50% | text-shadow | "0 0 48px rgba(212,175,55,.45)" |
| @keyframes goldPulse 100% | text-shadow | "0 0 24px rgba(212,175,55,.2)" |
| @keyframes particleDrift 0% | transform | translate3d(0,0,0) |
| @keyframes particleDrift 50% | transform | translate3d(-2.5vw, 1.5vh, 0) |
| @keyframes particleDrift 100% | transform | translate3d(0,0,0) |
| @keyframes particleDrift2 0% | transform | translate3d(0,0,0) |
| @keyframes particleDrift2 50% | transform | translate3d(2vw, -2vh, 0) |
| @keyframes particleDrift2 100% | transform | translate3d(0,0,0) |
| @keyframes nebulaPulse 0% | opacity | "0.55" |
| @keyframes nebulaPulse 50% | opacity | "0.9" |
| @keyframes nebulaPulse 100% | opacity | "0.55" |

*`motion`*

## layout

`layout` =

| selector | property | value |
|----------|----------|-------|
| * | box-sizing | border-box |
| html | scroll-behavior | smooth |
| html | -webkit-text-size-adjust | 100% |
| body | margin | "0" |
| body | min-height | 100vh |
| body | font-family | var(--sans) |
| body | font-weight | "400" |
| body | font-size | var(--text) |
| body | line-height | var(--leading) |
| body | color | var(--paper) |
| body | background | var(--ink) |
| body | background-image | "radial-gradient(ellipse 90% 55% at 50% -15%, rgba(107,30,58,.5), transparent 58%), radial-gradient(ellipse 55% 45% at 92% 75%, rgba(212,175,55,.14), transparent 55%), radial-gradient(ellipse 40% 35% at 8% 70%, rgba(107,30,58,.22), transparent 60%), linear-gradient(180deg, #0a0809 0%, #120c10 45%, #0a0809 100%)" |
| body | background-attachment | fixed |
| body | display | grid |
| body | grid-template-rows | "auto 1fr auto" |
| body | -webkit-font-smoothing | antialiased |
| body | position | relative |
| body | overflow-x | hidden |
| body::before | content | '""' |
| body::before | position | fixed |
| body::before | inset | "0" |
| body::before | z-index | "1" |
| body::before | pointer-events | none |
| body::before | opacity | "0.1" |
| body::before | background-image | "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E\")" |
| html::before | content | '""' |
| html::before | position | fixed |
| html::before | inset | "0" |
| html::before | z-index | "0" |
| html::before | pointer-events | none |
| html::before | background-image | "radial-gradient(ellipse 50% 40% at 15% 30%, rgba(107,30,58,.35), transparent 70%), radial-gradient(ellipse 45% 35% at 85% 60%, rgba(212,175,55,.12), transparent 65%), radial-gradient(ellipse 40% 30% at 50% 90%, rgba(107,30,58,.25), transparent 70%)" |
| html::before | animation | "nebulaPulse 12s ease-in-out infinite" |
| body::after | content | '""' |
| body::after | position | fixed |
| body::after | top | "0" |
| body::after | left | "0" |
| body::after | width | "2px" |
| body::after | height | "2px" |
| body::after | z-index | "2" |
| body::after | pointer-events | none |
| body::after | border-radius | 50% |
| body::after | background | transparent |
| body::after | box-shadow | "82vw 15vh 0px 0px rgba(212,175,55,0.55),29vw 18vh 0px 1px rgba(212,175,55,0.25),95vw 70vh 1px 0px rgba(212,175,55,0.55),4vw 12vh 0px 1px rgba(212,175,55,0.35),78vw 4vh 0px 1px rgba(245,230,200,0.5),84vw 90vh 1px 0px rgba(245,230,200,0.5),58vw 76vh 0px 0px rgba(255,248,240,0.4),90vw 55vh 0px 0px rgba(255,248,240,0.4),28vw 98vh 0px 0px rgba(255,248,240,0.4),49vw 13vh 0px 1px rgba(255,248,240,0.4),34vw 6vh 1px 1px rgba(212,175,55,0.25),16vw 49vh 0px 1px rgba(212,175,55,0.55),80vw 47vh 0px 1px rgba(245,230,200,0.5),9vw 6vh 0px 0px rgba(212,175,55,0.25),11vw 30vh 1px 0px rgba(212,175,55,0.55),59vw 82vh 0px 0px rgba(255,248,240,0.4),46vw 27vh 0px 1px rgba(212,175,55,0.25),88vw 83vh 0px 1px rgba(212,175,55,0.55),94vw 32vh 1px 0px rgba(212,175,55,0.35),35vw 82vh 0px 1px rgba(212,175,55,0.25),42vw 99vh 0px 0px rgba(212,175,55,0.55),41vw 52vh 0px 0px rgba(255,248,240,0.4),73vw 92vh 0px 1px rgba(255,248,240,0.4),64vw 51vh 1px 0px rgba(212,175,55,0.25),34vw 18vh 0px 1px rgba(212,175,55,0.35),75vw 55vh 1px 0px rgba(245,230,200,0.5),29vw 18vh 1px 0px rgba(245,230,200,0.5),97vw 7vh 0px 1px rgba(212,175,55,0.55),21vw 88vh 0px 0px rgba(107,30,58,0.45),49vw 77vh 0px 1px rgba(107,30,58,0.45),2vw 88vh 0px 1px rgba(212,175,55,0.25),69vw 97vh 0px 0px rgba(255,248,240,0.4),38vw 56vh 1px 0px rgba(212,175,55,0.35),93vw 93vh 0px 1px rgba(255,248,240,0.4),14vw 81vh 0px 0px rgba(255,248,240,0.4),48vw 98vh 0px 1px rgba(212,175,55,0.35),42vw 63vh 0px 0px rgba(212,175,55,0.55),40vw 31vh 0px 1px rgba(212,175,55,0.55),11vw 11vh 1px 0px rgba(212,175,55,0.25),98vw 69vh 0px 1px rgba(212,175,55,0.35),61vw 71vh 0px 1px rgba(212,175,55,0.35),78vw 55vh 0px 1px rgba(212,175,55,0.35),40vw 52vh 0px 0px rgba(212,175,55,0.25),67vw 58vh 0px 0px rgba(212,175,55,0.55),9vw 44vh 0px 1px rgba(212,175,55,0.55),29vw 1vh 0px 0px rgba(212,175,55,0.55),9vw 5vh 0px 1px rgba(255,248,240,0.4),31vw 36vh 1px 0px rgba(212,175,55,0.25),70vw 17vh 1px 0px rgba(212,175,55,0.25),61vw 53vh 0px 0px rgba(212,175,55,0.35),85vw 56vh 1px 0px rgba(255,248,240,0.4),60vw 94vh 0px 0px rgba(212,175,55,0.55),52vw 94vh 0px 0px rgba(255,248,240,0.4),25vw 25vh 1px 0px rgba(245,230,200,0.5),55vw 24vh 1px 0px rgba(255,248,240,0.4),10vw 57vh 0px 0px rgba(245,230,200,0.5),84vw 70vh 0px 0px rgba(212,175,55,0.55),22vw 53vh 1px 0px rgba(107,30,58,0.45),52vw 8vh 1px 0px rgba(212,175,55,0.35),50vw 34vh 0px 0px rgba(107,30,58,0.45),90vw 94vh 1px 0px rgba(245,230,200,0.5),25vw 38vh 0px 1px rgba(212,175,55,0.35),95vw 70vh 0px 0px rgba(212,175,55,0.55),7vw 75vh 0px 0px rgba(107,30,58,0.45),66vw 11vh 0px 1px rgba(212,175,55,0.35),9vw 87vh 1px 0px rgba(212,175,55,0.35),73vw 32vh 0px 1px rgba(245,230,200,0.5),11vw 54vh 0px 0px rgba(212,175,55,0.25),27vw 86vh 0px 0px rgba(212,175,55,0.25),34vw 51vh 0px 0px rgba(212,175,55,0.35),41vw 97vh 0px 0px rgba(212,175,55,0.55),80vw 73vh 0px 1px rgba(212,175,55,0.55)" |
| body::after | animation | "particleDrift 48s ease-in-out infinite" |
| html::after | content | '""' |
| html::after | position | fixed |
| html::after | top | "0" |
| html::after | left | "0" |
| html::after | width | "1px" |
| html::after | height | "1px" |
| html::after | z-index | "2" |
| html::after | pointer-events | none |
| html::after | border-radius | 50% |
| html::after | background | transparent |
| html::after | box-shadow | "28vw 65vh 0 0 rgba(255,248,240,0.25),17vw 45vh 0 0 rgba(212,175,55,0.3),32vw 48vh 0 0 rgba(255,248,240,0.25),21vw 57vh 0 0 rgba(107,30,58,0.3),91vw 39vh 0 0 rgba(107,30,58,0.3),84vw 68vh 0 0 rgba(212,175,55,0.3),86vw 71vh 0 0 rgba(255,248,240,0.25),85vw 14vh 0 0 rgba(212,175,55,0.3),34vw 15vh 0 0 rgba(212,175,55,0.3),96vw 71vh 0 0 rgba(212,175,55,0.3),35vw 37vh 0 0 rgba(107,30,58,0.3),27vw 92vh 0 0 rgba(255,248,240,0.25),27vw 88vh 0 0 rgba(107,30,58,0.3),34vw 65vh 0 0 rgba(255,248,240,0.25),33vw 7vh 0 0 rgba(212,175,55,0.3),82vw 55vh 0 0 rgba(255,248,240,0.25),6vw 1vh 0 0 rgba(255,248,240,0.25),99vw 17vh 0 0 rgba(107,30,58,0.3),34vw 21vh 0 0 rgba(107,30,58,0.3),57vw 71vh 0 0 rgba(107,30,58,0.3),55vw 72vh 0 0 rgba(212,175,55,0.3),15vw 10vh 0 0 rgba(107,30,58,0.3),20vw 70vh 0 0 rgba(212,175,55,0.3),48vw 75vh 0 0 rgba(107,30,58,0.3),19vw 56vh 0 0 rgba(212,175,55,0.3),6vw 40vh 0 0 rgba(255,248,240,0.25),6vw 46vh 0 0 rgba(212,175,55,0.3),88vw 32vh 0 0 rgba(107,30,58,0.3),14vw 46vh 0 0 rgba(107,30,58,0.3),53vw 80vh 0 0 rgba(107,30,58,0.3),20vw 31vh 0 0 rgba(212,175,55,0.3),23vw 53vh 0 0 rgba(212,175,55,0.3),23vw 95vh 0 0 rgba(255,248,240,0.25),53vw 86vh 0 0 rgba(107,30,58,0.3),32vw 35vh 0 0 rgba(212,175,55,0.3),90vw 14vh 0 0 rgba(255,248,240,0.25),5vw 61vh 0 0 rgba(212,175,55,0.3),26vw 59vh 0 0 rgba(255,248,240,0.25),40vw 30vh 0 0 rgba(212,175,55,0.3),4vw 85vh 0 0 rgba(212,175,55,0.3),52vw 43vh 0 0 rgba(255,248,240,0.25),9vw 99vh 0 0 rgba(255,248,240,0.25),45vw 83vh 0 0 rgba(107,30,58,0.3),52vw 87vh 0 0 rgba(107,30,58,0.3),43vw 4vh 0 0 rgba(212,175,55,0.3),34vw 23vh 0 0 rgba(107,30,58,0.3),34vw 5vh 0 0 rgba(212,175,55,0.3),77vw 56vh 0 0 rgba(255,248,240,0.25)" |
| html::after | animation | "particleDrift2 64s ease-in-out infinite" |
| body.has-sidebar | grid-template-columns | "14rem minmax(0, 1fr)" |
| body.has-sidebar | grid-template-areas | '"top top" "side main" "foot foot"' |
| body.no-sidebar | grid-template-areas | '"top" "main" "foot"' |
| body.layout-stacked | grid-template-areas | '"top" "main" "foot"' |
| body.is-desk | grid-template-areas | '"top" "main" "foot"' |
| header.topnav | grid-area | top |
| aside.side | grid-area | side |
| main.main | grid-area | main |
| footer.foot | grid-area | foot |
| header.topnav | position | relative |
| header.topnav | z-index | "20" |
| aside.side | position | relative |
| aside.side | z-index | "10" |
| main.main | position | relative |
| main.main | z-index | "10" |
| footer.foot | position | relative |
| footer.foot | z-index | "10" |
| main.main | width | "100%" |
| main.main | max-width | "min(78rem, 100%)" |
| main.main | padding | "var(--space-8) var(--space-6) 4rem" |
| main.main | animation | "rise .8s cubic-bezier(.22,1,.36,1) both" |
| a | color | var(--gold) |
| a | text-decoration | none |
| a | transition | color 0.25s ease |
| a:hover | color | #f5e6c8 |
| code | font-family | var(--mono) |
| code | font-size | "0.82em" |
| code | font-weight | "400" |
| code | background | rgba(0,0,0,0.35) |
| code | border | 1px solid var(--border-soft) |
| code | padding | "0.12em 0.45em" |
| code | border-radius | var(--radius-sm) |
| code | color | var(--gold) |

*`layout`*

## responsive

`responsive` =

| media | selector | property | value |
|-------|----------|----------|-------|
| (max-width: 860px) | body.has-sidebar | grid-template-columns | 1fr |
| (max-width: 860px) | body.has-sidebar | grid-template-areas | '"top" "main" "side" "foot"' |
| (max-width: 860px) | aside.side | border-right | "0" |
| (max-width: 860px) | aside.side | border-top | 1px solid var(--border-soft) |
| (max-width: 860px) | main.main | padding | "var(--space-6) var(--space-4) var(--space-8)" |
| (max-width: 860px) | .content.cards | grid-template-columns | 1fr |
| (max-width: 860px) | .site-form form | grid-template-columns | 1fr |
| (max-width: 860px) | .site-form label:nth-child(1) | grid-column | "1 / -1" |
| (max-width: 860px) | .site-form label:nth-child(2) | grid-column | "1 / -1" |
| (max-width: 860px) | header.topnav | padding | "0.85rem var(--space-4)" |
| (max-width: 860px) | .intro_title | font-size | "2.4rem" |
| (prefers-reduced-motion: reduce) | main.main | animation | none |
| (prefers-reduced-motion: reduce) | .kicker | animation | none |
| (prefers-reduced-motion: reduce) | article.card | transition | none |
| (prefers-reduced-motion: reduce) | body::after | animation | none |
| (prefers-reduced-motion: reduce) | html::after | animation | none |
| (prefers-reduced-motion: reduce) | html::before | animation | none |

*`responsive`*

## topnav

`topnav` =

| property | value |
|----------|-------|
| position | sticky |
| top | "0" |
| z-index | "40" |
| display | flex |
| align-items | center |
| justify-content | flex-start |
| gap | var(--space-4) |
| padding | "1rem var(--space-6)" |
| background | "linear-gradient(180deg, rgba(10,8,9,.92) 0%, rgba(10,8,9,.55) 70%, transparent 100%)" |
| backdrop-filter | blur(10px) |
| -webkit-backdrop-filter | blur(10px) |
| border-bottom | 1px solid var(--border-soft) |

*`topnav`*

## side_panel

`side_panel` =

| property | value |
|----------|-------|
| background | "rgba(10,8,9,.55)" |
| padding | "var(--space-6) var(--space-4)" |
| border-right | 1px solid var(--border-soft) |

*`side_panel`*

## footer

`footer` =

| property | value |
|----------|-------|
| color | var(--paper-faint) |
| font-size | "0.8rem" |
| letter-spacing | "0.08em" |
| padding | "var(--space-5) var(--space-6)" |
| border-top | 1px solid var(--border-soft) |
| background | "rgba(10,8,9,.75)" |

*`footer`*

## card_title

`card_title` =

| property | value |
|----------|-------|
| font-family | var(--serif) |
| font-size | "1.15rem" |
| font-weight | "600" |
| margin | "0 0 var(--space-2)" |
| color | var(--gold) |
| letter-spacing | "0.02em" |
| line-height | "1.35" |

*`card_title`*

## card_body

`card_body` =

| property | value |
|----------|-------|
| color | var(--paper-dim) |
| margin | "0" |
| line-height | "1.65" |
| font-size | "0.9rem" |
| font-weight | "400" |
| display | "-webkit-box" |
| -webkit-box-orient | vertical |
| -webkit-line-clamp | "3" |
| overflow | hidden |

*`card_body`*

## chrome

Panel + claim/steps group layout (field look comes from named bind styles).

`chrome` =

| selector | property | value |
|----------|----------|-------|
| .main-intro | background | var(--glass) |
| .main-intro | border | 1px solid var(--border) |
| .main-intro | border-radius | var(--radius) |
| .main-intro | padding | "2.25rem 1.75rem 1.85rem" |
| .main-intro | box-shadow | var(--shadow) |
| .main-intro | backdrop-filter | blur(16px) |
| .main-intro | -webkit-backdrop-filter | blur(16px) |
| .main-intro | margin-bottom | var(--space-8) |
| .main-intro > .claim | display | flex |
| .main-intro > .claim | flex-wrap | wrap |
| .main-intro > .claim | gap | var(--space-2) |
| .main-intro > .claim | margin | "0 0 var(--space-6)" |
| .main-intro > .steps | margin | "0" |
| .main-intro > .steps | padding | "var(--space-5) var(--space-5) var(--space-5) 1.5rem" |
| .main-intro > .steps | color | var(--paper-dim) |
| .main-intro > .steps | background | "rgba(0,0,0,.35)" |
| .main-intro > .steps | border | 1px solid var(--border-soft) |
| .main-intro > .steps | border-left | 2px solid var(--gold-dim) |
| .main-intro > .steps | border-radius | var(--radius) |
| .main-intro > .steps | font-size | "0.95rem" |
| .main-intro > .steps | letter-spacing | "0.02em" |
| .main-intro > .steps a | color | var(--gold) |
| .main-intro > .steps a | font-weight | "500" |
| .main-intro > .steps a:hover | color | #f5e6c8 |
| .article | background | var(--glass) |
| .article | border | 1px solid var(--border) |
| .article | border-radius | var(--radius) |
| .article | padding | "2rem 1.75rem" |
| .article | box-shadow | var(--shadow) |
| .article | backdrop-filter | blur(16px) |
| .article | -webkit-backdrop-filter | blur(16px) |
| .article-title | font-family | var(--serif) |
| .article-title | font-size | "clamp(1.75rem, 4vw, 2.35rem)" |
| .article-title | font-weight | "600" |
| .article-title | color | var(--gold) |
| .article-title | letter-spacing | "0.04em" |
| .article-title | margin | "0 0 var(--space-5)" |
| .article-title | line-height | "1.25" |
| .article-body | color | var(--paper-dim) |
| .article-body | font-family | var(--sans) |
| .article-body | font-size | "1.02rem" |
| .article-body | line-height | "1.85" |
| .article-body.md p | margin | "0 0 1.1rem" |
| .article-body.md a | color | var(--gold) |
| article.card | cursor | pointer |
| a.card-link | color | inherit |
| a.card-link | text-decoration | none |
| a.card-link:hover | color | inherit |

*`chrome`*

## kicker

`kicker` =

| property | value |
|----------|-------|
| letter-spacing | "0.32em" |
| text-transform | uppercase |
| font-size | "0.72rem" |
| color | var(--gold-dim) |
| margin | "0 0 var(--space-4)" |
| font-weight | "400" |
| font-family | var(--sans) |
| animation | "float 2.8s ease-in-out infinite" |

*`kicker`*

## intro_title

`intro_title` =

| property | value |
|----------|-------|
| font-family | var(--display) |
| font-size | "clamp(3rem, 10vw, 4.8rem)" |
| font-weight | "400" |
| line-height | "1.15" |
| letter-spacing | "0.04em" |
| margin | "0 0 var(--space-4)" |
| color | transparent |
| background | "linear-gradient(135deg, #f5e6c8 0%, var(--gold) 42%, var(--wine) 100%)" |
| -webkit-background-clip | text |
| background-clip | text |
| animation | "goldPulse 4.5s ease-in-out infinite" |

*`intro_title`*

## lede

`lede` =

| property | value |
|----------|-------|
| color | var(--paper-dim) |
| max-width | "34rem" |
| line-height | "1.85" |
| margin | "0 0 var(--space-5)" |
| font-size | "1.05rem" |
| letter-spacing | "0.02em" |
| font-weight | "400" |
| font-family | var(--sans) |

*`lede`*

## claim

`claim` =

| property | value |
|----------|-------|
| display | inline-flex |
| align-items | center |
| padding | "0.2rem 0.65rem" |
| border-radius | var(--radius-sm) |
| background | var(--wine-soft) |
| color | #f0c9d4 |
| font-size | "0.68rem" |
| font-weight | "500" |
| letter-spacing | "0.14em" |
| border | 1px solid rgba(107,30,58,0.5) |

*`claim`*

## step

`step` =

| property | value |
|----------|-------|
| margin | "0.55rem 0" |

*`step`*

## widgets

`widgets` =

| selector | property | value |
|----------|----------|-------|
| ul.nav | display | flex |
| ul.nav | flex-wrap | wrap |
| ul.nav | align-items | center |
| ul.nav | gap | "0.35rem 1rem" |
| ul.nav | list-style | none |
| ul.nav | margin | "0" |
| ul.nav | padding | "0" |
| ul.nav a | display | inline-flex |
| ul.nav a | align-items | center |
| ul.nav a | text-decoration | none |
| ul.nav a | color | var(--paper-dim) |
| ul.nav a | padding | "0.35rem 0" |
| ul.nav a | border-radius | "0" |
| ul.nav a | font-weight | "400" |
| ul.nav a | font-size | "0.8rem" |
| ul.nav a | letter-spacing | "0.14em" |
| ul.nav a | transition | color 0.25s ease |
| ul.nav a:hover | background | transparent |
| ul.nav a:hover | color | var(--gold) |
| ul.nav a:hover | text-decoration | none |
| ul.nav a[href="/admin"] | color | var(--gold) |
| ul.nav a[href="/admin"] | letter-spacing | "0.18em" |
| ul.nav a[href="/admin"] | border-bottom | 1px solid var(--gold-dim) |
| ul.side-nav a[href="/admin"] | color | var(--gold) |
| body.is-desk header.topnav | border-bottom | 1px solid rgba(107,30,58,0.55) |
| body.is-desk header.topnav | box-shadow | "0 12px 40px rgba(107,30,58,.18)" |
| body.is-desk .main-intro | border-left | 3px solid var(--wine) |
| body.is-desk .site-form | border-color | rgba(107,30,58,0.45) |
| body.is-desk .site-form | box-shadow | "0 0 0 1px rgba(212,175,55,.12), 0 18px 48px rgba(0,0,0,.35)" |
| body.is-desk .content.cards | margin-top | var(--space-6) |
| ul.nav li:first-child a | font-family | var(--display) |
| ul.nav li:first-child a | font-weight | "400" |
| ul.nav li:first-child a | color | var(--gold) |
| ul.nav li:first-child a | font-size | "1.85rem" |
| ul.nav li:first-child a | letter-spacing | "0.06em" |
| ul.nav li:first-child a | padding | "0.1rem 0.2rem 0.1rem 0" |
| ul.nav li:first-child a:hover | background | transparent |
| ul.nav li:first-child a:hover | color | #f5e6c8 |
| .side-label | display | block |
| .side-label | font-size | "0" |
| .side-label | margin | "0 0 var(--space-4)" |
| .side-label::before | content | '"Path"' |
| .side-label::before | display | block |
| .side-label::before | font-size | "0.68rem" |
| .side-label::before | letter-spacing | "0.28em" |
| .side-label::before | text-transform | uppercase |
| .side-label::before | color | var(--gold-dim) |
| .side-label::before | font-weight | "400" |
| ul.side-nav | list-style | none |
| ul.side-nav | margin | "0" |
| ul.side-nav | padding | "0" |
| ul.side-nav | display | grid |
| ul.side-nav | gap | "2px" |
| ul.side-nav | border-left | 1px solid var(--border) |
| ul.side-nav | padding-left | "0.85rem" |
| ul.side-nav | margin-left | "0.25rem" |
| ul.side-nav a | display | block |
| ul.side-nav a | text-decoration | none |
| ul.side-nav a | color | var(--paper-dim) |
| ul.side-nav a | padding | "0.45rem 0.35rem" |
| ul.side-nav a | border-radius | "0" |
| ul.side-nav a | font-weight | "400" |
| ul.side-nav a | font-size | "0.88rem" |
| ul.side-nav a | letter-spacing | "0.06em" |
| ul.side-nav a | border | "0" |
| ul.side-nav a | position | relative |
| ul.side-nav a:hover | background | transparent |
| ul.side-nav a:hover | border-color | transparent |
| ul.side-nav a:hover | color | var(--gold) |
| ul.side-nav a:hover | text-decoration | none |
| ul.side-nav a:hover | box-shadow | none |
| ul.foot-nav | display | flex |
| ul.foot-nav | gap | var(--space-4) |
| ul.foot-nav | list-style | none |
| ul.foot-nav | margin | "0" |
| ul.foot-nav | padding | "0" |
| ul.foot-nav a | color | var(--paper-faint) |
| ul.foot-nav a | font-weight | "400" |
| ul.foot-nav a | letter-spacing | "0.1em" |
| ul.foot-nav a | font-size | "0.78rem" |
| ul.foot-nav a:hover | color | var(--gold) |
| .content.cards | display | grid |
| .content.cards | gap | var(--space-4) |
| .content.cards | margin-top | "0" |
| .content.cards | grid-template-columns | "repeat(auto-fill, minmax(16rem, 1fr))" |
| article.card | background | "rgba(0,0,0,.35)" |
| article.card | border | 1px solid var(--border-soft) |
| article.card | border-radius | var(--radius-sm) |
| article.card | padding | "1.35rem 1.15rem" |
| article.card | box-shadow | none |
| article.card | transition | "border-color .35s ease, box-shadow .35s ease, transform .35s ease" |
| article.card:hover | transform | "translateY(-3px)" |
| article.card:hover | box-shadow | var(--shadow-card) |
| article.card:hover | border-color | var(--gold-dim) |
| .site-form | background | var(--glass) |
| .site-form | border | 1px solid var(--border) |
| .site-form | border-radius | var(--radius) |
| .site-form | padding | "var(--space-6) var(--space-6) var(--space-7, 1.75rem)" |
| .site-form | margin | "0 0 var(--space-6)" |
| .site-form | max-width | "100%" |
| .site-form | width | "100%" |
| .site-form | box-shadow | var(--shadow) |
| .site-form | backdrop-filter | blur(16px) |
| .site-form | -webkit-backdrop-filter | blur(16px) |
| .site-form .meta | display | none |
| .site-form form | display | grid |
| .site-form form | grid-template-columns | "minmax(0, 1fr) minmax(0, 1fr)" |
| .site-form form | column-gap | var(--space-5) |
| .site-form form | row-gap | var(--space-5) |
| .site-form form | gap | var(--space-5) |
| .site-form form | margin | "0" |
| .site-form form | width | "100%" |
| .site-form form | max-width | "100%" |
| .site-form label | display | grid |
| .site-form label | gap | var(--space-2) |
| .site-form label | margin | "0" |
| .site-form label | min-width | "0" |
| .site-form label | font-weight | "400" |
| .site-form label | color | var(--gold) |
| .site-form label | font-size | "0.78rem" |
| .site-form label | letter-spacing | "0.16em" |
| .site-form label | text-transform | uppercase |
| .site-form label:nth-child(1) | grid-column | "1" |
| .site-form label:nth-child(2) | grid-column | "2" |
| .site-form label:nth-child(n+3) | grid-column | "1 / -1" |
| .site-form .actions | grid-column | "1 / -1" |
| .site-form input | -webkit-appearance | none |
| .site-form textarea | -webkit-appearance | none |
| .site-form input | appearance | none |
| .site-form textarea | appearance | none |
| .site-form input | box-sizing | border-box |
| .site-form textarea | box-sizing | border-box |
| .site-form input | display | block |
| .site-form textarea | display | block |
| .site-form input | width | "100%" |
| .site-form textarea | width | "100%" |
| .site-form input | height | "2.75rem" |
| .site-form textarea | min-height | "10rem" |
| .site-form label:nth-child(4) textarea | min-height | "14rem" |
| .site-form textarea | resize | vertical |
| .site-form input | padding | "0 0.9rem" |
| .site-form textarea | padding | "0.75rem 0.9rem" |
| .site-form input | border | 1px solid var(--border) |
| .site-form textarea | border | 1px solid var(--border) |
| .site-form input | border-radius | var(--radius-sm) |
| .site-form textarea | border-radius | var(--radius-sm) |
| .site-form input | background | "rgba(0,0,0,.4)" |
| .site-form textarea | background | "rgba(0,0,0,.4)" |
| .site-form input | color | var(--paper) |
| .site-form textarea | color | var(--paper) |
| .site-form input | font | inherit |
| .site-form textarea | font | inherit |
| .site-form input | font-size | "0.95rem" |
| .site-form textarea | font-size | "0.95rem" |
| .site-form input | line-height | "1.4" |
| .site-form textarea | line-height | "1.65" |
| .site-form input | transition | "border-color .2s ease, box-shadow .2s ease" |
| .site-form textarea | transition | "border-color .2s ease, box-shadow .2s ease" |
| .site-form input:hover | border-color | var(--gold-dim) |
| .site-form textarea:hover | border-color | var(--gold-dim) |
| .site-form input:focus | outline | none |
| .site-form textarea:focus | outline | none |
| .site-form input:focus | border-color | var(--gold) |
| .site-form textarea:focus | border-color | var(--gold) |
| .site-form input:focus | box-shadow | var(--ring) |
| .site-form textarea:focus | box-shadow | var(--ring) |
| .site-form input[readonly] | background | "rgba(0,0,0,.25)" |
| .site-form input[readonly] | color | var(--paper-faint) |
| .site-form .err | color | var(--danger) |
| .site-form .err | font-size | "0.8rem" |
| .site-form .err | font-weight | "500" |
| .site-form .err | letter-spacing | "0.04em" |
| .site-form .actions | display | flex |
| .site-form .actions | align-items | center |
| .site-form .actions | gap | var(--space-4) |
| .site-form .actions | margin-top | var(--space-2) |
| .site-form .actions | flex-wrap | wrap |
| .site-form button | -webkit-appearance | none |
| .site-form button | appearance | none |
| .site-form button | height | "2.75rem" |
| .site-form button | padding | "0 1.35rem" |
| .site-form button | background | "linear-gradient(135deg, #f5e6c8, var(--gold))" |
| .site-form button | color | var(--ink) |
| .site-form button | border | "0" |
| .site-form button | border-radius | 999px |
| .site-form button | font-weight | "600" |
| .site-form button | font-size | "0.85rem" |
| .site-form button | letter-spacing | "0.12em" |
| .site-form button | cursor | pointer |
| .site-form button | box-shadow | "0 8px 28px rgba(212,175,55,.28)" |
| .site-form button | transition | "transform .2s ease, box-shadow .2s ease, filter .2s ease" |
| .site-form button:hover | filter | brightness(1.06) |
| .site-form button:hover | transform | "translateY(-1px)" |
| .site-form button:hover | box-shadow | "0 12px 36px rgba(212,175,55,.4)" |
| .site-form .actions a | color | var(--paper-faint) |
| .site-form .actions a | font-weight | "400" |
| .site-form .actions a | text-decoration | none |
| .site-form .actions a | font-size | "0.85rem" |
| .site-form .actions a | letter-spacing | "0.08em" |
| .site-form .actions a:hover | color | var(--gold) |
| .flash.err | color | var(--danger) |
| .flash.err | font-weight | "500" |

*`widgets`*

## css

**v = > vars**
**m = > motion**
**l = > layout**
**r = > responsive**
**c = > chrome**
**w = > widgets**
**css_v = > web.make_style name="vars" table=v**
**css_m = > web.make_style name="motion" table=m**
**css_l = > web.make_style name="layout" table=l**
**css_r = > web.make_style name="responsive" table=r**
**css_c = > web.make_style name="chrome" table=c**
**css_w = > web.make_style name="widgets" table=w**

`parts` =

| css |
|-----|
| `css_v` |
| `css_m` |
| `css_l` |
| `css_r` |
| `css_c` |
| `css_w` |

**out = > text.str_join xs=`parts` sep=""**
*out*
