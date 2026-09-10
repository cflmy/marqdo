// Package httpx implements blocking HTTP listen for the Marqdo web app bag.
package httpx

import (
	"compress/gzip"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/render"
)

// NewHandler builds the HTTP handler for an app bag (no listen).
// Relative static_dir is resolved against entryDir when non-empty, else cwd.
func NewHandler(appBag map[string]any, entryDir string) (http.Handler, error) {
	if appBag == nil {
		appBag = map[string]any{}
	}
	page, _ := appBag["page"].(map[string]any)
	if page == nil {
		page = map[string]any{}
	}
	dbURL := dbURLOf(appBag)
	routes := map[string]any{}
	if r, ok := appBag["routes"].(map[string]any); ok {
		routes = r
	}
	forms := map[string]any{}
	if f, ok := appBag["forms"].(map[string]any); ok {
		forms = cloneMap(f)
	}
	collectPageForms(page, forms)
	for _, p := range routes {
		if pm, ok := p.(map[string]any); ok {
			collectPageForms(pm, forms)
		}
	}

	staticMount := app.NormalizeStaticMount(strOpt(appBag, "static_mount", "/static"))
	var staticDir string
	if s, ok := appBag["static_dir"].(string); ok && strings.TrimSpace(s) != "" {
		staticDir = resolvePath(strings.TrimSpace(s), entryDir)
		fi, err := os.Stat(staticDir)
		if err != nil || !fi.IsDir() {
			return nil, fmt.Errorf("static dir `%s` is not a directory", staticDir)
		}
	}

	mw := middleware.Parse(appBag)
	st := &state{
		page:     page,
		dbURL:    dbURL,
		routes:   routes,
		forms:    forms,
		mw:       mw,
		entryDir: entryDir,
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /{$}", st.handleHome)
	mux.HandleFunc("GET /_part/{id}", st.handleHomePart)
	mux.HandleFunc("GET /_form/{id}", st.handleFormGet)
	mux.HandleFunc("POST /_form/{id}", st.handleFormPost)

	paths := sortedKeys(routes)
	for _, p := range paths {
		routePath := p
		pageV, _ := routes[routePath].(map[string]any)
		if pageV == nil {
			pageV = map[string]any{}
		}
		if strings.Contains(routePath, "{") {
			mux.HandleFunc("GET "+routePath, st.makeDynamicPage(pageV, routePath))
			mux.HandleFunc("GET "+routePath+"/_part/{id}", st.makeDynamicPart(pageV, routePath))
		} else {
			pv := pageV
			mux.HandleFunc("GET "+routePath, func(w http.ResponseWriter, r *http.Request) {
				st.writePage(w, pv)
			})
			mux.HandleFunc("GET "+routePath+"/_part/{id}", func(w http.ResponseWriter, r *http.Request) {
				st.writePart(w, pv, r.PathValue("id"))
			})
		}
	}

	for _, jr := range mw.JSONRoutes {
		route := jr
		pattern := route.Method + " " + route.Path
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			st.handleJSON(w, route)
		})
	}

	if staticDir != "" {
		fmt.Fprintf(os.Stderr, "marqdo web static: %s → %s\n", staticMount, staticDir)
		fs := http.StripPrefix(staticMount, http.FileServer(http.Dir(staticDir)))
		mux.Handle("GET "+staticMount+"/", fs)
		mux.Handle("HEAD "+staticMount+"/", fs)
		mux.Handle("GET "+staticMount, fs)
		mux.Handle("HEAD "+staticMount, fs)
	}

	return withMiddleware(mux, mw), nil
}

// Listen binds host:port from the app bag and serves until interrupt/kill.
func Listen(appBag map[string]any, entryDir string) (map[string]any, error) {
	if appBag == nil {
		appBag = map[string]any{}
	}
	handler, err := NewHandler(appBag, entryDir)
	if err != nil {
		return nil, err
	}
	host := strOpt(appBag, "host", "127.0.0.1")
	port := intFrom(appBag["port"], 18081)
	addr := fmt.Sprintf("%s:%d", host, port)
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("bind %s: %w", addr, err)
	}
	fmt.Fprintf(os.Stderr, "marqdo web listening on http://%s\n", addr)

	srv := &http.Server{Handler: handler}
	errCh := make(chan error, 1)
	go func() {
		errCh <- srv.Serve(ln)
	}()

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	defer signal.Stop(sigCh)
	select {
	case sig := <-sigCh:
		_ = srv.Close()
		fmt.Fprintf(os.Stderr, "marqdo web shutdown (%v)\n", sig)
	case err := <-errCh:
		if err != nil && err != http.ErrServerClosed {
			return nil, fmt.Errorf("serve: %w", err)
		}
	}
	return map[string]any{"ok": true}, nil
}

type state struct {
	page     map[string]any
	dbURL    string
	routes   map[string]any
	forms    map[string]any
	mw       middleware.Config
	entryDir string
}

func (st *state) handleHome(w http.ResponseWriter, r *http.Request) {
	st.writePage(w, st.page)
}

func (st *state) handleHomePart(w http.ResponseWriter, r *http.Request) {
	st.writePart(w, st.page, r.PathValue("id"))
}

func (st *state) writePage(w http.ResponseWriter, page map[string]any) {
	html := render.RenderPage(page, st.dbURL, "")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) writePart(w http.ResponseWriter, page map[string]any, id string) {
	html := render.RenderPage(page, st.dbURL, id)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) makeDynamicPage(page map[string]any, pattern string) http.HandlerFunc {
	paramNames := pathParamNames(pattern)
	return func(w http.ResponseWriter, r *http.Request) {
		p := cloneMap(page)
		params := map[string]any{}
		for _, name := range paramNames {
			params[name] = r.PathValue(name)
		}
		injectParams(p, params)
		st.writePage(w, p)
	}
}

func (st *state) makeDynamicPart(page map[string]any, pattern string) http.HandlerFunc {
	paramNames := pathParamNames(pattern)
	return func(w http.ResponseWriter, r *http.Request) {
		p := cloneMap(page)
		params := map[string]any{}
		for _, name := range paramNames {
			params[name] = r.PathValue(name)
		}
		injectParams(p, params)
		st.writePart(w, p, r.PathValue("id"))
	}
}

func (st *state) handleFormGet(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	frm, ok := st.forms[id].(map[string]any)
	if !ok {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		_, _ = io.WriteString(w, fmt.Sprintf("<p>unknown form %s</p>", id))
		return
	}
	html := form.Render(frm, id, nil, nil, "")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) handleFormPost(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	frm, ok := st.forms[id].(map[string]any)
	if !ok {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		_, _ = io.WriteString(w, fmt.Sprintf("<p>unknown form %s</p>", id))
		return
	}
	if st.dbURL == "" {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		_, _ = io.WriteString(w, "<p>no database</p>")
		return
	}
	if err := r.ParseForm(); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	data := map[string]any{}
	for k, vs := range r.PostForm {
		if len(vs) > 0 {
			data[k] = vs[0]
		}
	}
	res, err := form.Submit(frm, data, st.dbURL)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if ok, _ := res["ok"].(bool); ok {
		redir := strOpt(res, "redirect", "/")
		http.Redirect(w, r, redir, http.StatusSeeOther)
		return
	}
	html := form.Render(frm, id, data, res["errors"], "")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) handleJSON(w http.ResponseWriter, route middleware.JSONRouteMount) {
	if st.dbURL == "" {
		writeJSON(w, http.StatusInternalServerError, map[string]any{"error": "no database"})
		return
	}
	opts := db.SelectOpts{Where: route.Where, Order: route.Order}
	out, err := db.Select(st.dbURL, route.Table, route.Limit, opts)
	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]any{"error": err.Error()})
		return
	}
	writeJSON(w, http.StatusOK, out)
}

func withMiddleware(next http.Handler, mw middleware.Config) http.Handler {
	h := next
	if mw.BodyLimit != nil && *mw.BodyLimit > 0 {
		limit := int64(*mw.BodyLimit)
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			r.Body = http.MaxBytesReader(w, r.Body, limit)
			inner.ServeHTTP(w, r)
		})
	}
	if mw.CacheControl != "" {
		cc := mw.CacheControl
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Cache-Control", cc)
			inner.ServeHTTP(w, r)
		})
	}
	if mw.Compress {
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if !strings.Contains(r.Header.Get("Accept-Encoding"), "gzip") {
				inner.ServeHTTP(w, r)
				return
			}
			w.Header().Set("Content-Encoding", "gzip")
			w.Header().Add("Vary", "Accept-Encoding")
			gz := gzip.NewWriter(w)
			defer gz.Close()
			inner.ServeHTTP(&gzipResponseWriter{ResponseWriter: w, w: gz}, r)
		})
	}
	for _, hv := range mw.Security {
		name, ok := securityHeaderName(hv[0])
		if !ok {
			continue
		}
		val := hv[1]
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set(name, val)
			inner.ServeHTTP(w, r)
		})
	}
	if mw.CORS != nil {
		c := mw.CORS
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			applyCORS(w, r, c)
			if r.Method == http.MethodOptions {
				w.WriteHeader(http.StatusNoContent)
				return
			}
			inner.ServeHTTP(w, r)
		})
	}
	if mw.AccessLog {
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			rw := &statusRecorder{ResponseWriter: w, status: 200}
			inner.ServeHTTP(rw, r)
			fmt.Fprintf(os.Stderr, "marqdo web %s %s %d %dms\n",
				r.Method, r.URL.Path, rw.status, time.Since(start).Milliseconds())
		})
	}
	return h
}

func applyCORS(w http.ResponseWriter, r *http.Request, c *middleware.CORSConfig) {
	origin := r.Header.Get("Origin")
	if len(c.AllowOrigins) == 0 {
		w.Header().Set("Access-Control-Allow-Origin", "*")
	} else if origin != "" {
		for _, o := range c.AllowOrigins {
			if o == origin {
				w.Header().Set("Access-Control-Allow-Origin", origin)
				break
			}
		}
	}
	if len(c.Methods) == 0 {
		w.Header().Set("Access-Control-Allow-Methods", "*")
	} else {
		w.Header().Set("Access-Control-Allow-Methods", strings.Join(unique(c.Methods), ", "))
	}
	if len(c.Headers) == 0 {
		w.Header().Set("Access-Control-Allow-Headers", "*")
	} else {
		w.Header().Set("Access-Control-Allow-Headers", strings.Join(unique(c.Headers), ", "))
	}
	if len(c.ExposeHeaders) > 0 {
		w.Header().Set("Access-Control-Expose-Headers", strings.Join(unique(c.ExposeHeaders), ", "))
	}
	if c.Credentials {
		w.Header().Set("Access-Control-Allow-Credentials", "true")
	}
}

func securityHeaderName(raw string) (string, bool) {
	n := strings.ToLower(strings.ReplaceAll(raw, "_", "-"))
	switch n {
	case "x-frame-options":
		return "X-Frame-Options", true
	case "content-security-policy":
		return "Content-Security-Policy", true
	case "x-content-type-options":
		return "X-Content-Type-Options", true
	case "referrer-policy":
		return "Referrer-Policy", true
	case "strict-transport-security", "hsts":
		return "Strict-Transport-Security", true
	case "x-xss-protection":
		return "X-XSS-Protection", true
	default:
		return "", false
	}
}

type gzipResponseWriter struct {
	http.ResponseWriter
	w *gzip.Writer
}

func (g *gzipResponseWriter) Write(b []byte) (int, error) { return g.w.Write(b) }

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (s *statusRecorder) WriteHeader(code int) {
	s.status = code
	s.ResponseWriter.WriteHeader(code)
}

func writeJSON(w http.ResponseWriter, code int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(v)
}

func collectPageForms(page map[string]any, forms map[string]any) {
	if page == nil {
		return
	}
	if obj, ok := page["forms"].(map[string]any); ok {
		for k, v := range obj {
			forms[k] = v
		}
	}
	if id, ok := page["form_id"].(string); ok && id != "" {
		if f := page["form"]; f != nil {
			forms[id] = f
		}
	}
}

func injectParams(page map[string]any, params map[string]any) {
	page["params"] = params
	if parts, ok := page["parts"].(map[string]any); ok {
		for k, cfg := range parts {
			if m, ok := cfg.(map[string]any); ok {
				cp := cloneMap(m)
				cp["params"] = params
				parts[k] = cp
			}
		}
	}
}

func pathParamNames(pattern string) []string {
	var names []string
	for _, seg := range strings.Split(pattern, "/") {
		if strings.HasPrefix(seg, "{") && strings.HasSuffix(seg, "}") {
			name := strings.TrimSuffix(strings.TrimPrefix(seg, "{"), "}")
			name = strings.TrimPrefix(name, "*")
			if name != "" && name != "id" {
				names = append(names, name)
			}
		}
	}
	return names
}

func dbURLOf(appBag map[string]any) string {
	if dbObj, ok := appBag["db"].(map[string]any); ok {
		if s, ok := dbObj["url"].(string); ok && s != "" {
			return s
		}
	}
	if s, ok := appBag["db_url"].(string); ok {
		return s
	}
	return ""
}

func resolvePath(p, entryDir string) string {
	if filepath.IsAbs(p) {
		return p
	}
	base := entryDir
	if base == "" {
		base, _ = os.Getwd()
	}
	return filepath.Join(base, p)
}

func strOpt(m map[string]any, key, def string) string {
	if s, ok := m[key].(string); ok && s != "" {
		return s
	}
	return def
}

func intFrom(v any, def int) int {
	switch t := v.(type) {
	case float64:
		return int(t)
	case int:
		return t
	case int64:
		return int(t)
	case string:
		var n int
		if _, err := fmt.Sscanf(t, "%d", &n); err == nil {
			return n
		}
	}
	return def
}

func cloneMap(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func sortedKeys(m map[string]any) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	for i := 0; i < len(keys); i++ {
		for j := i + 1; j < len(keys); j++ {
			if keys[j] < keys[i] {
				keys[i], keys[j] = keys[j], keys[i]
			}
		}
	}
	return keys
}

func unique(ss []string) []string {
	seen := map[string]bool{}
	var out []string
	for _, s := range ss {
		if !seen[s] {
			seen[s] = true
			out = append(out, s)
		}
	}
	return out
}
