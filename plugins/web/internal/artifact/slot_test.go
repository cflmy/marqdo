package artifact

import (
	"strings"
	"testing"
)

func TestFindSlotKeepsNestedTypedResourcesAndRejectsAmbiguity(t *testing.T) {
	descriptor := map[string]any{"_type": "data_query", "source": "articles", "slot": "not-a-view-slot"}
	nodes := []any{map[string]any{"type": "row", "slot": "layout", "value": []any{
		map[string]any{"类型": "纵排", "槽": "main", "值": []any{
			map[string]any{"type": "resource", "slot": "articles", "value": descriptor},
		}},
		map[string]any{"type": "markdown", "slot": "sidebar", "value": "Separate"},
	}}}
	selected, found, err := FindSlot(nodes, "articles")
	if err != nil || !found {
		t.Fatalf("%v %v", found, err)
	}
	calls := 0
	html, err := RenderNodesWithResources(selected, func(value map[string]any) (string, error) {
		calls++
		if value["source"] != "articles" {
			t.Fatal(value)
		}
		return "<p>Actual request value</p>", nil
	})
	if err != nil || calls != 1 || !strings.Contains(html, "Actual request value") || strings.Contains(html, "Separate") {
		t.Fatalf("%d %v %s", calls, err, html)
	}
	if _, found, err := FindSlot(nodes, "not-a-view-slot"); err != nil || found {
		t.Fatalf("walked Resource internals: %v %v", found, err)
	}
	duplicate := append(nodes, map[string]any{"type": "text", "slot": "articles", "value": "Ambiguous"})
	if _, _, err := FindSlot(duplicate, "articles"); err == nil {
		t.Fatal("duplicate slot accepted")
	}
	if _, _, err := FindSlot([]any{42}, "articles"); err == nil {
		t.Fatal("malformed View accepted")
	}
}
