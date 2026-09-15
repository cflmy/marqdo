package render_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/render"
)

func TestRenderPageIntroNoDB(t *testing.T) {
	page := map[string]any{
		"title":      "smoke",
		"intro":      "<h1>smoke</h1>",
		"styles_css": ".x { color: red; }",
		"shell_css":  "minimal",
	}
	html := render.RenderPage(page, "", "")
	if html == "" {
		t.Fatal("empty html")
	}
	if !strings.Contains(html, "<h1>smoke</h1>") {
		t.Fatalf("missing intro: %s", html)
	}
	if !strings.Contains(html, ".x { color: red; }") {
		t.Fatalf("missing styles: %s", html)
	}
	if !strings.Contains(html, "<!DOCTYPE html>") {
		t.Fatalf("not a document: %s", html[:80])
	}
}

func TestShellOffStackedOmitsSidebarGrid(t *testing.T) {
	page := map[string]any{
		"title":     "t",
		"shell_css": "off",
		"layout":    "stacked",
		"sidebar": []any{
			map[string]any{"label": "A", "href": "/a"},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, "layout-stacked") {
		t.Fatalf("missing layout-stacked: %s", html)
	}
	if strings.Contains(html, "has-sidebar") {
		t.Fatalf("unexpected has-sidebar: %s", html)
	}
	if strings.Contains(html, "grid-template-columns:14rem") {
		t.Fatalf("unexpected sidebar grid: %s", html)
	}
}

func TestShellMinimalKeepsVarsNotCards(t *testing.T) {
	page := map[string]any{"title": "t", "shell_css": "minimal"}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, "--ink:") {
		t.Fatalf("missing vars: %s", html)
	}
	if strings.Contains(html, ".content.cards article") {
		t.Fatalf("unexpected cards css: %s", html)
	}
}

func TestLayoutBareSkipsAside(t *testing.T) {
	page := map[string]any{
		"title":  "t",
		"layout": "bare",
		"sidebar": []any{
			map[string]any{"label": "A", "href": "/a"},
		},
		"nav": []any{
			map[string]any{"label": "Home", "href": "/"},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, "layout-bare") {
		t.Fatalf("missing layout-bare: %s", html)
	}
	if strings.Contains(html, "<aside") {
		t.Fatalf("unexpected aside: %s", html)
	}
}

func TestDefaultFullKeepsSidebarGrid(t *testing.T) {
	page := map[string]any{
		"title": "t",
		"sidebar": []any{
			map[string]any{"label": "A", "href": "/a"},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, "has-sidebar") {
		t.Fatalf("missing has-sidebar: %s", html)
	}
	if !strings.Contains(html, "grid-template-columns:14rem") {
		t.Fatalf("missing sidebar grid: %s", html)
	}
}

func TestNavWhenHideOmitsItem(t *testing.T) {
	page := map[string]any{
		"title":     "t",
		"shell_css": "off",
		"nav": []any{
			map[string]any{"label": "Home", "href": "/"},
			map[string]any{"label": "Secret", "href": "/x", "when": "hide"},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, ">Home<") {
		t.Fatalf("missing Home: %s", html)
	}
	if strings.Contains(html, "/x") {
		t.Fatalf("hide leaked: %s", html)
	}
}

func TestNavWhenAuthGuestRespectsLoggedIn(t *testing.T) {
	nav := []any{
		map[string]any{"front": "Public", "back": "/"},
		map[string]any{"front": "Admin", "back": "/admin", "when": "auth"},
		map[string]any{"front": "Login", "back": "/login", "when": "guest"},
	}
	guest := map[string]any{
		"title":       "t",
		"shell_css":   "off",
		"_logged_in":  false,
		"nav":         nav,
	}
	g := render.RenderPage(guest, "", "")
	if !strings.Contains(g, "/login") {
		t.Fatalf("guest missing login: %s", g)
	}
	if strings.Contains(g, "/admin") {
		t.Fatalf("guest leaked admin: %s", g)
	}
	user := map[string]any{
		"title":       "t",
		"shell_css":   "off",
		"_logged_in":  true,
		"_nav_user":   "alice",
		"nav":         nav,
	}
	u := render.RenderPage(user, "", "")
	if !strings.Contains(u, "/admin") {
		t.Fatalf("user missing admin: %s", u)
	}
	if strings.Contains(u, "/login") {
		t.Fatalf("user leaked login: %s", u)
	}
}

func TestNavMediaEmitsClassAndCSS(t *testing.T) {
	page := map[string]any{
		"title":     "t",
		"shell_css": "off",
		"nav": []any{
			map[string]any{"label": "Wide", "href": "/w", "media": "(min-width: 900px)"},
			map[string]any{"label": "Home", "href": "/"},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, "nav-mq-0") {
		t.Fatalf("missing nav-mq-0: %s", html)
	}
	if !strings.Contains(html, "@media not (min-width: 900px)") {
		t.Fatalf("missing media css: %s", html)
	}
}

func TestHeadDeferVersionAndAssetVersion(t *testing.T) {
	page := map[string]any{
		"title":         "t",
		"asset_version": "appv",
		"head": []any{
			map[string]any{"rel": "script", "href": "/static/a.js", "defer": true, "version": "3"},
			map[string]any{"rel": "stylesheet", "href": "/static/t.css", "version": "1"},
			map[string]any{"rel": "script", "href": "/static/b.js", "defer": true},
		},
	}
	html := render.RenderPage(page, "", "")
	if !strings.Contains(html, `a.js?v=3`) {
		t.Fatalf("missing script version: %s", html)
	}
	if !strings.Contains(html, "script defer") {
		t.Fatalf("missing defer: %s", html)
	}
	if !strings.Contains(html, `t.css?v=1`) {
		t.Fatalf("missing css version: %s", html)
	}
	if !strings.Contains(html, `b.js?v=appv`) {
		t.Fatalf("missing asset_version fallback: %s", html)
	}
	syncPage := map[string]any{
		"title": "t",
		"head": []any{
			map[string]any{"rel": "script", "href": "/static/sync.js"},
		},
	}
	syncHTML := render.RenderPage(syncPage, "", "")
	if !strings.Contains(syncHTML, `sync.js`) {
		t.Fatalf("missing sync script: %s", syncHTML)
	}
	if strings.Contains(syncHTML, "script defer") {
		t.Fatalf("unexpected defer on sync: %s", syncHTML)
	}
}

func TestRenderPageNavChrome(t *testing.T) {
	page := map[string]any{
		"title": "t",
		"nav": []any{
			map[string]any{"front": "Home", "back": "/", "css": "", "media": "", "when": ""},
		},
		"sidebar": []any{
			map[string]any{"front": "A", "back": "/a", "css": "", "media": "", "when": ""},
		},
		"footer": []any{
			map[string]any{"front": "©", "back": "/", "css": "", "media": "", "when": ""},
		},
		"intro": "hi",
	}
	html := render.RenderPage(page, "", "")
	for _, needle := range []string{"<header", "<aside", "<footer", "Home", "侧栏", "has-sidebar"} {
		if !strings.Contains(html, needle) {
			t.Fatalf("missing %q in html", needle)
		}
	}
}

func TestResolveRouteParams(t *testing.T) {
	got := render.ResolveRouteParams("/post/{slug}", map[string]any{"slug": "hello-marqdo"})
	if got != "/post/hello-marqdo" {
		t.Fatalf("got %q", got)
	}
	got = render.ResolveRouteParams("/post/{slug}", map[string]any{})
	if got != "/post/{slug}" {
		t.Fatalf("unmatched must stay: %q", got)
	}
}

func TestPartSlotSrcUsesResolvedRoute(t *testing.T) {
	page := map[string]any{
		"title":  "post",
		"intro":  "",
		"_route": "/post/{slug}",
		"params": map[string]any{"slug": "hello-marqdo"},
		"parts": map[string]any{
			"index": map[string]any{"slot": "main"},
		},
	}
	html := render.RenderPage(page, "", "")
	if strings.Contains(html, "/post/{slug}/_part/") {
		t.Fatalf("slot src still has placeholder: %s", html)
	}
	if !strings.Contains(html, `data-slot-src="/post/hello-marqdo/_part/index"`) {
		t.Fatalf("missing resolved slot src: %s", html)
	}
}
