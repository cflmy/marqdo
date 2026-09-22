package ws

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
)

// RouteSpec is one ws_routes entry (mode + optional room_key / on_message / presence).
type RouteSpec struct {
	Mode        Mode
	RoomKey     string
	OnMessage   string
	Presence    bool
	RequireAuth bool
}

// RouteWS registers path → {mode, room_key?, on_message?, presence?} on app.ws_routes.
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
	onMsg := OnMessageFromArgs(args)
	presence := PresenceFromArgs(args)
	// Stateful fan-out routes are private by default. Echo and drain remain
	// useful for public health checks and demos; authors can opt out explicitly.
	requireAuth := mode == ModeBroadcast || mode == ModeRoom
	if v, ok := first(args, "require_auth", "需要登录"); ok && v != nil {
		requireAuth = boolish(v)
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
	if onMsg != "" {
		spec["on_message"] = onMsg
	}
	if presence {
		spec["presence"] = true
	}
	if requireAuth {
		spec["require_auth"] = true
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

// OnMessageFromArgs reads on_message / 消息钩子 (lib.member path).
func OnMessageFromArgs(args map[string]any) string {
	if v, ok := first(args, "on_message", "消息钩子", "钩子"); ok {
		return strings.TrimSpace(fmt.Sprint(v))
	}
	return ""
}

// PresenceFromArgs reads presence / 在场 flag.
func PresenceFromArgs(args map[string]any) bool {
	if v, ok := first(args, "presence", "在场"); ok {
		return boolish(v)
	}
	return false
}

// ExpandTemplate replaces `{name}` tokens using values from PathValue-style map.
func ExpandTemplate(tmpl string, values map[string]string) string {
	if tmpl == "" || len(values) == 0 {
		return tmpl
	}
	out := tmpl
	for k, v := range values {
		out = strings.ReplaceAll(out, "{"+k+"}", v)
	}
	return out
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
		if om, ok := t["on_message"]; ok {
			out.OnMessage = strings.TrimSpace(fmt.Sprint(om))
		} else if om, ok := t["消息钩子"]; ok {
			out.OnMessage = strings.TrimSpace(fmt.Sprint(om))
		}
		if p, ok := t["presence"]; ok {
			out.Presence = boolish(p)
		} else if p, ok := t["在场"]; ok {
			out.Presence = boolish(p)
		}
		if a, ok := t["require_auth"]; ok {
			out.RequireAuth = boolish(a)
		} else if a, ok := t["需要登录"]; ok {
			out.RequireAuth = boolish(a)
		}
	case string:
		out.Mode = ParseMode(t)
	default:
		out.Mode = ParseMode(spec)
	}
	return out
}
