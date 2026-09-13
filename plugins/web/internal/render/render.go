// Package render ports HTML shell rendering from the Rust web plugin.
package render

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/assets"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

func esc(s string) string {
	r := strings.NewReplacer(
		"&", "&amp;",
		"<", "&lt;",
		">", "&gt;",
		`"`, "&quot;",
	)
	return r.Replace(s)
}

func text(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case bool:
		if t {
			return "true"
		}
		return "false"
	case float64:
		if t == float64(int64(t)) {
			return fmt.Sprintf("%d", int64(t))
		}
		return fmt.Sprint(t)
	default:
		return fmt.Sprint(t)
	}
}

func classAttr(css string) string {
	if css == "" {
		return ""
	}
	return fmt.Sprintf(` class="%s"`, esc(css))
}

func isSimpleIdent(s string) bool {
	if s == "" {
		return false
	}
	for _, c := range s {
		if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' {
			continue
		}
		if c > 127 {
			continue
		}
		return false
	}
	return true
}

func isDBBindBack(back string) bool {
	s := table.NormalizeRef(back)
	if s == "" || strings.Contains(s, "://") || strings.HasPrefix(s, "/") ||
		strings.HasPrefix(s, "#") || strings.HasPrefix(s, "?") {
		return false
	}
	sp := table.ParseSitePath(s)
	switch sp.Kind {
	case table.SitePathDbField:
		return true
	case table.SitePathLibMember:
		return isSimpleIdent(sp.Lib) && isSimpleIdent(sp.Member)
	case table.SitePathPlain:
		parts := strings.Split(sp.Plain, ".")
		n := 0
		for _, p := range parts {
			if p != "" {
				n++
				if !isSimpleIdent(p) {
					return false
				}
			}
		}
		return n == 2
	}
	return false
}

type navLink struct {
	label string
	href  string
	class string
	media string
}

type navWhen int

const (
	navAlways navWhen = iota
	navHide
	navAuth
	navGuest
)

func parseNavWhen(raw string) navWhen {
	t := strings.TrimSpace(raw)
	switch t {
	case "", "*", "always", "Always", "真", "yes", "on", "show", "all":
		return navAlways
	case "hide", "never", "off", "no", "假", "否", "0", "false", "False":
		return navHide
	case "auth", "user", "登录", "已登录", "logged_in", "logged-in":
		return navAuth
	case "guest", "anon", "anonymous", "访客", "匿名", "未登录":
		return navGuest
	default:
		return navAlways
	}
}

func pageAuthState(page map[string]any) *bool {
	if page == nil {
		return nil
	}
	for _, k := range []string{"_nav_user", "user", "username", "_user"} {
		if v, ok := page[k]; ok {
			b := false
			switch t := v.(type) {
			case nil:
				b = false
			case bool:
				b = t
			case string:
				b = strings.TrimSpace(t) != ""
			case float64:
				b = int64(t) != 0
			default:
				b = true
			}
			return &b
		}
	}
	for _, k := range []string{"_logged_in", "logged_in"} {
		if v, ok := page[k]; ok {
			switch t := v.(type) {
			case bool:
				return &t
			case string:
				b := false
				switch strings.TrimSpace(t) {
				case "1", "true", "True", "yes", "on", "真", "是":
					b = true
				}
				return &b
			case float64:
				b := int64(t) != 0
				return &b
			}
		}
	}
	return nil
}

func navWhenVisible(when navWhen, auth *bool) bool {
	switch when {
	case navAlways:
		return true
	case navHide:
		return false
	case navAuth:
		if auth == nil {
			return true
		}
		return *auth
	case navGuest:
		if auth == nil {
			return true
		}
		return !*auth
	}
	return true
}

func selectPageData(url, tableName string, page map[string]any, limit, offset int64) map[string]any {
	var where any
	if q, ok := page["query"]; ok {
		where = q
	}
	order, _ := page["order"].(string)
	opts := db.SelectOpts{Where: where, Order: order}
	if _, ok := page["paginate"]; ok || offset > 0 {
		opts.Offset = &offset
	}
	out, err := db.Select(url, tableName, limit, opts)
	if err != nil {
		return map[string]any{"rows": []any{}}
	}
	return out
}

func resolveLinks(raw any, dbURL string, page map[string]any) []navLink {
	if raw == nil {
		return nil
	}
	auth := pageAuthState(page)
	bindsAny := table.AsBind(raw)
	arr, _ := bindsAny.([]any)
	var rows []map[string]any
	if len(arr) > 0 {
		hasDB := false
		for _, b := range arr {
			m, _ := b.(map[string]any)
			back, _ := m["back"].(string)
			if isDBBindBack(back) {
				hasDB = true
				break
			}
		}
		if hasDB {
			if dbURL == "" {
				return nil
			}
			tn, ok := table.BindTableName(arr)
			if !ok {
				return nil
			}
			data := selectPageData(dbURL, tn, map[string]any{}, 200, 0)
			rawRows, _ := data["rows"].([]any)
			projected := table.ProjectRows(arr, rawRows)
			parr, _ := projected.([]any)
			for _, it := range parr {
				if m, ok := it.(map[string]any); ok {
					rows = append(rows, m)
				}
			}
		} else {
			for _, b := range arr {
				if m, ok := b.(map[string]any); ok {
					rows = append(rows, m)
				}
			}
		}
	} else {
		rows = table.AsNavRows(raw)
	}

	var out []navLink
	for _, m := range rows {
		label, href := table.NavLabelHref(m)
		if label == "" {
			continue
		}
		media, whenRaw, class := table.NavMediaWhenClass(m)
		if whenRaw == "" {
			if v, ok := m["when"]; ok {
				whenRaw = text(v)
			} else if v, ok := m["当"]; ok {
				whenRaw = text(v)
			}
		}
		if media == "" {
			if v, ok := m["media"]; ok {
				media = text(v)
			} else if v, ok := m["媒体"]; ok {
				media = text(v)
			}
		}
		when := parseNavWhen(whenRaw)
		if !navWhenVisible(when, auth) {
			continue
		}
		out = append(out, navLink{label: label, href: href, class: class, media: media})
	}
	return out
}

func navMediaClassMapMany(groups ...[]navLink) (string, [][2]string) {
	var pairs [][2]string
	var css strings.Builder
	for _, links := range groups {
		for _, link := range links {
			media := strings.TrimSpace(link.media)
			if media == "" {
				continue
			}
			dup := false
			for _, p := range pairs {
				if p[0] == media {
					dup = true
					break
				}
			}
			if dup {
				continue
			}
			class := fmt.Sprintf("nav-mq-%d", len(pairs))
			css.WriteString(fmt.Sprintf(
				"@media not %s { li.%s { display:none !important; } }\n",
				media, class,
			))
			pairs = append(pairs, [2]string{media, class})
		}
	}
	return css.String(), pairs
}

func renderULWithMQ(links []navLink, class string, mqPairs [][2]string) string {
	var s strings.Builder
	s.WriteString(fmt.Sprintf(`<ul class="%s">`, esc(class)))
	for _, link := range links {
		mq := strings.TrimSpace(link.media)
		mqClass := ""
		if mq != "" {
			for _, p := range mqPairs {
				if p[0] == mq {
					mqClass = p[1]
					break
				}
			}
		}
		liClass := link.class
		if mqClass != "" {
			if liClass != "" {
				liClass += " "
			}
			liClass += mqClass
		}
		if liClass == "" {
			s.WriteString(fmt.Sprintf(`<li><a href="%s">%s</a></li>`, esc(link.href), esc(link.label)))
		} else {
			s.WriteString(fmt.Sprintf(
				`<li class="%s"><a href="%s">%s</a></li>`,
				esc(liClass), esc(link.href), esc(link.label),
			))
		}
	}
	s.WriteString("</ul>")
	return s.String()
}

func resolveMain(page map[string]any, dbURL string) (intro string, items []map[string]any, total *int64) {
	intro, _ = page["intro"].(string)
	main, ok := page["main"]
	if !ok {
		return intro, nil, nil
	}
	bindsAny := table.AsBind(main)
	arr, _ := bindsAny.([]any)
	if len(arr) == 0 {
		return intro, nil, nil
	}
	if dbURL == "" {
		return intro, nil, nil
	}
	tn, ok := table.BindTableName(arr)
	if !ok {
		return intro, nil, nil
	}
	limit := int64(200)
	offset := int64(0)
	if p, ok := page["paginate"].(map[string]any); ok {
		if v, ok := p["limit"].(float64); ok {
			limit = int64(v)
		}
		if v, ok := p["offset"].(float64); ok {
			offset = int64(v)
		}
	}
	data := selectPageData(dbURL, tn, page, limit, offset)
	if t, ok := data["total"].(float64); ok {
		ti := int64(t)
		total = &ti
	}
	rawRows, _ := data["rows"].([]any)
	projected := table.ProjectRows(arr, rawRows)
	parr, _ := projected.([]any)
	for _, v := range parr {
		if m, ok := v.(map[string]any); ok {
			items = append(items, m)
		}
	}
	return intro, items, total
}

func fieldCSS(obj map[string]any, field string) string {
	css, _ := obj["_css"].(map[string]any)
	if css == nil {
		return ""
	}
	s, _ := css[field].(string)
	return s
}

func partSrcPrefix(page map[string]any) string {
	r, _ := page["_route"].(string)
	if r == "" || r == "/" {
		return ""
	}
	for len(r) > 1 && strings.HasSuffix(r, "/") {
		r = r[:len(r)-1]
	}
	if strings.HasPrefix(r, "/") {
		return r
	}
	return "/" + r
}

func slotAttrs(slot string, parts map[string]any, page map[string]any) string {
	prefix := partSrcPrefix(page)
	a := fmt.Sprintf(` data-slot="%s"`, esc(slot))
	for id, cfg := range parts {
		cm, ok := cfg.(map[string]any)
		if !ok {
			continue
		}
		s, _ := cm["slot"].(string)
		if s == "" {
			s, _ = cm["fragment"].(string)
		}
		if s == slot || (slot == "sidebar" && id == "side") || (slot == "main" && id == "index") {
			a += fmt.Sprintf(` data-slot-src="%s/_part/%s"`, esc(prefix), esc(id))
			break
		}
	}
	return a
}

func slotClass(page map[string]any, slot, base string) string {
	extra := ""
	if sc, ok := page["slot_class"].(map[string]any); ok {
		extra, _ = sc[slot].(string)
	}
	if extra == "" {
		return base
	}
	return base + " " + extra
}

const shellVars = `
:root { --ink:#1c1917; --muted:#57534e; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; }
body { margin:0; font-family: "IBM Plex Sans", "Noto Sans SC", sans-serif; background:var(--paper); color:var(--ink); }
a { color:var(--ink); text-decoration:none; }
a:hover { color:var(--accent); }
`

const shellLayout = `
body { display:grid; min-height:100vh; grid-template-rows:auto 1fr auto; }
body.has-sidebar { grid-template-columns:14rem 1fr; grid-template-areas:"top top" "side main" "foot foot"; }
body.no-sidebar { grid-template-areas:"top" "main" "foot"; }
@media (max-width:720px) {
  body.has-sidebar { grid-template-columns:1fr; grid-template-areas:"top" "main" "foot" "side"; }
  body.has-sidebar aside.side { border-right:0; border-top:1px solid var(--line); }
}
body.layout-stacked { display:flex; flex-direction:column; min-height:100vh; }
body.layout-stacked header.topnav, body.layout-stacked aside.side, body.layout-stacked main.main, body.layout-stacked footer.foot { width:100%; }
body.layout-stacked aside.side { border-right:0; border-bottom:1px solid var(--line); }
body.layout-bare { display:block; min-height:100vh; }
body.layout-bare main.main { padding:1.5rem 1.25rem 2rem; }
header.topnav { grid-area:top; border-bottom:1px solid var(--line); padding:.85rem 1.25rem; background:#fff; }
aside.side { grid-area:side; border-right:1px solid var(--line); padding:1.25rem 1rem; background:#f5f5f4; }
main.main { grid-area:main; padding:1.5rem 1.25rem 2rem; }
footer.foot { grid-area:foot; border-top:1px solid var(--line); padding:.85rem 1.25rem; color:var(--muted); background:#fff; }
ul.nav, ul.side-nav, ul.foot-nav { list-style:none; margin:0; padding:0; display:flex; flex-wrap:wrap; gap:.35rem 1rem; }
ul.side-nav { flex-direction:column; }
`

const shellWidgets = `
.content.cards { display:grid; gap:1rem; margin-top:1.25rem; grid-template-columns:repeat(auto-fill,minmax(16rem,1fr)); }
.content.cards article { background:#fff; border:1px solid var(--line); border-radius:6px; padding:1rem; }
.main-intro h1 { margin:0 0 .75rem; font-size:2rem; }
.main-intro p { color:var(--muted); }
.site-form { margin-top:1.25rem; max-width:28rem; }
.article { background:#fff; border:1px solid var(--line); border-radius:8px; padding:2rem; margin-top:1.5rem; }
.article-title { margin:0 0 .75rem; font-size:2rem; }
.pagination { display:flex; gap:1rem; align-items:center; margin:1.5rem 0; font-size:.95rem; }
`

func shellCSSFor(mode string) string {
	switch strings.ToLower(strings.TrimSpace(mode)) {
	case "off", "none", "false", "0":
		return ""
	case "minimal", "min", "vars":
		return shellVars
	default:
		return shellVars + shellLayout + shellWidgets
	}
}

func resolveShellCSS(page map[string]any) string {
	raw := "full"
	if s, ok := page["shell_css"].(string); ok {
		raw = s
	} else if s, ok := page["壳样式"].(string); ok {
		raw = s
	}
	return shellCSSFor(raw)
}

func normalizeLayout(raw string) string {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "", "auto", "default":
		return ""
	case "sidebar", "side":
		return "sidebar"
	case "stacked", "stack", "column":
		return "stacked"
	case "bare", "main", "none":
		return "bare"
	case "rail":
		return "rail"
	default:
		return raw
	}
}

func resolveLayout(page map[string]any) string {
	raw := ""
	if s, ok := page["layout"].(string); ok {
		raw = s
	} else if s, ok := page["布局"].(string); ok {
		raw = s
	}
	return normalizeLayout(raw)
}

func headHTML(page map[string]any, defaultTitle string) string {
	pageTitle := defaultTitle
	if meta, ok := page["meta"].(map[string]any); ok {
		if t := text(meta["title"]); t != "" {
			pageTitle = t
		}
	}
	var s strings.Builder
	s.WriteString(fmt.Sprintf("<title>%s</title>", esc(pageTitle)))
	if meta, ok := page["meta"].(map[string]any); ok {
		for k, v := range meta {
			if k == "title" {
				continue
			}
			val := text(v)
			if val == "" {
				continue
			}
			switch {
			case k == "description":
				s.WriteString(fmt.Sprintf(`<meta name="description" content="%s"/>`, esc(val)))
			case k == "canonical":
				s.WriteString(fmt.Sprintf(`<link rel="canonical" href="%s"/>`, esc(val)))
			case k == "icon" || k == "favicon":
				s.WriteString(fmt.Sprintf(`<link rel="icon" href="%s"/>`, esc(val)))
			case strings.HasPrefix(k, "og:"):
				s.WriteString(fmt.Sprintf(`<meta property="%s" content="%s"/>`, esc(k), esc(val)))
			default:
				s.WriteString(fmt.Sprintf(`<meta name="%s" content="%s"/>`, esc(k), esc(val)))
			}
		}
	}
	if head, ok := page["head"]; ok {
		links := assets.HeadLinksFromJSON(head)
		assetVersion := ""
		if v, ok := page["asset_version"].(string); ok {
			assetVersion = v
		} else if v, ok := page["资源版本"].(string); ok {
			assetVersion = v
		}
		s.WriteString(assets.RenderHeadLinksWithVersion(links, assetVersion))
	}
	return s.String()
}

func renderCard(page map[string]any, it map[string]any) string {
	title := text(it["title"])
	body := text(it["body"])
	href := text(it["href"])
	meta := text(it["meta"])
	tag := text(it["tag"])
	tc := classAttr(fieldCSS(it, "title"))
	bc := classAttr(fieldCSS(it, "body"))
	var card strings.Builder
	card.WriteString(`<article class="card">`)
	if href != "" {
		prefix := "/post/"
		if p, ok := page["link_prefix"].(string); ok {
			prefix = p
		}
		card.WriteString(fmt.Sprintf(`<a class="card-link" href="%s">`, esc(prefix+href)))
	}
	if meta != "" {
		card.WriteString(fmt.Sprintf(`<div class="card-meta">%s</div>`, esc(meta)))
	}
	card.WriteString(fmt.Sprintf("<h2%s>%s</h2>", tc, esc(title)))
	if tag != "" {
		card.WriteString(fmt.Sprintf(`<div class="card-tag">%s</div>`, esc(tag)))
	}
	if body != "" {
		card.WriteString(fmt.Sprintf("<p%s>%s</p>", bc, esc(body)))
	}
	if href != "" {
		card.WriteString("</a>")
	}
	card.WriteString("</article>")
	return card.String()
}

func renderArticle(it map[string]any) string {
	title := text(it["title"])
	body := text(it["body"])
	meta := text(it["meta"])
	tag := text(it["tag"])
	var s strings.Builder
	s.WriteString(`<article class="article">`)
	if meta != "" {
		s.WriteString(fmt.Sprintf(`<div class="article-meta">%s</div>`, esc(meta)))
	}
	s.WriteString(fmt.Sprintf(`<h1 class="article-title">%s</h1>`, esc(title)))
	if tag != "" {
		s.WriteString(fmt.Sprintf(`<div class="article-tags">%s</div>`, esc(tag)))
	}
	if body != "" {
		// Minimal: escape as paragraphs (full markdown deferred).
		s.WriteString(fmt.Sprintf(`<div class="article-body"><p class="article-p">%s</p></div>`, esc(body)))
	}
	s.WriteString("</article>")
	return s.String()
}

func renderPagination(page map[string]any, total int64, itemCount int) string {
	p, ok := page["paginate"].(map[string]any)
	if !ok {
		return ""
	}
	offset := int64(0)
	limit := int64(10)
	if v, ok := p["offset"].(float64); ok {
		offset = int64(v)
	}
	if v, ok := p["limit"].(float64); ok {
		limit = int64(v)
	}
	if limit <= 0 {
		return ""
	}
	path := "/"
	if s, ok := p["path"].(string); ok {
		path = s
	}
	var s strings.Builder
	s.WriteString(`<nav class="pagination" aria-label="Pagination">`)
	if offset > 0 {
		prev := offset - limit
		if prev < 0 {
			prev = 0
		}
		s.WriteString(fmt.Sprintf(
			`<a class="page-prev" href="%s?offset=%d">← Previous</a> `,
			esc(path), prev,
		))
	}
	end := offset + int64(itemCount)
	statusEnd := end
	if statusEnd > total {
		statusEnd = total
	}
	s.WriteString(fmt.Sprintf(
		`<span class="page-status">%d–%d of %d</span> `,
		offset+1, statusEnd, total,
	))
	if end < total {
		s.WriteString(fmt.Sprintf(
			`<a class="page-next" href="%s?offset=%d">Next →</a>`,
			esc(path), end,
		))
	}
	s.WriteString("</nav>")
	return s.String()
}

func formMountID(target string) string {
	t := strings.TrimSpace(strings.TrimPrefix(strings.TrimSpace(target), "#"))
	if t == "" {
		return ""
	}
	for _, c := range t {
		if c == ' ' || c == '\t' || c == '\n' || c == '"' || c == '\'' || c == '<' || c == '>' {
			return ""
		}
	}
	return t
}

// injectHTMLIntoID puts payload inside the first element with id="…" / id='…'.
func injectHTMLIntoID(haystack, id, payload string) (string, bool) {
	id = formMountID(id)
	if id == "" {
		return "", false
	}
	patterns := []string{`id="` + id + `"`, `id='` + id + `'`}
	for _, pat := range patterns {
		idx := strings.Index(haystack, pat)
		if idx < 0 {
			continue
		}
		tagStart := strings.LastIndex(haystack[:idx], "<")
		if tagStart < 0 {
			continue
		}
		if tagStart+1 >= len(haystack) {
			continue
		}
		afterLt := haystack[tagStart+1]
		if afterLt == '/' || afterLt == '!' {
			continue
		}
		nameEnd := tagStart + 1
		for nameEnd < len(haystack) {
			c := haystack[nameEnd]
			if c == ' ' || c == '\t' || c == '\n' || c == '>' || c == '/' {
				break
			}
			nameEnd++
		}
		tagName := strings.TrimSpace(haystack[tagStart+1 : nameEnd])
		if tagName == "" {
			continue
		}
		afterID := idx + len(pat)
		rel := strings.IndexByte(haystack[afterID:], '>')
		if rel < 0 {
			continue
		}
		openGt := afterID + rel
		if openGt > 0 && haystack[openGt-1] == '/' {
			continue
		}
		close := "</" + tagName + ">"
		rest := haystack[openGt+1:]
		closeRel := strings.Index(strings.ToLower(rest), strings.ToLower(close))
		if closeRel < 0 {
			continue
		}
		closeIdx := openGt + 1 + closeRel
		var out strings.Builder
		out.Grow(len(haystack) + len(payload))
		out.WriteString(haystack[:openGt+1])
		out.WriteString(payload)
		out.WriteString(haystack[closeIdx:])
		return out.String(), true
	}
	return "", false
}

func pageFormHTML(page map[string]any) (formID string, html string) {
	if page == nil {
		return "", ""
	}
	id, _ := page["form_id"].(string)
	id = strings.TrimSpace(id)
	if id == "" {
		return "", ""
	}
	frm, ok := page["form"].(map[string]any)
	if !ok || frm == nil {
		return "", ""
	}
	return id, form.RenderBody(frm, id, nil, nil, "")
}

func pushIntro(buf *strings.Builder, intro string) {
	pushIntroAndForm(buf, intro, nil)
}

// pushIntroAndForm writes intro and optional composed form (GAP-11 form_target inject).
func pushIntroAndForm(buf *strings.Builder, intro string, page map[string]any) {
	formID, formHTML := pageFormHTML(page)
	target := ""
	if page != nil {
		if t, ok := page["form_target"].(string); ok {
			target = t
		} else if t, ok := page["target"].(string); ok {
			target = t
		} else if t, ok := page["form_slot"].(string); ok {
			target = t
		} else if t, ok := page["表单插槽"].(string); ok {
			target = t
		}
	}
	if formHTML != "" && target != "" && intro != "" {
		if injected, ok := injectHTMLIntoID(intro, target, formHTML); ok {
			buf.WriteString(fmt.Sprintf(`<div class="main-intro">%s</div>`, injected))
			return
		}
	}
	if intro != "" {
		buf.WriteString(fmt.Sprintf(`<div class="main-intro">%s</div>`, intro))
	}
	if formHTML != "" {
		buf.WriteString(formHTML)
		_ = formID
	}
}

func renderFragment(page map[string]any, dbURL, slot string) string {
	slot = table.NormalizeSlot(slot)
	switch slot {
	case "nav":
		links := resolveLinks(page["nav"], dbURL, page)
		return fmt.Sprintf(
			`<header class="topnav" data-slot="nav">%s</header>`,
			renderULWithMQ(links, "nav", nil),
		)
	case "sidebar":
		links := resolveLinks(page["sidebar"], dbURL, page)
		return fmt.Sprintf(
			`<aside class="side" data-slot="sidebar"><span class="side-label">侧栏</span>%s</aside>`,
			renderULWithMQ(links, "side-nav", nil),
		)
	case "footer":
		links := resolveLinks(page["footer"], dbURL, page)
		return fmt.Sprintf(
			`<footer class="foot" data-slot="footer">%s</footer>`,
			renderULWithMQ(links, "foot-nav", nil),
		)
	default:
		intro, items, _ := resolveMain(page, dbURL)
		var body strings.Builder
		pushIntroAndForm(&body, intro, page)
		if len(items) > 0 {
			isDetail, _ := page["detail"].(bool)
			if isDetail {
				body.WriteString(renderArticle(items[0]))
			} else {
				body.WriteString(`<section class="content cards">`)
				for _, it := range items {
					body.WriteString(renderCard(page, it))
				}
				body.WriteString(`</section>`)
			}
		}
		return fmt.Sprintf(`<main class="main" data-slot="main">%s</main>`, body.String())
	}
}

// RenderPage builds full-page HTML (or a part fragment when partID is set).
func RenderPage(page map[string]any, dbURL string, partID string) string {
	if page == nil {
		page = map[string]any{}
	}
	if partID != "" {
		parts, _ := page["parts"].(map[string]any)
		if parts != nil {
			if cfg, ok := parts[partID].(map[string]any); ok {
				merged := map[string]any{}
				for k, v := range page {
					merged[k] = v
				}
				for k, v := range cfg {
					merged[k] = v
				}
				slot, _ := cfg["slot"].(string)
				if slot == "" {
					slot, _ = cfg["fragment"].(string)
				}
				if slot == "" {
					slot = "main"
				}
				return renderFragment(merged, dbURL, slot)
			}
		}
		return renderFragment(page, dbURL, partID)
	}

	title := "Marqdo Web"
	if t, ok := page["title"].(string); ok && t != "" {
		title = t
	}
	shell := resolveShellCSS(page)
	layout := resolveLayout(page)
	parts, _ := page["parts"].(map[string]any)
	if parts == nil {
		parts = map[string]any{}
	}
	nav := resolveLinks(page["nav"], dbURL, page)
	side := resolveLinks(page["sidebar"], dbURL, page)
	foot := resolveLinks(page["footer"], dbURL, page)
	intro, items, total := resolveMain(page, dbURL)
	navCSS, mqPairs := navMediaClassMapMany(nav, side, foot)
	extra, _ := page["styles_css"].(string)
	if navCSS != "" {
		if extra != "" {
			extra += "\n"
		}
		extra += navCSS
	}

	_, hasSideKey := page["sidebar"]
	hasSide := hasSideKey || len(side) > 0
	bare := layout == "bare"
	var bodyClass string
	switch layout {
	case "bare":
		bodyClass = "layout-bare"
	case "stacked":
		bodyClass = "layout-stacked"
	case "rail":
		if hasSide {
			bodyClass = "has-rail has-sidebar"
		} else {
			bodyClass = "has-rail no-sidebar"
		}
	case "sidebar":
		if hasSide {
			bodyClass = "has-sidebar"
		} else {
			bodyClass = "no-sidebar"
		}
	case "":
		if hasSide {
			bodyClass = "has-sidebar"
		} else {
			bodyClass = "no-sidebar"
		}
	default:
		bodyClass = layout
	}

	var mainHTML strings.Builder
	if images, ok := page["images_html"].(string); ok && images != "" {
		mainHTML.WriteString(images)
	}
	pushIntroAndForm(&mainHTML, intro, page)
	if len(items) > 0 {
		isDetail, _ := page["detail"].(bool)
		if isDetail {
			mainHTML.WriteString(renderArticle(items[0]))
		} else {
			mainHTML.WriteString(`<section class="content cards">`)
			for _, it := range items {
				mainHTML.WriteString(renderCard(page, it))
			}
			mainHTML.WriteString(`</section>`)
		}
		if total != nil {
			mainHTML.WriteString(renderPagination(page, *total, len(items)))
		}
	}

	styleBlock := ""
	if shell != "" || extra != "" {
		styleBlock = fmt.Sprintf("<style>%s%s</style>", shell, extra)
	}

	showChrome := !bare
	sideHTML := ""
	if showChrome && hasSide {
		sideHTML = fmt.Sprintf(
			`<aside class="%s"%s><span class="side-label">侧栏</span>%s</aside>`,
			slotClass(page, "sidebar", "side"),
			slotAttrs("sidebar", parts, page),
			renderULWithMQ(side, "side-nav", mqPairs),
		)
	}
	headerHTML := ""
	if showChrome {
		if raw, ok := page["nav_html"].(string); ok && strings.TrimSpace(raw) != "" {
			headerHTML = raw
		} else {
			headerHTML = fmt.Sprintf(
				`<header class="%s"%s>%s</header>`,
				slotClass(page, "nav", "topnav"),
				slotAttrs("nav", parts, page),
				renderULWithMQ(nav, "nav", mqPairs),
			)
		}
	}
	footerHTML := ""
	if showChrome {
		if raw, ok := page["footer_html"].(string); ok && strings.TrimSpace(raw) != "" {
			footerHTML = raw
		} else {
			footerHTML = fmt.Sprintf(
				`<footer class="%s"%s>%s</footer>`,
				slotClass(page, "footer", "foot"),
				slotAttrs("footer", parts, page),
				renderULWithMQ(foot, "foot-nav", mqPairs),
			)
		}
	}

	if extraBody, ok := page["body_class"].(string); ok && strings.TrimSpace(extraBody) != "" {
		if bodyClass == "" {
			bodyClass = strings.TrimSpace(extraBody)
		} else {
			bodyClass = bodyClass + " " + strings.TrimSpace(extraBody)
		}
	}

	return fmt.Sprintf(`<!DOCTYPE html>
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
%s
%s
</head>
<body class="%s">
%s
%s
<main class="%s"%s>%s</main>
%s
</body></html>`,
		headHTML(page, title),
		styleBlock,
		bodyClass,
		headerHTML,
		sideHTML,
		slotClass(page, "main", "main"),
		slotAttrs("main", parts, page),
		mainHTML.String(),
		footerHTML,
	)
}
