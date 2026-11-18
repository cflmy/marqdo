---
markup: occlusion
title: ext/security/oidc
description: OIDC attach helper for app handles (ADR 0007). Prefer declared auth metadata on Endpoints.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

OIDC attaches provider configuration to an existing web application. Read [the security entry](index.mq.md) for the ownership boundaries before configuring a live identity provider. Registered callback URLs, origin checks and credentials require real deployment configuration; constructing a record alone does not validate a login flow. Keep provider failures distinct from permission decisions and exercise callbacks with controlled fixtures.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/security: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

## attach

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Existing web application resource |
| issuer | any | None | OIDC issuer URL; None requires explicit provider endpoint configuration |
| client_id | any | required | Client identifier registered with the identity provider |
| client_secret | any | required | Client credential; supply through trusted runtime configuration |
| redirect_uri | any | required | Registered absolute callback URL or callback path |
| redirect_origins | any | None | Comma-separated callback origin allowlist when using a path callback |
| scopes | any | "openid profile email" | Space-separated identity-provider scopes |
| callback_path | any | None | Explicit callback route override or None for the implementation default |
| authorize_url | any | None | Authorization endpoint override |
| token_url | any | None | Token-exchange endpoint override |
| userinfo_url | any | None | User-info endpoint override |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | (`/oidc/callback`). Path mode resolves the absolute callback from the login request Host (`X-Forwarded-Proto` / `X-Forwarded-Host` when behind a reverse proxy) and stores it in the session so token exchange uses the same string. Optional `redirect_origins` is a comma-separated allowlist (`https://qdqc.com,https://www.qdqc.com`). When set, path-mode origins must match. The IdP must still register every absolute callback URL that will be used. |

Use `app`, `issuer`=None, `client_id`, `client_secret`, `redirect_uri`, `redirect_origins`=None, `scopes`="openid profile email", `callback_path`=None, `authorize_url`=None, `token_url`=None, `userinfo_url`=None as the inputs for this operation.

`redirect_uri` may be an absolute URL (`https://qdqc.com/oidc/callback`) or a path
(`/oidc/callback`). Path mode resolves the absolute callback from the login request
Host (`X-Forwarded-Proto` / `X-Forwarded-Host` when behind a reverse proxy) and stores
it in the session so token exchange uses the same string.

Optional `redirect_origins` is a comma-separated allowlist
(`https://qdqc.com,https://www.qdqc.com`). When set, path-mode origins must match.
The IdP must still register every absolute callback URL that will be used.

We **[Load the native plugin required by this operation](_load())**.
Return *[(/oidc/callback)](web_app_oidc(app=`app`, issuer=`issuer`, client_id=`client_id`, client_secret=`client_secret`, redirect_uri=`redirect_uri`, redirect_origins=`redirect_origins`, scopes=`scopes`, callback_path=`callback_path`, authorize_url=`authorize_url`, token_url=`token_url`, userinfo_url=`userinfo_url`))*.
