package render

import (
	"regexp"
	"strings"
	"unicode"
	"unicode/utf8"
)

// formatMetaDate shortens ISO timestamps to YYYY-MM-DD for card/article meta.
func formatMetaDate(s string) string {
	s = strings.TrimSpace(s)
	if len(s) >= 10 && s[4] == '-' && s[7] == '-' {
		if len(s) == 10 || s[10] == 'T' || s[10] == ' ' {
			return s[:10]
		}
	}
	return s
}

func navBrandSpec(page map[string]any) map[string]any {
	if page == nil {
		return nil
	}
	if b, ok := page["nav_brand"].(map[string]any); ok {
		return b
	}
	return nil
}

func themeKeyOf(page map[string]any) string {
	if b := navBrandSpec(page); b != nil {
		if k := text(b["theme_key"]); k != "" {
			return k
		}
		if k := text(b["主题键"]); k != "" {
			return k
		}
	}
	return "mq-theme"
}

func renderNavBrandHTML(page map[string]any) string {
	b := navBrandSpec(page)
	if b == nil {
		return ""
	}
	title := text(b["title"])
	if title == "" {
		title = text(b["标题"])
	}
	if title == "" {
		title = text(page["title"])
	}
	href := text(b["href"])
	if href == "" {
		href = text(b["链接"])
	}
	if href == "" {
		href = "/"
	}
	logo := text(b["logo"])
	if logo == "" {
		logo = text(b["标志"])
	}
	var s strings.Builder
	s.WriteString(`<a id="nav-brand" class="nav-brand" href="`)
	s.WriteString(esc(href))
	s.WriteString(`" aria-label="`)
	s.WriteString(esc(title))
	s.WriteString(`">`)
	if logo != "" {
		s.WriteString(`<img class="nav-brand-logo" src="`)
		s.WriteString(esc(logo))
		s.WriteString(`" alt="`)
		s.WriteString(esc(title))
		s.WriteString(`" width="36" height="36" decoding="async"/>`)
	}
	if title != "" {
		s.WriteString(`<span class="nav-brand-text">`)
		s.WriteString(esc(title))
		s.WriteString(`</span>`)
	}
	s.WriteString(`</a>`)
	s.WriteString(`<button type="button" id="nav-menu-toggle" class="nav-menu-toggle" aria-controls="site-side-drawer" aria-expanded="false" aria-label="打开目录">`)
	s.WriteString(`<span class="nav-menu-bars" aria-hidden="true"><i></i><i></i><i></i></span>`)
	s.WriteString(`<span class="nav-menu-label">目录</span></button>`)
	return s.String()
}

func renderThemeToggleHTML(page map[string]any) string {
	if navBrandSpec(page) == nil {
		return ""
	}
	return `<button type="button" id="theme-toggle" class="theme-toggle on-dark" title="切换主题" aria-label="切换到浅色模式">浅色</button>`
}

func renderNavInner(page map[string]any, linksHTML string) string {
	brand := renderNavBrandHTML(page)
	toggle := renderThemeToggleHTML(page)
	return brand + linksHTML + toggle
}

func chromeExtrasHTML(page map[string]any) string {
	if navBrandSpec(page) == nil && pageClient(page) == nil {
		return ""
	}
	return `<div id="qd-progress" aria-hidden="true"></div>` +
		`<button type="button" id="nav-drawer-veil" class="nav-drawer-veil" aria-label="关闭目录" hidden></button>`
}

func pageClient(page map[string]any) map[string]any {
	if page == nil {
		return nil
	}
	if c, ok := page["client"].(map[string]any); ok {
		return c
	}
	return nil
}

func themeBootScript(page map[string]any) string {
	if navBrandSpec(page) == nil && pageClient(page) == nil {
		return ""
	}
	key := themeKeyOf(page)
	return `<script>(function(){try{var t=localStorage.getItem("` + key + `");` +
		`if(t==="light"||t==="dark")document.documentElement.setAttribute("data-theme",t);` +
		`else document.documentElement.setAttribute("data-theme","dark");` +
		`}catch(e){document.documentElement.setAttribute("data-theme","dark");}})();</script>`
}

func clientEmbedScript(page map[string]any) string {
	c := pageClient(page)
	if c == nil {
		return ""
	}
	source := text(c["source"])
	if source == "" {
		source = text(c["源"])
	}
	if source == "" {
		return ""
	}
	bridge := text(c["bridge"])
	if bridge == "" {
		bridge = text(c["桥"])
	}
	if bridge == "" {
		bridge = "/static/marqdo-bridge.js"
	}
	wasm := text(c["wasm"])
	if wasm == "" {
		wasm = "/static/marqdo_wasm.wasm"
	}
	return `<script type="module" src="` + esc(bridge) + `" data-mq-wasm="` + esc(wasm) +
		`" data-mq-source-url="` + esc(source) + `"></script>`
}

var firstPBlockRe = regexp.MustCompile(`(?is)^(\s*)(<p\b[^>]*>)(.*?)(</p>)`)

func shouldDropcap(plain string) bool {
	plain = strings.Join(strings.Fields(plain), "")
	if plain == "" {
		return false
	}
	if strings.HasPrefix(plain, "$$") {
		return false
	}
	r, _ := utf8.DecodeRuneInString(plain)
	if r == '$' || r == '\\' || r == '[' || r == '(' || unicode.IsDigit(r) {
		return false
	}
	if unicode.In(r, unicode.Han) {
		return true
	}
	return (r >= 'A' && r <= 'Z') || (r >= 'a' && r <= 'z')
}

func stripTags(s string) string {
	var b strings.Builder
	in := false
	for _, r := range s {
		switch {
		case r == '<':
			in = true
		case r == '>':
			in = false
		case !in:
			b.WriteRune(r)
		}
	}
	return b.String()
}

// tuneDropcapHTML adds has-dropcap to the first eligible paragraph in article markdown HTML.
func tuneDropcapHTML(inner string) string {
	m := firstPBlockRe.FindStringSubmatch(inner)
	if m == nil {
		return inner
	}
	plain := stripTags(m[3])
	if !shouldDropcap(plain) {
		return inner
	}
	open := m[2]
	if strings.Contains(open, "has-dropcap") {
		return inner
	}
	if strings.Contains(open, `class="`) {
		open = strings.Replace(open, `class="`, `class="has-dropcap `, 1)
	} else if strings.Contains(open, `class='`) {
		open = strings.Replace(open, `class='`, `class='has-dropcap `, 1)
	} else {
		open = strings.TrimSuffix(open, ">") + ` class="has-dropcap">`
	}
	loc := firstPBlockRe.FindStringIndex(inner)
	if loc == nil {
		return inner
	}
	return inner[:loc[0]] + m[1] + open + m[3] + m[4] + inner[loc[1]:]
}
