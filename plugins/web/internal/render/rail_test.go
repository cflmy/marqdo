package render

import (
	"strings"
	"testing"
)

func TestIsRailTarget(t *testing.T) {
	for _, in := range []string{"rail", "轨", "新闻轨", "#side-rail", "aside.side-rail"} {
		if !isRailTarget(in) {
			t.Fatalf("expected rail: %q", in)
		}
	}
	if isRailTarget("#comment-list") {
		t.Fatal("comment list must not be rail")
	}
}

func TestRenderNewsRail(t *testing.T) {
	html := renderNewsRail([]map[string]any{
		{"title": "A", "href": "https://ex.test/a", "meta": "2026-09-01", "tag": "IBM"},
	})
	for _, want := range []string{`side-rail--news`, `https://ex.test/a`, `target="_blank"`, `量子新闻`, `全部 →`} {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in %s", want, html)
		}
	}
}
