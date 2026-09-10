package page_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/page"
)

func TestNewDefaults(t *testing.T) {
	p := page.New(map[string]any{})
	if p["title"] != "Marqdo Web" {
		t.Fatalf("title=%v", p["title"])
	}
	if p["intro"] != "" {
		t.Fatalf("intro=%v", p["intro"])
	}
}

func TestNewZhAliases(t *testing.T) {
	p := page.New(map[string]any{
		"title":  "T",
		"壳样式":  "minimal",
		"布局":    "bare",
		"资源版本": "v1",
	})
	if p["shell_css"] != "minimal" || p["layout"] != "bare" || p["asset_version"] != "v1" {
		t.Fatalf("%v", p)
	}
}

func TestPaginateBag(t *testing.T) {
	p := page.New(map[string]any{"title": "List"})
	out := page.Paginate(p, 0, 5, "/")
	pg, ok := out["paginate"].(map[string]any)
	if !ok {
		t.Fatalf("paginate=%v", out["paginate"])
	}
	if pg["offset"] != int64(0) || pg["limit"] != int64(5) || pg["path"] != "/" {
		t.Fatalf("%v", pg)
	}
}
