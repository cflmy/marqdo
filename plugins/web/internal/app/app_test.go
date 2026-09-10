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

func TestRouteFreesAdminWhenDisabled(t *testing.T) {
	a := app.New(map[string]any{"admin": false})
	page := map[string]any{"title": "news"}
	out, err := app.Route(a, "/admin/news", page)
	if err != nil {
		t.Fatal(err)
	}
	routes := out["routes"].(map[string]any)
	if routes["/admin/news"] == nil {
		t.Fatal("missing route")
	}
}

func TestRouteCustomPrefixFreesAdmin(t *testing.T) {
	a := app.New(map[string]any{"admin": true, "admin_prefix": "/desk"})
	out, err := app.Route(a, "/admin/news", map[string]any{"title": "news"})
	if err != nil {
		t.Fatal(err)
	}
	if out["routes"].(map[string]any)["/admin/news"] == nil {
		t.Fatal("missing")
	}
	_, err = app.Route(a, "/desk/x", map[string]any{"title": "x"})
	if err == nil || !strings.Contains(err.Error(), "reserved") {
		t.Fatalf("want reserved under /desk, got %v", err)
	}
}

func TestAuthDefaultGate(t *testing.T) {
	a := app.New(map[string]any{"admin": true})
	users := map[string]any{
		"用户名": []any{"admin"},
		"密码":  []any{"secret"},
		"角色":  []any{"admin"},
	}
	out, err := app.Auth(a, users, nil)
	if err != nil {
		t.Fatal(err)
	}
	if out["login_path"] != "/admin/login" {
		t.Fatalf("login_path=%v", out["login_path"])
	}
	gates := out["gates"].([]any)
	g0 := gates[0].(map[string]any)
	if g0["path"] != "/admin" || g0["match"] != "prefix" || g0["on_deny"] != "redirect" {
		t.Fatalf("gate=%v", g0)
	}
	ex := g0["exclude"].([]any)
	if len(ex) != 1 || ex[0] != "/admin/login" {
		t.Fatalf("exclude=%v", ex)
	}
}

func TestAuthCustomPrefix(t *testing.T) {
	a := app.New(map[string]any{"admin": true})
	out, err := app.Auth(a, []any{}, map[string]any{
		"admin_prefix":    "/desk",
		"login_redirect":  "/desk",
		"logout_redirect": "/desk/login",
	})
	if err != nil {
		t.Fatal(err)
	}
	if out["admin_prefix"] != "/desk" || out["login_redirect"] != "/desk" {
		t.Fatalf("%v", out)
	}
	g0 := out["gates"].([]any)[0].(map[string]any)
	if g0["path"] != "/desk" {
		t.Fatalf("gate path=%v", g0["path"])
	}
}

func TestGateCustomFields(t *testing.T) {
	a := app.New(map[string]any{"admin": false})
	out, err := app.Gate(a, "/desk", map[string]any{
		"roles":   "admin",
		"match":   "prefix",
		"on_deny": "redirect",
		"exclude": "/desk/login",
	})
	if err != nil {
		t.Fatal(err)
	}
	g0 := out["gates"].([]any)[0].(map[string]any)
	if g0["path"] != "/desk" || g0["match"] != "prefix" || g0["on_deny"] != "redirect" {
		t.Fatalf("%v", g0)
	}
	if g0["exclude"] != "/desk/login" {
		t.Fatalf("exclude=%v", g0["exclude"])
	}
	out2, err := app.Gate(a, "/write*", map[string]any{"roles": "admin,author"})
	if err != nil {
		t.Fatal(err)
	}
	g1 := out2["gates"].([]any)[0].(map[string]any)
	roles := g1["roles"].([]any)
	if g1["path"] != "/write*" || len(roles) != 2 {
		t.Fatalf("%v", g1)
	}
}

func TestRedirectStoresMap(t *testing.T) {
	a := app.New(map[string]any{"page": map[string]any{"title": "Home"}})
	out, err := app.Redirect(a, "/old", "/new", true)
	if err != nil {
		t.Fatal(err)
	}
	rd, ok := out["redirects"].(map[string]any)
	if !ok {
		t.Fatalf("redirects=%v", out["redirects"])
	}
	entry, ok := rd["/old"].(map[string]any)
	if !ok || entry["to"] != "/new" || entry["permanent"] != true {
		t.Fatalf("entry=%v", entry)
	}
}

func TestErrorPage404(t *testing.T) {
	a := app.New(nil)
	p := map[string]any{"title": "Missing"}
	out, err := app.ErrorPage(a, 404, p)
	if err != nil {
		t.Fatal(err)
	}
	got, ok := out["page_404"].(map[string]any)
	if !ok || got["title"] != "Missing" {
		t.Fatalf("page_404=%v", out["page_404"])
	}
}

func TestRobotsDefaultBody(t *testing.T) {
	a := app.New(nil)
	out, err := app.Robots(a, "", "https://example.com/sitemap.xml")
	if err != nil {
		t.Fatal(err)
	}
	rb, ok := out["robots_body"].(string)
	if !ok || !strings.Contains(rb, "Sitemap: https://example.com/sitemap.xml") {
		t.Fatalf("robots_body=%v", out["robots_body"])
	}
}

func TestSitemapRouteBag(t *testing.T) {
	a := app.New(nil)
	items := []any{map[string]any{"loc": "/"}}
	out, err := app.Sitemap(a, "/sitemap.xml", "https://example.com", "", "path", 1000, items)
	if err != nil {
		t.Fatal(err)
	}
	routes, ok := out["sitemap_routes"].(map[string]any)
	if !ok {
		t.Fatalf("sitemap_routes=%v", out["sitemap_routes"])
	}
	entry, ok := routes["/sitemap.xml"].(map[string]any)
	if !ok || entry["base"] != "https://example.com" {
		t.Fatalf("entry=%v", entry)
	}
}
