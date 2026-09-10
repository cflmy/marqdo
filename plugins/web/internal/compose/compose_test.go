package compose_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/compose"
)

func TestComposeComponentsFakeCallLib(t *testing.T) {
	page := map[string]any{
		"title": "Home",
	}
	layout := []any{
		map[string]any{"组件": "site.nav", "样式": ""},
		map[string]any{"组件": "site.footer", "样式": ""},
	}
	callLib := func(path string) (any, error) {
		switch path {
		case "site.nav":
			return map[string]any{
				"属性": []any{"Home", "About"},
				"值":  []any{"/", "/about"},
			}, nil
		case "site.footer":
			return []any{
				map[string]any{"属性": "©", "值": "/"},
			}, nil
		default:
			t.Fatalf("unexpected callLib %q", path)
			return nil, nil
		}
	}
	out, err := compose.ComposeComponents(page, layout, callLib)
	if err != nil {
		t.Fatal(err)
	}
	obj := out.(map[string]any)
	nav, ok := obj["nav"].([]any)
	if !ok || len(nav) != 2 {
		t.Fatalf("nav=%v", obj["nav"])
	}
	n0 := nav[0].(map[string]any)
	if n0["front"] != "Home" || n0["back"] != "/" {
		t.Fatalf("nav[0]=%v", n0)
	}
	foot, ok := obj["footer"].([]any)
	if !ok || len(foot) != 1 {
		t.Fatalf("footer=%v", obj["footer"])
	}
	comp, ok := obj["compose"].([]any)
	if !ok || len(comp) != 2 {
		t.Fatalf("compose=%v", obj["compose"])
	}
	c0 := comp[0].(map[string]any)
	if c0["src"] != "nav" || c0["slot"] != "nav" {
		t.Fatalf("compose[0]=%v", c0)
	}
	parts, ok := obj["parts"].(map[string]any)
	if !ok {
		t.Fatalf("parts missing: %v", obj)
	}
	if _, ok := parts["nav"]; !ok {
		t.Fatalf("parts.nav missing: %v", parts)
	}
	if _, ok := parts["footer"]; !ok {
		t.Fatalf("parts.footer missing: %v", parts)
	}
}
