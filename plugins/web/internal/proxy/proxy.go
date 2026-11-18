// Package proxy ports offline proxy route bags (Rust proxy.rs).
package proxy

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// Options is one proxy mount (web_app_proxy args).
type Options struct {
	Path           string
	Upstream       string
	Stream         bool
	StripPrefix    string
	Methods        []string
	HeadersFromEnv string
	TimeoutMs      uint64
}

// RoutesFromTable normalizes `|路径|上游|流式|去前缀|方法|环境头|超时|` into proxy_routes map keys.
func RoutesFromTable(tableV any) map[string]any {
	rows, _ := table.AsRows(tableV).([]any)
	out := map[string]any{}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		path := normalizeTablePath(col(m, "路径", "path", "Path"))
		if path == "" || path == "/" {
			continue
		}
		upstream := col(m, "上游", "upstream", "Upstream")
		if upstream == "" {
			continue
		}
		stream := true
		if s := col(m, "流式", "stream", "Stream"); s != "" {
			stream = boolish(s)
		}
		stripPrefix := col(m, "去前缀", "strip_prefix", "Strip-Prefix")
		methodsRaw := col(m, "方法", "methods", "Methods")
		methods := parseMethods(methodsRaw)
		headersFromEnv := col(m, "环境头", "headers_from_env", "Headers-From-Env")
		timeoutMs := uint64(120_000)
		if s := col(m, "超时", "timeout_ms", "Timeout"); s != "" {
			if n, err := strconv.ParseUint(strings.TrimSpace(s), 10, 64); err == nil {
				timeoutMs = n
			}
		}
		key := strings.TrimPrefix(path, "/")
		out[key] = routeEntry(upstream, stream, stripPrefix, methods, headersFromEnv, timeoutMs)
	}
	return out
}

// Register adds one proxy route to app.proxy_routes (Rust web_app_proxy).
func Register(appBag map[string]any, opts Options) (map[string]any, error) {
	if strings.TrimSpace(opts.Upstream) == "" {
		return nil, fmt.Errorf("proxy requires `upstream`")
	}
	path, err := app.NormalizeRoutePath(opts.Path, appBag)
	if err != nil {
		return nil, err
	}
	key := strings.TrimPrefix(path, "/")
	out := clone(appBag)
	routes := map[string]any{}
	if r, ok := out["proxy_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	methods := opts.Methods
	if len(methods) == 0 {
		methods = []string{"POST"}
	}
	timeout := opts.TimeoutMs
	if timeout == 0 {
		timeout = 120_000
	}
	routes[key] = routeEntry(opts.Upstream, opts.Stream, opts.StripPrefix, methods, opts.HeadersFromEnv, timeout)
	out["proxy_routes"] = routes
	return out, nil
}

// OptionsFromArgs parses web_app_proxy ABI args.
func OptionsFromArgs(args map[string]any) (Options, error) {
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return Options{}, err
	}
	upstream, err := argStrReq(args, "upstream", "上游")
	if err != nil {
		return Options{}, err
	}
	stream := true
	if v, ok := first(args, "stream", "流式"); ok {
		stream = boolish(v)
	}
	stripPrefix, _ := argStr(args, "strip_prefix", "去前缀")
	headersFromEnv, _ := argStr(args, "headers_from_env", "环境头")
	timeoutMs := uint64(120_000)
	if v, ok := first(args, "timeout_ms", "超时"); ok {
		if n, ok := asUint64(v); ok {
			timeoutMs = n
		}
	}
	return Options{
		Path:           path,
		Upstream:       upstream,
		Stream:         stream,
		StripPrefix:    stripPrefix,
		Methods:        parseMethodsArg(args["methods"], args["方法"]),
		HeadersFromEnv: headersFromEnv,
		TimeoutMs:      timeoutMs,
	}, nil
}

func routeEntry(upstream string, stream bool, stripPrefix string, methods []string, headersFromEnv string, timeoutMs uint64) map[string]any {
	methodAny := make([]any, len(methods))
	for i, m := range methods {
		methodAny[i] = m
	}
	return map[string]any{
		"upstream":         upstream,
		"stream":           stream,
		"strip_prefix":     stripPrefix,
		"methods":          methodAny,
		"headers_from_env": headersFromEnv,
		"timeout_ms":       float64(timeoutMs),
	}
}

func normalizeTablePath(raw string) string {
	s := strings.TrimSpace(raw)
	if s == "" {
		return ""
	}
	if !strings.HasPrefix(s, "/") {
		return "/" + s
	}
	return s
}

func parseMethods(raw string) []string {
	if strings.TrimSpace(raw) == "" {
		return []string{"POST"}
	}
	var out []string
	for _, part := range strings.Split(raw, ",") {
		part = strings.TrimSpace(part)
		if part != "" {
			out = append(out, strings.ToUpper(part))
		}
	}
	if len(out) == 0 {
		return []string{"POST"}
	}
	return out
}

func parseMethodsArg(v any, alt any) []string {
	if v == nil {
		v = alt
	}
	if v == nil {
		return nil
	}
	switch t := v.(type) {
	case []any:
		var out []string
		for _, x := range t {
			if s, ok := x.(string); ok && strings.TrimSpace(s) != "" {
				out = append(out, strings.ToUpper(strings.TrimSpace(s)))
			}
		}
		return out
	case string:
		return parseMethods(t)
	}
	return nil
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
	}
	return false
}

func asUint64(v any) (uint64, bool) {
	switch t := v.(type) {
	case float64:
		if t >= 0 {
			return uint64(t), true
		}
	case int64:
		if t >= 0 {
			return uint64(t), true
		}
	case int:
		if t >= 0 {
			return uint64(t), true
		}
	case string:
		n, err := strconv.ParseUint(strings.TrimSpace(t), 10, 64)
		return n, err == nil
	}
	return 0, false
}

func first(args map[string]any, keys ...string) (any, bool) {
	for _, k := range keys {
		if v, ok := args[k]; ok && v != nil {
			return v, true
		}
	}
	return nil, false
}

func argStr(args map[string]any, keys ...string) (string, bool) {
	v, ok := first(args, keys...)
	if !ok {
		return "", false
	}
	s, _ := v.(string)
	return s, true
}

func argStrReq(args map[string]any, keys ...string) (string, error) {
	s, ok := argStr(args, keys...)
	if !ok || strings.TrimSpace(s) == "" {
		return "", fmt.Errorf("missing `%s`", keys[0])
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
