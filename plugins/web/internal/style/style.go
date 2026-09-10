// Package style implements web_style (CSS from GFM style tables).
package style

import (
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// Style matches Rust web_style: args name + table + optional strict → CSS string.
func Style(args map[string]any) (string, error) {
	name := ""
	if v, ok := args["name"]; ok && v != nil {
		name = cellStr(v)
	}
	var tbl any = []any{}
	if v, ok := args["table"]; ok && v != nil {
		tbl = v
	}
	strict := false
	for _, k := range []string{"strict", "strict_css_cells", "严格"} {
		if v, ok := args[k]; ok {
			strict = boolishStrict(v)
			break
		}
	}
	return table.AsCSSNamedChecked(name, tbl, strict)
}

func cellStr(v any) string {
	switch t := v.(type) {
	case string:
		return t
	case float64:
		if t == float64(int64(t)) {
			return strconv.FormatInt(int64(t), 10)
		}
		return strconv.FormatFloat(t, 'f', -1, 64)
	case bool:
		return strconv.FormatBool(t)
	default:
		return ""
	}
}

func boolishStrict(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case float64:
		return int64(t) != 0
	case string:
		s := strings.TrimSpace(t)
		switch s {
		case "1", "true", "True", "yes", "on", "真", "是":
			return true
		}
		return false
	default:
		return false
	}
}
