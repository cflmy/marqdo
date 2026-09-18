package oidc_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/oidc"
)

func TestUserInfoLocalRole(t *testing.T) {
	u := &oidc.UserInfo{PreferredUsername: "alice", IsAdmin: false}
	if u.LocalRole("member") != "member" {
		t.Fatal("expected member")
	}
	u.IsAdmin = true
	if u.LocalRole("member") != "admin" {
		t.Fatal("expected admin from is_admin")
	}
	u.IsAdmin = false
	u.AdminRole = "super_admin"
	if u.LocalRole("member") != "admin" {
		t.Fatal("expected admin from admin_role")
	}
	if u.LocalUsername() != "alice" {
		t.Fatalf("username=%s", u.LocalUsername())
	}
}

func TestPKCEAndParse(t *testing.T) {
	v, err := oidc.RandomURLString(32)
	if err != nil || v == "" {
		t.Fatal(err)
	}
	ch := oidc.PKCEChallengeS256(v)
	if len(ch) < 20 {
		t.Fatalf("challenge=%q", ch)
	}
	c := oidc.ParseConfig(map[string]any{
		"发行方":   "https://id.example.com",
		"客户端编号": "app_x",
		"客户端密钥": "secret",
		"回调":    "http://localhost:18085/oidc/callback",
	})
	if !c.Enabled() {
		t.Fatal("expected enabled")
	}
	if c.Callback() != "/oidc/callback" {
		t.Fatalf("callback=%s", c.Callback())
	}
}
