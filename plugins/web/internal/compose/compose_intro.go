package compose

import (
	"fmt"
	"html"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// ComposeIntro binds an intro table (|属性|值|样式| / |front|back|style|) onto the
// page — same shape as ComposeMain. Literals go in 值/back; 样式 resolves like
// card_title into styles_css + element class names. Sets page.intro HTML.
//
// Unlike AsBind, the 值/back column is not NormalizeRef'd so inline `code`
// markers survive for richText.
func ComposeIntro(page any, introTable any, callLib CallLib) (any, error) {
	obj := asObject(page)
	arr := asIntroRows(introTable)
	css := strField(obj, "styles_css")
	seenStyle := map[string]bool{}
	binds := make([]map[string]any, 0, len(arr))

	for _, bm := range arr {
		front := table.NormalizeRef(strField(bm, "front"))
		backRaw := strings.TrimSpace(strField(bm, "back"))
		cssRaw := table.NormalizeRef(strField(bm, "css"))

		back := backRaw
		switch sp := table.ParseSitePath(backRaw); sp.Kind {
		case table.SitePathLibMember:
			// Intro copy is almost always prose; only resolve clear lib.member refs
			// that look like style/module paths (no spaces).
			if !strings.ContainsAny(backRaw, " \t\n") {
				v, err := callLib(sp.Lib + "." + sp.Member)
				if err != nil {
					back = sp.Lib + "." + sp.Member
				} else if s, ok := v.(string); ok {
					back = s
				} else {
					back = fmt.Sprint(v)
				}
			}
		}

		cssName := ""
		if cssRaw != "" {
			switch sp := table.ParseSitePath(cssRaw); sp.Kind {
			case table.SitePathLibMember:
				cssName = sp.Member
				if !seenStyle[cssName] {
					st, err := callLib(sp.Lib + "." + sp.Member)
					if err != nil {
						return nil, err
					}
					css += table.AsCSSNamed(sp.Member, st)
					seenStyle[cssName] = true
				}
			case table.SitePathPlain:
				cssName = sp.Plain
			case table.SitePathDbField:
				cssName = table.NormalizeRef(cssRaw)
			}
		}

		if front == "" && back == "" {
			continue
		}
		binds = append(binds, map[string]any{
			"front": front,
			"back":  back,
			"css":   cssName,
		})
	}

	obj["intro"] = RenderIntroHTML(binds)
	obj["intro_bind"] = binds
	if css != "" {
		obj["styles_css"] = css
	}
	return obj, nil
}

// MakeIntroHTML builds intro HTML from a bind table without resolving lib styles
// (css column used as class name as-is). Preserves backticks in 值/back.
func MakeIntroHTML(introTable any) string {
	arr := asIntroRows(introTable)
	binds := make([]map[string]any, 0, len(arr))
	for _, bm := range arr {
		front := table.NormalizeRef(strField(bm, "front"))
		back := strings.TrimSpace(strField(bm, "back"))
		cssName := table.NormalizeRef(strField(bm, "css"))
		if front == "" && back == "" {
			continue
		}
		binds = append(binds, map[string]any{
			"front": front,
			"back":  back,
			"css":   cssName,
		})
	}
	return RenderIntroHTML(binds)
}

func asIntroRows(v any) []map[string]any {
	switch t := v.(type) {
	case []any:
		out := make([]map[string]any, 0, len(t))
		for _, row := range t {
			m, ok := row.(map[string]any)
			if !ok {
				continue
			}
			front, back, css := "", "", ""
			if x, ok := pickIntro(m, []string{"属性", "前端变量", "front", "field"}); ok {
				front = cellIntro(x)
			}
			if x, ok := pickIntro(m, []string{"值", "后端数据库", "back", "db", "text", "文"}); ok {
				back = cellIntro(x)
			}
			if x, ok := pickIntro(m, []string{"样式", "绑定css样式", "css", "class", "style"}); ok {
				css = cellIntro(x)
			}
			if front != "" || back != "" {
				out = append(out, map[string]any{"front": front, "back": back, "css": css})
			}
		}
		return out
	case map[string]any:
		// Column-oriented map of lists (GFM multi-row table).
		fronts, backs, csses := []string{}, []string{}, []string{}
		if x, ok := pickIntro(t, []string{"属性", "前端变量", "front", "field"}); ok {
			fronts = asIntroStrList(x)
		}
		if x, ok := pickIntro(t, []string{"值", "后端数据库", "back", "db", "text", "文"}); ok {
			backs = asIntroStrList(x)
		}
		if x, ok := pickIntro(t, []string{"样式", "绑定css样式", "css", "class", "style"}); ok {
			csses = asIntroStrList(x)
		}
		n := len(fronts)
		if len(backs) > n {
			n = len(backs)
		}
		out := make([]map[string]any, 0, n)
		for i := 0; i < n; i++ {
			front, back, css := "", "", ""
			if i < len(fronts) {
				front = fronts[i]
			}
			if i < len(backs) {
				back = backs[i]
			}
			if i < len(csses) {
				css = csses[i]
			}
			if front != "" || back != "" {
				out = append(out, map[string]any{"front": front, "back": back, "css": css})
			}
		}
		return out
	default:
		return nil
	}
}

func asIntroStrList(v any) []string {
	switch t := v.(type) {
	case []any:
		out := make([]string, 0, len(t))
		for _, x := range t {
			out = append(out, cellIntro(x))
		}
		return out
	case []string:
		return t
	default:
		s := cellIntro(v)
		if s == "" {
			return nil
		}
		return []string{s}
	}
}

func pickIntro(m map[string]any, keys []string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v, true
		}
	}
	return nil, false
}

func cellIntro(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case bool:
		if t {
			return "true"
		}
		return "false"
	case float64:
		if t == float64(int64(t)) {
			return fmt.Sprintf("%d", int64(t))
		}
		return fmt.Sprintf("%v", t)
	default:
		return fmt.Sprint(t)
	}
}

// RenderIntroHTML turns normalized intro binds into .main-intro inner HTML.
func RenderIntroHTML(binds []map[string]any) string {
	if len(binds) == 0 {
		return ""
	}
	var out strings.Builder
	i := 0
	for i < len(binds) {
		front := strings.TrimSpace(strField(binds[i], "front"))
		kind := introKind(front)
		switch kind {
		case "claim":
			j := i
			for j < len(binds) && introKind(strField(binds[j], "front")) == "claim" {
				j++
			}
			out.WriteString(`<p class="claim">`)
			for _, row := range binds[i:j] {
				cls := classAttr(strField(row, "css"))
				out.WriteString(fmt.Sprintf("<span%s>%s</span>", cls, richText(strField(row, "back"))))
			}
			out.WriteString(`</p>`)
			i = j
		case "step":
			j := i
			for j < len(binds) && introKind(strField(binds[j], "front")) == "step" {
				j++
			}
			out.WriteString(`<ol class="steps">`)
			for _, row := range binds[i:j] {
				cls := classAttr(strField(row, "css"))
				out.WriteString(fmt.Sprintf("<li%s>%s</li>", cls, richText(strField(row, "back"))))
			}
			out.WriteString(`</ol>`)
			i = j
		case "mount":
			id := strField(binds[i], "back")
			if id == "" {
				id = front
			}
			id = strings.TrimPrefix(id, "#")
			out.WriteString(fmt.Sprintf(`<div id="%s"></div>`, html.EscapeString(id)))
			i++
		case "html":
			out.WriteString(strField(binds[i], "back"))
			i++
		default:
			tag, baseClass := introTagClass(kind, front)
			cssName := strField(binds[i], "css")
			cls := mergeClass(baseClass, cssName)
			out.WriteString(fmt.Sprintf("<%s%s>%s</%s>", tag, classAttr(cls), richText(strField(binds[i], "back")), tag))
			i++
		}
	}
	return out.String()
}

func introKind(front string) string {
	s := strings.ToLower(strings.TrimSpace(front))
	switch s {
	case "kicker", "eyebrow", "眉题":
		return "kicker"
	case "title", "h1", "标题":
		return "title"
	case "lede", "lead", "导语":
		return "lede"
	case "claim", "badge", "标签":
		return "claim"
	case "step", "步骤":
		return "step"
	case "h2", "小标题":
		return "h2"
	case "p", "段落":
		return "p"
	case "mount", "插槽", "slot":
		return "mount"
	case "html", "原文", "raw":
		return "html"
	default:
		return s
	}
}

func introTagClass(kind, front string) (tag, baseClass string) {
	switch kind {
	case "kicker":
		return "p", "kicker"
	case "title":
		return "h1", ""
	case "lede":
		return "p", "lede"
	case "h2":
		return "h2", ""
	case "p":
		return "p", ""
	default:
		if front != "" {
			return "p", front
		}
		return "p", ""
	}
}

func mergeClass(base, style string) string {
	base = strings.TrimSpace(base)
	style = strings.TrimSpace(style)
	switch {
	case base == "":
		return style
	case style == "" || style == base:
		return base
	default:
		return base + " " + style
	}
}

func classAttr(class string) string {
	class = strings.TrimSpace(class)
	if class == "" {
		return ""
	}
	return ` class="` + html.EscapeString(class) + `"`
}

// richText escapes text then applies a tiny Markdown subset: [label](url), `code`, **bold**.
func richText(s string) string {
	if s == "" {
		return ""
	}
	type span struct {
		kind string // text|a|code|strong
		a    string
		b    string
	}
	var spans []span
	rest := s
	for len(rest) > 0 {
		// find earliest special
		iLink := strings.Index(rest, "[")
		iCode := strings.Index(rest, "`")
		iBold := strings.Index(rest, "**")
		next := -1
		which := ""
		if iLink >= 0 && (next < 0 || iLink < next) {
			next, which = iLink, "a"
		}
		if iCode >= 0 && (next < 0 || iCode < next) {
			next, which = iCode, "code"
		}
		if iBold >= 0 && (next < 0 || iBold < next) {
			next, which = iBold, "strong"
		}
		if next < 0 {
			spans = append(spans, span{kind: "text", a: rest})
			break
		}
		if next > 0 {
			spans = append(spans, span{kind: "text", a: rest[:next]})
		}
		rest = rest[next:]
		switch which {
		case "a":
			// [label](url)
			endLabel := strings.Index(rest, "](")
			if endLabel < 0 {
				spans = append(spans, span{kind: "text", a: rest[:1]})
				rest = rest[1:]
				continue
			}
			endURL := strings.Index(rest[endLabel+2:], ")")
			if endURL < 0 {
				spans = append(spans, span{kind: "text", a: rest[:1]})
				rest = rest[1:]
				continue
			}
			label := rest[1:endLabel]
			url := rest[endLabel+2 : endLabel+2+endURL]
			spans = append(spans, span{kind: "a", a: label, b: url})
			rest = rest[endLabel+2+endURL+1:]
		case "code":
			end := strings.Index(rest[1:], "`")
			if end < 0 {
				spans = append(spans, span{kind: "text", a: rest[:1]})
				rest = rest[1:]
				continue
			}
			spans = append(spans, span{kind: "code", a: rest[1 : 1+end]})
			rest = rest[1+end+1:]
		case "strong":
			end := strings.Index(rest[2:], "**")
			if end < 0 {
				spans = append(spans, span{kind: "text", a: rest[:2]})
				rest = rest[2:]
				continue
			}
			spans = append(spans, span{kind: "strong", a: rest[2 : 2+end]})
			rest = rest[2+end+2:]
		}
	}
	var out strings.Builder
	for _, sp := range spans {
		switch sp.kind {
		case "a":
			out.WriteString(`<a href="`)
			out.WriteString(html.EscapeString(sp.b))
			out.WriteString(`">`)
			out.WriteString(html.EscapeString(sp.a))
			out.WriteString(`</a>`)
		case "code":
			out.WriteString(`<code>`)
			out.WriteString(html.EscapeString(sp.a))
			out.WriteString(`</code>`)
		case "strong":
			out.WriteString(`<strong>`)
			out.WriteString(html.EscapeString(sp.a))
			out.WriteString(`</strong>`)
		default:
			out.WriteString(html.EscapeString(sp.a))
		}
	}
	return out.String()
}
