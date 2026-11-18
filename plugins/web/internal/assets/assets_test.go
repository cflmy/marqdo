package assets

import (
	"strings"
	"testing"
)

func TestImagesBasic(t *testing.T) {
	table := []any{
		map[string]any{"源": "/a.png", "替代": "A", "类": "logo", "加载": "eager"},
	}
	h := MakeImagesHTML(table)
	if !strings.Contains(h, "mq-images") {
		t.Fatalf("missing mq-images: %s", h)
	}
	if !strings.Contains(h, `src="/a.png"`) {
		t.Fatalf("missing src: %s", h)
	}
	if !strings.Contains(h, `loading="eager"`) {
		t.Fatalf("missing loading: %s", h)
	}
	if !strings.Contains(h, `class="mq-img logo"`) {
		t.Fatalf("missing class: %s", h)
	}
}

func TestHeadScriptAndIcon(t *testing.T) {
	table := []any{
		map[string]any{"关系": "icon", "地址": "/favicon.ico", "类型": "image/x-icon"},
		map[string]any{"关系": "script", "地址": "/static/a.js"},
	}
	h := MakeHeadHTML(table)
	if !strings.Contains(h, `rel="icon"`) {
		t.Fatalf("missing icon: %s", h)
	}
	if !strings.Contains(h, `<script src="/static/a.js"></script>`) {
		t.Fatalf("missing script: %s", h)
	}
	if strings.Contains(h, " defer") {
		t.Fatalf("unexpected defer: %s", h)
	}
}

func TestHeadDeferVersion(t *testing.T) {
	table := []any{
		map[string]any{"关系": "script", "地址": "/static/a.js", "推迟": true, "版本": "3"},
	}
	links := AsHeadLinks(table)
	h := RenderHeadLinksWithVersion(links, "2026-09-04")
	if !strings.Contains(h, `<script defer src="/static/a.js?v=3"></script>`) {
		t.Fatalf("defer+version: %s", h)
	}
}

func TestNormalizeIcons(t *testing.T) {
	table := []any{
		map[string]any{
			"path": "web-fixtures/public/favicon.png", "rel": "icon",
			"type": "image/png", "sizes": "32x32", "url": "/favicon.ico",
		},
	}
	icons, head, routes := NormalizeIcons(table)
	if len(icons) != 1 {
		t.Fatalf("icons=%d", len(icons))
	}
	if len(head) == 0 {
		t.Fatal("expected site head")
	}
	if len(routes) == 0 {
		t.Fatal("expected routes")
	}
}
