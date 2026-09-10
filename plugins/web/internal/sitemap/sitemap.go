// Package sitemap builds sitemap.xml and robots.txt helpers (W7 / W-G7).
package sitemap

import (
	"fmt"
	"strconv"
	"strings"
)

func cellStr(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case bool:
		return strconv.FormatBool(t)
	case float64:
		if t == float64(int64(t)) {
			return strconv.FormatInt(int64(t), 10)
		}
		return strconv.FormatFloat(t, 'f', -1, 64)
	default:
		return fmt.Sprint(t)
	}
}

func xmlEscape(s string) string {
	r := strings.NewReplacer(
		"&", "&amp;",
		"<", "&lt;",
		">", "&gt;",
		`"`, "&quot;",
		"'", "&apos;",
	)
	return r.Replace(s)
}

func rowsFromItems(items any) []map[string]any {
	switch t := items.(type) {
	case []any:
		out := make([]map[string]any, 0, len(t))
		for _, row := range t {
			switch r := row.(type) {
			case map[string]any:
				out = append(out, r)
			case string:
				out = append(out, map[string]any{"loc": r})
			default:
				out = append(out, map[string]any{"loc": cellStr(r)})
			}
		}
		return out
	case map[string]any:
		locs, ok := firstArray(t, "loc", "路径", "path", "url")
		if !ok {
			return nil
		}
		out := make([]map[string]any, 0, len(locs))
		for i, loc := range locs {
			obj := map[string]any{"loc": loc}
			for k, v := range t {
				if k == "loc" || k == "路径" || k == "path" || k == "url" {
					continue
				}
				if col, ok := v.([]any); ok && i < len(col) {
					obj[k] = col[i]
				}
			}
			out = append(out, obj)
		}
		return out
	default:
		return nil
	}
}

func firstArray(m map[string]any, keys ...string) ([]any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			if a, ok := v.([]any); ok {
				return a, true
			}
		}
	}
	return nil, false
}

// BuildSitemap builds a sitemap.xml document from rows.
func BuildSitemap(base string, items any) string {
	base = strings.TrimRight(base, "/")
	rows := rowsFromItems(items)
	out := `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
`
	for _, obj := range rows {
		locRaw := cellStr(first(obj, "loc", "url", "路径", "path"))
		if locRaw == "" {
			continue
		}
		loc := locRaw
		if strings.HasPrefix(locRaw, "http://") || strings.HasPrefix(locRaw, "https://") {
			loc = locRaw
		} else if strings.HasPrefix(locRaw, "/") {
			loc = base + locRaw
		} else {
			loc = base + "/" + locRaw
		}
		out += "  <url>\n"
		out += fmt.Sprintf("    <loc>%s</loc>\n", xmlEscape(loc))
		if lm := cellStr(first(obj, "lastmod", "更新")); lm != "" {
			out += fmt.Sprintf("    <lastmod>%s</lastmod>\n", xmlEscape(lm))
		}
		if cf := cellStr(first(obj, "changefreq", "频率")); cf != "" {
			out += fmt.Sprintf("    <changefreq>%s</changefreq>\n", xmlEscape(cf))
		}
		if pr := cellStr(first(obj, "priority", "优先级")); pr != "" {
			out += fmt.Sprintf("    <priority>%s</priority>\n", xmlEscape(pr))
		}
		out += "  </url>\n"
	}
	out += "</urlset>\n"
	return out
}

func first(m map[string]any, keys ...string) any {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v
		}
	}
	return nil
}

// BuildRobots returns a default robots.txt body with optional Sitemap line.
func BuildRobots(sitemapURL string) string {
	out := "User-agent: *\nAllow: /\n"
	if strings.TrimSpace(sitemapURL) != "" {
		out += "Sitemap: " + strings.TrimSpace(sitemapURL) + "\n"
	}
	return out
}

// SitemapJSON is the ABI helper returning {"xml":"…"}.
func SitemapJSON(base string, items any) map[string]any {
	return map[string]any{"xml": BuildSitemap(base, items)}
}
