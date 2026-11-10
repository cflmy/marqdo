package httpx

import (
	"fmt"
	"net/http"
	"regexp"
	"sort"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/artifact"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/oidc"
)

type recordedMux struct {
	validate *http.ServeMux
	patterns []string
}

func (mux *recordedMux) Handle(pattern string, _ http.Handler) {
	// Validate registration conflicts without retaining or running handlers.
	mux.validate.HandleFunc(pattern, func(http.ResponseWriter, *http.Request) {})
	mux.patterns = append(mux.patterns, pattern)
}
func (mux *recordedMux) HandleFunc(pattern string, handler func(http.ResponseWriter, *http.Request)) {
	mux.Handle(pattern, http.HandlerFunc(handler))
}

var tailWildcard = regexp.MustCompile(`\{([^{}]+)\.\.\.\}`)

func inspectionPath(path string) string {
	path = strings.TrimSuffix(path, "{$}")
	return tailWildcard.ReplaceAllString(path, "{*$1}")
}

// InspectApp executes only trusted registration logic. It does not create a
// session, reset global runtime state, query a DB, run Documents or listen.
func InspectApp(bag map[string]any) (report map[string]any, err error) {
	defer func() {
		if recovered := recover(); recovered != nil {
			report = nil
			err = fmt.Errorf("web.inspect: invalid route registration: %v", recovered)
		}
	}()
	if bag == nil {
		bag = map[string]any{}
	}
	routes := routeMapOf(bag, "routes")
	page, _ := bag["page"].(map[string]any)
	forms := map[string]any{}
	if existing, ok := bag["forms"].(map[string]any); ok {
		for key, value := range existing {
			forms[key] = value
		}
	}
	collectPageForms(page, forms)
	for _, value := range routes {
		if page, ok := value.(map[string]any); ok {
			collectPageForms(page, forms)
		}
	}
	root := strOpt(bag, "artifact_root", "")
	staticDir := strOpt(bag, "static_dir", "")
	if staticDir != "" {
		staticDir = resolvePath(staticDir, root)
	}
	icons, err := iconRoutesFromBag(bag, root, staticDir)
	if err != nil {
		return nil, err
	}
	st := &state{
		page: page, routes: routes, forms: forms, mw: middleware.Parse(bag), entryDir: root,
		iconRoutes: icons, redirects: redirectsOf(bag), sitemapRoutes: sitemapRoutesOf(bag),
		robotsBody: robotsBodyOf(bag), page404: pageOf(bag, "page_404"), page500: pageOf(bag, "page_500"),
		uploadRoutes: routeMapOf(bag, "upload_routes"), downloadRoutes: routeMapOf(bag, "download_routes"),
		wsRoutes: routeMapOf(bag, "ws_routes"), galleryRoutes: routeMapOf(bag, "gallery_routes"),
		proxyRoutes: routeMapOf(bag, "proxy_routes"), invokeRoutes: routeMapOf(bag, "invoke_routes"),
		artifactRoutes: routeMapOf(bag, "artifact_routes"), auth: authConfigOf(bag), oidc: oidc.ParseConfig(bag),
	}
	// Only a nonempty DB URL determines whether DB-backed routes are enabled.
	// Its existence and contents are intentionally not inspected here.
	st.dbURL = dbURLOf(bag)
	staticMount := app.NormalizeStaticMount(strOpt(bag, "static_mount", "/static"))
	mux := &recordedMux{validate: http.NewServeMux()}
	st.registerRoutes(mux, staticDir, staticMount, false)
	report = artifact.InspectBag(bag, nil)
	declarations := map[string]map[string]any{}
	byPath := map[string]map[string]any{}
	for _, row := range report["declared"].([]map[string]any) {
		path := inspectionPath(goMuxPattern(fmt.Sprint(row["path"])))
		method := fmt.Sprint(row["method"])
		declarations[method+" "+path] = row
		if _, exists := byPath[path]; !exists {
			byPath[path] = row
		}
	}
	// Classification uses the explicitly supplied Resource paths; the method
	// and presence still come from the actual recorded registration.
	for group, kind := range map[string]string{
		"ws_routes": "websocket", "upload_routes": "upload", "download_routes": "download",
		"gallery_routes": "gallery", "proxy_routes": "proxy", "redirects": "redirect", "sitemap_routes": "sitemap",
	} {
		for key, value := range routeMapOf(bag, group) {
			spec, _ := value.(map[string]any)
			path := strOpt(spec, "path", key)
			if parts := strings.SplitN(path, " ", 2); len(parts) == 2 {
				path = parts[1]
			}
			path = inspectionPath(goMuxPattern(path))
			byPath[path] = map[string]any{"path": path, "kind": kind}
		}
	}
	for _, query := range st.mw.JSONRoutes {
		row := map[string]any{"path": query.Path, "method": query.Method, "kind": "data_query"}
		declarations[query.Method+" "+query.Path] = row
	}
	var declared, system []map[string]any
	seen := map[string]bool{}
	for _, pattern := range mux.patterns {
		method, path := "*", pattern
		if parts := strings.SplitN(pattern, " ", 2); len(parts) == 2 {
			method, path = parts[0], parts[1]
		}
		path = inspectionPath(path)
		if method == "*" && (path == staticMount || path == staticMount+"/") && strOpt(bag, "static_dir", "") != "" {
			method = "GET|HEAD"
		}
		key := method + " " + path
		if seen[key] {
			continue
		}
		seen[key] = true
		row := map[string]any{"method": method, "path": path, "kind": "system"}
		source, found := declarations[key]
		if !found {
			source, found = byPath[path]
		}
		if found {
			for key, value := range source {
				if key != "method" && key != "path" {
					row[key] = value
				}
			}
			declared = append(declared, row)
		} else {
			system = append(system, row)
		}
	}
	less := func(rows []map[string]any) {
		sort.Slice(rows, func(i, j int) bool {
			a, b := fmt.Sprint(rows[i]["path"]), fmt.Sprint(rows[j]["path"])
			if a != b {
				return a < b
			}
			return fmt.Sprint(rows[i]["method"]) < fmt.Sprint(rows[j]["method"])
		})
	}
	less(declared)
	less(system)
	if declared == nil {
		declared = []map[string]any{}
	}
	if system == nil {
		system = []map[string]any{}
	}
	report["scope"] = "registered"
	report["declared"] = declared
	report["system"] = system
	return report, nil
}
