package form

import (
	"encoding/json"
	"strings"
	"time"
)

func normalizeSource(raw string) string {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "client"
	}
	switch strings.ToLower(s) {
	case "client", "用户", "input", "form":
		return "client"
	case "user", "username", "session.user", "session.username", "用户名":
		return "session.username"
	case "now", "utc.now", "server.now", "时间", "utc":
		return "now"
	default:
		return s
	}
}

// IsClientSource reports whether the field is filled by the browser.
func IsClientSource(source string) bool {
	return normalizeSource(source) == "client"
}

// RequestContext carries session user and route params for form source stamping
// (design: ext-web-request-context.md). Domain-agnostic — no table-name branches.
type RequestContext struct {
	Username   string
	Params     map[string]any
	ReturnPath string
	Path       string
	Method     string
}

// ApplySources strips client-supplied values for non-client fields, then stamps
// from ctx. Always overwrites server-owned keys.
func ApplySources(formBag map[string]any, data map[string]any, ctx *RequestContext) {
	if data == nil {
		return
	}
	if ctx == nil {
		ctx = &RequestContext{}
	}
	for _, f := range fieldSlice(formBag) {
		fm, ok := f.(map[string]any)
		if !ok {
			continue
		}
		name, _ := fm["name"].(string)
		if name == "" {
			continue
		}
		src := normalizeSource(strOf(fm, "source", "client"))
		if src == "client" {
			continue
		}
		delete(data, name)
		if v, ok := resolveSource(src, ctx); ok {
			data[name] = v
		}
	}
}

func resolveSource(src string, ctx *RequestContext) (any, bool) {
	src = normalizeSource(src)
	switch src {
	case "client":
		return nil, false
	case "session.username":
		if ctx.Username == "" {
			return nil, false
		}
		return ctx.Username, true
	case "now":
		return time.Now().UTC().Format("2006-01-02 15:04:05"), true
	}
	lower := strings.ToLower(src)
	for _, prefix := range []string{"route.", "param.", "params."} {
		if strings.HasPrefix(lower, prefix) {
			key := src[len(prefix):]
			if key == "" || ctx.Params == nil {
				return nil, false
			}
			if v, ok := ctx.Params[key]; ok {
				return v, true
			}
			for k, v := range ctx.Params {
				if strings.EqualFold(k, key) {
					return v, true
				}
			}
			return nil, false
		}
	}
	return nil, false
}

// ParseMQParams decodes the hidden `_mq_params` JSON posted with forms.
func ParseMQParams(raw string) map[string]any {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil
	}
	var out map[string]any
	if err := json.Unmarshal([]byte(raw), &out); err != nil {
		return nil
	}
	return out
}
