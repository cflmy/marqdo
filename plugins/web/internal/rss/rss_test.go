package rss

import (
	"strings"
	"testing"
)

func TestBuildRSSContainsItem(t *testing.T) {
	items := []any{
		map[string]any{"title": "Hi", "slug": "/post/a", "summary": "text"},
	}
	xml := BuildRSS("Blog", "http://example.com", "Feed", items)
	if !strings.Contains(xml, "<rss") {
		t.Fatalf("missing rss root: %s", xml)
	}
	if !strings.Contains(xml, "<title>Hi</title>") {
		t.Fatalf("missing item title: %s", xml)
	}
	if !strings.Contains(xml, "http://example.com/post/a") {
		t.Fatalf("missing full link: %s", xml)
	}
}
