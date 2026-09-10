// Package app ports minimal app bag helpers (new + route) for offline smoke.
package app

import (
	"fmt"
	"strings"
)

// New builds an app bag matching Rust web_app_new defaults used by zh-smoke.
func New(args map[string]any) map[string]any {
	page := args["page"]
	if page == nil {
		page = map[string]any{}
	}
	dbv := args["db"]
	host := strOpt(args, "host", "127.0.0.1")
	port := float64(18081)
	if v, ok := args["port"]; ok {
		switch t := v.(type) {
		case float64:
			port = t
		case string:
			// leave default if unparsable
			_ = t
		}
	}
	admin := boolish(args["admin"])
	adminPrefix := strOpt(args, "admin_prefix", "/admin")
	if s := strOpt(args, "后台前缀", ""); s != "" {
		adminPrefix = s
	}
	out := map[string]any{
		"page":         page,
		"db":           dbv,
		"host":         host,
		"port":         port,
		"admin":        admin,
		"admin_prefix": adminPrefix,
		"forms":        map[string]any{},
		"routes":       map[string]any{},
		"ws_routes":    map[string]any{},
		"static_dir":   nil,
		"static_mount": "/static",
	}
	if s := strOpt(args, "shell_css", ""); s == "" {
		s = strOpt(args, "壳样式", "")
		if s != "" {
			out["shell_css"] = s
		}
	} else {
		out["shell_css"] = s
	}
	if s := strOpt(args, "layout", ""); s == "" {
		s = strOpt(args, "布局", "")
		if s != "" {
			out["layout"] = s
		}
	} else {
		out["layout"] = s
	}
	if s := strOpt(args, "asset_version", ""); s == "" {
		s = strOpt(args, "资源版本", "")
		if s != "" {
			out["asset_version"] = s
		}
	} else {
		out["asset_version"] = s
	}
	return out
}

// Route registers path → page on app.routes and stamps page._route.
func Route(appBag map[string]any, path string, page any) (map[string]any, error) {
	path, err := normalizeRoutePath(path)
	if err != nil {
		return nil, err
	}
	out := clone(appBag)
	pageMap := map[string]any{}
	if m, ok := page.(map[string]any); ok {
		pageMap = clone(m)
	}
	pageMap["_route"] = path
	routes := map[string]any{}
	if r, ok := out["routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[path] = pageMap
	out["routes"] = routes
	return out, nil
}

// MountForm stores form under app.forms[id].
func MountForm(appBag map[string]any, id string, form any) (map[string]any, error) {
	id = strings.TrimSpace(id)
	if id == "" {
		return nil, fmt.Errorf("missing form id")
	}
	if form == nil {
		return nil, fmt.Errorf("missing `form`")
	}
	out := clone(appBag)
	forms := map[string]any{}
	if f, ok := out["forms"].(map[string]any); ok {
		forms = clone(f)
	}
	forms[id] = form
	out["forms"] = forms
	return out, nil
}

// NormalizeStaticMount mirrors Rust http::normalize_static_mount.
func NormalizeStaticMount(raw string) string {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "/static"
	}
	m := s
	if !strings.HasPrefix(m, "/") {
		m = "/" + m
	}
	for len(m) > 1 && strings.HasSuffix(m, "/") {
		m = strings.TrimSuffix(m, "/")
	}
	return m
}

// Static sets static_dir + static_mount (Rust web_app_static).
func Static(appBag map[string]any, dir, mount string) (map[string]any, error) {
	dir = strings.TrimSpace(dir)
	if dir == "" {
		return nil, fmt.Errorf("static `dir` is empty")
	}
	if mount == "" {
		mount = "/static"
	}
	mount = NormalizeStaticMount(mount)
	if mount == "/_form" || strings.HasPrefix(mount, "/_form/") ||
		mount == "/_part" || strings.HasPrefix(mount, "/_part/") {
		return nil, fmt.Errorf("static mount `%s` is reserved", mount)
	}
	if boolish(appBag["admin"]) {
		prefix := strOpt(appBag, "admin_prefix", "/admin")
		prefix = NormalizeStaticMount(prefix)
		if mount == prefix || strings.HasPrefix(mount, prefix+"/") {
			return nil, fmt.Errorf("static mount `%s` is reserved", mount)
		}
	}
	out := clone(appBag)
	out["static_dir"] = dir
	out["static_mount"] = mount
	return out, nil
}

func normalizeRoutePath(raw string) (string, error) {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "", fmt.Errorf("empty route path")
	}
	if !strings.HasPrefix(s, "/") {
		s = "/" + s
	}
	if s != "/" {
		s = strings.TrimRight(s, "/")
	}
	return s, nil
}

func clone(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func strOpt(args map[string]any, key, def string) string {
	v, ok := args[key]
	if !ok || v == nil {
		return def
	}
	if s, ok := v.(string); ok {
		return s
	}
	return def
}

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		return t == "true" || t == "True" || t == "1" || t == "yes"
	case float64:
		return t != 0
	}
	return false
}
