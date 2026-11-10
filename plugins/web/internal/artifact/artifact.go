// Package artifact implements Document/Endpoint View render, serve scan, and inspect (ADR 0007).
package artifact

import (
	"encoding/json"
	"fmt"
	"html"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/render"
)

// MetaType returns canonical kind: web | endpoint | "" from flat frontmatter map.
func MetaType(meta map[string]string) string {
	if v := strings.TrimSpace(meta["type"]); v != "" {
		return normalizeType(v)
	}
	if v := strings.TrimSpace(meta["类型"]); v != "" {
		return normalizeType(v)
	}
	return ""
}

func normalizeType(v string) string {
	switch v {
	case "web", "网页":
		return "web"
	case "endpoint", "端点":
		return "endpoint"
	default:
		return v
	}
}

// ParseFrontmatter reads flat key: value pairs from a .mq.md file head.
func ParseFrontmatter(src string) (meta map[string]string, body string) {
	meta = map[string]string{}
	s := strings.TrimPrefix(src, "\ufeff")
	if !strings.HasPrefix(s, "---\n") && !strings.HasPrefix(s, "---\r\n") {
		return meta, src
	}
	rest := s[4:]
	if strings.HasPrefix(rest, "\r\n") {
		rest = rest[2:]
	} else if strings.HasPrefix(rest, "\n") {
		rest = rest[1:]
	}
	end := strings.Index(rest, "\n---\n")
	if end < 0 {
		end = strings.Index(rest, "\r\n---\r\n")
	}
	if end < 0 {
		end = strings.Index(rest, "\n---\r\n")
	}
	if end < 0 {
		return meta, src
	}
	fm := rest[:end]
	body = rest[end:]
	if i := strings.Index(body, "---"); i >= 0 {
		body = body[i+3:]
		body = strings.TrimPrefix(body, "\r")
		body = strings.TrimPrefix(body, "\n")
		// Drop blank line(s) between closing --- and first content.
		for strings.HasPrefix(body, "\n") || strings.HasPrefix(body, "\r\n") {
			body = strings.TrimPrefix(body, "\r\n")
			body = strings.TrimPrefix(body, "\n")
		}
	}
	for _, line := range strings.Split(fm, "\n") {
		line = strings.TrimSpace(strings.TrimSuffix(line, "\r"))
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, "import ") || strings.HasPrefix(line, "导入 ") {
			continue
		}
		k, v, ok := strings.Cut(line, ":")
		if !ok {
			continue
		}
		k = strings.TrimSpace(k)
		v = strings.TrimSpace(v)
		if len(v) >= 2 {
			if (v[0] == '"' && v[len(v)-1] == '"') || (v[0] == '\'' && v[len(v)-1] == '\'') {
				v = v[1 : len(v)-1]
			}
		}
		if k != "" {
			meta[k] = v
		}
	}
	return meta, body
}

// RenderNodes renders the existing five-column View schema recursively. Container
// value cells hold more View rows; markdown leaves hold ordinary Markdown text.
func RenderNodes(nodes any) (string, error) {
	return RenderNodesWithResources(nodes, nil)
}

// ResourceRenderer resolves a typed Resource at the request boundary.
// Ordinary View construction remains pure; a nil renderer never starts IO.
type ResourceRenderer func(map[string]any) (string, error)

func RenderNodesWithResources(nodes any, resources ResourceRenderer) (string, error) {
	var b strings.Builder
	b.WriteString(`<style>` + LayoutCSS + `</style><div class="mq-view">`)
	if err := renderNodes(&b, nodes, 0, resources); err != nil {
		return "", err
	}
	b.WriteString(`</div>`)
	return b.String(), nil
}

// Fixed runtime geometry, shared by offline previews and served Documents.
// Authors choose row/column instead of embedding CSS or HTML in content cells.
const LayoutCSS = `.mq-view{max-width:72rem;margin:auto;padding:1.5rem;line-height:1.6;font-family:system-ui,sans-serif}.mq-view *{box-sizing:border-box}.mq-view .mq-row{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,16rem),1fr));gap:1.5rem}.mq-view .mq-column{display:flex;flex-direction:column;gap:1.5rem}.mq-view .mq-node{min-width:0;overflow-wrap:anywhere}.mq-view nav,.mq-view footer{padding-block:1rem}.mq-view footer{border-top:1px solid #d5dce5}.mq-view a{color:#2457a7}.mq-view img{max-width:100%;height:auto}.mq-view pre{overflow:auto}.mq-view input,.mq-view textarea,.mq-view select{max-width:100%;font:inherit}.mq-view textarea{width:100%}`

// Preserve typed value cells when normalizing column tables. General AsRows
// treats @ cells as text, which would stringify nested component tables.
func viewRows(nodes any) ([]any, error) {
	switch v := nodes.(type) {
	case nil:
		return nil, nil
	case []any:
		return v, nil
	case map[string]any:
		if _, ok := v["type"].(string); ok {
			return []any{v}, nil
		}
		if _, ok := v["类型"].(string); ok {
			return []any{v}, nil
		}
		n := -1
		for _, cell := range v {
			col, ok := cell.([]any)
			if !ok {
				return nil, fmt.Errorf("View: expected a row or column table")
			}
			if n < 0 {
				n = len(col)
			} else if n != len(col) {
				return nil, fmt.Errorf("View: column lengths differ")
			}
		}
		rows := []any{}
		for i := 0; i < n; i++ {
			row := map[string]any{}
			for key, cell := range v {
				row[key] = cell.([]any)[i]
			}
			rows = append(rows, row)
		}
		return rows, nil
	default:
		return nil, fmt.Errorf("View: expected rows, got %T", nodes)
	}
}

func containerTag(kind string) (string, string) {
	switch kind {
	case "row", "横排":
		return "div", " mq-row"
	case "column", "纵排":
		return "div", " mq-column"
	case "nav", "导航":
		return "nav", ""
	case "header", "页头":
		return "header", ""
	case "footer", "页脚":
		return "footer", ""
	case "section", "区域":
		return "section", ""
	case "aside", "侧栏":
		return "aside", ""
	case "form", "表单":
		return "form", ""
	case "list", "列表":
		return "ul", ""
	case "item", "列表项":
		return "li", ""
	case "select", "选择":
		return "select", ""
	}
	return "", ""
}

// FindSlot selects one declared View region without resolving any Resources.
// Container traversal is identical to rendering and never descends into a
// Resource's private map as if it were another View table.
func FindSlot(nodes any, slot string) (any, bool, error) {
	var selected []any
	var walk func(any, int) error
	walk = func(nodes any, depth int) error {
		if depth > 64 {
			return fmt.Errorf("View: nesting exceeds 64 levels")
		}
		rows, err := viewRows(nodes)
		if err != nil {
			return err
		}
		for _, value := range rows {
			row, ok := value.(map[string]any)
			if !ok {
				return fmt.Errorf("View slot: expected a row map")
			}
			if first(row, "slot", "槽") == slot {
				selected = append(selected, row)
			}
			kind := strings.ToLower(first(row, "type", "类型"))
			if tag, _ := containerTag(kind); tag != "" {
				children := row["value"]
				if children == nil {
					children = row["值"]
				}
				if children == nil {
					return fmt.Errorf("View %s: missing child rows", kind)
				}
				if err := walk(children, depth+1); err != nil {
					return err
				}
			}
		}
		return nil
	}
	if err := walk(nodes, 0); err != nil {
		return nil, false, err
	}
	if len(selected) > 1 {
		return nil, false, fmt.Errorf("View slot %q is ambiguous", slot)
	}
	if len(selected) == 0 {
		return nil, false, nil
	}
	return selected, true, nil
}

func renderNodes(b *strings.Builder, nodes any, depth int, resources ResourceRenderer) error {
	if depth > 64 {
		return fmt.Errorf("View: nesting exceeds 64 levels")
	}
	rows, err := viewRows(nodes)
	if err != nil {
		return err
	}
	for i, item := range rows {
		row, ok := item.(map[string]any)
		if !ok {
			return fmt.Errorf("View row %d: expected a map", i+1)
		}
		ty := strings.ToLower(first(row, "type", "类型"))
		slot := first(row, "slot", "槽")
		val := first(row, "value", "值")
		attrs := first(row, "attrs", "属性")
		attributes, err := renderAttributes(row)
		if err != nil {
			return fmt.Errorf("View row %d: %w", i+1, err)
		}
		style := first(row, "style", "样式")
		class := "mq-node"
		if style != "" {
			class += " " + html.EscapeString(style)
		}
		if slot != "" {
			class += " slot-" + html.EscapeString(slot)
		}
		tag, geometry := containerTag(ty)
		class += geometry
		if tag != "" {
			children := row["value"]
			if children == nil {
				children = row["值"]
			}
			if children == nil {
				return fmt.Errorf("View row %d (%s): missing child rows in value", i+1, ty)
			}
			b.WriteString(`<` + tag + ` class="` + class + `"` + attributes + `>`)
			if err := renderNodes(b, children, depth+1, resources); err != nil {
				return fmt.Errorf("View %s/%s: %w", ty, slot, err)
			}
			b.WriteString(`</` + tag + `>`)
			continue
		}
		switch ty {
		case "resource", "资源":
			value := row["value"]
			if value == nil {
				value = row["值"]
			}
			resource, ok := value.(map[string]any)
			if !ok || first(resource, "_type") == "" {
				return fmt.Errorf("View row %d: resource value must be a typed Resource", i+1)
			}
			if first(resource, "_type") == "document_head" {
				if _, err := ParseDocumentHead(resource); err != nil {
					return err
				}
				continue
			}
			if resources == nil {
				return fmt.Errorf("View row %d: resource requires an explicit request renderer", i+1)
			}
			fragment, err := resources(resource)
			if err != nil {
				return fmt.Errorf("View resource %s: %w", slot, err)
			}
			b.WriteString(`<section class="` + class + `"` + attributes + `>` + fragment + `</section>`)
		case "markdown", "正文":
			b.WriteString(`<div class="` + class + `"` + attributes + `>` + render.MarkdownToHTML(val) + `</div>`)
		case "title", "标题":
			b.WriteString(`<h1 class="` + class + `"` + attributes + `>` + inline(val) + `</h1>`)
		case "link", "链接":
			if !hasAttributeMap(row) {
				if attrs == "" {
					attrs = "#"
				}
				attributes = ` href="` + html.EscapeString(attrs) + `"`
			}
			b.WriteString(`<a class="` + class + `"` + attributes + `>` + inline(val) + `</a>`)
		case "input", "输入":
			b.WriteString(`<input class="` + class + `"` + attributes + ` value="` + html.EscapeString(val) + `">`)
		case "button", "按钮", "textarea", "多行输入", "label", "标签", "pre", "预格式", "option", "选项":
			tags := map[string]string{"button": "button", "按钮": "button", "textarea": "textarea", "多行输入": "textarea", "label": "label", "标签": "label", "pre": "pre", "预格式": "pre", "option": "option", "选项": "option"}
			tag := tags[ty]
			b.WriteString(`<` + tag + ` class="` + class + `"` + attributes + `>` + html.EscapeString(val) + `</` + tag + `>`)
		case "script", "脚本", "stylesheet", "样式表":
			url := strings.ToLower(strings.TrimSpace(val))
			if url == "" || strings.ContainsAny(url, "\r\n\t") || strings.HasPrefix(url, "javascript:") || strings.HasPrefix(url, "vbscript:") {
				return fmt.Errorf("View row %d: resource requires a nonempty non-script URL", i+1)
			}
			if attrs, ok := attributeValue(row).(map[string]any); ok {
				for _, reserved := range []string{"href", "src", "rel"} {
					if _, exists := attrs[reserved]; exists {
						return fmt.Errorf("View row %d: resource URL and relation belong to its kind/value, not attrs.%s", i+1, reserved)
					}
				}
			}
			if ty == "script" || ty == "脚本" {
				b.WriteString(`<script src="` + html.EscapeString(val) + `"` + attributes + `></script>`)
			} else {
				b.WriteString(`<link rel="stylesheet" href="` + html.EscapeString(val) + `"` + attributes + `>`)
			}
		case "html", "原文":
			b.WriteString(`<div class="` + class + `">` + val + `</div>`)
		default:
			value := row["value"]
			if value == nil {
				value = row["值"]
			}
			switch value.(type) {
			case []any, map[string]any:
				return fmt.Errorf("View row %d: unknown structural type %q", i+1, ty)
			}
			b.WriteString(`<p class="` + class + `"` + attributes + `>` + inline(val) + `</p>`)
		}
	}
	return nil
}

// Attribute records keep controls in the same five-column View schema.
// Legacy link rows may still carry a plain href string. Names and scalar values
// are checked before emission; handlers and inline style remain explicit escapes.
func attributeValue(row map[string]any) any {
	if value, ok := row["attrs"]; ok && value != nil {
		return value
	}
	return row["属性"]
}

func hasAttributeMap(row map[string]any) bool {
	_, ok := attributeValue(row).(map[string]any)
	return ok
}

func renderAttributes(row map[string]any) (string, error) {
	value := attributeValue(row)
	if value == nil || value == "" {
		return "", nil
	}
	attrs, ok := value.(map[string]any)
	if !ok {
		if _, legacy := value.(string); legacy {
			return "", nil
		}
		return "", fmt.Errorf("attrs must be a scalar attribute record or legacy link URL")
	}
	allowed := map[string]bool{
		"id": true, "title": true, "role": true, "name": true, "type": true,
		"href": true, "src": true, "action": true, "method": true, "enctype": true,
		"placeholder": true, "for": true, "rows": true, "cols": true, "size": true,
		"min": true, "max": true, "step": true, "minlength": true, "maxlength": true,
		"pattern": true, "autocomplete": true, "target": true, "rel": true,
		"tabindex": true, "download": true, "accept": true, "alt": true,
		"integrity": true, "crossorigin": true, "media": true, "referrerpolicy": true,
	}
	booleans := map[string]bool{"hidden": true, "disabled": true, "required": true, "readonly": true, "checked": true, "selected": true, "multiple": true, "autofocus": true, "async": true, "defer": true, "nomodule": true}
	keys := make([]string, 0, len(attrs))
	for name := range attrs {
		keys = append(keys, name)
	}
	sort.Strings(keys)
	var out strings.Builder
	for _, name := range keys {
		if !(allowed[name] || booleans[name] || validCustomAttribute(name)) {
			return "", fmt.Errorf("unsupported attribute %q; use attrs data/aria keys for client state", name)
		}
		v := attrs[name]
		if v == nil {
			continue
		}
		if booleans[name] {
			flag, ok := v.(bool)
			if !ok {
				return "", fmt.Errorf("attribute %s requires a boolean", name)
			}
			if flag {
				out.WriteString(" " + name)
			}
			continue
		}
		switch v.(type) {
		case string, bool, float64, int, int64, json.Number:
		default:
			return "", fmt.Errorf("attribute %s requires a scalar value", name)
		}
		text := fmt.Sprint(v)
		if name == "href" || name == "src" || name == "action" {
			url := strings.ToLower(strings.TrimSpace(text))
			if strings.HasPrefix(url, "javascript:") || strings.HasPrefix(url, "vbscript:") {
				return "", fmt.Errorf("attribute %s cannot execute a script URL", name)
			}
		}
		out.WriteString(" " + name + `="` + html.EscapeString(text) + `"`)
	}
	return out.String(), nil
}

func validCustomAttribute(name string) bool {
	if !(strings.HasPrefix(name, "data-") || strings.HasPrefix(name, "aria-")) {
		return false
	}
	if strings.HasSuffix(name, "-") {
		return false
	}
	for _, char := range name {
		if !(char >= 'a' && char <= 'z' || char >= '0' && char <= '9' || char == '-' || char == '_') {
			return false
		}
	}
	return true
}

// RenderFile explicitly opts a Document into startup-time component assembly.
// Documents without view: main retain their existing Markdown/data-source path.
// AssembleView runs the existing startup entry once and retains typed rows.
// Request resources are deliberately not resolved during assembly.
func AssembleView(f ArtifactFile, run func(string, map[string]any) (any, error)) (any, bool, error) {
	view := metaString(f.Meta, "view", "视图")
	if view == "" {
		return nil, false, nil
	}
	if view != "main" {
		return nil, false, fmt.Errorf("%s: view/视图 must be main", f.Rel)
	}
	if metaString(f.Meta, "data_source", "数据源") != "" {
		return nil, false, fmt.Errorf("%s: view main is startup assembly; use an ordinary Document for automatic data_source binding", f.Rel)
	}
	nodes, err := run(f.Path, nil)
	if err != nil {
		return nil, false, fmt.Errorf("%s: view main: %w", f.Rel, err)
	}
	if nodes == nil {
		return nil, false, fmt.Errorf("%s: view main must return View rows", f.Rel)
	}
	return nodes, true, nil
}

func RenderFile(f ArtifactFile, run func(string, map[string]any) (any, error)) (string, bool, error) {
	nodes, structured, err := AssembleView(f, run)
	if err != nil {
		return "", false, err
	}
	if !structured {
		return MarkdownHTML(f.Body), false, nil
	}
	html, err := RenderNodes(nodes)
	if err != nil {
		return "", false, fmt.Errorf("%s: %w", f.Rel, err)
	}
	return html, true, nil
}

func inline(s string) string {
	return html.EscapeString(s)
}

func first(row map[string]any, keys ...string) string {
	for _, k := range keys {
		if v, ok := row[k]; ok && v != nil {
			return fmt.Sprint(v)
		}
	}
	return ""
}

// RenderDocument builds a minimal HTML page from title + markdown body.
func RenderDocument(title, body string) string {
	return RenderHTMLPage(title, render.MarkdownToHTML(body))
}

// RenderHTMLPage wraps an HTML fragment in a minimal document shell.
func RenderHTMLPage(title, innerHTML string) string {
	t := html.EscapeString(title)
	if t == "" {
		t = "Marqdo"
	}
	return renderHTMLPageWithHead("<title>"+t+"</title>", innerHTML)
}

func renderHTMLPageWithHead(head, innerHTML string) string {
	return "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">" + head +
		"</head><body class=\"mq-document\"><main>" + innerHTML + "</main></body></html>"
}

// MarkdownHTML exposes render helper.
func MarkdownHTML(md string) string {
	return render.MarkdownToHTML(md)
}

// ScanRoot finds Artifact .mq.md files under root.
func ScanRoot(root string) ([]ArtifactFile, error) {
	var out []ArtifactFile
	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		name := d.Name()
		if d.IsDir() {
			if name == "data" || name == ".git" || name == "node_modules" || strings.HasPrefix(name, ".") {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(name, ".mq.md") {
			return nil
		}
		raw, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		meta, body := ParseFrontmatter(string(raw))
		kind := MetaType(meta)
		if kind != "web" && kind != "endpoint" {
			return nil
		}
		rel, _ := filepath.Rel(root, path)
		af := ArtifactFile{Path: path, Rel: rel, Kind: kind, Meta: meta, Body: body}
		if kind == "web" {
			af.Route = metaString(meta, "route", "路由")
			if af.Route == "" {
				af.Route = routeFromRel(rel)
			}
			af.Method = metaString(meta, "method", "方法")
			if af.Method == "" {
				af.Method = "GET"
			}
		} else {
			af.Route = metaString(meta, "path", "路径")
			af.Method = metaString(meta, "method", "方法")
			if af.Method == "" {
				af.Method = "GET"
			}
			if af.Route == "" {
				return fmt.Errorf("endpoint %s: missing path/路径", rel)
			}
		}
		out = append(out, af)
		return nil
	})
	return out, err
}

// ArtifactFile is one discovered Document or Endpoint.
type ArtifactFile struct {
	Path   string
	Rel    string
	Kind   string
	Route  string
	Method string
	Meta   map[string]string
	Body   string
}

func metaString(meta map[string]string, keys ...string) string {
	for _, k := range keys {
		if v := strings.TrimSpace(meta[k]); v != "" {
			return v
		}
	}
	return ""
}

func routeFromRel(rel string) string {
	rel = filepath.ToSlash(rel)
	if rel == "index.mq.md" {
		return "/"
	}
	rel = strings.TrimSuffix(rel, ".mq.md")
	rel = strings.TrimSuffix(rel, "/index")
	if !strings.HasPrefix(rel, "/") {
		rel = "/" + rel
	}
	return rel
}

// SystemRoutes lists runtime-owned routes for inspect.
func SystemRoutes() []map[string]any {
	return []map[string]any{
		{"method": "GET", "path": "/_part/*", "kind": "system"},
		{"method": "GET|POST", "path": "/_form/*", "kind": "system"},
		{"method": "POST", "path": "/_md", "kind": "system"},
		{"method": "POST", "path": "/_runtime/tasks", "kind": "system"},
		{"method": "GET", "path": "/_runtime/tasks/{id}", "kind": "system"},
		{"method": "POST", "path": "/_runtime/tasks/{id}/cancel", "kind": "system"},
		{"method": "GET", "path": "/_runtime/tasks/{id}/events", "kind": "system"},
	}
}

// InspectBag reads declarations only. A prepared app's actual handler inventory
// is generated by httpx.InspectApp through the same registrar as NewHandler.
func InspectBag(app map[string]any, declared []ArtifactFile) map[string]any {
	entries := map[string]map[string]any{}
	add := func(row map[string]any) {
		path := first(row, "path")
		method := first(row, "method")
		if method == "" {
			method = "GET"
		}
		if path == "" {
			return
		}
		key := method + " " + path
		if _, exists := entries[key]; !exists {
			entries[key] = row
		}
	}
	for _, file := range declared {
		add(map[string]any{"method": file.Method, "path": file.Route, "kind": file.Kind, "file": file.Rel})
	}
	if app != nil {
		switch rows := app["artifact_declared"].(type) {
		case []map[string]any:
			for _, row := range rows {
				add(row)
			}
		case []any:
			for _, value := range rows {
				if row, ok := value.(map[string]any); ok {
					add(row)
				}
			}
		}
		if app["page"] != nil {
			add(map[string]any{"method": "GET", "path": "/", "kind": "document"})
		}
		if routes, ok := app["routes"].(map[string]any); ok {
			for path := range routes {
				add(map[string]any{"method": "GET", "path": path, "kind": "document"})
			}
		}
		for _, group := range []string{"artifact_routes", "invoke_routes"} {
			if routes, ok := app[group].(map[string]any); ok {
				for key, value := range routes {
					spec, _ := value.(map[string]any)
					method, path := first(spec, "method"), first(spec, "path")
					if parts := strings.SplitN(key, " ", 2); len(parts) == 2 {
						if method == "" {
							method = parts[0]
						}
						if path == "" {
							path = parts[1]
						}
					} else if path == "" {
						path = key
					}
					if method == "" {
						method = "GET"
					}
					row := map[string]any{"method": method, "path": path, "kind": "endpoint"}
					for _, name := range []string{"file", "class", "fn"} {
						if value := first(spec, name); value != "" {
							row[name] = value
						}
					}
					add(row)
				}
			}
		}
	}
	keys := make([]string, 0, len(entries))
	for key := range entries {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]map[string]any, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, entries[key])
	}
	return map[string]any{"_type": "web_inspection", "scope": "metadata", "declared": rows, "system": SystemRoutes()}
}
