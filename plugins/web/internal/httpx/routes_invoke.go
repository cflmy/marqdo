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
		jsonErr(w, code, msg)
		return
	}

	switch route.ReturnMode {
	case "text":
		if value == nil {
			jsonErr(w, http.StatusInternalServerError, "null result")
			return
		}
		text := fmt.Sprint(value)
		if s, ok := value.(string); ok {
			text = s
		}
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, text)
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
