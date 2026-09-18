package httpx

import (
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func isLoginAuthPage(page any) bool {
	m, ok := page.(map[string]any)
	if !ok || m == nil {
		return false
	}
	af, ok := m["auth_form"].(map[string]any)
	if !ok || af == nil {
		return false
	}
	kind := strings.ToLower(strings.TrimSpace(fmt.Sprint(af["kind"])))
	action := strings.ToLower(strings.TrimSpace(fmt.Sprint(af["action"])))
	if kind == "register" || strings.Contains(action, "register") {
		return false
	}
	return true
}

func (st *state) pageForAuthPath(path string) map[string]any {
	if path == "" || st.routes == nil {
		return nil
	}
	if p, ok := st.routes[path].(map[string]any); ok && p != nil {
		return p
	}
	alt := strings.TrimRight(st.auth.adminPrefix, "/") + "/login"
	if path == alt {
		if p, ok := st.routes["/desk/login"].(map[string]any); ok {
			return p
		}
	}
	return nil
}

func localizeAuthErr(msg string) string {
	s := strings.TrimSpace(msg)
	lower := strings.ToLower(s)
	switch {
	case s == "":
		return "操作失败，请重试。"
	case strings.Contains(s, "Too many failed"):
		return "尝试次数过多，请约 15 分钟后再试。"
	case strings.Contains(s, "Invalid username or password"):
		return "用户名或密码错误。"
	case strings.Contains(s, "Invalid or missing CSRF"), strings.Contains(s, "Invalid CSRF"):
		return "登录令牌失效，请刷新页面后重试。"
	case strings.Contains(s, "Username") && strings.Contains(lower, "password"):
		return "请填写有效的用户名（≥2）与密码（≥4）。"
	case strings.Contains(lower, "already"), strings.Contains(lower, "exists"), strings.Contains(s, "存在"):
		return "该用户名已被注册，请换一个。"
	default:
		return s
	}
}

func safeAuthNext(raw string) string {
	next := strings.TrimSpace(raw)
	if next != "" && strings.HasPrefix(next, "/") && !strings.HasPrefix(next, "//") {
		return next
	}
	return ""
}

func (st *state) writeAuthFailure(w http.ResponseWriter, r *http.Request, msg string) {
	path := r.URL.Path
	page := st.pageForAuthPath(path)
	if page == nil && path == strings.TrimRight(st.auth.adminPrefix, "/")+"/login" {
		page = st.pageForAuthPath("/desk/login")
	}
	if page != nil {
		p := cloneMap(page)
		p["_flash_err"] = localizeAuthErr(msg)
		if next := safeAuthNext(r.FormValue("next")); next != "" {
			p["_auth_next"] = next
		} else if next := safeAuthNext(r.URL.Query().Get("next")); next != "" {
			p["_auth_next"] = next
		}
		st.writePage(w, r, p)
		return
	}
	_, csrf, setCookie := resolveSession(r)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	loc := localizeAuthErr(msg)
	_, _ = io.WriteString(w, loginPageHTML(st.auth.loginPath, &loc, csrf, r.FormValue("next")))
}

func (st *state) writeRegisterFailure(w http.ResponseWriter, r *http.Request, msg string) {
	path := r.URL.Path
	page := st.pageForAuthPath(path)
	if page == nil {
		page = st.pageForAuthPath(st.auth.registerPath)
	}
	if page != nil {
		p := cloneMap(page)
		p["_flash_err"] = localizeAuthErr(msg)
		st.writePage(w, r, p)
		return
	}
	_, csrf, setCookie := resolveSession(r)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	loc := localizeAuthErr(msg)
	_, _ = io.WriteString(w, registerPageHTML(st.auth.registerPath, &loc, csrf))
}

func (st *state) mountExtraLoginPosts(mux *http.ServeMux, already map[string]bool) {
	for path, pageV := range st.routes {
		if already[path] {
			continue
		}
		if isLoginAuthPage(pageV) {
			mux.HandleFunc("POST "+path, st.handleLoginPost)
			already[path] = true
		}
	}
}

func (st *state) authEntryRedirect(w http.ResponseWriter, r *http.Request, page map[string]any) bool {
	if page == nil || r == nil {
		return false
	}
	path := r.URL.Path
	isEntry := false
	if path == st.auth.loginPath || path == st.auth.registerPath {
		isEntry = true
	}
	alt := strings.TrimRight(st.auth.adminPrefix, "/") + "/login"
	if path == alt {
		isEntry = true
	}
	if _, ok := page["auth_form"]; ok {
		isEntry = true
	}
	if !isEntry {
		return false
	}
	if session.RoleFromCookie(r.Header.Get("Cookie")) == "visitor" {
		return false
	}
	fallback := st.auth.loginRedirect
	if af, ok := page["auth_form"].(map[string]any); ok {
		if n, ok := af["next"].(string); ok {
			if n = strings.TrimSpace(n); n != "" {
				fallback = n
			}
		}
	}
	if path == st.auth.registerPath {
		fallback = st.auth.loginRedirect
	}
	http.Redirect(w, r, loginDest(r, fallback), http.StatusSeeOther)
	return true
}
