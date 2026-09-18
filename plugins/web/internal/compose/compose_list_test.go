package compose

import "testing"

func TestComposeListAppends(t *testing.T) {
	page := map[string]any{"title": "p"}
	main := []any{
		map[string]any{"front": "title", "back": "comments.author", "css": ""},
		map[string]any{"front": "body", "back": "comments.body", "css": ""},
	}
	q := []any{map[string]any{"field": "post_slug", "op": "=", "value": "{slug}"}}
	out, err := ComposeList(page, main, q, "-created_at", "#comment-list", func(string) (any, error) {
		return nil, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	m := out.(map[string]any)
	lists, ok := m["lists"].([]any)
	if !ok || len(lists) != 1 {
		t.Fatalf("lists=%v", m["lists"])
	}
	entry := lists[0].(map[string]any)
	if entry["order"] != "-created_at" || entry["target"] != "#comment-list" {
		t.Fatalf("entry=%v", entry)
	}
	if entry["query"] == nil {
		t.Fatal("missing query")
	}
}
