# ext/web — Document / Endpoint / Resource

Official dynamic site extension (**ADR 0007**). Design: [`doc/design/ext-web-artifact.md`](../../doc/design/ext-web-artifact.md).

| Surface | Import |
|---------|--------|
| English | `import web:ext/web/web.mq.md` |
| Chinese | `导入 网页:ext/web/网页.mq.md` |

**Facade:** `page` · `route` · `serve` · `render` · `inspect` · `use`

**Related modules:** `ext/data` (db/cache/storage) · `ext/security` (auth/oidc/rbac) · `ext/net` (http/websocket/url)

```bash
bash scripts/build-web-plugin.sh
marqdo ext add web   # optional install
marqdo run examples/marqdo-blog/serve.mq.md
```

Hard rules: no author `ensure_plugin` / `compose_*` / `configure` junk drawer; View schema `|type|slot|value|attrs|style|` only; declared vs system routes via `inspect`.
