# web-site

This is the complete English document-native site: Home, About, New, the gated custom desk and slug detail each declare their own route. Parents compose structure and leaves contain Markdown. The original ink/wine/gold theme, typography, particles, breakpoints and form states remain declarative, with no author business JavaScript or hand-written CSS.

From the checkout root, use the current source CLI and prepare the engineering and Web plugins, then call the actual serving workflow:

```sh
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_engineering --effects process.exec --fs-root . --require-ok --json
target/debug/marqdo call skills/marqdo-dev/index.mq.md prepare_web --effects plugin --fs-root . --require-ok --json
target/debug/marqdo call examples/web-site/index.mq.md serve --effects plugin,fs.write,network.write --fs-root examples/web-site --require-ok --json
```

Open http://127.0.0.1:18081/ and use admin/demo only as local fixture credentials. Default SQLite belongs to the granted site root; restart preserves existing rows. The listener blocks in this process and the caller owns shutdown.

Read by responsibility:

- [Site entry](index.mq.md) — declare Home and compose actual data, form, session, gate and listener resources.
- [Layout entry](components/index.mq.md) — public rail and separate desk tree; terminal leaves live in components/content.
- [Article request resources](components/articles.mq.md) — current summary cards and actual captured-slug full Markdown detail.
- [Shared form](forms.mq.md) — actual fields/rules are in forms.tables.md; failed validation retains CSRF.
- [Theme entry](styles/index.mq.md) — retain original data and project only structural addresses plus wrapper geometry.
- [Native verification tool](verify.mq.md) — actually compare 412 original layout/motion declaration rows and return real CSS identities.

```sh
target/debug/marqdo call examples/web-site/verify.mq.md theme_values --effects plugin,fs.read --fs-root examples/web-site --require-ok --json
```

Theme output is generated at public/generated/theme.css; edit the adjacent source tables rather than that rebuildable artifact. Original Google Fonts links remain declared, with the original fallback families offline. Local browser acceptance does not claim external font-service delivery.

The public creation form remains public, while the custom desk is role-gated. Rendering the same form in Admin does not make its submission endpoint private. HTTP regression uses owned copied sites on Tree and Bytecode; real-browser cases verify styles, publishing, detail, session navigation, logout and narrow-screen order without modifying the example's existing database.

For the hands-on sequence see [DEMO.md](DEMO.md)。
