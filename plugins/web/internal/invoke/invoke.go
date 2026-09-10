// Package invoke ports offline invoke route bags (Rust invoke.rs).
package invoke

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// Options is one invoke mount (web_app_invoke args).
type Options struct {
	Path   string
	Method string
	Fn     string
	Body   string
	Return string
}

// RoutesFromTable normalizes `|路径|方法|函数|正文|返回|` into invoke_routes map keys.
func RoutesFromTable(tableV any) map[string]any {
	rows, _ := table.AsRows(tableV).([]any)
	out := map[string]any{}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		path := strings.Trim(strings.TrimSpace(col(m, "路径", "path", "Path")), "/")
		if path == "" {
			continue
		}
		fnPath := col(m, "函数", "fn", "function", "Function")
		if fnPath == "" {
			continue
		}
		method := strings.ToUpper(strings.TrimSpace(col(m, "方法", "method", "Method")))
		if method == "" {
			method = "POST"
		}
		body := strings.ToLower(strings.TrimSpace(col(m, "正文", "body", "Body")))
		if body == "" {
			body = "json"
		}
		ret := strings.ToLower(strings.TrimSpace(col(m, "返回", "return", "Return")))
		if ret == "" {
			ret = "json"
		}
		out[path] = routeEntry(method, fnPath, body, ret)
	}
	return out
}

// Register adds one invoke route to app.invoke_routes (Rust web_app_invoke).
func Register(appBag map[string]any, opts Options) (map[string]any, error) {
	fnPath := strings.TrimSpace(opts.Fn)
	if fnPath == "" {
		return nil, fmt.Errorf("invoke requires `fn`")
	}
	if !strings.Contains(fnPath, ".") || strings.Contains(fnPath, "/") || strings.Contains(fnPath, "#") {
		return nil, fmt.Errorf("invoke `fn` must be `lib.member` (imported on the entry module)")
	}
	path, err := app.NormalizeRoutePath(opts.Path, appBag)
	if err != nil {
		return nil, err
	}
	key := strings.TrimPrefix(path, "/")
	method := strings.ToUpper(strings.TrimSpace(opts.Method))
	if method == "" {
		method = "POST"
	}
	body := strings.ToLower(strings.TrimSpace(opts.Body))
	if body == "" {
		body = "json"
	}
	ret := strings.ToLower(strings.TrimSpace(opts.Return))
	if ret == "" {
		ret = "json"
	}
	out := clone(appBag)
	routes := map[string]any{}
	if r, ok := out["invoke_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[key] = routeEntry(method, fnPath, body, ret)
	out["invoke_routes"] = routes
	return out, nil
}

// OptionsFromArgs parses web_app_invoke ABI args.
func OptionsFromArgs(args map[string]any) (Options, error) {
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return Options{}, err
	}
	fnPath, err := argStrReq(args, "fn", "function", "函数")
	if err != nil {
		return Options{}, err
	}
	method, _ := argStr(args, "method", "方法")
	body, _ := argStr(args, "body", "正文")
	ret, _ := argStr(args, "return", "返回")
	return Options{
		Path:   path,
		Method: method,
		Fn:     fnPath,
		Body:   body,
		Return: ret,
	}, nil
}

func routeEntry(method, fnPath, body, ret string) map[string]any {
	return map[string]any{
		"method": method,
		"fn":     fnPath,
		"body":   body,
		"return": ret,
	}
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
	default:
		return fmt.Sprint(t)
	}
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
