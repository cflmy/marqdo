package app_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
)

func TestNormalizeStaticMount(t *testing.T) {
	cases := []struct {
		in, want string
	}{
		{"", "/static"},
		{"  ", "/static"},
		{"static", "/static"},
		{"/static/", "/static"},
		{"/assets", "/assets"},
		{"assets/", "/assets"},
	}
	for _, c := range cases {
		if got := app.NormalizeStaticMount(c.in); got != c.want {
			t.Fatalf("NormalizeStaticMount(%q)=%q want %q", c.in, got, c.want)
		}
	}
}

func TestStaticSetsDirAndMount(t *testing.T) {
	a := app.New(map[string]any{"page": map[string]any{"title": "Home"}})
	out, err := app.Static(a, "web-fixtures/public", "/static")
	if err != nil {
		t.Fatal(err)
	}
	if out["static_dir"] != "web-fixtures/public" {
		t.Fatalf("static_dir=%v", out["static_dir"])
	}
	if out["static_mount"] != "/static" {
		t.Fatalf("static_mount=%v", out["static_mount"])
	}
	out2, err := app.Static(out, "assets", "/assets/")
	if err != nil {
		t.Fatal(err)
	}
	if out2["static_mount"] != "/assets" {
		t.Fatalf("custom mount=%v", out2["static_mount"])
	}
}

func TestStaticRejectsReserved(t *testing.T) {
	a := app.New(nil)
	for _, mount := range []string{"/_form", "/_form/x", "/_part", "/_part/nav"} {
		_, err := app.Static(a, "pub", mount)
		if err == nil || !strings.Contains(err.Error(), "reserved") {
			t.Fatalf("mount %q: want reserved error, got %v", mount, err)
		}
	}
	admin := app.New(map[string]any{"admin": true, "admin_prefix": "/admin"})
	_, err := app.Static(admin, "pub", "/admin")
	if err == nil || !strings.Contains(err.Error(), "reserved") {
		t.Fatalf("admin mount: want reserved, got %v", err)
	}
}
