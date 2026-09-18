package httpx

import (
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/oidc"
	"github.com/marqdo/marqdo/plugins/web/internal/rbac"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

const oidcPasswordSentinel = "!oidc!"

var (
	oidcHTTPOnce sync.Once
	oidcHTTP     *http.Client
)

func oidcClient() *http.Client {
	oidcHTTPOnce.Do(func() {
		oidcHTTP = &http.Client{Timeout: 20 * time.Second}
	})
	return oidcHTTP
}

func (st *state) oidcEnabled() bool {
	return st.oidc.Enabled()
}

func (st *state) ensureOIDCEndpoints() error {
	if !st.oidc.Enabled() {
		return fmt.Errorf("oidc not configured")
	}
	return st.oidc.ResolveEndpoints(oidcClient())
}

func (st *state) isOIDCEntryPath(path string) bool {
	if !st.oidcEnabled() || path == "" {
		return false
	}
	if path == st.auth.loginPath || path == st.auth.registerPath {
		return true
	}
	alt := strings.TrimRight(st.auth.adminPrefix, "/") + "/login"
	if path == alt || path == "/desk/login" || path == "/admin/login" {
		return true
	}
	return false
}

// oidcFallbackNext picks where to land after IdP login when ?next= is absent.
// Admin entry pages must not fall back to the public homepage.
func (st *state) oidcFallbackNext(r *http.Request) string {
	path := ""
	if r != nil {
		path = r.URL.Path
	}
	switch path {
	case "/desk/login", "/admin/login", "/oidc/login":
		return "/admin"
	case "/desk", "/admin":
		return "/admin"
	}
	if strings.HasPrefix(path, "/desk/") || strings.HasPrefix(path, "/admin/") {
		return "/admin"
	}
	if st.auth.loginRedirect != "" {
		return st.auth.loginRedirect
	}
	return "/"
}

func (st *state) mountOIDCRoutes(mux *http.ServeMux) {
	if !st.oidcEnabled() {
		return
	}
	cb := st.oidc.Callback()
	mux.HandleFunc("GET "+cb, st.handleOIDCCallback)
	mux.HandleFunc("GET /oidc/login", st.handleOIDCStart)
	mux.HandleFunc("GET /oidc/register", st.handleOIDCStart)
}

func (st *state) handleOIDCStart(w http.ResponseWriter, r *http.Request) {
	if !st.oidcEnabled() {
		http.NotFound(w, r)
		return
	}
	// Already signed in → skip IdP round-trip when destination is allowed.
	cookie := r.Header.Get("Cookie")
	if session.RoleFromCookie(cookie) != "visitor" {
		next := loginDest(r, st.oidcFallbackNext(r))
		needsDesk := strings.HasPrefix(next, "/admin") || strings.HasPrefix(next, "/desk")
		held := session.PermissionsFromCookie(cookie)
		if needsDesk && !session.PermissionAllowed(held, []string{"desk:access"}) {
			http.Redirect(w, r, "/", http.StatusSeeOther)
			return
		}
		http.Redirect(w, r, next, http.StatusSeeOther)
		return
	}
	if err := st.ensureOIDCEndpoints(); err != nil {
		http.Error(w, "oidc discovery failed: "+err.Error(), http.StatusBadGateway)
		return
	}
	sid, _, setCookie := resolveSession(r)
	state, err := oidc.RandomURLString(24)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	verifier, err := oidc.RandomURLString(32)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	session.SetValue(sid, "oidc_state", state)
	session.SetValue(sid, "oidc_verifier", verifier)
	next := loginDest(r, st.oidcFallbackNext(r))
	session.SetValue(sid, "oidc_next", next)
	loc, err := st.oidc.AuthorizeRedirect(state, oidc.PKCEChallengeS256(verifier))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	appendSetCookie(w, setCookie)
	w.Header().Add("Set-Cookie", session.IssueCookie(sid))
	http.Redirect(w, r, loc, http.StatusFound)
}

func (st *state) handleOIDCCallback(w http.ResponseWriter, r *http.Request) {
	if !st.oidcEnabled() {
		http.NotFound(w, r)
		return
	}
	if err := st.ensureOIDCEndpoints(); err != nil {
		http.Error(w, "oidc discovery failed: "+err.Error(), http.StatusBadGateway)
		return
	}
	q := r.URL.Query()
	if errCode := q.Get("error"); errCode != "" {
		msg := errCode
		if d := q.Get("error_description"); d != "" {
			msg = errCode + ": " + d
		}
		http.Error(w, "oidc: "+msg, http.StatusBadRequest)
		return
	}
	code := q.Get("code")
	state := q.Get("state")
	if code == "" || state == "" {
		http.Error(w, "oidc: missing code or state", http.StatusBadRequest)
		return
	}
	sid, _, setCookie := resolveSession(r)
	want, _ := session.GetValue(sid, "oidc_state")
	wantS, _ := want.(string)
	if wantS == "" || wantS != state {
		http.Error(w, "oidc: state mismatch", http.StatusBadRequest)
		return
	}
	verifier, _ := session.GetValue(sid, "oidc_verifier")
	verifierS, _ := verifier.(string)
	nextVal, _ := session.GetValue(sid, "oidc_next")
	next, _ := nextVal.(string)
	if next == "" {
		next = st.auth.loginRedirect
	}
	session.DelKey(sid, "oidc_state")
	session.DelKey(sid, "oidc_verifier")
	session.DelKey(sid, "oidc_next")

	tok, err := st.oidc.ExchangeCode(oidcClient(), code, verifierS)
	if err != nil {
		http.Error(w, "oidc token exchange: "+err.Error(), http.StatusBadGateway)
		return
	}
	info, err := st.oidc.FetchUserInfo(oidcClient(), tok.AccessToken)
	if err != nil {
		http.Error(w, "oidc userinfo: "+err.Error(), http.StatusBadGateway)
		return
	}
	username := info.LocalUsername()
	if username == "" {
		http.Error(w, "oidc: empty username", http.StatusBadGateway)
		return
	}
	role := info.LocalRole(st.auth.defaultRole)
	if role == "admin" {
		// System RBAC seed uses superadmin; keep session role "admin" for ExpandRole.
		if st.dbURL != "" && st.auth.rbac {
			_ = rbac.EnsureUserWithRole(st.dbURL, username, oidcPasswordSentinel, "superadmin")
		}
	} else if st.dbURL != "" && st.auth.rbac {
		_ = rbac.EnsureUserWithRole(st.dbURL, username, oidcPasswordSentinel, role)
	}
	newID := session.NewID(st.auth.sessionTTL)
	session.SetValue(newID, "username", username)
	session.SetValue(newID, "role", role)
	session.SetValue(newID, "oidc_sub", info.Sub)
	if info.Email != "" {
		session.SetValue(newID, "email", info.Email)
	}
	if info.AdminRole != "" {
		session.SetValue(newID, "admin_role", info.AdminRole)
	}
	st.attachOIDCPermissions(newID, username, role)

	held := session.PermissionsFromSession(newID)
	if (strings.HasPrefix(next, "/desk") || strings.HasPrefix(next, "/admin")) &&
		!session.PermissionAllowed(held, []string{"desk:access"}) {
		next = "/"
	}
	appendSetCookie(w, setCookie)
	w.Header().Add("Set-Cookie", session.IssueCookie(newID))
	http.Redirect(w, r, next, http.StatusSeeOther)
}

func (st *state) attachOIDCPermissions(sessID, username, role string) {
	// Prefer IdP-derived role expansion so platform admins always get desk access
	// even before local RBAC rows catch up.
	perms := rbac.ExpandRole(role)
	if len(perms) == 0 {
		perms = expandLegacyRole(role)
	}
	if st.dbURL != "" && st.auth.rbac {
		if dbPerms, err := rbac.PermissionsForUser(st.dbURL, username); err == nil && len(dbPerms) > 0 {
			// Union IdP role perms with any local grants.
			seen := map[string]bool{}
			var out []string
			for _, p := range append(perms, dbPerms...) {
				if p == "" || seen[p] {
					continue
				}
				seen[p] = true
				out = append(out, p)
			}
			perms = out
		}
	}
	auth.AttachPermissions(sessID, perms)
}
