// Package artifact implements Document/Endpoint View render, serve scan, and inspect (ADR 0007).
package artifact

import (
	"fmt"
	"html"
	"os"
	"path/filepath"
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
	var b strings.Builder
	b.WriteString(`<style>` + LayoutCSS + `</style><div class="mq-view">`)
	if err := renderNodes(&b, nodes, 0); err != nil {
		return "", err
	}
	b.WriteString(`</div>`)
	return b.String(), nil
}

// Fixed runtime geometry, shared by offline previews and served Documents.
// Authors choose row/column instead of embedding CSS or HTML in content cells.
const LayoutCSS = `.mq-view{max-width:72rem;margin:auto;padding:1.5rem;line-height:1.6;font-family:system-ui,sans-serif}.mq-view *{box-sizing:border-box}.mq-view .mq-row{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,16rem),1fr));gap:1.5rem}.mq-view .mq-column{display:flex;flex-direction:column;gap:1.5rem}.mq-view .mq-node{min-width:0}.mq-view nav,.mq-view footer{padding-block:1rem}.mq-view footer{border-top:1px solid #d5dce5}.mq-view a{color:#2457a7}.mq-view img{max-width:100%;height:auto}.mq-view pre{overflow:auto}`

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

func renderNodes(b *strings.Builder, nodes any, depth int) error {
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
		style := first(row, "style", "样式")
		class := "mq-node"
		if style != "" {
			class += " " + html.EscapeString(style)
		}
		if slot != "" {
			class += " slot-" + html.EscapeString(slot)
		}
		tag := ""
		switch ty {
		case "row", "横排":
			tag = "div"
			class += " mq-row"
		case "column", "纵排":
			tag = "div"
			class += " mq-column"
		case "nav", "导航":
			tag = "nav"
		case "header", "页头":
			tag = "header"
		case "footer", "页脚":
			tag = "footer"
		case "section", "区域":
			tag = "section"
		case "aside", "侧栏":
			tag = "aside"
		}
		if tag != "" {
			children := row["value"]
			if children == nil {
				children = row["值"]
			}
			if children == nil {
				return fmt.Errorf("View row %d (%s): missing child rows in value", i+1, ty)
			}
			b.WriteString(`<` + tag + ` class="` + class + `">`)
			if err := renderNodes(b, children, depth+1); err != nil {
				return fmt.Errorf("View %s/%s: %w", ty, slot, err)
			}
			b.WriteString(`</` + tag + `>`)
			continue
		}
		switch ty {
		case "markdown", "正文":
			b.WriteString(`<div class="` + class + `">` + render.MarkdownToHTML(val) + `</div>`)
		case "title", "标题":
			b.WriteString(`<h1 class="` + class + `">` + inline(val) + `</h1>`)
		case "link", "链接":
			if attrs == "" {
				attrs = "#"
			}
			b.WriteString(`<a class="` + class + `" href="` + html.EscapeString(attrs) + `">` + inline(val) + `</a>`)
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
			b.WriteString(`<p class="` + class + `">` + inline(val) + `</p>`)
		}
	}
	return nil
}

// RenderFile explicitly opts a Document into startup-time component assembly.
// Documents without view: main retain their existing Markdown/data-source path.
func RenderFile(f ArtifactFile, run func(string, map[string]any) (any, error)) (string, bool, error) {
	view := metaString(f.Meta, "view", "视图")
	if view == "" {
		return MarkdownHTML(f.Body), false, nil
	}
	if view != "main" {
		return "", false, fmt.Errorf("%s: view/视图 must be main", f.Rel)
	}
	if metaString(f.Meta, "data_source", "数据源") != "" {
		return "", false, fmt.Errorf("%s: view main is startup assembly; use an ordinary Document for automatic data_source binding", f.Rel)
	}
	nodes, err := run(f.Path, nil)
	if err != nil {
		return "", false, fmt.Errorf("%s: view main: %w", f.Rel, err)
	}
	if nodes == nil {
		return "", false, fmt.Errorf("%s: view main must return View rows", f.Rel)
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
	return "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>" + t +
		"</title></head><body class=\"mq-document\"><main>" + innerHTML + "</main></body></html>"
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
		{"method": "POST", "path": "/_runtime/tasks", "kind": "system"},
		{"method": "GET", "path": "/_runtime/tasks/{id}", "kind": "system"},
		{"method": "POST", "path": "/_runtime/tasks/{id}/cancel", "kind": "system"},
		{"method": "GET", "path": "/_runtime/tasks/{id}/events", "kind": "system"},
	}
}

// InspectBag builds {declared, system} from an app bag and/or scanned files.
func InspectBag(app map[string]any, declared []ArtifactFile) map[string]any {
	var dec []map[string]any
	for _, a := range declared {
		dec = append(dec, map[string]any{
			"method": a.Method,
			"path":   a.Route,
			"kind":   a.Kind,
			"file":   a.Rel,
		})
	}
	if app != nil {
		if routes, ok := app["routes"].(map[string]any); ok {
			for path := range routes {
				dec = append(dec, map[string]any{"method": "GET", "path": path, "kind": "declared"})
			}
		}
		if inv, ok := app["invoke_routes"].(map[string]any); ok {
			for key, v := range inv {
				row := map[string]any{"path": key, "kind": "endpoint"}
				if m, ok := v.(map[string]any); ok {
					if c, ok := m["class"].(string); ok && c != "" {
						row["class"] = c
					}
					if method, ok := m["method"].(string); ok && method != "" {
						row["method"] = method
					}
					if fn, ok := m["fn"].(string); ok && fn != "" {
						row["fn"] = fn
					}
				}
				dec = append(dec, row)
			}
		}
		if art, ok := app["artifact_routes"].(map[string]any); ok {
			for key, v := range art {
				m, _ := v.(map[string]any)
				dec = append(dec, map[string]any{
					"path":   key,
					"kind":   "endpoint",
					"method": fmt.Sprint(m["method"]),
					"file":   fmt.Sprint(m["file"]),
				})
			}
		}
	}
	return map[string]any{
		"_type":    "web_inspection",
		"declared": dec,
		"system":   SystemRoutes(),
	}
}
