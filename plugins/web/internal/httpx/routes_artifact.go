package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

// artifactRoute is one Endpoint file registered by web.serve (ADR 0007).
type artifactRoute struct {
	Method   string
	Path     string
	File     string
	Request  string // json | form | none | query
	Response string // json | text | html
	Auth     string // role name or "" / "none"
}

func parseArtifactRoute(v any) artifactRoute {
	m, _ := v.(map[string]any)
	return artifactRoute{
		Method:   strings.ToUpper(strOpt(m, "method", "GET")),
		Path:     strOpt(m, "path", ""),
		File:     strOpt(m, "file", ""),
		Request:  strings.ToLower(strOpt(m, "request", "json")),
		Response: strings.ToLower(strOpt(m, "response", "json")),
		Auth:     strOpt(m, "auth", ""),
	}
}

func (st *state) mountArtifactRoutes(mux *http.ServeMux) {
	for _, key := range sortedKeys(st.artifactRoutes) {
		cfg := parseArtifactRoute(st.artifactRoutes[key])
		if cfg.File == "" || cfg.Path == "" {
			continue
		}
		method := cfg.Method
		if method == "" {
			method = http.MethodGet
		}
		pattern := method + " " + goMuxPattern(cfg.Path)
		routeCfg := cfg
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			if r.Method != method {
				http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
				return
			}
			st.handleArtifact(w, r, routeCfg)
		})
	}
}

func (st *state) handleArtifact(w http.ResponseWriter, r *http.Request, route artifactRoute) {
	if route.Auth != "" && !strings.EqualFold(route.Auth, "none") {
		cookie := r.Header.Get("Cookie")
		user := session.UsernameFromCookie(cookie)
		if user == "" {
			jsonErr(w, http.StatusUnauthorized, "auth required")
			return
		}
		role := session.RoleFromCookie(cookie)
		if role != "" && role != route.Auth && role != "admin" && role != "superadmin" {
			jsonErr(w, http.StatusForbidden, "forbidden")
			return
		}
	}

	payload := map[string]any{}
	switch route.Request {
	case "none":
		payload = queryToMap(r.URL.RawQuery)
	case "query":
		payload = queryToMap(r.URL.RawQuery)
	case "form":
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		payload = formToMap(body)
	default: // json
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		q := queryToMap(r.URL.RawQuery)
		if len(body) == 0 {
			payload = q
		} else {
			var raw map[string]any
			if err := json.Unmarshal(body, &raw); err != nil {
				jsonErr(w, http.StatusBadRequest, fmt.Sprintf("json: %v", err))
				return
			}
			if raw == nil {
				payload = q
			} else {
				for k, v := range q {
					if _, exists := raw[k]; !exists {
						raw[k] = v
					}
				}
				payload = raw
			}
		}
	}
	for _, name := range pathParamNames(route.Path) {
		if v := r.PathValue(name); v != "" {
			payload[name] = v
		}
	}

	value, err := plugin.RunArtifact(route.File, payload)
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}

	switch route.Response {
	case "text":
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, fmt.Sprint(value))
	case "html", "text/html":
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, fmt.Sprint(value))
	default:
		code, body, err := jsonWrapInvokeValue(value)
		if err != nil {
			jsonErr(w, code, err.Error())
			return
		}
		writeJSON(w, code, body)
	}
}
