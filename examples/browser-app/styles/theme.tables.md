# Browser example theme

This literal data preserves the original browser-app palette, typefaces, spacing, backgrounds and controls. Region selectors are scoped to the native View frame. Callable compilation is documented in index.mq.md beside this data.

## original

Every original declaration appears in its original order. Only selectors gain the known frame address, keeping basic preview defaults from overriding the authored theme.

| selector | property | value |
|---|---|---|
| ":root" | "--ink" | "#1a1f1c" |
| ":root" | "--paper" | "#e8efe6" |
| ":root" | "--accent" | "#2f6f4e" |
| ":root" | "--muted" | "#5a6b60" |
| "*" | "box-sizing" | "border-box" |
| "body" | "margin" | "0" |
| "body" | "min-height" | "100vh" |
| "body" | "font-family" | "\"IBM Plex Sans\", \"Noto Sans SC\", sans-serif" |
| "body" | "color" | "var(--ink)" |
| "body" | "background" | "linear-gradient(165deg, #f3f7f2, var(--paper))" |
| "body" | "padding" | "2rem clamp(1rem, 4vw, 3rem)" |
| ".mq-document .browser-app h1" | "font-family" | "\"IBM Plex Serif\", \"Noto Serif SC\", serif" |
| ".mq-document .browser-app h1" | "margin" | "0 0 0.35rem" |
| ".mq-document .browser-app .lead" | "color" | "var(--muted)" |
| ".mq-document .browser-app .lead" | "max-width" | "42rem" |
| ".mq-document .browser-app nav a" | "margin-right" | "0.75rem" |
| ".mq-document .browser-app nav a" | "color" | "var(--accent)" |
| ".mq-document .browser-app section" | "margin" | "1.5rem 0" |
| ".mq-document .browser-app section" | "padding" | "1rem 0" |
| ".mq-document .browser-app section" | "border-top" | "1px solid #c5d4c9" |
| ".mq-document .browser-app #path" | "font-family" | "ui-monospace, monospace" |
| ".mq-document .browser-app #path" | "color" | "var(--muted)" |
| ".mq-document .browser-app #items" | "list-style" | "none" |
| ".mq-document .browser-app #items" | "padding" | "0" |
| ".mq-document .browser-app #items" | "margin" | "0.75rem 0" |
| ".mq-document .browser-app #items li" | "display" | "flex" |
| ".mq-document .browser-app #items li" | "align-items" | "center" |
| ".mq-document .browser-app #items li" | "gap" | "0.5rem" |
| ".mq-document .browser-app #items li" | "padding" | "0.4rem 0.6rem" |
| ".mq-document .browser-app #items li" | "margin-bottom" | "0.35rem" |
| ".mq-document .browser-app #items li" | "background" | "rgba(255,255,255,0.65)" |
| ".mq-document .browser-app #items li" | "border" | "1px solid #b7c7bb" |
| ".mq-document .browser-app #items button" | "margin-left" | "auto" |
| ".mq-document .browser-app input[type=\"text\"]" | "padding" | "0.45rem 0.65rem" |
| ".mq-document .browser-app input[type=\"text\"]" | "border" | "1px solid #b7c7bb" |
| ".mq-document .browser-app input[type=\"text\"]" | "width" | "min(16rem, 70%)" |
| ".mq-document .browser-app button" | "padding" | "0.45rem 0.9rem" |
| ".mq-document .browser-app button" | "border" | "0" |
| ".mq-document .browser-app button" | "background" | "var(--accent)" |
| ".mq-document .browser-app button" | "color" | "#f5faf6" |
| ".mq-document .browser-app button" | "font" | "inherit" |
| ".mq-document .browser-app button" | "cursor" | "pointer" |
| ".mq-document .browser-app button" | "margin" | "0.2rem 0.35rem 0.2rem 0" |
| ".mq-document .browser-app button.secondary" | "background" | "#3a4740" |
| ".mq-document .browser-app #tick, #net-out" | "font-variant-numeric" | "tabular-nums" |
| ".mq-document .browser-app #net-out" | "min-height" | "2.5rem" |
| ".mq-document .browser-app #net-out" | "padding" | "0.6rem 0.75rem" |
| ".mq-document .browser-app #net-out" | "background" | "rgba(255,255,255,0.55)" |
| ".mq-document .browser-app #net-out" | "border" | "1px solid #b7c7bb" |
| ".mq-document .browser-app #net-out" | "white-space" | "pre-wrap" |
| ".mq-document .browser-app #net-out" | "word-break" | "break-all" |
| ".mq-document .browser-app pre" | "margin-top" | "1rem" |
| ".mq-document .browser-app pre" | "padding" | "0.85rem 1rem" |
| ".mq-document .browser-app pre" | "background" | "#122018" |
| ".mq-document .browser-app pre" | "color" | "#d7eadc" |
| ".mq-document .browser-app pre" | "font-size" | "0.8rem" |
| ".mq-document .browser-app pre" | "overflow" | "auto" |
| ".mq-document .browser-app pre" | "max-width" | "42rem" |
| ".mq-document .browser-app .err" | "color" | "#ffb4a8" |

## geometry

These additions adapt the preview wrappers and keep native controls and literal long text within a narrow viewport. They do not replace the retained palette or typography.

| selector | property | value |
|---|---|---|
| ".mq-document .mq-view" | "max-width" | "none" |
| ".mq-document .mq-view" | "margin" | "0" |
| ".mq-document .mq-view" | "padding" | "0" |
| ".mq-document .mq-view" | "font-family" | "inherit" |
| ".mq-document .mq-view" | "line-height" | "inherit" |
| ".mq-document .browser-app" | "display" | "block" |
| ".mq-document .browser-app .mq-column" | "display" | "block" |
| ".mq-document .browser-app .mq-row" | "flex-wrap" | "wrap" |
| ".mq-document .browser-app #items span" | "overflow-wrap" | "anywhere" |
| ".mq-document .browser-app button" | "max-width" | "100%" |
| ".mq-document .browser-app pre" | "white-space" | "pre-wrap" |
| ".mq-document .browser-app pre" | "overflow-wrap" | "anywhere" |
