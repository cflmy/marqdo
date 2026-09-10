// Package auth ports login/check/logout and credential checks from the Rust web plugin.
package auth

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/password"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

var (
	userKeys = []string{"用户名", "用户", "username", "user", "账号"}
	passKeys = []string{"密码", "password", "pass", "口令"}
	roleKeys = []string{"角色", "role", "roles"}
)

// New builds an auth bag: {"users":…,"session_ttl":N} (web_auth_new).
func New(users any, sessionTTL uint64) map[string]any {
	if sessionTTL == 0 {
		sessionTTL = 3600
	}
	return map[string]any{
		"users":       users,
		"session_ttl": float64(sessionTTL),
	}
}

// CheckCredentials validates username/password against a users table (GFM or row list).
// Missing role defaults to "admin". Returns username, role, ok.
func CheckCredentials(users any, username, pass string) (string, string, bool) {
	rowsAny := table.AsRows(users)
	rows, ok := rowsAny.([]any)
	if !ok {
		return "", "", false
	}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		u := pickCell(m, userKeys)
		p := pickCell(m, passKeys)
		if u == username && password.Verify(pass, p) {
			role := pickCell(m, roleKeys)
			if role == "" {
				role = "admin"
			}
			return u, strings.ToLower(role), true
		}
	}
	return "", "", false
}

func pickCell(m map[string]any, keys []string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return cellStr(v)
		}
	}
	return ""
}

func cellStr(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case bool:
		return strconv.FormatBool(t)
	case float64:
		if t == float64(int64(t)) {
			return strconv.FormatInt(int64(t), 10)
		}
		return strconv.FormatFloat(t, 'f', -1, 64)
	case int:
		return strconv.Itoa(t)
	case int64:
		return strconv.FormatInt(t, 10)
	default:
		return fmt.Sprint(t)
	}
}

// Login validates credentials and creates a session.
// Success: {"ok":true,"session_id":…,"username":…,"role":…}
// Failure: {"ok":false}
func Login(username, pass string, users any, sessionTTL uint64) map[string]any {
	u, role, ok := CheckCredentials(users, username, pass)
	if !ok {
		return map[string]any{"ok": false}
	}
	id := session.NewID(sessionTTL)
	session.SetValue(id, "username", u)
	session.SetValue(id, "role", role)
	return map[string]any{
		"ok":         true,
		"session_id": id,
		"username":   u,
		"role":       role,
	}
}

// Check returns session auth state (web_auth_check).
func Check(sessionID string) map[string]any {
	v, ok := session.GetValue(sessionID, "username")
	if !ok {
		return map[string]any{"ok": false}
	}
	u, _ := v.(string)
	if u == "" {
		return map[string]any{"ok": false}
	}
	role := "admin"
	if rv, rok := session.GetValue(sessionID, "role"); rok {
		if s, ok := rv.(string); ok && s != "" {
			role = s
		}
	}
	return map[string]any{"ok": true, "username": u, "role": role}
}

// Logout destroys the session (web_auth_logout).
func Logout(sessionID string) map[string]any {
	return map[string]any{"ok": session.DestroyID(sessionID)}
}
