// Package rss builds RSS 2.0 feeds (content-site W4 / W-G7).
package rss

import (
	"fmt"
	"strconv"
	"strings"
)

func esc(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	return s
}

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

func pick(m map[string]any, keys ...string) any {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v
		}
	}
	return nil
}

// BuildRSS assembles RSS 2.0 XML from channel metadata and row maps.
func BuildRSS(title, link, description string, items []any) string {
	body := fmt.Sprintf(`<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel>
<title>%s</title>
<link>%s</link>
<description>%s</description>
`, esc(title), esc(link), esc(description))
	for _, row := range items {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		itemTitle := cellStr(pick(m, "title", "标题"))
		itemLink := cellStr(pick(m, "link", "href", "链接", "slug"))
		itemDesc := cellStr(pick(m, "description", "summary", "摘要", "body", "正文"))
		pubDate := cellStr(pick(m, "pubDate", "published", "created_at", "发布"))
		fullLink := link
		if itemLink != "" {
			if strings.HasPrefix(itemLink, "http") {
				fullLink = itemLink
			} else if itemLink[0] == '/' {
				fullLink = link + itemLink
			} else {
				fullLink = link + "/" + itemLink
			}
		}
		body += "<item>"
		body += fmt.Sprintf("<title>%s</title>", esc(itemTitle))
		body += fmt.Sprintf("<link>%s</link>", esc(fullLink))
		body += fmt.Sprintf("<description>%s</description>", esc(itemDesc))
		if pubDate != "" {
			body += fmt.Sprintf("<pubDate>%s</pubDate>", esc(pubDate))
		}
		body += "</item>"
	}
	body += "</channel></rss>"
	return body
}
