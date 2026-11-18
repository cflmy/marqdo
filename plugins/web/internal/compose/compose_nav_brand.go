package compose

import (
	"strings"
)

// ComposeNavBrand stamps page.nav_brand for SSR topnav lockup (logo + title),
// theme toggle, and drawer button chrome. Client WASM only wires behavior.
func ComposeNavBrand(page any, title, href, logo, logoLight, themeKey string) (any, error) {
	obj := asObject(page)
	spec := map[string]any{}
	if s := strings.TrimSpace(title); s != "" {
		spec["title"] = s
	}
	if s := strings.TrimSpace(href); s != "" {
		spec["href"] = s
	} else {
		spec["href"] = "/"
	}
	if s := strings.TrimSpace(logo); s != "" {
		spec["logo"] = s
	}
	if s := strings.TrimSpace(logoLight); s != "" {
		spec["logo_light"] = s
	}
	if s := strings.TrimSpace(themeKey); s != "" {
		spec["theme_key"] = s
	} else {
		spec["theme_key"] = "mq-theme"
	}
	obj["nav_brand"] = spec
	return obj, nil
}

// ComposeClient stamps page.client so render injects official bridge auto-mount
// (data-mq-wasm / data-mq-source-url). Authors ship client.mq.md, not business JS.
func ComposeClient(page any, bridge, wasm, source string) (any, error) {
	obj := asObject(page)
	spec := map[string]any{}
	if s := strings.TrimSpace(bridge); s != "" {
		spec["bridge"] = s
	} else {
		spec["bridge"] = "/static/marqdo-bridge.js"
	}
	if s := strings.TrimSpace(wasm); s != "" {
		spec["wasm"] = s
	} else {
		spec["wasm"] = "/static/marqdo_wasm.wasm"
	}
	if s := strings.TrimSpace(source); s != "" {
		spec["source"] = s
	}
	obj["client"] = spec
	return obj, nil
}
