package httpx

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/ratelimit"
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
	path      string
	roles     []string
	matchMode gateMatch
	onDeny    onDeny
	exclude   []string
}

type authConfig struct {
	users           any
	sessionTTL      uint64
	cookieSecure    bool
	admin           bool
	adminPrefix     string
	loginPath       string
	loginRedirect   string
	logoutRedirect  string
	gates           []gate
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
	if authBag, ok := appBag["auth"].(map[string]any); ok {
		cfg.users = authBag["users"]
		if n, ok := asUint64(authBag["session_ttl"]); ok && n > 0 {
			cfg.sessionTTL = n
		} else if n, ok := asUint64(authBag["ttl"]); ok && n > 0 {
			cfg.sessionTTL = n
		}
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
		if len(roles) == 0 {
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
			path:      path,
			roles:     roles,
			matchMode: matchMode,
			onDeny:    on,
			exclude:   parseExclude(m["exclude"], m["排除"]),
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
			if !session.RoleAllowed(role, g.roles) {
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
	if st.auth.users == nil {
		return
	}
	lp := st.auth.loginPath
	mux.HandleFunc("GET "+lp, st.handleLoginGet)
	mux.HandleFunc("POST "+lp, st.handleLoginPost)
	altLogin := strings.TrimRight(st.auth.adminPrefix, "/") + "/login"
	if altLogin != lp {
		mux.HandleFunc("GET "+altLogin, st.handleLoginGet)
		mux.HandleFunc("POST "+altLogin, st.handleLoginPost)
	}
	if st.auth.admin {
		logoutPath := strings.TrimRight(st.auth.adminPrefix, "/") + "/logout"
		mux.HandleFunc("GET "+logoutPath, st.handleLogout)
		adminHome := strings.TrimRight(st.auth.adminPrefix, "/")
		if adminHome == "" {
			adminHome = "/admin"
		}
		mux.HandleFunc("GET "+adminHome+"/{$}", st.handleAdminHome)
	}
}

func (st *state) handleLoginGet(w http.ResponseWriter, r *http.Request) {
	if st.auth.users == nil {
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
	if st.auth.users == nil {
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
	res := auth.Login(username, password, st.auth.users, st.auth.sessionTTL)
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
	dest := loginDest(r, st.auth.loginRedirect)
	w.Header().Add("Set-Cookie", session.IssueCookie(sessID))
	http.Redirect(w, r, dest, http.StatusSeeOther)
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
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Admin Login</title>
<style>
:root { --ink:#1c1917; --muted:#78716c; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; --err:#b91c1c; }
* { box-sizing:border-box; }
body { margin:0; min-height:100vh; display:grid; place-items:center; background:var(--paper); color:var(--ink); font-family:"IBM Plex Sans","Noto Sans SC",sans-serif; }
.login { background:rgba(255,255,255,.45); border:1px solid rgba(255,255,255,.5); border-radius:14px; padding:2rem 2.25rem; width:min(92vw,22rem); box-shadow:0 8px 24px rgba(253,189,219,.18); backdrop-filter:blur(14px); }
.login h1 { margin:0 0 .25rem; font-size:1.4rem; color:#a85878; }
.login .sub { color:#666; margin:0 0 1.25rem; font-size:.9rem; }
.login form { display:grid; gap:.9rem; }
.login label { display:grid; gap:.25rem; font-size:.9rem; }
.login input { padding:.55rem .65rem; border:1px solid rgba(253,189,219,.55); border-radius:999px; font:inherit; background:rgba(255,255,255,.7); }
.login button { background:linear-gradient(120deg,#f0a8c4,#fdbbdb); color:#a85878; border:0; padding:.6rem 1rem; border-radius:999px; cursor:pointer; font:inherit; font-weight:600; }
.login button:hover { filter:brightness(1.05); }
.flash.err { background:#fef2f2; color:#b91c1c; border:1px solid #fecaca; padding:.6rem .8rem; border-radius:6px; margin:0 0 .9rem; font-size:.9rem; }
</style>
</head>
<body style="background:#f8f9fa url('/static/img/bg-body-960.webp') center/cover fixed;">
<div class="login">
<h1>暗恋见君</h1>
<p class="sub">登录后发帖与管理。</p>
%s
<form method="post" action="%s">
%s%s
<label>Username<input name="username" autocomplete="username" required autofocus/></label>
<label>Password<input name="password" type="password" autocomplete="current-password" required/></label>
<button type="submit">Sign in</button>
</form>
</div>
</body></html>`, errHTML, esc(loginAction), csrfField, nextField)
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
