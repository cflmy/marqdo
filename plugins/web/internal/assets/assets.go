// Package assets ports head/images/icons HTML assembly (W8 / W-G6).
package assets

import (
	"fmt"
	"path"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// HeadLink is one <link> or <script> resource.
type HeadLink struct {
	Rel         string
	Href        string
	Type        string
	Sizes       string
	Media       string
	As          string
	CrossOrigin string
	Defer       *bool
	Async       bool
	Version     string
}

// IconRoute maps a URL to a filesystem path for static serving.
type IconRoute struct {
	URL         string
	Path        string
	ContentType string
}

var (
	relKeys   = []string{"关系", "rel", "Rel"}
	hrefKeys  = []string{"地址", "href", "src", "url", "Href", "Src", "URL"}
	typeKeys  = []string{"类型", "type", "Type"}
	sizesKeys = []string{"尺寸", "sizes", "Sizes"}
	mediaKeys = []string{"媒体", "media", "Media"}
	asKeys    = []string{"作为", "as", "As"}
	corsKeys  = []string{"跨域", "crossorigin", "crossOrigin"}
	deferKeys = []string{"推迟", "defer", "Defer"}
	asyncKeys = []string{"异步", "async", "Async"}
	verKeys   = []string{"版本", "version", "Version", "v"}
	pathKeys  = []string{"路径", "path", "file", "Path"}
	urlKeys   = []string{"地址", "url", "href", "Href", "URL"}
	imgSrc    = []string{"源", "src", "Src", "url", "URL"}
	imgAlt    = []string{"替代", "alt", "Alt"}
	imgTitle  = []string{"标题", "title", "Title"}
	imgClass  = []string{"类", "class", "Class"}
	imgHref   = []string{"链接", "href", "link", "Href", "Link"}
	imgWidth  = []string{"宽度", "width", "Width"}
	imgHeight = []string{"高度", "height", "Height"}
	imgLoad   = []string{"加载", "loading", "Loading"}
	imgCap    = []string{"图注", "caption", "Caption"}
)

func cellStr(v any) string {
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
		return fmt.Sprintf("%v", t)
	default:
		return fmt.Sprint(t)
	}
}

func pick(m map[string]any, keys []string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v, true
		}
	}
	return nil, false
}

func cellBoolish(v any) *bool {
	switch t := v.(type) {
	case bool:
		b := t
		return &b
	case float64:
		b := int64(t) != 0
		return &b
	case string:
		s := strings.TrimSpace(t)
		if s == "" {
			return nil
		}
		switch s {
		case "1", "true", "True", "yes", "on", "真", "是":
			b := true
			return &b
		case "0", "false", "False", "no", "off", "假", "否":
			b := false
			return &b
		}
	}
	return nil
}

func rowBool(m map[string]any, keys []string) *bool {
	if v, ok := pick(m, keys); ok {
		return cellBoolish(v)
	}
	return nil
}

func rowStr(m map[string]any, keys []string) string {
	if v, ok := pick(m, keys); ok {
		return strings.TrimSpace(table.NormalizeRef(cellStr(v)))
	}
	return ""
}

func rowsOf(v any) []map[string]any {
	switch t := v.(type) {
	case []any:
		out := make([]map[string]any, 0, len(t))
		for _, row := range t {
			if m, ok := row.(map[string]any); ok {
				out = append(out, m)
			}
		}
		return out
	case map[string]any:
		keys := make([]string, 0, len(t))
		for k := range t {
			keys = append(keys, k)
		}
		if len(keys) == 0 {
			return nil
		}
		maxLen := 0
		allArrays := true
		for _, v := range t {
			a, ok := v.([]any)
			if !ok {
				allArrays = false
				break
			}
			if len(a) > maxLen {
				maxLen = len(a)
			}
		}
		if !allArrays || maxLen == 0 {
			return []map[string]any{t}
		}
		out := make([]map[string]any, 0, maxLen)
		for i := 0; i < maxLen; i++ {
			row := map[string]any{}
			for _, k := range keys {
				if a, ok := t[k].([]any); ok {
					if i < len(a) {
						row[k] = a[i]
					} else {
						row[k] = nil
					}
				} else if i == 0 {
					row[k] = t[k]
				}
			}
			out = append(out, row)
		}
		return out
	default:
		return nil
	}
}

// HrefWithVersion appends ?v= / &v= for cache busting.
func HrefWithVersion(href, version string) string {
	version = strings.TrimSpace(version)
	if version == "" {
		return href
	}
	if strings.Contains(href, "?") {
		return href + "&v=" + version
	}
	return href + "?v=" + version
}

func esc(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	return s
}

// MimeForPath infers Content-Type from file extension.
func MimeForPath(p string) string {
	lower := strings.ToLower(p)
	switch {
	case strings.HasSuffix(lower, ".ico"):
		return "image/x-icon"
	case strings.HasSuffix(lower, ".png"):
		return "image/png"
	case strings.HasSuffix(lower, ".svg"):
		return "image/svg+xml"
	case strings.HasSuffix(lower, ".webp"):
		return "image/webp"
	case strings.HasSuffix(lower, ".jpg"), strings.HasSuffix(lower, ".jpeg"):
		return "image/jpeg"
	case strings.HasSuffix(lower, ".gif"):
		return "image/gif"
	case strings.HasSuffix(lower, ".webmanifest"), strings.HasSuffix(lower, ".json"):
		return "application/manifest+json"
	case strings.HasSuffix(lower, ".css"):
		return "text/css; charset=utf-8"
	case strings.HasSuffix(lower, ".js"), strings.HasSuffix(lower, ".mjs"):
		return "text/javascript; charset=utf-8"
	default:
		return "application/octet-stream"
	}
}

// AsHeadLinks normalizes a Head resource table → list of HeadLink.
func AsHeadLinks(v any) []HeadLink {
	var out []HeadLink
	for _, m := range rowsOf(v) {
		rel := rowStr(m, relKeys)
		href := rowStr(m, hrefKeys)
		if href == "" {
			continue
		}
		if rel == "" {
			rel = "stylesheet"
		}
		out = append(out, HeadLink{
			Rel:         rel,
			Href:        href,
			Type:        rowStr(m, typeKeys),
			Sizes:       rowStr(m, sizesKeys),
			Media:       rowStr(m, mediaKeys),
			As:          rowStr(m, asKeys),
			CrossOrigin: rowStr(m, corsKeys),
			Defer:       rowBool(m, deferKeys),
			Async:       boolDefault(rowBool(m, asyncKeys), false),
			Version:     rowStr(m, verKeys),
		})
	}
	return out
}

func boolDefault(p *bool, def bool) bool {
	if p == nil {
		return def
	}
	return *p
}

// HeadLinksToJSON serializes links for page.head / site_head bags.
func HeadLinksToJSON(links []HeadLink) []any {
	out := make([]any, 0, len(links))
	for _, l := range links {
		o := map[string]any{
			"rel":         l.Rel,
			"href":        l.Href,
			"type":        l.Type,
			"sizes":       l.Sizes,
			"media":       l.Media,
			"as":          l.As,
			"crossorigin": l.CrossOrigin,
			"async":       l.Async,
			"version":     l.Version,
		}
		if l.Defer != nil {
			o["defer"] = *l.Defer
		}
		out = append(out, o)
	}
	return out
}

// HeadLinksFromJSON reads links from page.head JSON.
func HeadLinksFromJSON(v any) []HeadLink {
	if arr, ok := v.([]any); ok {
		var out []HeadLink
		for _, item := range arr {
			m, ok := item.(map[string]any)
			if !ok {
				continue
			}
			href := rowStr(m, hrefKeys)
			if href == "" {
				continue
			}
			out = append(out, HeadLink{
				Rel:         rowStr(m, relKeys),
				Href:        href,
				Type:        rowStr(m, typeKeys),
				Sizes:       rowStr(m, sizesKeys),
				Media:       rowStr(m, mediaKeys),
				As:          rowStr(m, asKeys),
				CrossOrigin: rowStr(m, corsKeys),
				Defer:       rowBool(m, deferKeys),
				Async:       boolDefault(rowBool(m, asyncKeys), false),
				Version:     rowStr(m, verKeys),
			})
		}
		return out
	}
	return AsHeadLinks(v)
}

// RenderHeadLinks renders HeadLink list to HTML.
func RenderHeadLinks(links []HeadLink) string {
	return RenderHeadLinksWithVersion(links, "")
}

// RenderHeadLinksWithVersion renders with per-link or fallback asset version.
func RenderHeadLinksWithVersion(links []HeadLink, assetVersion string) string {
	var s strings.Builder
	for _, l := range links {
		rel := strings.ToLower(strings.TrimSpace(l.Rel))
		ver := strings.TrimSpace(l.Version)
		if ver == "" {
			ver = assetVersion
		}
		href := HrefWithVersion(l.Href, ver)
		if rel == "script" || rel == "module" {
			tag := "<script"
			isModule := rel == "module" || l.Type == "module"
			if isModule {
				tag += ` type="module"`
			} else if l.Type != "" {
				tag += fmt.Sprintf(` type="%s"`, esc(l.Type))
			}
			if !isModule && l.Defer != nil && *l.Defer {
				tag += " defer"
			}
			if l.Async {
				tag += " async"
			}
			if l.CrossOrigin != "" {
				tag += fmt.Sprintf(` crossorigin="%s"`, esc(l.CrossOrigin))
			}
			tag += fmt.Sprintf(` src="%s"></script>`, esc(href))
			s.WriteString(tag)
			continue
		}
		tag := fmt.Sprintf(`<link rel="%s" href="%s"`, esc(l.Rel), esc(href))
		if l.Type != "" {
			tag += fmt.Sprintf(` type="%s"`, esc(l.Type))
		}
		if l.Sizes != "" {
			tag += fmt.Sprintf(` sizes="%s"`, esc(l.Sizes))
		}
		if l.Media != "" {
			tag += fmt.Sprintf(` media="%s"`, esc(l.Media))
		}
		if l.As != "" {
			tag += fmt.Sprintf(` as="%s"`, esc(l.As))
		}
		if l.CrossOrigin != "" {
			tag += fmt.Sprintf(` crossorigin="%s"`, esc(l.CrossOrigin))
		}
		tag += "/>"
		s.WriteString(tag)
	}
	return s.String()
}

// MakeHeadHTML assembles head table → HTML string.
func MakeHeadHTML(v any) string {
	return RenderHeadLinks(AsHeadLinks(v))
}

// NormalizeIcons converts icons table → (icons json, site_head links, file routes).
func NormalizeIcons(v any) ([]any, []HeadLink, []IconRoute) {
	var icons []any
	var head []HeadLink
	var routes []IconRoute
	sawFavicon := false
	for _, m := range rowsOf(v) {
		p := rowStr(m, pathKeys)
		rel := rowStr(m, relKeys)
		if rel == "" {
			rel = "icon"
		}
		type_ := rowStr(m, typeKeys)
		sizes := rowStr(m, sizesKeys)
		url := rowStr(m, urlKeys)
		if type_ == "" && p != "" {
			type_ = MimeForPath(p)
		}
		if url == "" {
			if p == "" {
				continue
			}
			name := path.Base(p)
			lower := strings.ToLower(p)
			if strings.Contains(strings.ToLower(rel), "icon") && strings.HasSuffix(lower, ".ico") {
				url = "/favicon.ico"
			} else {
				url = "/icons/" + name
			}
		}
		if url == "/favicon.ico" {
			sawFavicon = true
		}
		icons = append(icons, map[string]any{
			"path":  p,
			"rel":   rel,
			"type":  type_,
			"sizes": sizes,
			"url":   url,
		})
		if url != "" {
			head = append(head, HeadLink{
				Rel: rel, Href: url, Type: type_, Sizes: sizes,
			})
		}
		if p != "" && url != "" {
			ct := type_
			if ct == "" {
				ct = MimeForPath(p)
			}
			routes = append(routes, IconRoute{URL: url, Path: p, ContentType: ct})
		}
	}
	if !sawFavicon {
		for _, ic := range icons {
			m, ok := ic.(map[string]any)
			if !ok {
				continue
			}
			rel, _ := m["rel"].(string)
			p, _ := m["path"].(string)
			if p == "" || !strings.Contains(strings.ToLower(rel), "icon") {
				continue
			}
			type_ := cellStr(m["type"])
			if type_ == "" {
				type_ = MimeForPath(p)
			}
			routes = append(routes, IconRoute{
				URL: "/favicon.ico", Path: p, ContentType: type_,
			})
			sizes := cellStr(m["sizes"])
			head = append([]HeadLink{{
				Rel: "icon", Href: "/favicon.ico", Type: type_, Sizes: sizes,
			}}, head...)
			break
		}
	}
	return icons, head, routes
}

// MergeSiteHead merges site_head into page.head (dedupe by href+rel).
func MergeSiteHead(page map[string]any, siteHead []HeadLink) {
	if len(siteHead) == 0 {
		return
	}
	existing := HeadLinksFromJSON(page["head"])
	for _, link := range siteHead {
		dup := false
		for _, e := range existing {
			if e.Href == link.Href && strings.EqualFold(e.Rel, link.Rel) {
				dup = true
				break
			}
		}
		if !dup {
			existing = append(existing, link)
		}
	}
	page["head"] = HeadLinksToJSON(existing)
}

// MakeImagesHTML converts GFM image table → HTML fragment.
func MakeImagesHTML(v any) string {
	var figures strings.Builder
	for _, m := range rowsOf(v) {
		src := rowStr(m, imgSrc)
		if src == "" {
			continue
		}
		alt := rowStr(m, imgAlt)
		title := rowStr(m, imgTitle)
		class := rowStr(m, imgClass)
		href := rowStr(m, imgHref)
		width := rowStr(m, imgWidth)
		height := rowStr(m, imgHeight)
		loading := rowStr(m, imgLoad)
		if loading == "" {
			loading = "lazy"
		}
		caption := rowStr(m, imgCap)
		img := fmt.Sprintf(`<img src="%s" alt="%s"`, esc(src), esc(alt))
		if title != "" {
			img += fmt.Sprintf(` title="%s"`, esc(title))
		}
		if width != "" {
			img += fmt.Sprintf(` width="%s"`, esc(width))
		}
		if height != "" {
			img += fmt.Sprintf(` height="%s"`, esc(height))
		}
		img += fmt.Sprintf(` loading="%s"/>`, esc(loading))
		inner := img
		if href != "" {
			inner = fmt.Sprintf(`<a href="%s">%s</a>`, esc(href), img)
		}
		figClass := "mq-img"
		if class != "" {
			figClass = "mq-img " + class
		}
		figures.WriteString(fmt.Sprintf(`<figure class="%s">%s`, figClass, inner))
		if caption != "" {
			figures.WriteString(fmt.Sprintf("<figcaption>%s</figcaption>", esc(caption)))
		}
		figures.WriteString("</figure>")
	}
	if figures.Len() == 0 {
		return ""
	}
	return `<div class="mq-images" data-slot="main">` + figures.String() + `</div>`
}

// AttachHead merges head table links onto page.head.
func AttachHead(page map[string]any, table any) map[string]any {
	out := cloneMap(page)
	links := AsHeadLinks(table)
	existing := HeadLinksFromJSON(out["head"])
	for _, link := range links {
		dup := false
		for _, e := range existing {
			if e.Href == link.Href && strings.EqualFold(e.Rel, link.Rel) {
				dup = true
				break
			}
		}
		if !dup {
			existing = append(existing, link)
		}
	}
	out["head"] = HeadLinksToJSON(existing)
	return out
}

// AttachImages appends images_html on page.
func AttachImages(page map[string]any, table any) map[string]any {
	out := cloneMap(page)
	html := MakeImagesHTML(table)
	prev, _ := out["images_html"].(string)
	switch {
	case prev == "":
		out["images_html"] = html
	case html == "":
		out["images_html"] = prev
	default:
		out["images_html"] = prev + html
	}
	return out
}

// AttachMeta sets page.meta from SEO table.
func AttachMeta(page map[string]any, meta any) map[string]any {
	out := cloneMap(page)
	out["meta"] = table.AsMetaMap(meta)
	return out
}

func cloneMap(m map[string]any) map[string]any {
	if m == nil {
		return map[string]any{}
	}
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}
