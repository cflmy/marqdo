// Package form ports minimal form bag helpers for assemble smoke (W-G1/W-G3 start).
package form

import "github.com/marqdo/marqdo/plugins/web/internal/table"

// New matches Rust form_new.
func New(tableName, action, id string) map[string]any {
	out := map[string]any{
		"_type":  "form",
		"action": action,
		"fields": []any{},
		"rules":  []any{},
	}
	if action == "" {
		out["action"] = "insert"
	}
	if tableName != "" {
		out["table"] = tableName
	}
	if id != "" {
		out["id"] = id
	}
	return out
}

// SetFields attaches normalized field rows to the form bag.
func SetFields(formBag map[string]any, fields any) map[string]any {
	out := map[string]any{}
	for k, v := range formBag {
		out[k] = v
	}
	out["fields"] = normalizeFormFields(fields)
	return out
}

func normalizeFormFields(fields any) []any {
	// Prefer list-of-maps shape from GFM @ tables; fall back to AsFields-like maps.
	switch t := fields.(type) {
	case []any:
		var out []any
		for _, row := range t {
			m, ok := row.(map[string]any)
			if !ok {
				continue
			}
			name := pickStr(m, "字段", "name", "列", "column")
			if name == "" {
				continue
			}
			item := map[string]any{
				"name":  name,
				"label": pickStr(m, "标签", "label", "title"),
				"type":  pickStrDef(m, "text", "类型", "type"),
			}
			if v, ok := m["必填"]; ok {
				item["required"] = boolish(v)
			} else if v, ok := m["required"]; ok {
				item["required"] = boolish(v)
			}
			if v, ok := m["默认"]; ok {
				item["default"] = v
			} else if v, ok := m["default"]; ok {
				item["default"] = v
			}
			out = append(out, item)
		}
		if len(out) > 0 {
			return out
		}
	}
	// Columnar / schema fallback
	af := table.AsFields(fields)
	if arr, ok := af.([]any); ok {
		return arr
	}
	return []any{}
}

func pickStr(m map[string]any, keys ...string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok && v != nil {
			if s, ok := v.(string); ok {
				return s
			}
		}
	}
	return ""
}

func pickStrDef(m map[string]any, def string, keys ...string) string {
	if s := pickStr(m, keys...); s != "" {
		return s
	}
	return def
}

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		switch t {
		case "1", "true", "True", "yes", "是", "可":
			return true
		}
	case float64:
		return t != 0
	}
	return false
}
