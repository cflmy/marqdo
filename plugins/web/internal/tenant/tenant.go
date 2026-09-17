// Package tenant resolves multi-tenant request context for ext/web.
package tenant

import (
	"context"
	"fmt"
	"net/http"
	"strings"
)

type ctxKey int

const requestKey ctxKey = 1

// Config is stored on the app bag under key "tenant".
type Config struct {
	Mode         string // path | subdomain | header
	Param        string // path capture / query / header name
	Column       string // DB column (default tenant_id)
	DefaultScope bool   // auto-scope all selects/inserts when true
}

// FromBag reads tenant config from an app bag.
func FromBag(appBag map[string]any) Config {
	cfg := Config{Column: "tenant_id"}
	raw, ok := appBag["tenant"].(map[string]any)
	if !ok {
		return cfg
	}
	cfg.Mode = strings.ToLower(strings.TrimSpace(str(raw, "mode", "模式")))
	cfg.Param = strings.TrimSpace(str(raw, "param", "参数", "tenant_param"))
	if c := str(raw, "column", "列", "tenant_column"); c != "" {
		cfg.Column = c
	}
	if v, ok := raw["default_scope"]; ok {
		cfg.DefaultScope = boolish(v)
	} else if v, ok := raw["默认作用域"]; ok {
		cfg.DefaultScope = boolish(v)
	} else if v, ok := raw["scope"]; ok {
		cfg.DefaultScope = boolish(v)
	}
	return cfg
}

// Enabled reports whether tenant resolution is configured.
func (c Config) Enabled() bool {
	return c.Mode != ""
}

// Configure merges tenant options onto the app bag.
func Configure(appBag map[string]any, opts map[string]any) (map[string]any, error) {
	if opts == nil {
		opts = map[string]any{}
	}
	mode := strings.ToLower(strings.TrimSpace(str(opts, "mode", "模式")))
	if mode == "" {
		return nil, fmt.Errorf("tenant: mode required (path|subdomain|header)")
	}
	switch mode {
	case "path", "路径", "subdomain", "子域", "header", "头":
	default:
		return nil, fmt.Errorf("tenant: unknown mode %q", mode)
	}
	if mode == "路径" {
		mode = "path"
	}
	if mode == "子域" {
		mode = "subdomain"
	}
	if mode == "头" {
		mode = "header"
	}
	param := strings.TrimSpace(str(opts, "param", "参数", "tenant_param"))
	if param == "" {
		switch mode {
		case "path":
			param = "t"
		case "header":
			param = "X-Tenant-Id"
		case "subdomain":
			param = ""
		}
	}
	col := strings.TrimSpace(str(opts, "column", "列", "tenant_column"))
	if col == "" {
		col = "tenant_id"
	}
	entry := map[string]any{
		"mode":   mode,
		"param":  param,
		"column": col,
	}
	if v, ok := first(opts, "default_scope", "默认作用域", "scope"); ok {
		entry["default_scope"] = boolish(v)
	}
	out := clone(appBag)
	out["tenant"] = entry
	return out, nil
}

// Resolve extracts tenant id from the HTTP request.
func Resolve(r *http.Request, cfg Config) string {
	if !cfg.Enabled() {
		return ""
	}
	switch cfg.Mode {
	case "path":
		if cfg.Param != "" {
			if v := r.PathValue(cfg.Param); v != "" {
				return sanitize(v)
			}
			if v := r.URL.Query().Get(cfg.Param); v != "" {
				return sanitize(v)
			}
			// /t/{tenant}/… when param is t: take segment after /t/
			parts := strings.Split(strings.Trim(r.URL.Path, "/"), "/")
			if cfg.Param == "t" && len(parts) >= 2 && parts[0] == "t" {
				return sanitize(parts[1])
			}
			for i, p := range parts {
				if p == cfg.Param && i+1 < len(parts) {
					return sanitize(parts[i+1])
				}
			}
		}
	case "subdomain":
		host := r.Host
		if h, _, ok := strings.Cut(host, ":"); ok {
			host = h
		}
		labels := strings.Split(host, ".")
		if len(labels) >= 3 {
			return sanitize(labels[0])
		}
	case "header":
		name := cfg.Param
		if name == "" {
			name = "X-Tenant-Id"
		}
		return sanitize(r.Header.Get(name))
	}
	return ""
}

// WithContext stores tenant id on the request context.
func WithContext(ctx context.Context, id string) context.Context {
	return context.WithValue(ctx, requestKey, id)
}

// FromContext reads tenant id (empty if unset).
func FromContext(ctx context.Context) string {
	if ctx == nil {
		return ""
	}
	if s, ok := ctx.Value(requestKey).(string); ok {
		return s
	}
	return ""
}

// MergeWhere adds tenant_id equality to a where value (map or list of maps).
func MergeWhere(where any, column, tenantID string) any {
	if tenantID == "" || column == "" {
		return where
	}
	clause := map[string]any{"字段": column, "field": column, "操作": "=", "op": "=", "值": tenantID, "value": tenantID}
	switch t := where.(type) {
	case nil:
		return []any{clause}
	case []any:
		out := make([]any, 0, len(t)+1)
		out = append(out, t...)
		out = append(out, clause)
		return out
	case map[string]any:
		out := cloneMap(t)
		out[column] = tenantID
		return out
	default:
		return []any{where, clause}
	}
}

// StampRow sets tenant column on an insert/update row map if missing.
func StampRow(row map[string]any, column, tenantID string) map[string]any {
	if row == nil || tenantID == "" || column == "" {
		return row
	}
	if _, ok := row[column]; ok {
		return row
	}
	out := cloneMap(row)
	out[column] = tenantID
	return out
}

func sanitize(s string) string {
	s = strings.TrimSpace(strings.ToLower(s))
	var b strings.Builder
	for _, r := range s {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') || r == '-' || r == '_' {
			b.WriteRune(r)
		}
	}
	return b.String()
}

func str(m map[string]any, keys ...string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok && v != nil {
			if s, ok := v.(string); ok {
				return s
			}
			return fmt.Sprint(v)
		}
	}
	return ""
}

func first(m map[string]any, keys ...string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok && v != nil {
			return v, true
		}
	}
	return nil, false
}

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		switch strings.TrimSpace(t) {
		case "true", "True", "1", "yes", "on", "真":
			return true
		}
	case float64:
		return t != 0
	}
	return false
}

func clone(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func cloneMap(m map[string]any) map[string]any { return clone(m) }
