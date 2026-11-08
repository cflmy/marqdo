# ext/web — Document / Endpoint / Resource

Official dynamic site extension (**ADR 0007**).

| Doc | Link |
|-----|------|
| Artifact model | [`doc/design/ext-web-artifact.md`](../../doc/design/ext-web-artifact.md) |
| **Layout engineering** | [`doc/design/ext-web-layout-paradigm.md`](../../doc/design/ext-web-layout-paradigm.md) |
| skill=tools paradigm | [`doc/design/ext-skill-tools-paradigm.md`](../../doc/design/ext-skill-tools-paradigm.md) |

| Surface | Path |
|---------|------|
| EN root skill (zero-context site playbook) | `ext/web/index.mq.md` |
| EN author facade | `import web:ext/web/web.mq.md` |
| ZH root skill (zero-context playbook) | `ext/web/网页/索引.mq.md` |
| ZH author facade | `导入 网页:ext/web/网页.mq.md` |
| Modules | `page/` · `route/` · `component/` · `dom/` · `client/` · `layout/` |
| ZH modules | `网页/页面/` · `网页/路由/` · `网页/组件/` · … |
| Layout tools | `ext/web/layout/index.mq.md` · `ext/web/布局/索引.mq.md` |

**Facade:** `page` · `route` · `serve` · `render` · `inspect` · `use`

**Layout law:** partition pages into files (nav / sidebar / main / footer / …) until each piece is simple; bind via `|type|slot|value|attrs|style|`. Do **not** build sites by HTML/CSS injection (`dom.raw` is escape hatch only).

**Related modules:** `ext/data` · `ext/security` · `ext/net`

```bash
bash scripts/build-web-plugin.sh
marqdo ext add web
marqdo call ext/web/layout/index.mq.md rules --json
marqdo call ext/web/index.mq.md preview_markdown 'body="# Hi"' 'title="Demo"' --effects plugin --json
marqdo run examples/marqdo-blog/serve.mq.md
```

Hard rules: no author `ensure_plugin` / `compose_*` / `configure` junk drawer; View schema only; declared vs system routes via `inspect`; no `MARQDO_EXT=repo/ext` self-install.

The [executable handbook](index.mq.md) and [Chinese handbook](网页/索引.mq.md)
now include recursive layout constructors and tested source examples. Start with
[the reference site](../../examples/web-layout-handbook/index.mq.md): parents
compose one relationship, terminal content.md files stay ordinary Markdown.
The footer splits into brand/links, and links splits again into docs/community.
See [implementation and acceptance](../../doc/design/ext-web-handbook.md).

Composed Documents explicitly declare `view: main`; main returns nested View
rows for startup assembly. Ordinary Markdown Documents keep their existing path.
Use `web.inspect root=...` to scan disk declarations; empty fresh app state cannot
inspect another process. `call` previews, while `run serve.mq.md` starts a server.
