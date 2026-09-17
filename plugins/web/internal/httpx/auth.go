package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/password"
	"github.com/marqdo/marqdo/plugins/web/internal/ratelimit"
	"github.com/marqdo/marqdo/plugins/web/internal/rbac"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

type gateMatch int

const (
	gateMatchPrefix gateMatch = 0
	gateMatchExact  gateMatch = 1
)

type onDeny int

const (
	onDenyForbid   onDeny = 0
	onDenyRedirect onDeny = 1
)

type gate struct {
	path        string
	roles       []string
	permissions []string
	matchMode   gateMatch
	onDeny      onDeny
	exclude     []string
}

type authConfig struct {
	users          any
	sessionTTL     uint64
	cookieSecure   bool
	admin          bool
	adminPrefix    string
	loginPath      string
	loginRedirect  string
	logoutRedirect string
	gates          []gate
	rbac           bool
	register       bool
	registerPath   string
	defaultRole    string
}

func authConfigOf(appBag map[string]any) authConfig {
	cfg := authConfig{
		admin:       boolish(appBag["admin"]),
		adminPrefix: app.AdminPrefix(appBag),
		sessionTTL:  3600,
	}
	cfg.loginPath = strOpt(appBag, "login_path", "")
	if cfg.loginPath == "" {
		cfg.loginPath = cfg.adminPrefix + "/login"
	}
	cfg.loginRedirect = strOpt(appBag, "login_redirect", "")
	if cfg.loginRedirect == "" {
		cfg.loginRedirect = cfg.adminPrefix
	}
	cfg.logoutRedirect = strOpt(appBag, "logout_redirect", "")
	if cfg.logoutRedirect == "" {
		cfg.logoutRedirect = cfg.loginPath
	}
	if v, ok := appBag["cookie_secure"].(bool); ok {
		cfg.cookieSecure = v
	}
	cfg.registerPath = strOpt(appBag, "register_path", "")
	if authBag, ok := appBag["auth"].(map[string]any); ok {
		cfg.users = authBag["users"]
		if n, ok := asUint64(authBag["session_ttl"]); ok && n > 0 {
			cfg.sessionTTL = n
		} else if n, ok := asUint64(authBag["ttl"]); ok && n > 0 {
			cfg.sessionTTL = n
		}
		cfg.rbac = boolish(authBag["rbac"])
		cfg.register = boolish(authBag["register"])
		if p := strOpt(authBag, "register_path", ""); p != "" {
			cfg.registerPath = p
		}
		cfg.defaultRole = strOpt(authBag, "default_role", "")
		if cfg.defaultRole == "" {
			cfg.defaultRole = "member"
		}
	}
	if cfg.registerPath == "" {
		cfg.registerPath = "/register"
	}
	cfg.gates = parseGates(appBag["gates"])
	return cfg
}

func parseGates(v any) []gate {
	arr, ok := v.([]any)
	if !ok {
		return nil
	}
	out := make([]gate, 0, len(arr))
	for _, item := range arr {
		m, ok := item.(map[string]any)
		if !ok {
			continue
		}
		path := strings.TrimSpace(strOpt(m, "path", ""))
		if path == "" {
			path = strings.TrimSpace(strOpt(m, "路径", ""))
		}
		if path == "" {
			continue
		}
		starred := strings.HasSuffix(path, "*")
		if starred {
			path = strings.TrimSuffix(path, "*")
		}
		roles := parseGateRoles(m["roles"])
		if roles == nil {
			roles = parseGateRoles(m["角色"])
		}
		perms := parseGateRoles(m["permissions"])
		if perms == nil {
			perms = parseGateRoles(m["权限"])
		}
		if len(roles) == 0 && len(perms) == 0 {
			roles = []string{"admin"}
		}
		matchS := strings.ToLower(strings.TrimSpace(strOpt(m, "match", "")))
		if matchS == "" {
			matchS = strings.ToLower(strings.TrimSpace(strOpt(m, "匹配", "")))
		}
		matchMode := gateMatchPrefix
		switch matchS {
		case "exact", "精确":
			matchMode = gateMatchExact
		case "prefix", "前缀":
			matchMode = gateMatchPrefix
		default:
			if starred {
				matchMode = gateMatchPrefix
			}
		}
		denyS := strings.ToLower(strings.TrimSpace(strOpt(m, "on_deny", "")))
		if denyS == "" {
			denyS = strings.ToLower(strings.TrimSpace(strOpt(m, "拒绝", "")))
		}
		on := onDenyForbid
		switch denyS {
		case "redirect", "重定向", "login":
			on = onDenyRedirect
		case "forbid", "禁止", "403":
			on = onDenyForbid
		}
		out = append(out, gate{
			path:        path,
			roles:       roles,
			permissions: perms,
			matchMode:   matchMode,
			onDeny:      on,
			exclude:     parseExclude(m["exclude"], m["排除"]),
		})
	}
	return out
}

func parseGateRoles(v any) []string {
	switch t := v.(type) {
	case string:
		return session.ParseRolesCSV(t)
	case []any:
		var out []string
		for _, item := range t {
			if s, ok := item.(string); ok && strings.TrimSpace(s) != "" {
				out = append(out, strings.ToLower(strings.TrimSpace(s)))
			}
		}
		return out
	case []string:
		var out []string
		for _, s := range t {
			if strings.TrimSpace(s) != "" {
				out = append(out, strings.ToLower(strings.TrimSpace(s)))
			}
		}
		return out
	default:
		return nil
	}
}

func parseExclude(primary, alt any) []string {
	if primary == nil {
		primary = alt
	}
	switch t := primary.(type) {
	case string:
		var out []string
		for _, part := range strings.Split(t, ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				out = append(out, part)
			}
		}
		return out
	case []any:
		var out []string
		for _, item := range t {
			if s, ok := item.(string); ok {
				s = strings.TrimSpace(s)
				if s != "" {
					out = append(out, s)
				}
			}
		}
		return out
	default:
		return nil
	}
}

func gateMatches(path string, g gate) bool {
	pat := strings.TrimRight(strings.TrimSuffix(g.path, "*"), "/")
	if pat == "" {
		pat = "/"
	}
	switch g.matchMode {
	case gateMatchExact:
		return path == pat || path == g.path
	default:
		return path == pat || strings.HasPrefix(path, pat+"/")
	}
}

func pathExcluded(path string, exclude []string, loginPath string) bool {
	if path == loginPath || strings.HasPrefix(path, loginPath+"?") {
		return true
	}
	for _, ex := range exclude {
		ex = strings.TrimRight(strings.TrimSuffix(ex, "*"), "/")
		if ex == "" {
			continue
		}
		if path == ex || strings.HasPrefix(path, ex+"/") {
			return true
		}
	}
	return false
}

func withRBAC(next http.Handler, gates []gate, loginPath string) http.Handler {
	if len(gates) == 0 {
		return next
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		path := r.URL.Path
		cookie := r.Header.Get("Cookie")
		for _, g := range gates {
			if !gateMatches(path, g) {
				continue
			}
			if pathExcluded(path, g.exclude, loginPath) {
				break
			}
			role := session.RoleFromCookie(cookie)
			allowed := false
			if len(g.permissions) > 0 {
				held := session.PermissionsFromCookie(cookie)
				if len(held) == 0 && role != "visitor" {
					held = expandLegacyRole(role)
				}
				allowed = session.PermissionAllowed(held, g.permissions)
			} else {
				allowed = session.RoleAllowed(role, g.roles)
			}
			if !allowed {
				if role == "visitor" && g.onDeny == onDenyRedirect {
					nextQ := urlEncodePath(path)
					dest := loginPath
					if nextQ != "" && path != loginPath {
						dest = loginPath + "?next=" + nextQ
					}
					http.Redirect(w, r, dest, http.StatusSeeOther)
					return
				}
				w.Header().Set("Content-Type", "text/html; charset=utf-8")
				w.WriteHeader(http.StatusForbidden)
				_, _ = io.WriteString(w, fmt.Sprintf(
					"<!doctype html><html><body><h1>403 Forbidden</h1><p>Role `%s` cannot access `%s`.</p></body></html>",
					esc(role), esc(path)))
				return
			}
			break
		}
		next.ServeHTTP(w, r)
	})
}

func expandLegacyRole(role string) []string {
	switch strings.ToLower(strings.TrimSpace(role)) {
	case "superadmin", "admin":
		return []string{
			"comments:create", "comments:delete", "comments:delete_own",
			"desk:access", "roles:manage", "posts:edit", "users:manage",
		}
	case "member", "user":
		return []string{"comments:create", "comments:delete_own"}
	case "editor", "author":
		return []string{"comments:create", "comments:delete_own", "posts:edit", "desk:access"}
	default:
		return nil
	}
}

func urlEncodePath(path string) string {
	var out strings.Builder
	for _, b := range []byte(path) {
		switch {
		case b >= 'A' && b <= 'Z', b >= 'a' && b <= 'z', b >= '0' && b <= '9',
			b == '-', b == '_', b == '.', b == '~', b == '/':
			out.WriteByte(b)
		default:
			out.WriteString(fmt.Sprintf("%%%02X", b))
		}
	}
	return out.String()
}

func withNavAuth(page map[string]any, cookieHeader string) {
	sid, ok := session.IDFromCookie(cookieHeader)
	loggedIn := false
	var username string
	if ok && sid != "" {
		if v, has := session.GetValue(sid, "username"); has {
			if u, ok := v.(string); ok && u != "" {
				loggedIn = true
				username = u
			}
		}
	}
	page["_logged_in"] = loggedIn
	if loggedIn {
		page["_nav_user"] = username
	} else {
		delete(page, "_nav_user")
	}
}

func (st *state) mountAuthRoutes(mux *http.ServeMux) {
	if st.auth.users == nil && !(st.auth.rbac && st.auth.register) {
		return
	}
	if st.auth.users != nil || st.auth.rbac {
		lp := st.auth.loginPath
		mux.HandleFunc("GET "+lp, st.handleLoginGet)
		mux.HandleFunc("POST "+lp, st.handleLoginPost)
		altLogin := strings.TrimRight(st.auth.adminPrefix, "/") + "/login"
		if altLogin != lp {
			mux.HandleFunc("GET "+altLogin, st.handleLoginGet)
			mux.HandleFunc("POST "+altLogin, st.handleLoginPost)
		}
		logoutPath := strings.TrimRight(st.auth.adminPrefix, "/") + "/logout"
		mux.HandleFunc("GET "+logoutPath, st.handleLogout)
	}
	if st.auth.register && st.auth.rbac {
		rp := st.auth.registerPath
		mux.HandleFunc("GET "+rp, st.handleRegisterGet)
		mux.HandleFunc("POST "+rp, st.handleRegisterPost)
	}
	if st.auth.rbac && st.dbURL != "" {
		mux.HandleFunc("GET /_rbac/roles", st.handleRbacListRoles)
		mux.HandleFunc("GET /_rbac/permissions", st.handleRbacListPermissions)
		mux.HandleFunc("POST /_rbac/roles", st.handleRbacCreateRole)
		mux.HandleFunc("POST /_rbac/roles/{id}/permissions", st.handleRbacSetRolePerms)
		mux.HandleFunc("POST /_rbac/assign", st.handleRbacAssign)
		mux.HandleFunc("GET /_rbac/desk", st.handleRbacDesk)
	}
	if st.auth.admin {
		adminHome := strings.TrimRight(st.auth.adminPrefix, "/")
		if adminHome == "" {
			adminHome = "/admin"
		}
		mux.HandleFunc("GET "+adminHome+"/{$}", st.handleAdminHome)
	}
}

func (st *state) handleLoginGet(w http.ResponseWriter, r *http.Request) {
	if st.auth.users == nil && !st.auth.rbac {
		http.Redirect(w, r, st.auth.loginRedirect, http.StatusSeeOther)
		return
	}
	cookieHdr := r.Header.Get("Cookie")
	if session.RoleFromCookie(cookieHdr) != "visitor" {
		http.Redirect(w, r, loginDest(r, st.auth.loginRedirect), http.StatusSeeOther)
		return
	}
	_, csrf, setCookie := resolveSession(r)
	html := loginPageHTML(st.auth.loginPath, nil, csrf, r.URL.Query().Get("next"))
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	_, _ = io.WriteString(w, html)
}

func (st *state) handleLoginPost(w http.ResponseWriter, r *http.Request) {
	if st.auth.users == nil && !st.auth.rbac {
		http.Redirect(w, r, st.auth.loginRedirect, http.StatusSeeOther)
		return
	}
	if err := r.ParseForm(); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	cookieHdr := r.Header.Get("Cookie")
	sid, csrf, setCookie := resolveSession(r)
	_, hadCookie := session.IDFromCookie(cookieHdr)
	csrfToken := r.FormValue("_csrf")
	if csrfToken != "" || hadCookie {
		if !session.ValidateCSRF(sid, csrfToken) {
			w.Header().Set("Content-Type", "text/html; charset=utf-8")
			appendSetCookie(w, setCookie)
			_, _ = io.WriteString(w, "<p class=\"flash err\">Invalid or missing CSRF token. Refresh and try again.</p>")
			return
		}
	}
	username := r.FormValue("username")
	password := r.FormValue("password")
	ip := clientIP(r)
	if err := ratelimit.Check(ip, username); err != nil {
		msg := err.Error()
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		appendSetCookie(w, setCookie)
		_, _ = io.WriteString(w, loginPageHTML(st.auth.loginPath, &msg, csrf, r.FormValue("next")))
		return
	}
	res := st.tryLogin(username, password)
	if ok, _ := res["ok"].(bool); !ok {
		ratelimit.RecordFailure(ip, username)
		errMsg := "Invalid username or password."
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		appendSetCookie(w, setCookie)
		_, _ = io.WriteString(w, loginPageHTML(st.auth.loginPath, &errMsg, csrf, r.FormValue("next")))
		return
	}
	ratelimit.ClearSuccess(ip, username)
	sessID, _ := res["session_id"].(string)
	role, _ := res["role"].(string)
	st.attachLoginPermissions(sessID, username, role)
	dest := loginDest(r, st.auth.loginRedirect)
	w.Header().Add("Set-Cookie", session.IssueCookie(sessID))
	http.Redirect(w, r, dest, http.StatusSeeOther)
}

// tryLogin: GFM users first, then web_users when RBAC is on.
func (st *state) tryLogin(username, password string) map[string]any {
	if st.auth.users != nil {
		res := auth.Login(username, password, st.auth.users, st.auth.sessionTTL)
		if ok, _ := res["ok"].(bool); ok {
			u, _ := res["username"].(string)
			role, _ := res["role"].(string)
			st.seedGFMUserIntoRBAC(u, password, role)
			return res
		}
	}
	if st.auth.rbac && st.dbURL != "" {
		role, ok, err := rbac.Authenticate(st.dbURL, username, password, passwordVerify)
		if err == nil && ok {
			id := session.NewID(st.auth.sessionTTL)
			session.SetValue(id, "username", username)
			session.SetValue(id, "role", role)
			return map[string]any{
				"ok": true, "session_id": id, "username": username, "role": role,
			}
		}
	}
	return map[string]any{"ok": false}
}

func (st *state) attachLoginPermissions(sessID, username, role string) {
	var perms []string
	if st.dbURL != "" && st.auth.rbac {
		if dbPerms, err := rbac.PermissionsForUser(st.dbURL, username); err == nil && len(dbPerms) > 0 {
			perms = dbPerms
		}
	}
	if len(perms) == 0 {
		perms = rbac.ExpandRole(role)
	}
	if len(perms) == 0 {
		perms = expandLegacyRole(role)
	}
	auth.AttachPermissions(sessID, perms)
}

func (st *state) handleLogout(w http.ResponseWriter, r *http.Request) {
	if sid, ok := session.IDFromCookie(r.Header.Get("Cookie")); ok {
		auth.Logout(sid)
	}
	w.Header().Add("Set-Cookie", session.ClearCookie())
	http.Redirect(w, r, st.auth.logoutRedirect, http.StatusSeeOther)
}

func (st *state) handleAdminHome(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, "<!doctype html><html><body><h1>Admin</h1><p>Site administration.</p></body></html>")
}

func loginDest(r *http.Request, fallback string) string {
	next := strings.TrimSpace(r.FormValue("next"))
	if next == "" {
		next = strings.TrimSpace(r.URL.Query().Get("next"))
	}
	if next != "" && strings.HasPrefix(next, "/") && !strings.HasPrefix(next, "//") {
		return next
	}
	return fallback
}

func loginPageHTML(loginAction string, errMsg *string, csrf string, next string) string {
	errHTML := ""
	if errMsg != nil && *errMsg != "" {
		errHTML = fmt.Sprintf("<p class=\"flash err\">%s</p>", esc(*errMsg))
	}
	csrfField := ""
	if csrf != "" {
		csrfField = fmt.Sprintf("<input type=\"hidden\" name=\"_csrf\" value=\"%s\"/>", esc(csrf))
	}
	nextField := ""
	if next != "" && strings.HasPrefix(next, "/") && !strings.HasPrefix(next, "//") {
		nextField = fmt.Sprintf("<input type=\"hidden\" name=\"next\" value=\"%s\"/>", esc(next))
	}
	return fmt.Sprintf(`<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Admin Login</title>
<style>
:root { --ink:#0a0809; --paper:#f5e6c8; --gold:#d4af37; --wine:#6b1e3a; --err:#b91c1c; }
* { box-sizing:border-box; }
body { margin:0; min-height:100vh; display:grid; place-items:center; color:var(--paper); font-family:"Noto Sans SC","IBM Plex Sans",system-ui,sans-serif;
  background: radial-gradient(ellipse 90%% 55%% at 50%% -15%%, rgba(107,30,58,.5), transparent 58%%), linear-gradient(180deg,#0a0809,#120c10 45%%,#0a0809); }
.login { background:rgba(0,0,0,.45); border:1px solid rgba(212,175,55,.28); border-radius:12px; padding:2rem 2.25rem; width:min(92vw,22rem); box-shadow:0 24px 80px rgba(0,0,0,.55); backdrop-filter:blur(14px); }
.login h1 { margin:0 0 .25rem; font-size:1.55rem; color:var(--gold); letter-spacing:.04em; }
.login .sub { color:rgba(245,230,200,.72); margin:0 0 1.25rem; font-size:.9rem; line-height:1.55; }
.login .hint { color:rgba(245,230,200,.5); margin:1rem 0 0; font-size:.78rem; letter-spacing:.04em; }
.login form { display:grid; gap:.9rem; }
.login label { display:grid; gap:.25rem; font-size:.85rem; letter-spacing:.08em; }
.login input { padding:.6rem .75rem; border:1px solid rgba(212,175,55,.35); border-radius:8px; font:inherit; color:var(--paper); background:rgba(10,8,9,.7); }
.login input:focus { outline:none; border-color:var(--gold); box-shadow:0 0 0 2px rgba(212,175,55,.25); }
.login button { background:linear-gradient(135deg,#f5e6c8,var(--gold)); color:var(--ink); border:0; padding:.65rem 1rem; border-radius:999px; cursor:pointer; font:inherit; font-weight:600; letter-spacing:.1em; }
.login button:hover { filter:brightness(1.06); }
.flash.err { background:rgba(185,28,28,.15); color:#fecaca; border:1px solid rgba(185,28,28,.45); padding:.6rem .8rem; border-radius:8px; margin:0 0 .9rem; font-size:.9rem; }
a.back { color:var(--gold); text-decoration:none; font-size:.85rem; }
a.back:hover { color:#f5e6c8; }
</style>
</head>
<body>
<div class="login">
<h1>Marqdo Admin</h1>
<p class="sub">Sign in to open the table-declared desk.</p>
%s
<form method="post" action="%s">
%s%s
<label>Username<input name="username" autocomplete="username" required autofocus/></label>
<label>Password<input name="password" type="password" autocomplete="current-password" required/></label>
<button type="submit">Sign in</button>
</form>
<p class="hint">Demo · admin / demo</p>
<p style="margin:1.1rem 0 0"><a class="back" href="/">← Site</a></p>
</div>
</body></html>`, errHTML, esc(loginAction), csrfField, nextField)
}

func passwordVerify(pass, encoded string) bool {
	return password.Verify(pass, encoded)
}

func (st *state) seedGFMUserIntoRBAC(username, plaintext, role string) {
	if !st.auth.rbac || st.dbURL == "" || username == "" {
		return
	}
	exists, err := rbac.UserExists(st.dbURL, username)
	if err != nil || exists {
		return
	}
	hash, err := password.Hash(plaintext)
	if err != nil {
		return
	}
	target := "member"
	switch strings.ToLower(role) {
	case "admin", "superadmin":
		target = "superadmin"
	case "editor", "author":
		// custom roles may not exist yet; fall back to member + ExpandRole at session
		target = "member"
	}
	_ = rbac.EnsureUserWithRole(st.dbURL, username, hash, target)
}

func (st *state) handleRegisterGet(w http.ResponseWriter, r *http.Request) {
	_, csrf, setCookie := resolveSession(r)
	html := registerPageHTML(st.auth.registerPath, nil, csrf)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	_, _ = io.WriteString(w, html)
}

func (st *state) handleRegisterPost(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseForm(); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	sid, csrf, setCookie := resolveSession(r)
	csrfToken := r.FormValue("_csrf")
	if csrfToken != "" {
		if !session.ValidateCSRF(sid, csrfToken) {
			msg := "Invalid CSRF token."
			w.Header().Set("Content-Type", "text/html; charset=utf-8")
			appendSetCookie(w, setCookie)
			_, _ = io.WriteString(w, registerPageHTML(st.auth.registerPath, &msg, csrf))
			return
		}
	}
	username := strings.TrimSpace(r.FormValue("username"))
	pass := r.FormValue("password")
	if len(username) < 2 || len(pass) < 4 {
		msg := "Username (≥2) and password (≥4) required."
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		appendSetCookie(w, setCookie)
		_, _ = io.WriteString(w, registerPageHTML(st.auth.registerPath, &msg, csrf))
		return
	}
	hash, err := password.Hash(pass)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if err := rbac.RegisterUser(st.dbURL, username, hash, st.auth.defaultRole); err != nil {
		msg := err.Error()
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		appendSetCookie(w, setCookie)
		_, _ = io.WriteString(w, registerPageHTML(st.auth.registerPath, &msg, csrf))
		return
	}
	res := st.tryLogin(username, pass)
	if ok, _ := res["ok"].(bool); ok {
		sessID, _ := res["session_id"].(string)
		role, _ := res["role"].(string)
		st.attachLoginPermissions(sessID, username, role)
		w.Header().Add("Set-Cookie", session.IssueCookie(sessID))
		dest := st.auth.loginRedirect
		held := session.PermissionsFromSession(sessID)
		if !session.PermissionAllowed(held, []string{"desk:access"}) {
			dest = "/"
		}
		http.Redirect(w, r, dest, http.StatusSeeOther)
		return
	}
	http.Redirect(w, r, st.auth.loginPath, http.StatusSeeOther)
}

func registerPageHTML(action string, errMsg *string, csrf string) string {
	errHTML := ""
	if errMsg != nil && *errMsg != "" {
		errHTML = fmt.Sprintf("<p class=\"flash err\">%s</p>", esc(*errMsg))
	}
	csrfField := ""
	if csrf != "" {
		csrfField = fmt.Sprintf("<input type=\"hidden\" name=\"_csrf\" value=\"%s\"/>", esc(csrf))
	}
	return fmt.Sprintf(`<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Register</title>
<style>
:root { --ink:#0a0809; --paper:#f5e6c8; --gold:#d4af37; --err:#b91c1c; }
* { box-sizing:border-box; }
body { margin:0; min-height:100vh; display:grid; place-items:center; color:var(--paper); font-family:"Noto Sans SC","IBM Plex Sans",system-ui,sans-serif;
  background: radial-gradient(ellipse 90%% 55%% at 50%% -15%%, rgba(107,30,58,.5), transparent 58%%), linear-gradient(180deg,#0a0809,#120c10 45%%,#0a0809); }
.login { background:rgba(0,0,0,.45); border:1px solid rgba(212,175,55,.28); border-radius:12px; padding:2rem 2.25rem; width:min(92vw,22rem); }
.login h1 { margin:0 0 .25rem; font-size:1.55rem; color:var(--gold); }
.login .sub { color:rgba(245,230,200,.72); margin:0 0 1.25rem; font-size:.9rem; }
.login form { display:grid; gap:.9rem; }
.login label { display:grid; gap:.25rem; font-size:.85rem; }
.login input { padding:.6rem .75rem; border:1px solid rgba(212,175,55,.35); border-radius:8px; font:inherit; color:var(--paper); background:rgba(10,8,9,.7); }
.login button { background:linear-gradient(135deg,#f5e6c8,var(--gold)); color:var(--ink); border:0; padding:.65rem 1rem; border-radius:999px; cursor:pointer; font:inherit; font-weight:600; }
.flash.err { background:rgba(185,28,28,.15); color:#fecaca; border:1px solid rgba(185,28,28,.45); padding:.6rem .8rem; border-radius:8px; margin:0 0 .9rem; font-size:.9rem; }
a.back { color:var(--gold); text-decoration:none; font-size:.85rem; }
</style>
</head>
<body>
<div class="login">
<h1>Create account</h1>
<p class="sub">New members get the default role (comments).</p>
%s
<form method="post" action="%s">
%s
<label>Username<input name="username" autocomplete="username" required autofocus/></label>
<label>Password<input name="password" type="password" autocomplete="new-password" required/></label>
<button type="submit">Register</button>
</form>
<p style="margin:1.1rem 0 0"><a class="back" href="/login">← Sign in</a></p>
</div>
</body></html>`, errHTML, esc(action), csrfField)
}

func (st *state) requireRolesManage(w http.ResponseWriter, r *http.Request) bool {
	held := session.PermissionsFromCookie(r.Header.Get("Cookie"))
	if !session.PermissionAllowed(held, []string{"roles:manage"}) {
		http.Error(w, "forbidden", http.StatusForbidden)
		return false
	}
	return true
}

func (st *state) writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func (st *state) handleRbacListRoles(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	roles, err := rbac.ListRoles(st.dbURL)
	if err != nil {
		st.writeJSON(w, http.StatusInternalServerError, map[string]any{"ok": false, "error": err.Error()})
		return
	}
	st.writeJSON(w, http.StatusOK, map[string]any{"ok": true, "roles": roles})
}

func (st *state) handleRbacListPermissions(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	perms, err := rbac.ListPermissions(st.dbURL)
	if err != nil {
		st.writeJSON(w, http.StatusInternalServerError, map[string]any{"ok": false, "error": err.Error()})
		return
	}
	st.writeJSON(w, http.StatusOK, map[string]any{"ok": true, "permissions": perms})
}

func (st *state) handleRbacCreateRole(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	var body struct {
		Name string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil || strings.TrimSpace(body.Name) == "" {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": "name required"})
		return
	}
	id, err := rbac.CreateRole(st.dbURL, strings.TrimSpace(body.Name))
	if err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": err.Error()})
		return
	}
	st.writeJSON(w, http.StatusOK, map[string]any{"ok": true, "id": id, "name": body.Name})
}

func (st *state) handleRbacSetRolePerms(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	idStr := r.PathValue("id")
	roleID, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": "bad id"})
		return
	}
	var body struct {
		Permissions []string `json:"permissions"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": "bad json"})
		return
	}
	grantable := session.PermissionsFromCookie(r.Header.Get("Cookie"))
	if err := rbac.SetRolePermissions(st.dbURL, roleID, body.Permissions, grantable); err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": err.Error()})
		return
	}
	st.writeJSON(w, http.StatusOK, map[string]any{"ok": true})
}

func (st *state) handleRbacAssign(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	var body struct {
		Username string `json:"username"`
		Role     string `json:"role"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": "bad json"})
		return
	}
	if err := rbac.AssignRole(st.dbURL, body.Username, body.Role); err != nil {
		st.writeJSON(w, http.StatusBadRequest, map[string]any{"ok": false, "error": err.Error()})
		return
	}
	st.writeJSON(w, http.StatusOK, map[string]any{"ok": true})
}

func (st *state) handleRbacDesk(w http.ResponseWriter, r *http.Request) {
	if !st.requireRolesManage(w, r) {
		return
	}
	roles, _ := rbac.ListRoles(st.dbURL)
	perms, _ := rbac.ListPermissions(st.dbURL)
	var roleOpts, permChecks strings.Builder
	for _, role := range roles {
		name, _ := role["name"].(string)
		id := role["id"]
		sys, _ := role["is_system"].(bool)
		tag := ""
		if sys {
			tag = " (system)"
		}
		fmt.Fprintf(&roleOpts, `<option value="%v">%s%s</option>`, id, esc(name), tag)
	}
	for _, p := range perms {
		code, _ := p["code"].(string)
		desc, _ := p["description"].(string)
		fmt.Fprintf(&permChecks, `<label style="display:block;margin:.25rem 0"><input type="checkbox" name="perm" value="%s"/> %s — %s</label>`,
			esc(code), esc(code), esc(desc))
	}
	html := fmt.Sprintf(`<!DOCTYPE html>
<html lang="zh"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Roles · RBAC</title>
<style>
body{font-family:"Noto Sans SC",system-ui,sans-serif;max-width:42rem;margin:2rem auto;padding:0 1rem;line-height:1.5}
h1{font-size:1.4rem} section{margin:1.5rem 0;padding:1rem;border:1px solid #ddd;border-radius:8px}
button{padding:.5rem 1rem;cursor:pointer} input,select{padding:.4rem;margin:.25rem 0;width:100%%;max-width:20rem}
#msg{min-height:1.2rem;color:#b91c1c}
</style></head><body>
<h1>角色与权限</h1>
<p>需 <code>roles:manage</code>。系统角色权限不可在此改写；可新建角色并勾选权限，或给用户赋角色。</p>
<p id="msg"></p>
<section>
<h2>新建角色</h2>
<input id="new-role" placeholder="角色名"/><button type="button" id="btn-create">创建</button>
</section>
<section>
<h2>设置角色权限</h2>
<select id="role-sel">%s</select>
<div>%s</div>
<button type="button" id="btn-set">保存权限</button>
</section>
<section>
<h2>赋角色给用户</h2>
<input id="assign-user" placeholder="用户名"/>
<input id="assign-role" placeholder="角色名（如 member）"/>
<button type="button" id="btn-assign">赋权</button>
</section>
<p><a href="/desk">← 后台</a></p>
<script>
const msg=document.getElementById('msg');
async function j(url,opt){const r=await fetch(url,Object.assign({credentials:'same-origin',headers:{'Content-Type':'application/json'}},opt||{}));const d=await r.json();if(!d.ok)throw new Error(d.error||r.status);return d;}
document.getElementById('btn-create').onclick=async()=>{try{const name=document.getElementById('new-role').value.trim();await j('/_rbac/roles',{method:'POST',body:JSON.stringify({name})});msg.textContent='已创建 '+name;location.reload();}catch(e){msg.textContent=e.message;}};
document.getElementById('btn-set').onclick=async()=>{try{const id=document.getElementById('role-sel').value;const permissions=[...document.querySelectorAll('input[name=perm]:checked')].map(x=>x.value);await j('/_rbac/roles/'+id+'/permissions',{method:'POST',body:JSON.stringify({permissions})});msg.textContent='权限已保存';}catch(e){msg.textContent=e.message;}};
document.getElementById('btn-assign').onclick=async()=>{try{const username=document.getElementById('assign-user').value.trim();const role=document.getElementById('assign-role').value.trim();await j('/_rbac/assign',{method:'POST',body:JSON.stringify({username,role})});msg.textContent='已赋 '+role+' → '+username;}catch(e){msg.textContent=e.message;}};
</script>
</body></html>`, roleOpts.String(), permChecks.String())
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func clientIP(r *http.Request) string {
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		if i := strings.Index(xff, ","); i >= 0 {
			return strings.TrimSpace(xff[:i])
		}
		return strings.TrimSpace(xff)
	}
	if xri := r.Header.Get("X-Real-IP"); xri != "" {
		return strings.TrimSpace(xri)
	}
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		return r.RemoteAddr
	}
	return host
}

func asUint64(v any) (uint64, bool) {
	switch t := v.(type) {
	case float64:
		if t > 0 {
			return uint64(t), true
		}
	case int:
		if t > 0 {
			return uint64(t), true
		}
	case int64:
		if t > 0 {
			return uint64(t), true
		}
	case uint64:
		if t > 0 {
			return t, true
		}
	case string:
		var n uint64
		if _, err := fmt.Sscanf(strings.TrimSpace(t), "%d", &n); err == nil && n > 0 {
			return n, true
		}
	}
	return 0, false
}
