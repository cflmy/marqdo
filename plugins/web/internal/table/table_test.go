package table_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

func TestNormalizeRefBackticks(t *testing.T) {
	cases := []struct {
		in, want string
	}{
		{"`foo`", "foo"},
		{"  `a.b`  ", "a.b"},
		{"plain", "plain"},
		{"`a` and `b`", "a and b"},
		{"`unclosed", "`unclosed"},
		{"", ""},
	}
	for _, c := range cases {
		got := table.NormalizeRef(c.in)
		if got != c.want {
			t.Errorf("NormalizeRef(%q)=%q want %q", c.in, got, c.want)
		}
	}
}

func TestAsBindChineseHeaders(t *testing.T) {
	tbl := map[string]any{
		"属性": []any{"title", "body"},
		"值":  []any{"posts.title", "posts.body"},
		"样式": []any{"`hero`", ""},
	}
	got := table.AsBind(tbl)
	arr, ok := got.([]any)
	if !ok || len(arr) != 2 {
		t.Fatalf("AsBind=%v", got)
	}
	r0 := arr[0].(map[string]any)
	if r0["front"] != "title" || r0["back"] != "posts.title" || r0["css"] != "hero" {
		t.Fatalf("row0=%v", r0)
	}
	r1 := arr[1].(map[string]any)
	if r1["front"] != "body" || r1["back"] != "posts.body" || r1["css"] != "" {
		t.Fatalf("row1=%v", r1)
	}
}

func TestAsCSSNamedSimpleSelector(t *testing.T) {
	tbl := []any{
		map[string]any{"选择器": ".card", "属性": "color", "值": "red"},
		map[string]any{"选择器": ".card", "属性": "padding", "值": "1rem"},
	}
	css := table.AsCSSNamed("card", tbl)
	if !strings.Contains(css, ".card {") {
		t.Fatalf("missing selector: %q", css)
	}
	if !strings.Contains(css, "color: red;") || !strings.Contains(css, "padding: 1rem;") {
		t.Fatalf("missing decls: %q", css)
	}
}
