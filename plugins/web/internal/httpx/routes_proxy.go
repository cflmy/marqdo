package httpx

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

type proxyRoute struct {
	Upstream       string
	Stream         bool
	StripPrefix    string
	Methods        []string
	HeadersFromEnv []envHeader
	TimeoutMs      uint64
}

type envHeader struct {
	Env    string
	Header string
}

func parseProxyRoute(v any) proxyRoute {
	m, _ := v.(map[string]any)
	stream := true
	if s, ok := m["stream"].(bool); ok {
		stream = s
	}
	timeout := uint64(120_000)
	if n, ok := m["timeout_ms"].(float64); ok && n > 0 {
		timeout = uint64(n)
	}
	var methods []string
	if arr, ok := m["methods"].([]any); ok {
		for _, x := range arr {
			if s, ok := x.(string); ok && s != "" {
				methods = append(methods, strings.ToUpper(s))
			}
		}
	}
	if len(methods) == 0 {
		methods = []string{"POST"}
	}
	return proxyRoute{
		Upstream:       strOpt(m, "upstream", ""),
		Stream:         stream,
		StripPrefix:    strOpt(m, "strip_prefix", ""),
		Methods:        methods,
		HeadersFromEnv: parseHeadersFromEnv(m["headers_from_env"]),
		TimeoutMs:      timeout,
	}
}

func parseHeadersFromEnv(v any) []envHeader {
	switch t := v.(type) {
	case string:
		return parseHeadersFromEnvStr(t)
	case []any:
		var out []envHeader
		for _, x := range t {
			if s, ok := x.(string); ok {
				out = append(out, parseHeadersFromEnvStr(s)...)
			}
		}
		return out
	default:
		return nil
	}
}

func parseHeadersFromEnvStr(s string) []envHeader {
	var out []envHeader
	for _, part := range strings.FieldsFunc(s, func(r rune) bool { return r == ',' || r == ';' }) {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}
		env, hdr, _ := strings.Cut(part, "=")
		env = strings.TrimSpace(env)
		hdr = strings.TrimSpace(hdr)
		if env == "" || hdr == "" {
			continue
		}
		out = append(out, envHeader{Env: env, Header: hdr})
	}
	return out
}

func (st *state) mountProxyRoutes(mux *http.ServeMux) {
	for _, key := range sortedKeys(st.proxyRoutes) {
		cfg := parseProxyRoute(st.proxyRoutes[key])
		path := routePathKey(key)
		for _, method := range cfg.Methods {
			pattern := method + " " + path
			routeCfg := cfg
			mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
				if r.Method != method {
					http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
					return
				}
				st.handleProxy(w, r, routeCfg)
			})
		}
	}
}

func (st *state) handleProxy(w http.ResponseWriter, r *http.Request, route proxyRoute) {
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 64<<20))
	if err != nil {
		http.Error(w, fmt.Sprintf("read body: %v", err), http.StatusBadRequest)
		return
	}

	target, err := buildUpstreamURL(route, r.URL.Path, r.URL.RawQuery)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadGateway)
		return
	}

	timeout := time.Duration(route.TimeoutMs) * time.Millisecond
	client := &http.Client{Timeout: timeout}

	req, err := http.NewRequestWithContext(r.Context(), r.Method, target, nil)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if len(body) > 0 {
		req.Body = io.NopCloser(bytes.NewReader(body))
		req.ContentLength = int64(len(body))
	}

	for name, vals := range r.Header {
		lname := strings.ToLower(name)
		if isHopByHop(lname) || lname == "host" {
			continue
		}
		for _, v := range vals {
			req.Header.Add(name, v)
		}
	}

	for _, eh := range route.HeadersFromEnv {
		val := os.Getenv(eh.Env)
		if val == "" {
			continue
		}
		if strings.EqualFold(eh.Header, "authorization") &&
			!strings.HasPrefix(strings.ToLower(val), "bearer ") {
			val = "Bearer " + val
		}
		req.Header.Set(eh.Header, val)
	}

	resp, err := client.Do(req)
	if err != nil {
		http.Error(w, fmt.Sprintf("upstream: %v", err), http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	for name, vals := range resp.Header {
		lname := strings.ToLower(name)
		if isHopByHop(lname) {
			continue
		}
		for _, v := range vals {
			w.Header().Add(name, v)
		}
	}
	if route.Stream {
		w.Header().Set("Cache-Control", "no-cache, no-transform")
		w.Header().Set("X-Accel-Buffering", "no")
	}
	w.WriteHeader(resp.StatusCode)
	if route.Stream {
		_, _ = io.Copy(w, resp.Body)
		return
	}
	out, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	_, _ = w.Write(out)
}

func buildUpstreamURL(route proxyRoute, reqPath, query string) (string, error) {
	base := expandEnv(route.Upstream)
	if base == "" {
		return "", fmt.Errorf("proxy upstream empty after env expand")
	}
	path := reqPath
	strip := strings.TrimSpace(route.StripPrefix)
	if strip != "" {
		stripN := routePathKey(strip)
		if strings.HasPrefix(path, stripN) {
			path = path[len(stripN):]
			if path == "" {
				path = "/"
			} else if !strings.HasPrefix(path, "/") {
				path = "/" + path
			}
		}
	}

	if !strings.Contains(base, "://") {
		return "", fmt.Errorf("proxy upstream must be absolute URL, got `%s`", base)
	}

	trimmed := strings.TrimRight(base, "/")
	idx := strings.Index(trimmed, "://")
	after := ""
	if idx >= 0 {
		after = trimmed[idx+3:]
	}
	var url string
	if strings.Contains(after, "/") {
		if path == "/" {
			url = trimmed
		} else {
			url = trimmed + path
		}
	} else {
		url = trimmed + path
	}

	if query != "" {
		if strings.Contains(url, "?") {
			url += "&" + query
		} else {
			url += "?" + query
		}
	}
	return url, nil
}

func expandEnv(s string) string {
	var out strings.Builder
	b := s
	i := 0
	for i < len(b) {
		if b[i] == '$' {
			if i+1 < len(b) && b[i+1] == '{' {
				if end := strings.Index(b[i+2:], "}"); end >= 0 {
					name := b[i+2 : i+2+end]
					out.WriteString(os.Getenv(name))
					i += 3 + end
					continue
				}
			} else {
				start := i + 1
				end := start
				for end < len(b) && (isAlnum(b[end]) || b[end] == '_') {
					end++
				}
				if end > start {
					out.WriteString(os.Getenv(b[start:end]))
					i = end
					continue
				}
			}
		}
		out.WriteByte(b[i])
		i++
	}
	return out.String()
}

func isAlnum(c byte) bool {
	return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9')
}

func isHopByHop(name string) bool {
	switch name {
	case "connection", "keep-alive", "proxy-authenticate", "proxy-authorization",
		"te", "trailers", "transfer-encoding", "upgrade", "content-length":
		return true
	default:
		return false
	}
}
