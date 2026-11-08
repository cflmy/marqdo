# Layout decomposition reference

Read [the page](index.mq.md), then follow [components](components/index.mq.md) to
navigation, main and footer. Footer splits into brand and links; links splits
again into docs and community. Every terminal content.md is plain Markdown.
Parent .mq.md files only load/compose child rows and choose one relationship.

From the repository root, preview with:

```bash
marqdo run examples/web-layout-handbook/preview.mq.md
```

To serve, change into this directory, then run:

```bash
marqdo run serve.mq.md
```

Check /, /about and /api/ping. Geometry comes from the runtime's row/column
containers. No authored HTML or CSS is needed. view: main is evaluated once at
startup; restart after edits. Request-time interaction belongs in Endpoints.
