package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
)

type invokeRoute struct {
	Method     string
	FnPath     string
	BodyMode   string
	ReturnMode string
}

func parseInvokeRoute(v any) invokeRoute {
	m, _ := v.(map[string]any)
	method := strings.ToUpper(strOpt(m, "method", "POST"))
	body := strings.ToLower(strOpt(m, "body", "json"))
	ret := strings.ToLower(strOpt(m, "return", "json"))
	fn := strOpt(m, "fn", "")
	if fn == "" {
		fn = strOpt(m, "function", "")
	}
	return invokeRoute{
		Method:     method,
		FnPath:     fn,
		BodyMode:   body,
		ReturnMode: ret,
	}
}

func (st *state) mountInvokeRoutes(mux *http.ServeMux) {
	for _, key := range sortedKeys(st.invokeRoutes) {
		cfg := parseInvokeRoute(st.invokeRoutes[key])
		path := routePathKey(key)
		method := cfg.Method
		if method == "" {
			method = http.MethodPost
		}
		pattern := method + " " + path
		routeCfg := cfg
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			if r.Method != method {
				http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
				return
			}
			st.handleInvoke(w, r, routeCfg)
		})
	}
}

func (st *state) handleInvoke(w http.ResponseWriter, r *http.Request, route invokeRoute) {
	queryMap := queryToMap(r.URL.RawQuery)

	var payload map[string]any
	switch route.BodyMode {
	case "query":
		payload = queryMap
	case "raw":
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		payload = map[string]any{"payload": string(body)}
	case "form":
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		payload = formToMap(body)
	default:
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		if len(body) == 0 {
			payload = queryMap
		} else {
			var raw map[string]any
			if err := json.Unmarshal(body, &raw); err != nil {
				jsonErr(w, http.StatusBadRequest, fmt.Sprintf("json: %v", err))
				return
			}
			if raw == nil {
				payload = queryMap
			} else {
				for k, v := range queryMap {
					if _, exists := raw[k]; !exists {
						raw[k] = v
					}
				}
				payload = raw
			}
		}
	}

	// Bind common auth / client headers for OpenAI-compatible and API-key surfaces.
	if auth := strings.TrimSpace(r.Header.Get("Authorization")); auth != "" {
		if _, exists := payload["authorization"]; !exists {
			payload["authorization"] = auth
		}
	}
	if key := strings.TrimSpace(r.Header.Get("X-Api-Key")); key != "" {
		if _, exists := payload["api_key"]; !exists {
			payload["api_key"] = key
		}
	}

	args := invokeArgsFromPayload(payload)
	value, err := plugin.CallLibPath(route.FnPath, args)
	if err != nil {
		code := http.StatusInternalServerError
		msg := err.Error()
		if strings.Contains(msg, "unknown library") ||
			strings.Contains(msg, "call_lib_path: need") ||
			strings.Contains(msg, "not allowed") ||
			strings.Contains(msg, "no site") ||
			strings.Contains(msg, "no active host") {
			code = http.StatusBadRequest
		}
		if strings.Contains(msg, "unauthorized") || strings.Contains(msg, "Unauthorized") {
			code = http.StatusUnauthorized
		}
		jsonErr(w, code, msg)
		return
	}

	// OpenAI-compatible handlers may return `{sse: "..."}` to force event-stream
	// even when the route's declared return mode is json.
	if m, ok := value.(map[string]any); ok {
		if s, ok := m["sse"].(string); ok && strings.TrimSpace(s) != "" {
			writeSSE(w, m)
			return
		}
	}

	switch route.ReturnMode {
	case "text", "html", "text/html":
		if value == nil {
			jsonErr(w, http.StatusInternalServerError, "null result")
			return
		}
		text := fmt.Sprint(value)
		if s, ok := value.(string); ok {
			text = s
		}
		ct := "text/plain; charset=utf-8"
		if route.ReturnMode == "html" || route.ReturnMode == "text/html" {
			ct = "text/html; charset=utf-8"
		}
		w.Header().Set("Content-Type", ct)
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, text)
	case "sse", "event-stream", "text/event-stream":
		writeSSE(w, value)
	case "status":
		if value == nil {
			jsonErr(w, http.StatusInternalServerError, "null result")
			return
		}
		w.WriteHeader(http.StatusNoContent)
	default:
		code, body, err := jsonWrapInvokeValue(value)
		if err != nil {
			jsonErr(w, code, err.Error())
			return
		}
		writeJSON(w, code, body)
	}
}
