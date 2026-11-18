package render

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
)

func TestResolveMainDataSource(t *testing.T) {
	dir := t.TempDir()
	url := "sqlite:" + filepath.Join(dir, "t.db")
	fields := []any{
		map[string]any{"字段": "title", "类型": "text", "可空": false},
		map[string]any{"字段": "slug", "类型": "text", "可空": false},
		map[string]any{"字段": "summary", "类型": "text", "可空": true},
		map[string]any{"字段": "content", "类型": "text", "可空": true},
	}
	if _, err := db.Init(url, "posts", fields); err != nil {
		t.Fatal(err)
	}
	rows := []any{
		map[string]any{
			"title": "Hello", "slug": "hello",
			"summary": "sum", "content": "# Body",
		},
	}
	if _, err := db.Insert(url, "posts", rows); err != nil {
		t.Fatal(err)
	}
	page := map[string]any{
		"intro":       "<p>hi</p>",
		"data_source": "posts",
		"order":       "-id",
		"link_prefix": "/post/",
	}
	intro, items, _ := resolveMain(page, url)
	if intro != "<p>hi</p>" {
		t.Fatalf("intro=%q", intro)
	}
	if len(items) != 1 {
		t.Fatalf("items=%d", len(items))
	}
	if items[0]["title"] != "Hello" || items[0]["href"] != "hello" {
		t.Fatalf("item=%v", items[0])
	}
	html := RenderPage(page, url, "")
	if !strings.Contains(html, `class="card"`) || !strings.Contains(html, "Hello") {
		t.Fatalf("html missing card: %s", html[:min(400, len(html))])
	}
	_ = os.RemoveAll(dir)
}

func TestResolveMainDataWhereDetail(t *testing.T) {
	dir := t.TempDir()
	url := "sqlite:" + filepath.Join(dir, "t.db")
	fields := []any{
		map[string]any{"字段": "title", "类型": "text", "可空": false},
		map[string]any{"字段": "slug", "类型": "text", "可空": false},
		map[string]any{"字段": "content", "类型": "text", "可空": true},
	}
	if _, err := db.Init(url, "posts", fields); err != nil {
		t.Fatal(err)
	}
	rows := []any{
		map[string]any{"title": "A", "slug": "a", "content": "alpha"},
		map[string]any{"title": "B", "slug": "b", "content": "beta"},
	}
	if _, err := db.Insert(url, "posts", rows); err != nil {
		t.Fatal(err)
	}
	page := map[string]any{
		"data_source": "posts",
		"query":       map[string]any{"slug": "{slug}"},
		"params":      map[string]any{"slug": "b"},
		"detail":      true,
	}
	_, items, _ := resolveMain(page, url)
	if len(items) != 1 || items[0]["title"] != "B" {
		t.Fatalf("items=%v", items)
	}
	html := RenderPage(page, url, "")
	if !strings.Contains(html, "article-title") || !strings.Contains(html, "B") {
		t.Fatalf("detail html: %s", html[:min(500, len(html))])
	}
}
