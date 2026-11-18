# Complete-site walkthrough

Start through the callable workflow in [README](README.md). For a fresh owned database the site seeds eight articles; an existing database retains its actual rows and must not be reset merely to reproduce that count.

| Step | Actual action | Evidence to inspect |
|---|---|---|
| 1 | Open Home | Original theme, Markdown introduction, badges, reading steps and summary cards |
| 2 | Open a card | Declared /post/{slug} route; full Markdown body rather than its summary |
| 3 | Read index and follow components/index | Actual parent/child tree, terminal Markdown and typed data/form/navigation resources |
| 4 | Submit New with an empty title | Actual server rule message; retry retains current CSRF |
| 5 | Submit title, slug, summary and body | New row appears on Home and in the gated desk without restart |
| 6 | Follow Login, use admin/demo, open Admin | Real route gate; separate desk navigation, no public rail, shared form and current cards |
| 7 | Return Home and log out | Session-specific links switch with the real identity; direct Admin access redirects again |
| 8 | Narrow the browser | The original 860px breakpoint puts main before the side rail without overflow |

Source responsibilities remain separate. Route metadata lives in each Document, form data is [forms.tables.md](forms.tables.md), current data behavior is [components/articles](components/articles.mq.md), SEO/fonts are [components/head](components/head.mq.md), and theme source plus explicit View mapping begins at [styles/index](styles/index.mq.md). No compose mutation chain or author page HTML is required.

The callable [verification tool](verify.mq.md) compares 412 original declaration rows and returns actual baseline/projected CSS hashes. It does not replace visual or HTTP acceptance. The real-browser suite deliberately aborts external font requests and verifies declared font families and local fallback rendering; it makes no claim about external service delivery.

Use native file discovery for author-asset checks, excluding the explicitly generated stylesheet directory:

```sh
target/debug/marqdo call lib/fs.mq.md find 'path="."' 'pattern="**/*.js"' 'exclude=["public/generated/**"]' --effects fs.read --fs-root examples/web-site --json
target/debug/marqdo call lib/fs.mq.md find 'path="."' 'pattern="**/*.css"' 'exclude=["public/generated/**"]' --effects fs.read --fs-root examples/web-site --json
```

Both author-asset lists are empty. This says nothing about generated runtime resources or native rendering internals, which are separate declared boundaries.
