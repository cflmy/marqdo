// Package artifact implements Document/Endpoint View render, serve scan, and inspect (ADR 0007).
package artifact

import (
	"fmt"
	"html"
	"os"
	"path/filepath"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/render"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
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

// RenderNodes turns a unified View table into an HTML fragment.
// Columns: type|slot|value|attrs|style (ZH: 类型|槽|值|属性|样式).
func RenderNodes(nodes any) (string, error) {
	raw := table.AsRows(nodes)
	list, _ := raw.([]any)
	var b strings.Builder
	b.WriteString(`<div class="mq-view">`)
	for _, item := range list {
		row, ok := item.(map[string]any)
		if !ok {
			continue
		}
		ty := first(row, "type", "类型")
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
		switch strings.ToLower(ty) {
		case "title", "标题":
			b.WriteString(`<h1 class="` + class + `">` + inline(val) + `</h1>`)
		case "link", "链接":
			href := attrs
			if href == "" {
				href = "#"
			}
			b.WriteString(`<a class="` + class + `" href="` + html.EscapeString(href) + `">` + inline(val) + `</a>`)
		case "html", "原文":
			b.WriteString(`<div class="` + class + `">` + val + `</div>`)
		default: // text
			b.WriteString(`<p class="` + class + `">` + inline(val) + `</p>`)
		}
	}
	b.WriteString(`</div>`)
	return b.String(), nil
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
	return "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>" + t +
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
			for key := range inv {
				dec = append(dec, map[string]any{"path": key, "kind": "endpoint"})
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
		"declared": dec,
		"system":   SystemRoutes(),
	}
}
