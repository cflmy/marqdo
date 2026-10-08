package oidc_test

import (
	"crypto/tls"
	"net/http"
	"net/url"
	"strings"
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
	if c.IsPathRedirect() {
		t.Fatal("absolute redirect should not be path mode")
	}
}

func TestPathRedirectParseAndCallback(t *testing.T) {
	c := oidc.ParseConfig(map[string]any{
		"issuer":           "https://id.example.com",
		"client_id":        "app_x",
		"client_secret":    "secret",
		"redirect_uri":     "/oidc/callback",
		"redirect_origins": "https://qdqc.com,https://www.qdqc.com",
	})
	if !c.Enabled() {
		t.Fatal("expected enabled")
	}
	if !c.IsPathRedirect() {
		t.Fatal("expected path mode")
	}
	if c.Callback() != "/oidc/callback" {
		t.Fatalf("callback=%s", c.Callback())
	}
	origins := c.AllowedOrigins()
	if len(origins) != 2 || origins[0] != "https://qdqc.com" {
		t.Fatalf("origins=%v", origins)
	}
}

func TestResolveRedirectURIAbsolute(t *testing.T) {
	c := oidc.Config{RedirectURI: "https://qdqc.com/oidc/callback"}
	r, _ := http.NewRequest(http.MethodGet, "http://www.qdqc.com/oidc/login", nil)
	r.Host = "www.qdqc.com"
	got, err := c.ResolveRedirectURI(r)
	if err != nil {
		t.Fatal(err)
	}
	if got != "https://qdqc.com/oidc/callback" {
		t.Fatalf("got=%s", got)
	}
}

func TestResolveRedirectURIPathFromHost(t *testing.T) {
	c := oidc.Config{RedirectURI: "/oidc/callback"}
	r, _ := http.NewRequest(http.MethodGet, "http://qdqc.com/oidc/login", nil)
	r.Host = "qdqc.com"
	got, err := c.ResolveRedirectURI(r)
	if err != nil {
		t.Fatal(err)
	}
	if got != "http://qdqc.com/oidc/callback" {
		t.Fatalf("got=%s", got)
	}
}

func TestResolveRedirectURIPathTLS(t *testing.T) {
	c := oidc.Config{RedirectURI: "/oidc/callback"}
	r, _ := http.NewRequest(http.MethodGet, "https://qdqc.com/oidc/login", nil)
	r.Host = "qdqc.com"
	r.TLS = &tls.ConnectionState{}
	got, err := c.ResolveRedirectURI(r)
	if err != nil {
		t.Fatal(err)
	}
	if got != "https://qdqc.com/oidc/callback" {
		t.Fatalf("got=%s", got)
	}
}

func TestResolveRedirectURIXForwarded(t *testing.T) {
	c := oidc.Config{RedirectURI: "/oidc/callback"}
	r, _ := http.NewRequest(http.MethodGet, "http://127.0.0.1:18085/oidc/login", nil)
	r.Host = "127.0.0.1:18085"
	r.Header.Set("X-Forwarded-Proto", "https")
	r.Header.Set("X-Forwarded-Host", "www.qdqc.com, other.example")
	got, err := c.ResolveRedirectURI(r)
	if err != nil {
		t.Fatal(err)
	}
	if got != "https://www.qdqc.com/oidc/callback" {
		t.Fatalf("got=%s", got)
	}
}

func TestResolveRedirectURIAllowlistReject(t *testing.T) {
	c := oidc.Config{
		RedirectURI:     "/oidc/callback",
		RedirectOrigins: "https://qdqc.com,https://www.qdqc.com",
	}
	r, _ := http.NewRequest(http.MethodGet, "https://evil.example/oidc/login", nil)
	r.Host = "evil.example"
	r.Header.Set("X-Forwarded-Proto", "https")
	_, err := c.ResolveRedirectURI(r)
	if err == nil || !strings.Contains(err.Error(), "redirect_origins") {
		t.Fatalf("expected allowlist error, got %v", err)
	}
}

func TestResolveRedirectURIAllowlistOK(t *testing.T) {
	c := oidc.Config{
		RedirectURI:     "/oidc/callback",
		RedirectOrigins: "https://qdqc.com,https://www.qdqc.com",
	}
	r, _ := http.NewRequest(http.MethodGet, "https://www.qdqc.com/oidc/login", nil)
	r.Host = "www.qdqc.com"
	r.Header.Set("X-Forwarded-Proto", "https")
	got, err := c.ResolveRedirectURI(r)
	if err != nil {
		t.Fatal(err)
	}
	if got != "https://www.qdqc.com/oidc/callback" {
		t.Fatalf("got=%s", got)
	}
}

func TestAuthorizeRedirectUsesExplicitURI(t *testing.T) {
	c := oidc.Config{
		ClientID:     "app_x",
		AuthorizeURL: "https://id.example.com/oauth2/authorize",
		RedirectURI:  "/oidc/callback",
	}
	loc, err := c.AuthorizeRedirect("st", "ch", "https://www.qdqc.com/oidc/callback")
	if err != nil {
		t.Fatal(err)
	}
	u, err := url.Parse(loc)
	if err != nil {
		t.Fatal(err)
	}
	if got := u.Query().Get("redirect_uri"); got != "https://www.qdqc.com/oidc/callback" {
		t.Fatalf("redirect_uri=%s", got)
	}
	if u.Query().Get("state") != "st" {
		t.Fatal("state")
	}
}
