package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func esc(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	return s
}

func weakETag(bytes []byte) string {
	var h uint64 = 0xcbf29ce484222325
	for _, b := range bytes {
		h ^= uint64(b)
		h *= 0x0100000001b3
	}
	return fmt.Sprintf("W/\"%x-%d\"", h, len(bytes))
}

func jsonErr(w http.ResponseWriter, code int, msg string) {
	writeJSON(w, code, map[string]any{"ok": false, "error": msg})
}

func routeMapOf(appBag map[string]any, key string) map[string]any {
	if r, ok := appBag[key].(map[string]any); ok {
		return r
	}
	return map[string]any{}
}

func routePathKey(path string) string {
	p := strings.TrimSpace(path)
	if !strings.HasPrefix(p, "/") {
		p = "/" + p
	}
	return p
}

// goMuxPattern converts Marqdo route patterns (`{*key}`) to Go ServeMux (`{key...}`).
func goMuxPattern(path string) string {
	out := path
	for {
		start := strings.Index(out, "{*")
		if start < 0 {
			break
		}
		end := strings.Index(out[start:], "}")
		if end < 0 {
			break
		}
		end += start
		name := out[start+2 : end]
		out = out[:start] + "{" + name + "...}" + out[end+1:]
	}
	return out
}

func resolveSession(r *http.Request) (sid, csrf string, setCookie *string) {
	cookieHdr := r.Header.Get("Cookie")
	id, hadCookie := session.IDFromCookie(cookieHdr)
	if hadCookie && id != "" {
		if tok, ok := session.CSRFFor(id); ok && tok != "" {
			return id, tok, nil
		}
		// Stale/expired/missing session: mint a fresh session and replace the cookie.
		// Previously we returned the dead sid with empty CSRF, so login forms had no
		// _csrf while POST still saw hadCookie=true → "登录令牌失效".
	}
	sid = session.NewID(0)
	tok, _ := session.CSRFFor(sid)
	// Use configured session TTL (never Max-Age=0): browsers drop Max-Age=0
	// immediately, so CSRF cookies from GET /login would never stick.
	c := session.IssueCookie(sid)
	return sid, tok, &c
}

func appendSetCookie(w http.ResponseWriter, setCookie *string) {
	if setCookie != nil && *setCookie != "" {
		w.Header().Add("Set-Cookie", *setCookie)
	}
}

func queryToMap(q string) map[string]any {
	out := map[string]any{}
	if q == "" {
		return out
	}
	for _, pair := range strings.Split(q, "&") {
		if pair == "" {
			continue
		}
		k, v, _ := strings.Cut(pair, "=")
		k, _ = url.QueryUnescape(k)
		v, _ = url.QueryUnescape(v)
		if k != "" {
			out[k] = v
		}
	}
	return out
}

func formToMap(body []byte) map[string]any {
	return queryToMap(string(body))
}

func jsonWrapInvokeValue(value any) (int, map[string]any, error) {
	if value == nil {
		return http.StatusInternalServerError, nil, fmt.Errorf("null result")
	}
	switch t := value.(type) {
	case map[string]any:
		code := http.StatusOK
		if ok, has := t["ok"]; has {
			if b, isBool := ok.(bool); isBool && !b {
				if errMsg, _ := t["error"].(string); strings.EqualFold(errMsg, "unauthorized") ||
					strings.Contains(strings.ToLower(errMsg), "unauthorized") {
					code = http.StatusUnauthorized
				}
			}
		}
		return code, t, nil
	default:
		return http.StatusOK, map[string]any{"ok": true, "result": value}, nil
	}
}

// writeSSE writes OpenAI-compatible (or generic) Server-Sent Events.
// value may be a string body, or a map with key "sse" / "body" / "chunks".
func writeSSE(w http.ResponseWriter, value any) {
	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.WriteHeader(http.StatusOK)
	flusher, _ := w.(http.Flusher)

	writeChunk := func(s string) {
		if s == "" {
			return
		}
		if !strings.HasSuffix(s, "\n") {
			s += "\n"
		}
		_, _ = io.WriteString(w, s)
		if flusher != nil {
			flusher.Flush()
		}
	}

	switch t := value.(type) {
	case string:
		writeChunk(t)
	case map[string]any:
		if s, ok := t["sse"].(string); ok {
			writeChunk(s)
			return
		}
		if s, ok := t["body"].(string); ok {
			writeChunk(s)
			return
		}
		if chunks, ok := t["chunks"].([]any); ok {
			for _, c := range chunks {
				writeChunk(fmt.Sprint(c))
			}
			return
		}
		b, err := json.Marshal(t)
		if err != nil {
			writeChunk("data: {\"error\":\"sse_encode\"}\n\n")
			return
		}
		writeChunk("data: " + string(b) + "\n\n")
		writeChunk("data: [DONE]\n\n")
	default:
		writeChunk("data: " + fmt.Sprint(value) + "\n\n")
		writeChunk("data: [DONE]\n\n")
	}
}

func invokeArgsFromPayload(payload map[string]any) map[string]any {
	args := map[string]any{}
	for k, v := range payload {
		args[k] = v
	}
	if _, ok := args["payload"]; !ok {
		args["payload"] = payload
	}
	return args
}

// payloadWantsStream detects OpenAI-compatible stream:true on the request body.
func payloadWantsStream(payload map[string]any) bool {
	if payload == nil {
		return false
	}
	if v, ok := payload["stream"]; ok {
		return boolishAny(v)
	}
	return false
}

func boolishAny(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		s := strings.ToLower(strings.TrimSpace(t))
		return s == "1" || s == "true" || s == "yes" || s == "on"
	case float64:
		return t != 0
	case int:
		return t != 0
	case int64:
		return t != 0
	default:
		return false
	}
}

func readJSONBody(r *http.Request, max int64) (map[string]any, error) {
	r.Body = http.MaxBytesReader(nil, r.Body, max)
	defer r.Body.Close()
	var raw map[string]any
	if err := json.NewDecoder(r.Body).Decode(&raw); err != nil {
		return nil, err
	}
	if raw == nil {
		raw = map[string]any{}
	}
	return raw, nil
}
