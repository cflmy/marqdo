package auth_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/password"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func usersEN() any {
	return []any{
		map[string]any{"username": "admin", "password": "secret"},
		map[string]any{"用户": "站长", "密码": "pw123"},
	}
}

func usersZHTable() any {
	return map[string]any{
		"行":  []any{"1"},
		"用户": []any{"admin"},
		"密码": []any{"secret"},
	}
}

func TestLoginCheckLogout(t *testing.T) {
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)
	r := auth.Login("admin", "secret", usersEN(), 0)
	if r["ok"] != true {
		t.Fatalf("login: %v", r)
	}
	sid, _ := r["session_id"].(string)
	if sid == "" || r["username"] != "admin" || r["role"] != "admin" {
		t.Fatalf("login fields: %v", r)
	}
	check := auth.Check(sid)
	if check["ok"] != true || check["username"] != "admin" {
		t.Fatalf("check: %v", check)
	}
	bad := auth.Login("admin", "wrong", usersEN(), 0)
	if bad["ok"] != false {
		t.Fatalf("bad login: %v", bad)
	}
	out := auth.Logout(sid)
	if out["ok"] != true {
		t.Fatalf("logout: %v", out)
	}
	if auth.Check(sid)["ok"] != false {
		t.Fatal("after logout still ok")
	}
}

func TestGFMUsersTable(t *testing.T) {
	session.Reset(3600)
	r := auth.Login("admin", "secret", usersZHTable(), 120)
	if r["ok"] != true {
		t.Fatalf("gfm login: %v", r)
	}
}

func TestHashedPasswordLogin(t *testing.T) {
	session.Reset(3600)
	hash, err := password.Hash("secret")
	if err != nil {
		t.Fatal(err)
	}
	users := []any{map[string]any{"username": "admin", "password": hash}}
	r := auth.Login("admin", "secret", users, 0)
	if r["ok"] != true {
		t.Fatalf("hashed login: %v", r)
	}
}

func TestAuthNew(t *testing.T) {
	bag := auth.New(usersEN(), 120)
	if bag["session_ttl"] != float64(120) {
		t.Fatalf("%v", bag)
	}
}
