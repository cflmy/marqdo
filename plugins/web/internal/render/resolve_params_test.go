package render

import (
	"path/filepath"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
)

func TestResolvePlaceholdersColumnarAndRows(t *testing.T) {
	params := map[string]any{"slug": "hello-marqdo"}

	col := map[string]any{
		"字段": []any{"slug"},
		"操作": []any{"="},
		"值":  []any{"{slug}"},
	}
	got := resolvePlaceholders(col, params).(map[string]any)
	vals := got["值"].([]any)
	if vals[0] != "hello-marqdo" {
		t.Fatalf("columnar 值=%v", vals)
	}

	rows := []any{
		map[string]any{"字段": "slug", "操作": "=", "值": "{slug}"},
	}
	gotRows := resolvePlaceholders(rows, params).([]any)
	m := gotRows[0].(map[string]any)
	if m["值"] != "hello-marqdo" {
		t.Fatalf("row 值=%v", m["值"])
	}

	top := map[string]any{"slug": "{slug}"}
	gotTop := resolvePlaceholders(top, params).(map[string]any)
	if gotTop["slug"] != "hello-marqdo" {
		t.Fatalf("map string=%v", gotTop["slug"])
	}
}

func TestSelectPageDataResolvesSlug(t *testing.T) {
	dir := t.TempDir()
	dbPath := filepath.Join(dir, "t.db")
	url := "sqlite:" + dbPath
	t.Cleanup(db.ResetPool)

	fields := []any{
		map[string]any{"name": "slug", "type": "text", "nullable": false},
		map[string]any{"name": "title", "type": "text", "nullable": false},
		map[string]any{"name": "body", "type": "text", "nullable": true},
	}
	if _, err := db.Init(url, "posts", fields); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Insert(url, "posts", []any{
		map[string]any{"slug": "hello-marqdo", "title": "Hello", "body": "## Hi\n\n*world*"},
	}); err != nil {
		t.Fatal(err)
	}

	page := map[string]any{
		"title":  "post",
		"detail": true,
		"_route": "/post/{slug}",
		"params": map[string]any{"slug": "hello-marqdo"},
		"query": map[string]any{
			"字段": []any{"slug"},
			"操作": []any{"="},
			"值":  []any{"{slug}"},
		},
		"main": []any{
			map[string]any{"front": "title", "back": "posts.title"},
			map[string]any{"front": "body", "back": "posts.body"},
		},
		"parts": map[string]any{
			"index": map[string]any{"slot": "main"},
		},
	}
	html := RenderPage(page, url, "")
	if !strings.Contains(html, `class="article-title"`) || !strings.Contains(html, "Hello") {
		t.Fatalf("missing article title: %s", html)
	}
	if !strings.Contains(html, `class="article-body md"`) {
		t.Fatalf("missing markdown body wrapper: %s", html)
	}
	if !strings.Contains(html, "world") {
		t.Fatalf("missing markdown content: %s", html)
	}
	if strings.Contains(html, "{slug}") {
		t.Fatalf("literal {slug} still present: %s", html)
	}
	if !strings.Contains(html, `/post/hello-marqdo/_part/`) {
		t.Fatalf("slot prefix not resolved: %s", html)
	}
}

func TestMarkdownToHTML(t *testing.T) {
	out := markdownToHTML("## Title\n\n**bold** and `code`")
	if !strings.Contains(out, `class="article-body md"`) {
		t.Fatalf("%s", out)
	}
	if !strings.Contains(out, "<h2") || !strings.Contains(out, "<strong>bold</strong>") {
		t.Fatalf("%s", out)
	}
}

func TestCardHrefAbsoluteAndPrefixed(t *testing.T) {
	page := map[string]any{"link_prefix": "/post/"}
	if got := cardHref(page, "https://example.com/a"); got != "https://example.com/a" {
		t.Fatalf("absolute: %q", got)
	}
	if got := cardHref(page, "/about"); got != "/about" {
		t.Fatalf("root: %q", got)
	}
	if got := cardHref(page, "hello"); got != "/post/hello" {
		t.Fatalf("prefixed: %q", got)
	}
	html := renderCard(page, map[string]any{"title": "T", "href": "https://ex.test/x", "body": "b"})
	if !strings.Contains(html, `href="https://ex.test/x"`) {
		t.Fatalf("card absolute: %s", html)
	}
}
