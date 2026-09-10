// Package app ports minimal app bag helpers (new + route + auth + gate) for offline smoke.
package app

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/assets"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

// New builds an app bag matching Rust web_app_new defaults used by zh-smoke.
func New(args map[string]any) map[string]any {
	page := args["page"]
	if page == nil {
		page = map[string]any{}
	}
	dbv := args["db"]
	host := strOpt(args, "host", "127.0.0.1")
	port := float64(18081)
	if v, ok := args["port"]; ok {
		switch t := v.(type) {
		case float64:
			port = t
		case string:
			// leave default if unparsable
			_ = t
		}
	}
	admin := boolish(args["admin"])
	adminPrefix := strOpt(args, "admin_prefix", "/admin")
	if s := strOpt(args, "后台前缀", ""); s != "" {
		adminPrefix = s
	}
	if p, err := NormalizeAdminPrefix(adminPrefix); err == nil {
		adminPrefix = p
	}
	out := map[string]any{
		"page":         page,
		"db":           dbv,
		"host":         host,
		"port":         port,
		"admin":        admin,
		"admin_prefix": adminPrefix,
		"forms":        map[string]any{},
		"routes":       map[string]any{},
		"ws_routes":    map[string]any{},
		"static_dir":   nil,
		"static_mount": "/static",
	}
	if s := strOpt(args, "shell_css", ""); s == "" {
		s = strOpt(args, "壳样式", "")
		if s != "" {
			out["shell_css"] = s
		}
	} else {
		out["shell_css"] = s
	}
	if s := strOpt(args, "layout", ""); s == "" {
		s = strOpt(args, "布局", "")
		if s != "" {
			out["layout"] = s
		}
	} else {
		out["layout"] = s
	}
	if s := strOpt(args, "asset_version", ""); s == "" {
		s = strOpt(args, "资源版本", "")
		if s != "" {
			out["asset_version"] = s
		}
	} else {
		out["asset_version"] = s
	}
	return out
}

// Route registers path → page on app.routes and stamps page._route.
// When admin=true, paths under admin_prefix are reserved (c0 frees /admin when admin=false).
func Route(appBag map[string]any, path string, page any) (map[string]any, error) {
	path, err := NormalizeRoutePath(path, appBag)
	if err != nil {
		return nil, err
	}
	out := clone(appBag)
	pageMap := map[string]any{}
	if m, ok := page.(map[string]any); ok {
		pageMap = clone(m)
	}
	pageMap["_route"] = path
	routes := map[string]any{}
	if r, ok := out["routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[path] = pageMap
	out["routes"] = routes
	return out, nil
}

// MountForm stores form under app.forms[id].
func MountForm(appBag map[string]any, id string, form any) (map[string]any, error) {
	id = strings.TrimSpace(id)
	if id == "" {
		return nil, fmt.Errorf("missing form id")
	}
	if form == nil {
		return nil, fmt.Errorf("missing `form`")
	}
	out := clone(appBag)
	forms := map[string]any{}
	if f, ok := out["forms"].(map[string]any); ok {
		forms = clone(f)
	}
	forms[id] = form
	out["forms"] = forms
	return out, nil
}

// NormalizeStaticMount mirrors Rust http::normalize_static_mount.
func NormalizeStaticMount(raw string) string {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "/static"
	}
	m := s
	if !strings.HasPrefix(m, "/") {
		m = "/" + m
	}
	for len(m) > 1 && strings.HasSuffix(m, "/") {
		m = strings.TrimSuffix(m, "/")
	}
	return m
}

// Static sets static_dir + static_mount (Rust web_app_static).
func Static(appBag map[string]any, dir, mount string) (map[string]any, error) {
	dir = strings.TrimSpace(dir)
	if dir == "" {
		return nil, fmt.Errorf("static `dir` is empty")
	}
	if mount == "" {
		mount = "/static"
	}
	mount = NormalizeStaticMount(mount)
	if mount == "/_form" || strings.HasPrefix(mount, "/_form/") ||
		mount == "/_part" || strings.HasPrefix(mount, "/_part/") {
		return nil, fmt.Errorf("static mount `%s` is reserved", mount)
	}
	if boolish(appBag["admin"]) {
		prefix := AdminPrefix(appBag)
		if pathUnderPrefix(mount, prefix) {
			return nil, fmt.Errorf("static mount `%s` is reserved", mount)
		}
	}
	out := clone(appBag)
	out["static_dir"] = dir
	out["static_mount"] = mount
	return out, nil
}

// Auth wires users + default admin gate (Rust web_app_auth).
// opts may include session_ttl, admin_prefix/后台前缀, login_redirect/登录回跳,
// logout_redirect/登出回跳, login_path/登录路径.
func Auth(appBag map[string]any, users any, opts map[string]any) (map[string]any, error) {
	if opts == nil {
		opts = map[string]any{}
	}
	out := clone(appBag)
	sessionTTL := uint64(3600)
	if v, ok := opts["session_ttl"]; ok {
		if n, ok := asUint64(v); ok && n > 0 {
			sessionTTL = n
		}
	}
	if p := firstStr(opts, "admin_prefix", "后台前缀"); p != "" {
		np, err := NormalizeAdminPrefix(p)
		if err != nil {
			return nil, err
		}
		out["admin_prefix"] = np
	}
	if v := firstStr(opts, "login_redirect", "登录回跳"); v != "" {
		out["login_redirect"] = v
	}
	if v := firstStr(opts, "logout_redirect", "登出回跳"); v != "" {
		out["logout_redirect"] = v
	}
	if v := firstStr(opts, "login_path", "登录路径"); v != "" {
		out["login_path"] = v
	}
	prefix := AdminPrefix(out)
	loginPath := strOpt(out, "login_path", "")
	if loginPath == "" {
		loginPath = prefix + "/login"
	}
	out["login_path"] = loginPath
	out["auth"] = map[string]any{
		"users":       users,
		"session_ttl": float64(sessionTTL),
	}
	gates := gatesOf(out)
	if !hasAdminGate(gates, prefix) {
		gates = append(gates, map[string]any{
			"path":    prefix,
			"roles":   []any{"admin"},
			"match":   "prefix",
			"on_deny": "redirect",
			"exclude": []any{loginPath},
		})
		out["gates"] = gates
	} else {
		out["gates"] = gates
	}
	return out, nil
}

// Gate appends an RBAC gate (Rust web_app_gate).
// opts: roles/角色, match/匹配, on_deny/拒绝, exclude/排除.
func Gate(appBag map[string]any, path string, opts map[string]any) (map[string]any, error) {
	if opts == nil {
		opts = map[string]any{}
	}
	path = strings.TrimSpace(path)
	if path == "" {
		return nil, fmt.Errorf("missing `path`")
	}
	rolesRaw := firstStr(opts, "roles", "角色")
	if rolesRaw == "" {
		rolesRaw = "admin"
	}
	roles := session.ParseRolesCSV(rolesRaw)
	roleAny := make([]any, len(roles))
	for i, r := range roles {
		roleAny[i] = r
	}
	matchMode := firstStr(opts, "match", "匹配")
	if matchMode == "" {
		matchMode = "prefix"
	}
	onDeny := firstStr(opts, "on_deny", "拒绝")
	if onDeny == "" {
		onDeny = "forbid"
	}
	entry := map[string]any{
		"path":    path,
		"roles":   roleAny,
		"match":   matchMode,
		"on_deny": onDeny,
	}
	if v, ok := first(opts, "exclude", "排除"); ok && v != nil {
		entry["exclude"] = v
	}
	out := clone(appBag)
	gates := gatesOf(out)
	gates = append(gates, entry)
	out["gates"] = gates
	return out, nil
}

// NormalizeAdminPrefix cleans and validates admin_prefix.
func NormalizeAdminPrefix(raw string) (string, error) {
	path, err := cleanMountPath(raw)
	if err != nil {
		return "", err
	}
	if path == "/" {
		return "", fmt.Errorf("admin_prefix cannot be `/`")
	}
	if path == "/_form" || strings.HasPrefix(path, "/_form/") ||
		path == "/_part" || strings.HasPrefix(path, "/_part/") {
		return "", fmt.Errorf("admin_prefix `%s` collides with framework paths", path)
	}
	return path, nil
}

// AdminPrefix returns the active admin mount from an app bag.
func AdminPrefix(appBag map[string]any) string {
	s := strOpt(appBag, "admin_prefix", "/admin")
	if p, err := NormalizeAdminPrefix(s); err == nil {
		return p
	}
	return "/admin"
}

// NormalizeRoutePath applies Rust reserved-path rules for the app bag.
func NormalizeRoutePath(raw string, appBag map[string]any) (string, error) {
	path, err := cleanMountPath(raw)
	if err != nil {
		return "", err
	}
	if path == "/" {
		return "", fmt.Errorf("route path `/` is reserved for the home page")
	}
	staticMount := NormalizeStaticMount(strOpt(appBag, "static_mount", "/static"))
	if isFrameworkReserved(path, staticMount) {
		return "", fmt.Errorf("route path `%s` is reserved", path)
	}
	if boolish(appBag["admin"]) && pathUnderPrefix(path, AdminPrefix(appBag)) {
		return "", fmt.Errorf("route path `%s` is reserved", path)
	}
	return path, nil
}

func cleanMountPath(raw string) (string, error) {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "", fmt.Errorf("path is empty")
	}
	if !strings.HasPrefix(s, "/") {
		s = "/" + s
	}
	for len(s) > 1 && strings.HasSuffix(s, "/") {
		s = strings.TrimSuffix(s, "/")
	}
	return s, nil
}

func pathUnderPrefix(path, prefix string) bool {
	return path == prefix || strings.HasPrefix(path, prefix+"/")
}

func isFrameworkReserved(path, staticMount string) bool {
	return path == "/_form" || strings.HasPrefix(path, "/_form/") ||
		path == "/_part" || strings.HasPrefix(path, "/_part/") ||
		pathUnderPrefix(path, staticMount)
}

func gatesOf(appBag map[string]any) []any {
	if g, ok := appBag["gates"].([]any); ok {
		out := make([]any, len(g))
		copy(out, g)
		return out
	}
	return []any{}
}

func hasAdminGate(gates []any, prefix string) bool {
	want := strings.TrimSuffix(prefix, "/")
	for _, g := range gates {
		m, ok := g.(map[string]any)
		if !ok {
			continue
		}
		p, _ := m["path"].(string)
		cleaned := strings.TrimSuffix(strings.TrimSuffix(p, "*"), "/")
		if cleaned == want {
			return true
		}
	}
	return false
}

func first(m map[string]any, keys ...string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok && v != nil {
			return v, true
		}
	}
	return nil, false
}

func firstStr(m map[string]any, keys ...string) string {
	v, ok := first(m, keys...)
	if !ok {
		return ""
	}
	s, _ := v.(string)
	return strings.TrimSpace(s)
}

func asUint64(v any) (uint64, bool) {
	switch t := v.(type) {
	case uint64:
		return t, true
	case int:
		if t < 0 {
			return 0, false
		}
		return uint64(t), true
	case int64:
		if t < 0 {
			return 0, false
		}
		return uint64(t), true
	case float64:
		if t < 0 {
			return 0, false
		}
		return uint64(t), true
	default:
		return 0, false
	}
}

func clone(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func strOpt(args map[string]any, key, def string) string {
	v, ok := args[key]
	if !ok || v == nil {
		return def
	}
	if s, ok := v.(string); ok {
		return s
	}
	return def
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

// StorageURL extracts a storage url string or handle map.
func StorageURL(v any) (string, error) {
	if v == nil {
		return "", fmt.Errorf("missing storage")
	}
	if s, ok := v.(string); ok && strings.TrimSpace(s) != "" {
		return strings.TrimSpace(s), nil
	}
	if m, ok := v.(map[string]any); ok {
		if s, ok := m["url"].(string); ok && strings.TrimSpace(s) != "" {
			return strings.TrimSpace(s), nil
		}
	}
	return "", fmt.Errorf("storage must be a url string or storage handle")
}

// Upload registers an upload route on the app bag (offline stub; live HTTP in listen).
func Upload(appBag map[string]any, path, field, storageURL, prefix string, maxBytes uint64, types any) (map[string]any, error) {
	out := clone(appBag)
	if path == "" {
		path = "/_upload"
	}
	path, err := NormalizeRoutePath(path, out)
	if err != nil {
		return nil, err
	}
	if field == "" {
		field = "file"
	}
	if prefix == "" {
		prefix = "uploads/"
	}
	routes := map[string]any{}
	if r, ok := out["upload_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	entry := map[string]any{
		"field":       field,
		"storage_url": storageURL,
		"prefix":      prefix,
		"max_bytes":   float64(maxBytes),
	}
	if types != nil {
		entry["types"] = types
	}
	routes[path] = entry
	out["upload_routes"] = routes
	return out, nil
}

// Download registers a media download route with {key} capture.
func Download(appBag map[string]any, path, storageURL, disposition string) (map[string]any, error) {
	path = strings.TrimSpace(path)
	if path == "" {
		path = "/_media/{*key}"
	}
	if !strings.HasPrefix(path, "/") {
		path = "/" + path
	}
	for len(path) > 1 && strings.HasSuffix(path, "/") {
		path = strings.TrimSuffix(path, "/")
	}
	if !strings.Contains(path, "{") {
		return nil, fmt.Errorf("download path must include a `{key}` or `{*key}` capture")
	}
	if disposition == "" {
		disposition = "attachment"
	}
	if strings.TrimSpace(storageURL) == "" {
		return nil, fmt.Errorf("missing storage")
	}
	out := clone(appBag)
	routes := map[string]any{}
	if r, ok := out["download_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[path] = map[string]any{
		"storage_url": storageURL,
		"disposition": disposition,
	}
	out["download_routes"] = routes
	return out, nil
}

// Gallery registers a gallery page route (offline bag; live HTML in listen).
func Gallery(appBag map[string]any, path, storageURL, prefix, title, downloadBase string) (map[string]any, error) {
	out := clone(appBag)
	if path == "" {
		path = "/gallery"
	}
	var err error
	path, err = NormalizeRoutePath(path, out)
	if err != nil {
		return nil, err
	}
	if prefix == "" {
		prefix = "uploads/"
	}
	if title == "" {
		title = "Gallery"
	}
	if downloadBase == "" {
		downloadBase = "/_media"
	}
	storageURL, err = StorageURL(storageURL)
	if err != nil {
		return nil, err
	}
	routes := map[string]any{}
	if r, ok := out["gallery_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[path] = map[string]any{
		"storage":       storageURL,
		"prefix":        prefix,
		"title":         title,
		"download_base": downloadBase,
	}
	out["gallery_routes"] = routes
	return out, nil
}

// Redirect registers a redirect from → to on app.redirects.
func Redirect(appBag map[string]any, from, to string, permanent bool) (map[string]any, error) {
	out := clone(appBag)
	from, err := NormalizeRoutePath(from, out)
	if err != nil {
		return nil, err
	}
	redirects := map[string]any{}
	if r, ok := out["redirects"].(map[string]any); ok {
		redirects = clone(r)
	}
	redirects[from] = map[string]any{
		"to":        to,
		"permanent": permanent,
	}
	out["redirects"] = redirects
	return out, nil
}

// ErrorPage stores a custom page for HTTP 404 or 500.
func ErrorPage(appBag map[string]any, status uint16, page any) (map[string]any, error) {
	if page == nil {
		return nil, fmt.Errorf("missing `page`")
	}
	out := clone(appBag)
	key := "page_404"
	if status == 500 {
		key = "page_500"
	}
	out[key] = page
	return out, nil
}

// Sitemap registers a sitemap route bag on app.sitemap_routes.
func Sitemap(appBag map[string]any, path, base string, table, loc string, limit int64, items any) (map[string]any, error) {
	out := clone(appBag)
	if path == "" {
		path = "/sitemap.xml"
	}
	path, err := NormalizeRoutePath(path, out)
	if err != nil {
		return nil, err
	}
	if loc == "" {
		loc = "path"
	}
	if items == nil {
		items = []any{}
	}
	routes := map[string]any{}
	if r, ok := out["sitemap_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	entry := map[string]any{
		"base":  base,
		"loc":   loc,
		"limit": float64(limit),
		"items": items,
	}
	if table != "" {
		entry["table"] = table
	}
	routes[path] = entry
	out["sitemap_routes"] = routes
	return out, nil
}

// Robots sets app.robots_body from body or a default Allow-all robots.txt.
func Robots(appBag map[string]any, body, sitemapURL string) (map[string]any, error) {
	out := clone(appBag)
	text := strings.TrimSpace(body)
	if text == "" {
		text = defaultRobotsBody(sitemapURL)
	}
	out["robots_body"] = text
	return out, nil
}

func defaultRobotsBody(sitemapURL string) string {
	out := "User-agent: *\nAllow: /\n"
	if strings.TrimSpace(sitemapURL) != "" {
		out += "Sitemap: " + strings.TrimSpace(sitemapURL) + "\n"
	}
	return out
}

// RouteRSS registers an RSS feed route on app.rss_routes.
func RouteRSS(appBag map[string]any, path, table, order, title, link, description string, limit int64) (map[string]any, error) {
	out := clone(appBag)
	path, err := NormalizeRoutePath(path, out)
	if err != nil {
		return nil, err
	}
	if order == "" {
		order = "-created_at"
	}
	if title == "" {
		title = "Feed"
	}
	if link == "" {
		link = "/"
	}
	routes := map[string]any{}
	if r, ok := out["rss_routes"].(map[string]any); ok {
		routes = clone(r)
	}
	routes[path] = map[string]any{
		"table":       table,
		"limit":       float64(limit),
		"order":       order,
		"title":       title,
		"link":        link,
		"description": description,
	}
	out["rss_routes"] = routes
	return out, nil
}

// Icons normalizes icons table → app.icons + app.site_head.
func Icons(appBag map[string]any, table any) (map[string]any, error) {
	out := clone(appBag)
	icons, siteHead, _ := assets.NormalizeIcons(table)
	out["icons"] = icons
	out["site_head"] = assets.HeadLinksToJSON(siteHead)
	return out, nil
}
