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
