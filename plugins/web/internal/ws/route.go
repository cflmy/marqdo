package ws

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
)

// RouteSpec is one ws_routes entry (mode + optional room_key template).
type RouteSpec struct {
	Mode    Mode
	RoomKey string
}

// RouteWS registers path → {mode, room_key?} on app.ws_routes (Rust web_app_route_ws).
func RouteWS(appBag map[string]any, path string, args map[string]any) (map[string]any, error) {
	path, err := app.NormalizeRoutePath(path, appBag)
	if err != nil {
		return nil, err
	}
	mode := ModeFromArgs(args)
	roomKey := RoomKeyFromArgs(args)
	if mode == ModeRoom && roomKey == "" {
		return nil, fmt.Errorf("mode=room requires room_key")
	}
	out := clone(appBag)
	wsRoutes := map[string]any{}
	if r, ok := out["ws_routes"].(map[string]any); ok {
		wsRoutes = clone(r)
	}
	spec := map[string]any{"mode": string(mode)}
	if roomKey != "" {
		spec["room_key"] = roomKey
	}
	wsRoutes[path] = spec
	out["ws_routes"] = wsRoutes
	return out, nil
}

// RoomKeyFromArgs reads room_key / 房间 from route_ws ABI args.
func RoomKeyFromArgs(args map[string]any) string {
	if v, ok := first(args, "room_key", "房间"); ok {
		return strings.TrimSpace(fmt.Sprint(v))
	}
	return ""
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
	spec, err := RouteSpecOf(appBag, path)
	if err != nil {
		return "", err
	}
	return spec.Mode, nil
}

// RouteSpecOf returns mode and room_key for one ws route path.
func RouteSpecOf(appBag map[string]any, path string) (RouteSpec, error) {
	routes, ok := appBag["ws_routes"].(map[string]any)
	if !ok {
		return RouteSpec{}, fmt.Errorf("missing ws route %s", path)
	}
	spec, ok := routes[path]
	if !ok {
		return RouteSpec{}, fmt.Errorf("missing ws route %s", path)
	}
	return parseRouteSpec(spec), nil
}

// ParseRouteSpec reads mode and room_key from a ws_routes value.
func ParseRouteSpec(spec any) RouteSpec {
	return parseRouteSpec(spec)
}

func parseRouteSpec(spec any) RouteSpec {
	out := RouteSpec{Mode: ModeEcho}
	switch t := spec.(type) {
	case map[string]any:
		if mode, ok := t["mode"]; ok {
			out.Mode = ParseMode(mode)
		} else {
			out.Mode = ParseMode(t)
		}
		if rk, ok := t["room_key"]; ok {
			out.RoomKey = strings.TrimSpace(fmt.Sprint(rk))
		}
	case string:
		out.Mode = ParseMode(t)
	default:
		out.Mode = ParseMode(spec)
	}
	return out
}
