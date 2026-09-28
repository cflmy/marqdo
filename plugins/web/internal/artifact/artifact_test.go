package artifact

import "testing"

func TestMetaType(t *testing.T) {
	if MetaType(map[string]string{"type": "web"}) != "web" {
		t.Fatal("en web")
	}
	if MetaType(map[string]string{"类型": "网页"}) != "web" {
		t.Fatal("zh web")
	}
	if MetaType(map[string]string{"type": "endpoint"}) != "endpoint" {
		t.Fatal("endpoint")
	}
}

func TestParseFrontmatter(t *testing.T) {
	src := "---\ntype: web\nroute: /x\n---\n\n# Hi\n"
	meta, body := ParseFrontmatter(src)
	if meta["type"] != "web" || meta["route"] != "/x" {
		t.Fatalf("meta=%v", meta)
	}
	if body == "" || body[0] != '#' {
		t.Fatalf("body=%q", body)
	}
}

func TestRenderNodes(t *testing.T) {
	nodes := map[string]any{
		"type":  []any{"title", "text"},
		"slot":  []any{"title", "lede"},
		"value": []any{"Marqdo", "Hello"},
		"attrs": []any{"", ""},
		"style": []any{"title", "lede"},
	}
	html, err := RenderNodes(nodes)
	if err != nil {
		t.Fatal(err)
	}
	if !contains(html, "Marqdo") || !contains(html, "mq-view") {
		t.Fatalf("html=%s", html)
	}
}

func contains(s, sub string) bool {
	return len(s) >= len(sub) && (s == sub || len(sub) == 0 ||
		(func() bool {
			for i := 0; i+len(sub) <= len(s); i++ {
				if s[i:i+len(sub)] == sub {
					return true
				}
			}
			return false
		})())
}
