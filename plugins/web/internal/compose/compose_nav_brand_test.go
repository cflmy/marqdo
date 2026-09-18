package compose_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/compose"
)

func TestComposeNavBrandAndClient(t *testing.T) {
	page := map[string]any{"title": "t"}
	out, err := compose.ComposeNavBrand(page, "求道量子", "/", "/static/logo.png", "/static/logo-light.png", "mq-theme")
	if err != nil {
		t.Fatal(err)
	}
	m := out.(map[string]any)
	b, ok := m["nav_brand"].(map[string]any)
	if !ok || b["title"] != "求道量子" || b["logo"] != "/static/logo.png" {
		t.Fatalf("nav_brand: %#v", m["nav_brand"])
	}
	out, err = compose.ComposeClient(m, "", "", "/static/client.mq.md")
	if err != nil {
		t.Fatal(err)
	}
	m = out.(map[string]any)
	c, ok := m["client"].(map[string]any)
	if !ok || c["source"] != "/static/client.mq.md" {
		t.Fatalf("client: %#v", m["client"])
	}
	if c["bridge"] != "/static/marqdo-bridge.js" {
		t.Fatalf("bridge default: %#v", c)
	}
}
