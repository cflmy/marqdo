# ext/web — Document / Endpoint / Resource

Official dynamic site extension (**ADR 0007**).

| Doc | Link |
|-----|------|
| Artifact model | [`doc/design/ext-web-artifact.md`](../../doc/design/ext-web-artifact.md) |
| **Layout engineering** | [`doc/design/ext-web-layout-paradigm.md`](../../doc/design/ext-web-layout-paradigm.md) |
| skill=tools paradigm | [`doc/design/ext-skill-tools-paradigm.md`](../../doc/design/ext-skill-tools-paradigm.md) |

| Surface | Path |
|---------|------|
| EN root skill (agents first) | `ext/web/index.mq.md` |
| EN author facade | `import web:ext/web/web.mq.md` |
| ZH root / facade | `导入 网页:ext/web/网页.mq.md` |
| Layout tools | `ext/web/layout/index.mq.md` · `ext/web/布局/索引.mq.md` |

**Facade:** `page` · `route` · `serve` · `render` · `inspect` · `use`

**Layout law:** partition pages into files (nav / sidebar / main / footer / …) until each piece is simple; bind via `|type|slot|value|attrs|style|`. Do **not** build sites by HTML/CSS injection (`dom.raw` is escape hatch only).

**Related modules:** `ext/data` · `ext/security` · `ext/net`

```bash
bash scripts/build-web-plugin.sh
marqdo ext add web
marqdo call ext/web/layout/index.mq.md rules --json
marqdo run examples/marqdo-blog/serve.mq.md
```

Hard rules: no author `ensure_plugin` / `compose_*` / `configure` junk drawer; View schema only; declared vs system routes via `inspect`; no `MARQDO_EXT=repo/ext` self-install.
