package form

import "testing"

func TestApplySourcesOverwritesClient(t *testing.T) {
	frm := SetFields(New("comments", "insert", "c"), []any{
		map[string]any{"字段": "body", "标签": "body", "类型": "textarea", "必填": true, "来源": "client"},
		map[string]any{"字段": "author", "标签": "a", "类型": "text", "必填": true, "来源": "session.username"},
		map[string]any{"字段": "post_slug", "标签": "s", "类型": "text", "必填": true, "来源": "route.slug"},
		map[string]any{"字段": "created_at", "标签": "t", "类型": "text", "来源": "now"},
	})
	data := map[string]any{
		"body":      "hi",
		"author":    "forged",
		"post_slug": "evil",
	}
	ApplySources(frm, data, &RequestContext{
		Username: "alice",
		Params:   map[string]any{"slug": "hello"},
	})
	if data["author"] != "alice" {
		t.Fatalf("author=%v", data["author"])
	}
	if data["post_slug"] != "hello" {
		t.Fatalf("slug=%v", data["post_slug"])
	}
	if data["body"] != "hi" {
		t.Fatalf("body=%v", data["body"])
	}
	if data["created_at"] == nil || data["created_at"] == "" {
		t.Fatalf("created_at missing")
	}
}

func TestRenderSkipsServerSources(t *testing.T) {
	frm := SetFields(New("comments", "insert", "c"), []any{
		map[string]any{"字段": "body", "标签": "Body", "类型": "textarea", "必填": true},
		map[string]any{"字段": "author", "标签": "Author", "类型": "text", "来源": "session.username"},
	})
	html := RenderBodyCtx(frm, "c", nil, nil, "", &RequestContext{
		Username:   "alice",
		Params:     map[string]any{"slug": "x"},
		ReturnPath: "/post/x",
	})
	if stringsContains(html, `name="author"`) {
		t.Fatalf("author field should not render: %s", html)
	}
	if !stringsContains(html, `name="body"`) {
		t.Fatalf("body missing")
	}
	if !stringsContains(html, `_mq_params`) || !stringsContains(html, `_mq_return`) {
		t.Fatalf("missing mq context fields: %s", html)
	}
}

func stringsContains(s, sub string) bool {
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
