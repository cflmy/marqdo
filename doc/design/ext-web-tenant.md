# ext/web multi-tenant (path / subdomain / header)

| | |
|---|---|
| 状态 | **Accepted · MVP** |
| 日期 | 2026-09-17 |
| 相关 | [ext-web-rbac.md](ext-web-rbac.md) · full-ambition plan |

## Author API

```markdown
**应用 = > `应用`.tenant mode="path" param="t" column="tenant_id" default_scope=False**
```

JSON route table may set `tenant_scope` / `租户作用域` = true so `GET /api/items?t=acme` only returns rows with `tenant_id=acme`.

## Runtime

1. `withTenant` resolves id into request context.
2. Scoped JSON selects `AND tenant_id=?`; missing tenant → 400.
3. `default_scope=True` also stamps form POST bodies.

Demo: `examples/web-tenant-saas/`.
