// Package middleware ports declarative CORS/security/compress/JSON configure
// from the Rust web plugin (配置即数据、装配即函数).
package middleware

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// Config is the runtime view of app["middleware"].
type Config struct {
	CORS         *CORSConfig
	Security     [][2]string
	Compress     bool
	BodyLimit    *uint64
	AccessLog    bool
	CacheControl string
	JSONRoutes   []JSONRouteMount
}

// CORSConfig matches middleware.cors map shape.
type CORSConfig struct {
	AllowOrigins  []string
	Methods       []string
	Headers       []string
	ExposeHeaders []string
	Credentials   bool
}

// JSONRouteMount is one JSON API route with leading-slash path.
type JSONRouteMount struct {
	Path   string
	Method string
	Table  string
	Where  any
	Order  string
	Limit  int64
}

// Configure merges configure tables into app.middleware (Rust web_app_middleware).
func Configure(appBag map[string]any, args map[string]any) (map[string]any, error) {
	out := clone(appBag)
	mw, _ := out["middleware"].(map[string]any)
	if mw == nil {
		mw = map[string]any{}
	} else {
		mw = clone(mw)
	}

	if v, ok := args["cors"]; ok && v != nil {
		mw["cors"] = CorsFromTable(v)
	}
	if v, ok := args["security"]; ok && v != nil {
		mw["security"] = SecurityFromTable(v)
	}
	if v, ok := args["compress"]; ok && v != nil {
		mw["compress"] = boolish(v)
	}
	if v, ok := first(args, "access_log", "访问日志"); ok {
		mw["access_log"] = boolish(v)
	}
	if v, ok := first(args, "cache_control", "缓存控制"); ok {
		if s := strings.TrimSpace(cellText(v)); s != "" {
			mw["cache_control"] = s
		}
	}
	if v, ok := args["body_limit"]; ok && v != nil {
		if n, ok := asUint64(v); ok && n > 0 {
			mw["body_limit"] = float64(n)
		}
	}
	if v, ok := first(args, "json_routes", "json"); ok && v != nil {
		mw["json_routes"] = JSONRoutesFromTable(v)
	}
	out["middleware"] = mw
	return out, nil
}

// Parse reads app["middleware"] into Config (Rust middleware::parse).
func Parse(app map[string]any) Config {
	var cfg Config
	obj, _ := app["middleware"].(map[string]any)
	if obj == nil {
		return cfg
	}
	if c, ok := obj["cors"].(map[string]any); ok {
		cfg.CORS = parseCORS(c)
	}
	if sec, ok := obj["security"].(map[string]any); ok {
		for k, v := range sec {
			s := cellText(v)
			if s != "" {
				cfg.Security = append(cfg.Security, [2]string{k, s})
			}
		}
	}
	cfg.Compress = boolish(obj["compress"])
	cfg.AccessLog = boolish(obj["access_log"])
	if s := strings.TrimSpace(cellText(obj["cache_control"])); s != "" {
		cfg.CacheControl = s
	}
	if n, ok := asUint64(obj["body_limit"]); ok && n > 0 {
		cfg.BodyLimit = &n
	}
	if routes, ok := obj["json_routes"].(map[string]any); ok {
		for path, spec := range routes {
			if r, ok := parseJSONRoute(path, spec); ok {
				cfg.JSONRoutes = append(cfg.JSONRoutes, r)
			}
		}
		sortJSONRoutes(cfg.JSONRoutes)
	}
	return cfg
}

func parseCORS(obj map[string]any) *CORSConfig {
	return &CORSConfig{
		AllowOrigins:  strList(obj["allow_origins"]),
		Methods:       strList(obj["methods"]),
		Headers:       strList(obj["headers"]),
		ExposeHeaders: strList(obj["expose_headers"]),
		Credentials:   boolish(obj["credentials"]),
	}
}

func parseJSONRoute(path string, spec any) (JSONRouteMount, bool) {
	m, ok := spec.(map[string]any)
	if !ok {
		return JSONRouteMount{}, false
	}
	tableName := strings.TrimSpace(cellText(m["table"]))
	if tableName == "" {
		return JSONRouteMount{}, false
	}
	method := strings.ToUpper(strings.TrimSpace(cellText(m["method"])))
	if method == "" {
		method = "GET"
	}
	limit := int64(200)
	if n, ok := asInt64(m["limit"]); ok {
		limit = n
	}
	order := strings.TrimSpace(cellText(m["order"]))
	p := "/" + strings.TrimPrefix(strings.TrimSpace(path), "/")
	r := JSONRouteMount{
		Path:   p,
		Method: method,
		Table:  tableName,
		Order:  order,
		Limit:  limit,
	}
	if w, ok := m["where"]; ok && w != nil {
		r.Where = w
	}
	return r, true
}

func sortJSONRoutes(rs []JSONRouteMount) {
	for i := 0; i < len(rs); i++ {
		for j := i + 1; j < len(rs); j++ {
			if rs[j].Path < rs[i].Path {
				rs[i], rs[j] = rs[j], rs[i]
			}
		}
	}
}

// CorsFromTable normalizes `|允许来源|方法|头|暴露头|凭证|` into middleware.cors.
func CorsFromTable(tableV any) map[string]any {
	rows, _ := table.AsRows(tableV).([]any)
	var allowOrigins, methods, headers, expose []string
	credentials := false
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		origin := col(m, "允许来源", "origin", "Origin")
		if origin != "" && origin != "*" {
			allowOrigins = append(allowOrigins, origin)
		}
		for _, part := range strings.Split(col(m, "方法", "methods", "Methods"), ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				methods = append(methods, part)
			}
		}
		for _, part := range strings.Split(col(m, "头", "headers", "Headers"), ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				headers = append(headers, part)
			}
		}
		for _, part := range strings.Split(col(m, "暴露头", "expose_headers", "Expose-Headers"), ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				expose = append(expose, part)
			}
		}
		cred := col(m, "凭证", "credentials", "Credentials")
		if strings.EqualFold(cred, "true") || cred == "1" || strings.EqualFold(cred, "yes") {
			credentials = true
		}
	}
	return map[string]any{
		"allow_origins":  toAnyList(allowOrigins),
		"methods":        toAnyList(methods),
		"headers":        toAnyList(headers),
		"expose_headers": toAnyList(expose),
		"credentials":    credentials,
	}
}

// SecurityFromTable normalizes `|头|值|` into middleware.security map.
func SecurityFromTable(tableV any) map[string]any {
	rows, _ := table.AsRows(tableV).([]any)
	out := map[string]any{}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		k := col(m, "头", "header", "Header", "name")
		v := col(m, "值", "value", "Value")
		if k != "" {
			out[k] = v
		}
	}
	return out
}

// JSONRoutesFromTable normalizes `|路径|方法|表|条件|排序|上限|` into json_routes.
func JSONRoutesFromTable(tableV any) map[string]any {
	rows, _ := table.AsRows(tableV).([]any)
	out := map[string]any{}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		path := strings.TrimPrefix(strings.TrimSpace(col(m, "路径", "path", "Path")), "/")
		if path == "" {
			continue
		}
		method := strings.ToUpper(strings.TrimSpace(col(m, "方法", "method", "Method")))
		if method == "" {
			method = "GET"
		}
		tbl := col(m, "表", "table", "Table")
		if tbl == "" {
			continue
		}
		order := col(m, "排序", "order", "Order")
		limit := int64(200)
		if s := col(m, "上限", "limit", "Limit"); s != "" {
			if n, err := strconv.ParseInt(s, 10, 64); err == nil {
				limit = n
			}
		}
		spec := map[string]any{
			"method": method,
			"table":  tbl,
			"limit":  float64(limit),
		}
		if order != "" {
			spec["order"] = order
		}
		out[path] = spec
	}
	return out
}

func col(m map[string]any, keys ...string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return cellText(v)
		}
	}
	return ""
}

func cellText(v any) string {
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
	case int:
		return strconv.Itoa(t)
	case int64:
		return strconv.FormatInt(t, 10)
	default:
		return fmt.Sprint(t)
	}
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
		return false
	case float64:
		return t != 0
	case int:
		return t != 0
	case int64:
		return t != 0
	}
	return false
}

func asUint64(v any) (uint64, bool) {
	switch t := v.(type) {
	case float64:
		if t > 0 {
			return uint64(t), true
		}
	case int64:
		if t > 0 {
			return uint64(t), true
		}
	case int:
		if t > 0 {
			return uint64(t), true
		}
	case string:
		n, err := strconv.ParseUint(strings.TrimSpace(t), 10, 64)
		return n, err == nil && n > 0
	}
	return 0, false
}

func asInt64(v any) (int64, bool) {
	switch t := v.(type) {
	case float64:
		return int64(t), true
	case int64:
		return t, true
	case int:
		return int64(t), true
	case string:
		n, err := strconv.ParseInt(strings.TrimSpace(t), 10, 64)
		return n, err == nil
	}
	return 0, false
}

func strList(v any) []string {
	switch t := v.(type) {
	case []any:
		out := make([]string, 0, len(t))
		for _, x := range t {
			if s, ok := x.(string); ok {
				out = append(out, s)
			}
		}
		return out
	case []string:
		return append([]string{}, t...)
	}
	return nil
}

func toAnyList(ss []string) []any {
	out := make([]any, len(ss))
	for i, s := range ss {
		out[i] = s
	}
	return out
}

func first(args map[string]any, keys ...string) (any, bool) {
	for _, k := range keys {
		if v, ok := args[k]; ok && v != nil {
			return v, true
		}
	}
	return nil, false
}

func clone(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}
