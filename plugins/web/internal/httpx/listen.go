// Package httpx implements blocking HTTP listen for the Marqdo web app bag.
package httpx

import (
	"bufio"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/assets"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/ratelimit"
	"github.com/marqdo/marqdo/plugins/web/internal/rbac"
	"github.com/marqdo/marqdo/plugins/web/internal/render"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
	"github.com/marqdo/marqdo/plugins/web/internal/sitemap"
	"github.com/marqdo/marqdo/plugins/web/internal/tenant"
)

// NewHandler builds the HTTP handler for an app bag (no listen).
// Relative static_dir is resolved against entryDir when non-empty, else cwd.
func NewHandler(appBag map[string]any, entryDir string) (http.Handler, error) {
	if appBag == nil {
		appBag = map[string]any{}
	}
	authCfg := authConfigOf(appBag)
	dbURL := dbURLOf(appBag)
	if authCfg.rbac && dbURL != "" {
		if err := rbac.EnsureSchema(dbURL); err != nil {
			return nil, fmt.Errorf("rbac ensure: %w", err)
		}
	}
	session.Configure(session.Config{
		DBURL:        dbURL,
		RedisURL:     sessionURLOf(appBag),
		TTLSec:       authCfg.sessionTTL,
		CookieSecure: authCfg.cookieSecure,
	})
	session.Reset(authCfg.sessionTTL)
	ratelimit.Reset()

	page, _ := appBag["page"].(map[string]any)
	if page == nil {
		page = map[string]any{}
	}
	if _, ok := page["asset_version"]; !ok {
		if v := strOpt(appBag, "asset_version", ""); v != "" {
			page["asset_version"] = v
		}
	}
	routes := map[string]any{}
	if r, ok := appBag["routes"].(map[string]any); ok {
		routes = r
	}
	if av, ok := page["asset_version"]; ok {
		for _, p := range routes {
			if pm, ok := p.(map[string]any); ok {
				if _, has := pm["asset_version"]; !has {
					pm["asset_version"] = av
				}
			}
		}
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
	siteHead := assets.HeadLinksFromJSON(appBag["site_head"])
	iconRoutes, err := iconRoutesFromBag(appBag, entryDir, staticDir)
	if err != nil {
		return nil, err
	}
	st := &state{
		page:           page,
		dbURL:          dbURL,
		routes:         routes,
		forms:          forms,
		mw:             mw,
		entryDir:       entryDir,
		siteHead:       siteHead,
		iconRoutes:     iconRoutes,
		redirects:      redirectsOf(appBag),
		sitemapRoutes:  sitemapRoutesOf(appBag),
		robotsBody:     robotsBodyOf(appBag),
		page404:        pageOf(appBag, "page_404"),
		page500:        pageOf(appBag, "page_500"),
		uploadRoutes:   routeMapOf(appBag, "upload_routes"),
		downloadRoutes: routeMapOf(appBag, "download_routes"),
		wsRoutes:       routeMapOf(appBag, "ws_routes"),
		galleryRoutes:  routeMapOf(appBag, "gallery_routes"),
		proxyRoutes:    routeMapOf(appBag, "proxy_routes"),
		invokeRoutes:   routeMapOf(appBag, "invoke_routes"),
		auth:           authCfg,
		tenant:         tenant.FromBag(appBag),
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /{$}", st.handleHome)
	// Part slot uses {part} (not {id}) so dynamic routes like /desk/posts/{id}
	// do not collide with Go ServeMux duplicate wildcard names.
	mux.HandleFunc("GET /_part/{part}", st.handleHomePart)
	mux.HandleFunc("GET /_form/{id}", st.handleFormGet)
	mux.HandleFunc("POST /_form/{id}", st.handleFormPost)
	mux.HandleFunc("POST /_md", st.handleMarkdownPreview)

	paths := sortedKeys(routes)
	for _, p := range paths {
		routePath := p
		pageV, _ := routes[routePath].(map[string]any)
		if pageV == nil {
			pageV = map[string]any{}
		}
		if strings.Contains(routePath, "{") {
			mux.HandleFunc("GET "+goMuxPattern(routePath), st.makeDynamicPage(pageV, routePath))
			mux.HandleFunc("GET "+goMuxPattern(routePath)+"/_part/{part}", st.makeDynamicPart(pageV, routePath))
		} else {
			pv := pageV
			mux.HandleFunc("GET "+routePath, func(w http.ResponseWriter, r *http.Request) {
				st.writePage(w, r, pv)
			})
			mux.HandleFunc("GET "+routePath+"/_part/{part}", func(w http.ResponseWriter, r *http.Request) {
				st.writePart(w, r, pv, r.PathValue("part"))
			})
		}
	}

	for _, jr := range mw.JSONRoutes {
		route := jr
		pattern := route.Method + " " + route.Path
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			st.handleJSON(w, r, route)
		})
	}

	// Icon routes before the static prefix so /static/favicon.svg does not
	// conflict with Go 1.22+ ServeMux method/path overlap rules.
	for _, ir := range iconRoutes {
		url := ir.URL
		path := ir.Path
		ct := ir.ContentType
		if staticDir != "" && (url == staticMount || strings.HasPrefix(url, staticMount+"/")) {
			// FileServer under staticMount already serves these paths.
			fmt.Fprintf(os.Stderr, "marqdo web icon: %s → static %s (via %s)\n", url, path, staticMount)
			continue
		}
		fmt.Fprintf(os.Stderr, "marqdo web icon: %s → %s (%s)\n", url, path, ct)
		mux.HandleFunc("GET "+url, func(w http.ResponseWriter, r *http.Request) {
			st.serveIcon(w, path, ct)
		})
		mux.HandleFunc("HEAD "+url, func(w http.ResponseWriter, r *http.Request) {
			st.serveIcon(w, path, ct)
		})
	}

	if staticDir != "" {
		fmt.Fprintf(os.Stderr, "marqdo web static: %s → %s\n", staticMount, staticDir)
		fs := http.StripPrefix(staticMount, http.FileServer(http.Dir(staticDir)))
		// Register without method so GET+HEAD share one pattern (avoids conflicts
		// with more-specific GET/HEAD icon paths under the same prefix).
		serveStatic := func(w http.ResponseWriter, r *http.Request) {
			if r.Method != http.MethodGet && r.Method != http.MethodHead {
				http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
				return
			}
			// Long browser cache; FileServer still honors Last-Modified / 304.
			// Prefer ?v= / asset_version on HTML refs when assets change.
			w.Header().Set("Cache-Control", "public, max-age=604800")
			fs.ServeHTTP(w, r)
		}
		mux.Handle(staticMount+"/", http.HandlerFunc(serveStatic))
		mux.Handle(staticMount, http.HandlerFunc(serveStatic))
	}

	for _, from := range sortedKeys(st.redirects) {
		spec, _ := st.redirects[from].(map[string]any)
		to := strOpt(spec, "to", "/")
		permanent := boolish(spec["permanent"])
		mux.HandleFunc("GET "+from, func(w http.ResponseWriter, r *http.Request) {
			code := http.StatusTemporaryRedirect
			if permanent {
				code = http.StatusMovedPermanently
			}
			http.Redirect(w, r, to, code)
		})
	}

	for _, path := range sortedKeys(st.sitemapRoutes) {
		cfg, _ := st.sitemapRoutes[path].(map[string]any)
		mux.HandleFunc("GET "+path, func(w http.ResponseWriter, r *http.Request) {
			st.writeSitemap(w, cfg)
		})
	}

	if st.robotsBody != "" {
		body := st.robotsBody
		mux.HandleFunc("GET /robots.txt", func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "text/plain; charset=utf-8")
			_, _ = io.WriteString(w, body)
		})
	}

	st.mountWSRoutes(mux)
	st.mountUploadRoutes(mux)
	st.mountDownloadRoutes(mux)
	st.mountGalleryRoutes(mux)
	st.mountProxyRoutes(mux)
	st.mountInvokeRoutes(mux)
	st.mountAuthRoutes(mux)

	mux.HandleFunc("/{path...}", st.handleNotFound)

	handler := withMiddleware(mux, mw)
	handler = withRBAC(handler, authCfg.gates, authCfg.loginPath)
	handler = withTenant(handler, st.tenant)
	return handler, nil
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
	page           map[string]any
	dbURL          string
	routes         map[string]any
	forms          map[string]any
	mw             middleware.Config
	entryDir       string
	siteHead       []assets.HeadLink
	iconRoutes     []assets.IconRoute
	redirects      map[string]any
	sitemapRoutes  map[string]any
	robotsBody     string
	page404        map[string]any
	page500        map[string]any
	uploadRoutes   map[string]any
	downloadRoutes map[string]any
	wsRoutes       map[string]any
	galleryRoutes  map[string]any
	proxyRoutes    map[string]any
	invokeRoutes   map[string]any
	auth           authConfig
	tenant         tenant.Config
}

func (st *state) handleHome(w http.ResponseWriter, r *http.Request) {
	st.writePage(w, r, st.page)
}

func (st *state) handleHomePart(w http.ResponseWriter, r *http.Request) {
	st.writePart(w, r, st.page, r.PathValue("part"))
}

func (st *state) handleMarkdownPreview(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	html := render.MarkdownToHTML(string(body))
	if html == "" {
		html = `<p class="lede mq-md-empty">预览</p>`
	}
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) preparePage(page map[string]any, r *http.Request) map[string]any {
	p := cloneMap(page)
	assets.MergeSiteHead(p, st.siteHead)
	if r != nil {
		withNavAuth(p, r.Header.Get("Cookie"))
		params, _ := p["params"].(map[string]any)
		if params == nil {
			params = map[string]any{}
		} else {
			params = cloneMap(params)
		}
		for k, vs := range r.URL.Query() {
			if len(vs) == 0 {
				continue
			}
			if _, exists := params[k]; !exists {
				params[k] = vs[0]
			}
		}
		p["params"] = params
	}
	return p
}

// attachCSRF stamps page._csrf so custom login/register pages expose a token
// in meta and (via InjectAuthFields) in POST forms — progressive enhancement, no author JS.
func (st *state) attachCSRF(page map[string]any, r *http.Request) *string {
	if page == nil || r == nil {
		return nil
	}
	if st.auth.users == nil && !st.auth.rbac {
		return nil
	}
	_, csrf, setCookie := resolveSession(r)
	if csrf != "" {
		page["_csrf"] = csrf
	}
	return setCookie
}

func (st *state) writePage(w http.ResponseWriter, r *http.Request, page map[string]any) {
	p := st.preparePage(page, r)
	if st.authEntryRedirect(w, r, p) {
		return
	}
	if next := safeAuthNext(r.URL.Query().Get("next")); next != "" {
		p["_auth_next"] = next
	}
	setCookie := st.attachCSRF(p, r)
	html := render.RenderPage(p, st.dbURL, "")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	_, _ = io.WriteString(w, html)
}

func (st *state) writePart(w http.ResponseWriter, r *http.Request, page map[string]any, id string) {
	p := st.preparePage(page, r)
	setCookie := st.attachCSRF(p, r)
	html := render.RenderPage(p, st.dbURL, id)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	appendSetCookie(w, setCookie)
	_, _ = io.WriteString(w, html)
}

func (st *state) serveIcon(w http.ResponseWriter, path, contentType string) {
	data, err := os.ReadFile(path)
	if err != nil {
		http.NotFound(w, nil)
		return
	}
	w.Header().Set("Content-Type", contentType)
	w.Header().Set("Cache-Control", "public, max-age=86400")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

func (st *state) writeSitemap(w http.ResponseWriter, cfg map[string]any) {
	base := strOpt(cfg, "base", "")
	items := cfg["items"]
	if tbl, ok := cfg["table"].(string); ok && tbl != "" && st.dbURL != "" {
		locCol := strOpt(cfg, "loc", "path")
		limit := int64(1000)
		if n, ok := cfg["limit"].(float64); ok && n > 0 {
			limit = int64(n)
		}
		opts := db.SelectOpts{}
		out, err := db.Select(st.dbURL, tbl, limit, opts)
		if err == nil {
			if rows, ok := out["rows"].([]any); ok {
				norm := make([]any, 0, len(rows))
				for _, row := range rows {
					m, ok := row.(map[string]any)
					if !ok {
						continue
					}
					cp := cloneMap(m)
					if _, has := cp["loc"]; !has {
						if v, ok := cp[locCol]; ok {
							cp["loc"] = v
						}
					}
					norm = append(norm, cp)
				}
				items = norm
			}
		}
	}
	xml := sitemap.BuildSitemap(base, items)
	w.Header().Set("Content-Type", "application/xml; charset=utf-8")
	_, _ = io.WriteString(w, xml)
}

func (st *state) handleNotFound(w http.ResponseWriter, r *http.Request) {
	if st.page404 != nil {
		html := render.RenderPage(st.preparePage(st.page404, r), st.dbURL, "")
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusNotFound)
		_, _ = io.WriteString(w, html)
		return
	}
	http.NotFound(w, r)
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
		st.writePage(w, r, p)
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
		st.writePart(w, r, p, r.PathValue("part"))
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
	if st.tenant.Enabled() && st.tenant.DefaultScope {
		if tid := tenant.FromContext(r.Context()); tid != "" {
			data = tenant.StampRow(data, st.tenant.Column, tid)
		}
	}
	ctx := st.requestContext(r, data)
	delete(data, "_mq_params")
	delete(data, "_mq_return")
	delete(data, "_csrf")
	form.ApplySources(frm, data, ctx)
	res, err := form.Submit(frm, data, st.dbURL)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if ok, _ := res["ok"].(bool); ok {
		// Prefer an explicit form redirect (e.g. cancel_href list URL) over the
		// composing page path stamped as _mq_return.
		redir := strOpt(res, "redirect", "/")
		if (redir == "" || redir == "/") && ctx.ReturnPath != "" {
			redir = ctx.ReturnPath
		}
		http.Redirect(w, r, redir, http.StatusSeeOther)
		return
	}
	html := form.RenderBodyCtx(frm, id, data, res["errors"], "", ctx)
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, html)
}

func (st *state) requestContext(r *http.Request, data map[string]any) *form.RequestContext {
	ctx := &form.RequestContext{
		Username: session.UsernameFromCookie(r.Header.Get("Cookie")),
		Path:     r.URL.Path,
		Method:   r.Method,
	}
	if data != nil {
		if p := form.ParseMQParams(fmt.Sprint(data["_mq_params"])); len(p) > 0 {
			ctx.Params = p
		}
		if ret, ok := data["_mq_return"].(string); ok {
			if back := sameHostPath(ret, r.Host); back != "" {
				ctx.ReturnPath = back
			} else if strings.HasPrefix(ret, "/") && !strings.HasPrefix(ret, "//") {
				ctx.ReturnPath = ret
			}
		}
	}
	if ctx.ReturnPath == "" {
		if back := sameHostPath(r.Referer(), r.Host); back != "" {
			ctx.ReturnPath = back
		}
	}
	return ctx
}

func sameHostPath(raw, host string) string {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ""
	}
	u, err := url.Parse(raw)
	if err != nil || u == nil {
		return ""
	}
	if u.Host != "" && host != "" && !strings.EqualFold(u.Host, host) {
		return ""
	}
	path := u.Path
	if path == "" {
		path = "/"
	}
	if u.RawQuery != "" {
		path += "?" + u.RawQuery
	}
	if !strings.HasPrefix(path, "/") || strings.HasPrefix(path, "//") {
		return ""
	}
	return path
}

func (st *state) handleJSON(w http.ResponseWriter, r *http.Request, route middleware.JSONRouteMount) {
	if st.dbURL == "" {
		writeJSON(w, http.StatusInternalServerError, map[string]any{"error": "no database"})
		return
	}
	where := route.Where
	if (route.TenantScope || st.tenant.DefaultScope) && st.tenant.Enabled() {
		tid := tenant.FromContext(r.Context())
		if tid == "" {
			writeJSON(w, http.StatusBadRequest, map[string]any{"error": "tenant required"})
			return
		}
		where = tenant.MergeWhere(where, st.tenant.Column, tid)
	}
	opts := db.SelectOpts{Where: where, Order: route.Order}
	out, err := db.Select(st.dbURL, route.Table, route.Limit, opts)
	if err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]any{"error": err.Error()})
		return
	}
	writeJSON(w, http.StatusOK, out)
}

func withTenant(next http.Handler, cfg tenant.Config) http.Handler {
	if !cfg.Enabled() {
		return next
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id := tenant.Resolve(r, cfg)
		ctx := tenant.WithContext(r.Context(), id)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

func withMiddleware(next http.Handler, mw middleware.Config) http.Handler {
	h := next
	if mw.BodyLimit != nil && *mw.BodyLimit > 0 {
		limit := int64(*mw.BodyLimit)
		inner := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.ContentLength > limit {
				http.Error(w, "request body too large", http.StatusRequestEntityTooLarge)
				return
			}
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
			setHeader(w, "content-encoding", "gzip")
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
			setHeader(w, strings.ToLower(name), val)
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
		setHeader(w, "access-control-allow-origin", "*")
	} else if origin != "" {
		for _, o := range c.AllowOrigins {
			if o == origin {
				setHeader(w, "access-control-allow-origin", origin)
				break
			}
		}
	}
	if len(c.Methods) == 0 {
		setHeader(w, "access-control-allow-methods", "*")
	} else {
		setHeader(w, "access-control-allow-methods", strings.Join(unique(c.Methods), ", "))
	}
	if len(c.Headers) == 0 {
		setHeader(w, "access-control-allow-headers", "*")
	} else {
		setHeader(w, "access-control-allow-headers", strings.Join(unique(c.Headers), ", "))
	}
	if len(c.ExposeHeaders) > 0 {
		expose := strings.ToLower(strings.Join(unique(c.ExposeHeaders), ", "))
		setHeader(w, "access-control-expose-headers", expose)
	}
	if c.Credentials {
		setHeader(w, "access-control-allow-credentials", "true")
	}
}

func setHeader(w http.ResponseWriter, name, value string) {
	w.Header()[name] = []string{value}
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

func (g *gzipResponseWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	if h, ok := g.ResponseWriter.(http.Hijacker); ok {
		return h.Hijack()
	}
	return nil, nil, fmt.Errorf("underlying ResponseWriter does not support hijacking")
}

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (s *statusRecorder) WriteHeader(code int) {
	s.status = code
	s.ResponseWriter.WriteHeader(code)
}

func (s *statusRecorder) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	if h, ok := s.ResponseWriter.(http.Hijacker); ok {
		return h.Hijack()
	}
	return nil, nil, fmt.Errorf("underlying ResponseWriter does not support hijacking")
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
	// Stamp concrete path onto `_route` so part slot URLs use /post/slug not /post/{slug}.
	if r, ok := page["_route"].(string); ok && r != "" && len(params) > 0 {
		page["_route"] = render.ResolveRouteParams(r, params)
	}
	if parts, ok := page["parts"].(map[string]any); ok {
		for k, cfg := range parts {
			if m, ok := cfg.(map[string]any); ok {
				cp := cloneMap(m)
				cp["params"] = params
				if r, ok := cp["_route"].(string); ok && r != "" {
					cp["_route"] = render.ResolveRouteParams(r, params)
				}
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
			// Include "id" — part slots use {part}, so route {id} is safe for PathValue.
			if name != "" {
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

func sessionURLOf(appBag map[string]any) string {
	if s, ok := appBag["session_url"].(string); ok && s != "" {
		return s
	}
	if auth, ok := appBag["auth"].(map[string]any); ok {
		if s, ok := auth["session_url"].(string); ok && s != "" {
			return s
		}
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

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		return t == "true" || t == "True" || t == "1" || t == "yes"
	case float64:
		return t != 0
	}
	return false
}

func redirectsOf(appBag map[string]any) map[string]any {
	if r, ok := appBag["redirects"].(map[string]any); ok {
		return r
	}
	return map[string]any{}
}

func sitemapRoutesOf(appBag map[string]any) map[string]any {
	if r, ok := appBag["sitemap_routes"].(map[string]any); ok {
		return r
	}
	return map[string]any{}
}

func robotsBodyOf(appBag map[string]any) string {
	if s, ok := appBag["robots_body"].(string); ok {
		return s
	}
	return ""
}

func pageOf(appBag map[string]any, key string) map[string]any {
	if p, ok := appBag[key].(map[string]any); ok {
		return p
	}
	return nil
}

func iconRoutesFromBag(appBag map[string]any, entryDir, staticDir string) ([]assets.IconRoute, error) {
	var routes []assets.IconRoute
	if icons, ok := appBag["icons"].([]any); ok {
		for _, ic := range icons {
			m, ok := ic.(map[string]any)
			if !ok {
				continue
			}
			p := strOpt(m, "path", "")
			url := strOpt(m, "url", "")
			if p == "" || url == "" {
				continue
			}
			if !strings.HasPrefix(url, "/") {
				return nil, fmt.Errorf("icon url `%s` must start with /", url)
			}
			abs := resolvePath(p, entryDir)
			fi, err := os.Stat(abs)
			if err != nil || fi.IsDir() {
				return nil, fmt.Errorf("icon file `%s` not found for `%s`", abs, url)
			}
			ct := strOpt(m, "type", "")
			if ct == "" {
				ct = assets.MimeForPath(p)
			}
			routes = append(routes, assets.IconRoute{
				URL: url, Path: abs, ContentType: ct,
			})
		}
	}
	if len(routes) == 0 && staticDir != "" {
		for _, name := range []string{"favicon.ico", "favicon.png", "favicon.svg"} {
			p := filepath.Join(staticDir, name)
			fi, err := os.Stat(p)
			if err == nil && !fi.IsDir() {
				routes = append(routes, assets.IconRoute{
					URL:         "/favicon.ico",
					Path:        p,
					ContentType: assets.MimeForPath(name),
				})
				break
			}
		}
	}
	return routes, nil
}
