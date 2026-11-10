package artifact

import (
	"strings"
	"testing"
)

func headNode(meta, links any) any {
	return map[string]any{"type": "resource", "slot": "head", "value": map[string]any{"_type": "document_head", "meta": meta, "links": links}}
}

func TestDocumentHeadPreservesMetadataAndLinksInTheActualHeadOnly(t *testing.T) {
	meta := map[string]any{"key": []any{"description", "og:title", "canonical"}, "value": []any{`Text <tag> "quote" & literal`, "Actual OG", "/articles"}}
	links := []any{
		map[string]any{"关系": "preconnect", "地址": "https://fonts.example.test", "跨域": "anonymous"},
		map[string]any{"rel": "stylesheet", "href": "/theme.css?x=1&y=2"},
		map[string]any{"rel": "module", "href": "/client.js", "defer": false},
	}
	// Repeated imports should deduplicate equal links, including boolean flags.
	nodes := []any{map[string]any{"type": "column", "value": []any{
		headNode(meta, links), headNode(nil, links),
		map[string]any{"type": "markdown", "value": "# Actual page"},
	}}}
	got, err := RenderViewDocument("Actual document", nodes, nil)
	if err != nil {
		t.Fatal(err)
	}
	head, body, ok := strings.Cut(got, "</head>")
	if !ok {
		t.Fatal(got)
	}
	for _, want := range []string{
		"<title>Actual document</title>", `<meta name="description" content="Text &lt;tag&gt; &quot;quote&quot; &amp; literal"`,
		`<meta property="og:title" content="Actual OG"`, `<link rel="canonical" href="/articles"`,
		`<link rel="preconnect" href="https://fonts.example.test" crossorigin="anonymous"`,
		`<link rel="stylesheet" href="/theme.css?x=1&amp;y=2"`, `<script type="module" src="/client.js"></script>`,
	} {
		if !strings.Contains(head, want) {
			t.Fatalf("missing %s: %s", want, head)
		}
	}
	if strings.Count(head, `href="/theme.css`) != 1 || strings.Count(head, `src="/client.js"`) != 1 {
		t.Fatal("duplicate head resource: " + head)
	}
	if strings.Contains(body, "Actual OG") || strings.Contains(body, "fonts.example") || strings.Contains(body, "slot-head") || !strings.Contains(body, "Actual page") {
		t.Fatal(body)
	}
	calls := 0
	fragment, err := RenderNodesWithResources(nodes, func(map[string]any) (string, error) { calls++; return "unexpected", nil })
	if err != nil || calls != 0 || strings.Contains(fragment, "<meta") || strings.Contains(fragment, "fonts.example") {
		t.Fatalf("%d %v %s", calls, err, fragment)
	}
}

func TestDocumentHeadRejectsConflictsMalformedTablesAndOpaqueMutation(t *testing.T) {
	valid := map[string]any{"_type": "document_head", "meta": nil, "links": nil}
	for _, resource := range []map[string]any{
		{"_type": "other"},
		{"_type": "document_head", "html": "<script>x</script>"},
		{"_type": "document_head", "body_class": "admin"},
		{"_type": "document_head", "meta": []any{42}},
		{"_type": "document_head", "meta": map[string]any{"key": []any{"a", "b"}, "value": []any{"one"}}},
		{"_type": "document_head", "meta": map[string]any{"key": "description", "value": []any{"nested"}}},
		{"_type": "document_head", "meta": map[string]any{"key": "viewport", "value": "owned"}},
		{"_type": "document_head", "meta": []any{map[string]any{"key": "description", "value": "a"}, map[string]any{"key": "description", "value": "b"}}},
		{"_type": "document_head", "links": []any{map[string]any{"rel": "script", "href": "/a.js", "onclick": "x"}}},
		{"_type": "document_head", "links": []any{map[string]any{"rel": "stylesheet", "href": "/a.css", "media": map[string]any{"x": "nested"}}}},
		{"_type": "document_head", "links": []any{map[string]any{"rel": "stylesheet"}}},
	} {
		if _, err := ParseDocumentHead(resource); err == nil {
			t.Fatalf("accepted %#v", resource)
		}
	}
	if _, err := ParseDocumentHead(valid); err != nil {
		t.Fatal(err)
	}
	for _, url := range []string{"javascript:alert(1)", "data:text/javascript,x", "//other.test/a", "relative.css", "/\\other.test/a", "https://", "/a\n.js"} {
		for _, resource := range []map[string]any{
			{"_type": "document_head", "links": []any{map[string]any{"rel": "script", "href": url}}},
			{"_type": "document_head", "meta": map[string]any{"key": "canonical", "value": url}},
		} {
			if _, err := ParseDocumentHead(resource); err == nil {
				t.Fatalf("accepted %q", url)
			}
		}
	}
	conflict := []any{headNode(map[string]any{"key": "description", "value": "first"}, nil),
		map[string]any{"类型": "纵排", "值": []any{headNode(map[string]any{"键": "description", "值": "second"}, nil)}}}
	if _, err := CollectDocumentHead(conflict); err == nil {
		t.Fatal("nested head conflict silently overwrote metadata")
	}
	private := []any{map[string]any{"type": "resource", "value": map[string]any{"_type": "data_query", "where": headNode(nil, nil)}}}
	head, err := CollectDocumentHead(private)
	if err != nil || len(head.Links) != 0 || len(head.Meta) != 0 {
		t.Fatalf("walked private Resource: %v %#v", err, head)
	}
	var deep any = []any{headNode(nil, nil)}
	for i := 0; i < 66; i++ {
		deep = []any{map[string]any{"type": "row", "value": deep}}
	}
	if _, err := CollectDocumentHead(deep); err == nil {
		t.Fatal("unbounded head traversal")
	}
}
