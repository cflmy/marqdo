package page

import (
	"strconv"
	"strings"
)

// New builds a page bag (ABI web_page_new), matching the Rust plugin shape.
func New(args map[string]any) map[string]any {
	title := strOpt(args, "title", "Marqdo Web")
	intro := strOpt(args, "intro", "")
	out := map[string]any{
		"title": title,
		"intro": intro,
	}
	if s, ok := strAny(args, "shell_css", "壳样式"); ok {
		out["shell_css"] = s
	}
	if s, ok := strAny(args, "layout", "布局"); ok {
		out["layout"] = s
	}
	if s, ok := strAny(args, "asset_version", "资源版本"); ok {
		out["asset_version"] = s
	}
	return out
}

// Paginate stamps page.paginate with offset/limit/path (Rust web_page_paginate).
func Paginate(page map[string]any, offset, limit int64, path string) map[string]any {
	out := clone(page)
	if path == "" {
		path = "/"
	}
	out["paginate"] = map[string]any{
		"offset": offset,
		"limit":  limit,
		"path":   path,
	}
	return out
}

// SetQuery stamps page.query (Rust web_page_query).
func SetQuery(page map[string]any, query any) map[string]any {
	out := clone(page)
	if query == nil {
		query = map[string]any{}
	}
	out["query"] = query
	return out
}

// SetOrder stamps page.order (Rust web_page_order).
func SetOrder(page map[string]any, order string) map[string]any {
	out := clone(page)
	out["order"] = order
	return out
}

// SetLinkPrefix stamps page.link_prefix (Rust web_page_link_prefix).
func SetLinkPrefix(page map[string]any, prefix string) map[string]any {
	out := clone(page)
	out["link_prefix"] = prefix
	return out
}

// AppendCSS appends raw CSS to page.styles_css (Rust web_page_css).
func AppendCSS(page map[string]any, css string) map[string]any {
	out := clone(page)
	css = strings.TrimSpace(css)
	if css == "" {
		return out
	}
	prev, _ := out["styles_css"].(string)
	if prev == "" {
		out["styles_css"] = css
	} else {
		out["styles_css"] = prev + "\n" + css
	}
	return out
}

// SetDetail stamps page.detail bool (Rust web_page_detail).
func SetDetail(page map[string]any, on bool) map[string]any {
	out := clone(page)
	out["detail"] = on
	return out
}

func clone(page map[string]any) map[string]any {
	out := map[string]any{}
	for k, v := range page {
		out[k] = v
	}
	return out
}

func strOpt(args map[string]any, key, def string) string {
	if s, ok := strAny(args, key); ok {
		return s
	}
	return def
}

func strAny(args map[string]any, keys ...string) (string, bool) {
	for _, k := range keys {
		v, ok := args[k]
		if !ok || v == nil {
			continue
		}
		switch t := v.(type) {
		case string:
			return t, true
		case float64:
			if t == float64(int64(t)) {
				return strconv.FormatInt(int64(t), 10), true
			}
			return strconv.FormatFloat(t, 'f', -1, 64), true
		case bool:
			return strconv.FormatBool(t), true
		}
	}
	return "", false
}

// AsBool interprets ABI detail flags (bool / 0|1 / 真|是|true).
func AsBool(v any, def bool) bool {
	if v == nil {
		return def
	}
	switch t := v.(type) {
	case bool:
		return t
	case float64:
		return t != 0
	case string:
		s := strings.TrimSpace(t)
		switch s {
		case "1", "true", "True", "yes", "on", "真", "是":
			return true
		case "0", "false", "False", "no", "off", "假", "否", "":
			return false
		}
	}
	return def
}
