---
title: ext/security/oidc
description: OIDC attach helper for app handles (ADR 0007). Prefer declared auth metadata on Endpoints.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/security: native web plugin not found
  > sys.exit code=1
****

## attach
    + `app`
    + `issuer`=None
    + `client_id`
    + `client_secret`
    + `redirect_uri`
    + `redirect_origins`=None
    + `scopes`="openid profile email"
    + `callback_path`=None
    + `authorize_url`=None
    + `token_url`=None
    + `userinfo_url`=None

`redirect_uri` may be an absolute URL (`https://qdqc.com/oidc/callback`) or a path
(`/oidc/callback`). Path mode resolves the absolute callback from the login request
Host (`X-Forwarded-Proto` / `X-Forwarded-Host` when behind a reverse proxy) and stores
it in the session so token exchange uses the same string.

Optional `redirect_origins` is a comma-separated allowlist
(`https://qdqc.com,https://www.qdqc.com`). When set, path-mode origins must match.
The IdP must still register every absolute callback URL that will be used.

> _load
*> web_app_oidc app=`app` issuer=`issuer` client_id=`client_id` client_secret=`client_secret` redirect_uri=`redirect_uri` redirect_origins=`redirect_origins` scopes=`scopes` callback_path=`callback_path` authorize_url=`authorize_url` token_url=`token_url` userinfo_url=`userinfo_url`*
