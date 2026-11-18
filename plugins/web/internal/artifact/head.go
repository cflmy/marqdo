package artifact

import (
	"fmt"
	"net/url"
	"reflect"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/assets"
	"github.com/marqdo/marqdo/plugins/web/internal/render"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// DocumentHead is a closed, pure View Resource. It contributes only metadata
// and existing native head links; it cannot mutate page layout or server state.
type DocumentHead struct {
	Meta  map[string]any
	Links []assets.HeadLink
}

func headRows(value any) ([]map[string]any, error) {
	if value == nil {
		return nil, nil
	}
	switch data := value.(type) {
	case []any:
		rows := make([]map[string]any, 0, len(data))
		for _, value := range data {
			row, ok := value.(map[string]any)
			if !ok {
				return nil, fmt.Errorf("document_head: expected table records")
			}
			rows = append(rows, row)
		}
		return rows, nil
	case map[string]any:
		if len(data) == 0 {
			return nil, nil
		}
		arrays, scalars, count := 0, 0, -1
		for _, value := range data {
			if column, ok := value.([]any); ok {
				arrays++
				if count >= 0 && count != len(column) {
					return nil, fmt.Errorf("document_head: column lengths differ")
				}
				count = len(column)
			} else {
				scalars++
			}
		}
		if arrays > 0 && scalars > 0 {
			return nil, fmt.Errorf("document_head: mixed column and scalar cells")
		}
		if arrays > 0 && count == 0 {
			return nil, nil
		}
		rows, _ := table.AsRows(data).([]any)
		return headRows(rows)
	default:
		return nil, fmt.Errorf("document_head: expected a table or None")
	}
}

func headURL(raw string) error {
	if raw == "" || strings.ContainsAny(raw, "\r\n\t") || raw != strings.TrimSpace(raw) ||
		strings.Contains(raw, "\\") || strings.HasPrefix(raw, "//") {
		return fmt.Errorf("document_head: resource URL must be site-relative or http(s)")
	}
	parsed, err := url.Parse(raw)
	if err != nil || parsed == nil {
		return fmt.Errorf("document_head: invalid resource URL")
	}
	if parsed.Scheme == "" {
		if !strings.HasPrefix(raw, "/") {
			return fmt.Errorf("document_head: relative URL must start with /")
		}
	} else if (parsed.Scheme != "http" && parsed.Scheme != "https") || parsed.Host == "" {
		return fmt.Errorf("document_head: resource URL must be site-relative or http(s)")
	}
	return nil
}

// ParseDocumentHead validates before either preview or listening. No URL is
// fetched and no script is executed by this native projection.
func ParseDocumentHead(resource map[string]any) (DocumentHead, error) {
	out := DocumentHead{Meta: map[string]any{}}
	if first(resource, "_type") != "document_head" {
		return out, fmt.Errorf("document_head: incorrect resource type")
	}
	for key := range resource {
		if key != "_type" && key != "meta" && key != "links" {
			return out, fmt.Errorf("document_head: unsupported field %q", key)
		}
	}
	rows, err := headRows(resource["meta"])
	if err != nil {
		return out, err
	}
	for _, row := range rows {
		for key, value := range row {
			if key != "key" && key != "键" && key != "name" && key != "value" && key != "值" && key != "@" && key != "行" && key != "row" && key != "id" {
				return out, fmt.Errorf("document_head: unsupported metadata field %q", key)
			}
			switch value.(type) {
			case nil, string, bool, float64:
			default:
				return out, fmt.Errorf("document_head: metadata cells must be scalar values")
			}
		}
		key := first(row, "键", "key", "name")
		value := first(row, "值", "value")
		if key == "" {
			return out, fmt.Errorf("document_head: metadata requires key/value rows")
		}
		if strings.EqualFold(key, "charset") || strings.EqualFold(key, "viewport") || strings.EqualFold(key, "csrf-token") {
			return out, fmt.Errorf("document_head: %s is owned by the document runtime", key)
		}
		if key == "canonical" || key == "icon" || key == "favicon" {
			if err := headURL(value); err != nil {
				return out, err
			}
		}
		if existing, ok := out.Meta[key]; ok && existing != value {
			return out, fmt.Errorf("document_head: conflicting metadata %q", key)
		}
		out.Meta[key] = value
	}
	rows, err = headRows(resource["links"])
	if err != nil {
		return out, err
	}
	// Use the same link field aliases and native rendering as ordinary Documents.
	for _, row := range rows {
		for key, value := range row {
			switch value.(type) {
			case nil, string, bool, float64:
			default:
				return out, fmt.Errorf("document_head: link cells must be scalar values")
			}
			if value == nil {
				continue
			}
			switch key {
			case "关系", "rel", "Rel", "地址", "href", "src", "url", "Href", "Src", "URL",
				"类型", "type", "Type", "尺寸", "sizes", "Sizes", "媒体", "media", "Media",
				"作为", "as", "As", "跨域", "crossorigin", "crossOrigin",
				"推迟", "defer", "Defer", "异步", "async", "Async", "版本", "version", "Version", "v", "@", "行", "row", "id":
			default:
				return out, fmt.Errorf("document_head: unsupported link field %q", key)
			}
		}
		if err := headURL(first(row, "地址", "href", "src", "url", "Href", "Src", "URL")); err != nil {
			return out, err
		}
		links := assets.AsHeadLinks([]any{row})
		if len(links) != 1 {
			return out, fmt.Errorf("document_head: each link requires href")
		}
		if err := headURL(links[0].Href); err != nil {
			return out, err
		}
		out.Links = append(out.Links, links[0])
	}
	return out, nil
}

// CollectDocumentHead traverses only real View containers. Resource private
// records are never interpreted as nodes, and conflicting declarations fail.
func CollectDocumentHead(nodes any) (DocumentHead, error) {
	out := DocumentHead{Meta: map[string]any{}}
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
				return fmt.Errorf("View: expected row map")
			}
			kind := strings.ToLower(first(row, "type", "类型"))
			child := row["value"]
			if child == nil {
				child = row["值"]
			}
			if tag, _ := containerTag(kind); tag != "" {
				if child == nil {
					return fmt.Errorf("View %s: missing child rows", kind)
				}
				if err := walk(child, depth+1); err != nil {
					return err
				}
			} else if kind == "resource" || kind == "资源" {
				resource, ok := child.(map[string]any)
				if !ok || first(resource, "_type") != "document_head" {
					continue
				}
				head, err := ParseDocumentHead(resource)
				if err != nil {
					return err
				}
				for key, value := range head.Meta {
					if existing, ok := out.Meta[key]; ok && existing != value {
						return fmt.Errorf("document_head: conflicting metadata %q", key)
					}
					out.Meta[key] = value
				}
				for _, link := range head.Links {
					duplicate := false
					for _, existing := range out.Links {
						if reflect.DeepEqual(existing, link) {
							duplicate = true
							break
						}
					}
					if !duplicate {
						out.Links = append(out.Links, link)
					}
				}
			}
		}
		return nil
	}
	err := walk(nodes, 0)
	return out, err
}

// RenderViewDocument shares a single pure shell for offline and request-time
// nodes. Metadata never becomes visible content or a fragment response.
func RenderViewDocument(title string, nodes any, resources ResourceRenderer) (string, error) {
	head, err := CollectDocumentHead(nodes)
	if err != nil {
		return "", err
	}
	body, err := RenderNodesWithResources(nodes, resources)
	if err != nil {
		return "", err
	}
	return renderHTMLPageWithHead(render.DocumentHead(title, head.Meta, head.Links), body), nil
}
