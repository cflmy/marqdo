package compose

import "testing"

func TestComposeFormLoad(t *testing.T) {
	page := map[string]any{"title": "Edit"}
	out, err := ComposeFormLoad(page, "posts", "")
	if err != nil {
		t.Fatal(err)
	}
	obj, ok := out.(map[string]any)
	if !ok {
		t.Fatalf("want map, got %T", out)
	}
	load, ok := obj["form_load"].(map[string]any)
	if !ok {
		t.Fatalf("missing form_load: %#v", obj)
	}
	if load["table"] != "posts" {
		t.Fatalf("table=%v", load["table"])
	}
	if load["id_param"] != "id" {
		t.Fatalf("id_param=%v", load["id_param"])
	}
}

func TestComposeFormLoadRequiresTable(t *testing.T) {
	_, err := ComposeFormLoad(map[string]any{}, "  ", "id")
	if err == nil {
		t.Fatal("expected error for empty table")
	}
}
