package artifact

import (
	"strings"
	"testing"
)

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

func TestNestedLayoutKeepsMarkdownLeavesAndSiblingOrder(t *testing.T) {
	leaf := func(slot, text string) any { return map[string]any{"type": "markdown", "slot": slot, "value": text} }
	nodes := []any{map[string]any{"type": "footer", "value": []any{
		map[string]any{"type": "row", "slot": "footer", "value": []any{
			leaf("brand", "### Marqdo\n\nReadable **documents**."),
			map[string]any{"类型": "横排", "槽": "links", "值": []any{
				leaf("docs", "### Docs\n\n[Guide](/guide)"),
				leaf("community", "### Community\n\n[About](/about)"),
			}},
		}},
	}}}
	got, err := RenderNodes(nodes)
	if err != nil {
		t.Fatal(err)
	}
	for _, want := range []string{"<footer", "mq-row", "slot-links", "<strong>documents</strong>", `href="/guide"`, `href="/about"`} {
		if !strings.Contains(got, want) {
			t.Fatalf("missing %q: %s", want, got)
		}
	}
	if strings.Index(got, "slot-brand") > strings.Index(got, "slot-docs") || strings.Index(got, "slot-docs") > strings.Index(got, "slot-community") {
		t.Fatal("component order changed")
	}
	if strings.Contains(got, "map[") {
		t.Fatal("nested data was stringified")
	}
}

func TestNestedColumnTableRetainsTypedValue(t *testing.T) {
	nodes := map[string]any{"@": []any{1}, "type": []any{"column"}, "value": []any{[]any{map[string]any{"type": "markdown", "value": "# Leaf"}}}}
	got, err := RenderNodes(nodes)
	if err != nil || !strings.Contains(got, "<h1") || !strings.Contains(got, "Leaf") {
		t.Fatalf("%v: %s", err, got)
	}
}

func TestInvalidChildrenAreDiagnosed(t *testing.T) {
	for _, value := range []any{"<div>not child rows</div>", []any{"bad row"}} {
		if _, err := RenderNodes([]any{map[string]any{"type": "row", "value": value}}); err == nil {
			t.Fatalf("accepted %v", value)
		}
	}
	var deep any = []any{map[string]any{"type": "text", "value": "end"}}
	for i := 0; i < 66; i++ {
		deep = []any{map[string]any{"type": "column", "value": deep}}
	}
	if _, err := RenderNodes(deep); err == nil {
		t.Fatal("unbounded nesting")
	}
}

func TestViewDocumentExecutesOnlyAnExplicitMain(t *testing.T) {
	calls := 0
	run := func(path string, args map[string]any) (any, error) {
		calls++
		return []any{map[string]any{"type": "markdown", "value": "# Composed"}}, nil
	}
	f := ArtifactFile{Path: "index.mq.md", Rel: "index.mq.md", Meta: map[string]string{}, Body: "# Ordinary"}
	body, structured, err := RenderFile(f, run)
	if err != nil || structured || calls != 0 || !strings.Contains(body, "Ordinary") {
		t.Fatalf("ordinary: %v %v %d", err, structured, calls)
	}
	f.Meta["视图"] = "main"
	body, structured, err = RenderFile(f, run)
	if err != nil || !structured || calls != 1 || !strings.Contains(body, "Composed") {
		t.Fatalf("view: %v %v %d", err, structured, calls)
	}
	f.Meta["视图"] = "typo"
	if _, _, err = RenderFile(f, run); err == nil || calls != 1 {
		t.Fatal("invalid view must fail before execution")
	}
}

func TestViewDocumentRejectsAnEmptyAssemblyAndAutomaticDataSource(t *testing.T) {
	f := ArtifactFile{Path: "index.mq.md", Rel: "index.mq.md", Meta: map[string]string{"view": "main"}}
	calls := 0
	run := func(string, map[string]any) (any, error) { calls++; return nil, nil }
	if _, _, err := RenderFile(f, run); err == nil || !strings.Contains(err.Error(), "must return View rows") {
		t.Fatalf("empty assembly: %v", err)
	}
	f.Meta["data_source"] = "posts"
	if _, _, err := RenderFile(f, run); err == nil || calls != 1 {
		t.Fatalf("data source silently ignored: %v", err)
	}
}
