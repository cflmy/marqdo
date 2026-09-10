// Package ws ports WebSocket route registration and client connect from the Rust web plugin.
package ws

import (
	"strings"
)

// Mode is the server-side WebSocket handler mode for a path.
type Mode string

const (
	ModeEcho      Mode = "echo"
	ModeBroadcast Mode = "broadcast"
	ModeDrain     Mode = "drain"
	ModeRoom      Mode = "room"
)

// ParseMode mirrors Rust ws_hub::WsMode::parse.
func ParseMode(v any) Mode {
	switch t := v.(type) {
	case bool:
		if t {
			return ModeEcho
		}
		return ModeDrain
	case string:
		switch strings.TrimSpace(strings.ToLower(t)) {
		case "broadcast", "广播":
			return ModeBroadcast
		case "room", "房间":
			return ModeRoom
		case "drain", "false", "0", "no", "drain_only":
			return ModeDrain
		default:
			return ModeEcho
		}
	case map[string]any:
		if m, ok := t["mode"]; ok {
			return ParseMode(m)
		}
		if m, ok := t["模式"]; ok {
			return ParseMode(m)
		}
		if e, ok := t["echo"]; ok {
			return ParseMode(e)
		}
		if e, ok := t["回显"]; ok {
			return ParseMode(e)
		}
		return ModeEcho
	default:
		return ModeEcho
	}
}

// ModeFromArgs picks mode from route_ws ABI args (mode/模式 or echo/回显, default echo).
func ModeFromArgs(args map[string]any) Mode {
	if v, ok := first(args, "mode", "模式"); ok {
		return ParseMode(v)
	}
	echo := true
	if v, ok := first(args, "echo", "回显"); ok {
		echo = boolish(v)
	}
	if echo {
		return ModeEcho
	}
	return ModeDrain
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
