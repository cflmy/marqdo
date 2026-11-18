package sitemap

import (
	"strings"
	"testing"
)

func TestBuildSitemapRowTable(t *testing.T) {
	items := []any{
		map[string]any{"loc": "/", "更新": "2026-08-28"},
		map[string]any{"loc": "/about", "更新": "2026-08-28"},
	}
	xml := BuildSitemap("https://example.com", items)
	if !strings.Contains(xml, "https://example.com/") {
		t.Fatalf("missing home loc: %s", xml)
	}
	if !strings.Contains(xml, "https://example.com/about") {
		t.Fatalf("missing about loc: %s", xml)
	}
	if !strings.Contains(xml, "<lastmod>2026-08-28</lastmod>") {
		t.Fatalf("missing lastmod: %s", xml)
	}
}

func TestBuildRobotsWithSitemap(t *testing.T) {
	body := BuildRobots("https://example.com/sitemap.xml")
	if !strings.Contains(body, "User-agent: *") {
		t.Fatalf("missing user-agent: %s", body)
	}
	if !strings.Contains(body, "Sitemap: https://example.com/sitemap.xml") {
		t.Fatalf("missing sitemap line: %s", body)
	}
}

func TestSitemapJSONShape(t *testing.T) {
	j := SitemapJSON("https://x.com", []any{map[string]any{"loc": "/a"}})
	if j["xml"] == nil || j["xml"] == "" {
		t.Fatalf("expected xml: %v", j)
	}
}
