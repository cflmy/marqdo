package ws

import (
	"fmt"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
)

// RouteWS registers path → {mode} on app.ws_routes (Rust web_app_route_ws).
func RouteWS(appBag map[string]any, path string, args map[string]any) (map[string]any, error) {
	path, err := app.NormalizeRoutePath(path, appBag)
	if err != nil {
		return nil, err
	}
	mode := ModeFromArgs(args)
	out := clone(appBag)
	wsRoutes := map[string]any{}
	if r, ok := out["ws_routes"].(map[string]any); ok {
		wsRoutes = clone(r)
	}
	wsRoutes[path] = map[string]any{"mode": string(mode)}
	out["ws_routes"] = wsRoutes
	return out, nil
}

// RoutesOf reads ws_routes from an app bag.
func RoutesOf(appBag map[string]any) map[string]Mode {
	out := map[string]Mode{}
	obj, ok := appBag["ws_routes"].(map[string]any)
	if !ok {
		return out
	}
	for path, spec := range obj {
		switch t := spec.(type) {
		case map[string]any:
			if m, ok := t["mode"]; ok {
				out[path] = ParseMode(m)
				continue
			}
			out[path] = ParseMode(t)
		case string:
			out[path] = ParseMode(t)
		default:
			out[path] = ParseMode(spec)
		}
	}
	return out
}

func clone(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

// ModeOf returns the mode for one ws route path, or error if missing.
func ModeOf(appBag map[string]any, path string) (Mode, error) {
	routes, ok := appBag["ws_routes"].(map[string]any)
	if !ok {
		return "", fmt.Errorf("missing ws route %s", path)
	}
	spec, ok := routes[path]
	if !ok {
		return "", fmt.Errorf("missing ws route %s", path)
	}
	if m, ok := spec.(map[string]any); ok {
		if mode, ok := m["mode"]; ok {
			return ParseMode(mode), nil
		}
		return ParseMode(m), nil
	}
	return ParseMode(spec), nil
}
