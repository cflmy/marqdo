package page

import "strconv"

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
