---
title: ext/security/oidc
description: OIDC attach helper for app handles (ADR 0007). Prefer declared auth metadata on Endpoints.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
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
    + `scopes`="openid profile email"
    + `callback_path`=None
    + `authorize_url`=None
    + `token_url`=None
    + `userinfo_url`=None

> _load
*> web_app_oidc app=`app` issuer=`issuer` client_id=`client_id` client_secret=`client_secret` redirect_uri=`redirect_uri` scopes=`scopes` callback_path=`callback_path` authorize_url=`authorize_url` token_url=`token_url` userinfo_url=`userinfo_url`*
