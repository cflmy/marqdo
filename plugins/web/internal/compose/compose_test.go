package compose_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/compose"
)

func TestComposeIntroBindAndRichText(t *testing.T) {
	page := map[string]any{"title": "Home"}
	intro := []any{
		map[string]any{"属性": "kicker", "值": "WWW Demo", "样式": "shell.kicker"},
		map[string]any{"属性": "title", "值": "Marqdo", "样式": "shell.intro_title"},
		map[string]any{"属性": "lede", "值": "Zero author `JS`.", "样式": "shell.lede"},
		map[string]any{"属性": "claim", "值": "0 JS", "样式": "shell.claim"},
		map[string]any{"属性": "claim", "值": "GFM", "样式": "shell.claim"},
		map[string]any{"属性": "step", "值": "Open [New](/new) now.", "样式": "shell.step"},
		map[string]any{"属性": "step", "值": "Compare source.", "样式": "shell.step"},
	}
	callLib := func(path string) (any, error) {
		switch path {
		case "shell.kicker", "shell.intro_title", "shell.lede", "shell.claim", "shell.step":
			return map[string]any{
				"property": []any{"color"},
				"value":    []any{"red"},
			}, nil
		default:
			t.Fatalf("unexpected callLib %q", path)
			return nil, nil
		}
	}
	out, err := compose.ComposeIntro(page, intro, callLib)
	if err != nil {
		t.Fatal(err)
	}
	obj := out.(map[string]any)
	html, _ := obj["intro"].(string)
	for _, want := range []string{
		`class="kicker"`,
		`<h1 class="intro_title">Marqdo</h1>`,
		`<code>JS</code>`,
		`<p class="claim">`,
		`<span class="claim">0 JS</span>`,
		`<ol class="steps">`,
		`<a href="/new">New</a>`,
	} {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in html=%q", want, html)
		}
	}
	css, _ := obj["styles_css"].(string)
	if !strings.Contains(css, ".kicker") || !strings.Contains(css, ".intro_title") {
		t.Fatalf("styles_css incomplete: %q", css)
	}
}

func TestMakeIntroHTMLZHAliases(t *testing.T) {
	tbl := []any{
		map[string]any{"属性": "眉题", "值": "演示", "样式": "kicker"},
		map[string]any{"属性": "标题", "值": "站名", "样式": "intro_title"},
		map[string]any{"属性": "标签", "值": "A", "样式": "claim"},
		map[string]any{"属性": "步骤", "值": "第一步", "样式": "step"},
	}
	html := compose.MakeIntroHTML(tbl)
	for _, want := range []string{
		`class="kicker"`,
		`<h1 class="intro_title">站名</h1>`,
		`<p class="claim">`,
		`<ol class="steps">`,
	} {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in %q", want, html)
		}
	}
}

func TestComposeComponentsFakeCallLib(t *testing.T) {
	page := map[string]any{
		"title": "Home",
	}
	layout := []any{
		map[string]any{"组件": "site.nav", "样式": ""},
		map[string]any{"组件": "site.footer", "样式": ""},
	}
	callLib := func(path string) (any, error) {
		switch path {
		case "site.nav":
			return map[string]any{
				"属性": []any{"Home", "About"},
				"值":  []any{"/", "/about"},
			}, nil
		case "site.footer":
			return []any{
				map[string]any{"属性": "©", "值": "/"},
			}, nil
		default:
			t.Fatalf("unexpected callLib %q", path)
			return nil, nil
		}
	}
	out, err := compose.ComposeComponents(page, layout, callLib)
	if err != nil {
		t.Fatal(err)
	}
	obj := out.(map[string]any)
	nav, ok := obj["nav"].([]any)
	if !ok || len(nav) != 2 {
		t.Fatalf("nav=%v", obj["nav"])
	}
	n0 := nav[0].(map[string]any)
	if n0["front"] != "Home" || n0["back"] != "/" {
		t.Fatalf("nav[0]=%v", n0)
	}
	foot, ok := obj["footer"].([]any)
	if !ok || len(foot) != 1 {
		t.Fatalf("footer=%v", obj["footer"])
	}
	comp, ok := obj["compose"].([]any)
	if !ok || len(comp) != 2 {
		t.Fatalf("compose=%v", obj["compose"])
	}
	c0 := comp[0].(map[string]any)
	if c0["src"] != "nav" || c0["slot"] != "nav" {
		t.Fatalf("compose[0]=%v", c0)
	}
	parts, ok := obj["parts"].(map[string]any)
	if !ok {
		t.Fatalf("parts missing: %v", obj)
	}
	if _, ok := parts["nav"]; !ok {
		t.Fatalf("parts.nav missing: %v", parts)
	}
	if _, ok := parts["footer"]; !ok {
		t.Fatalf("parts.footer missing: %v", parts)
	}
}
