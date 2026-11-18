// Package table ports GFM table helpers + site path parsing from the Rust web plugin.
package table

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

var (
	fieldKeys    = []string{"字段", "name", "列", "column"}
	typeKeys     = []string{"类型", "type"}
	nullKeys     = []string{"可空", "null", "nullable"}
	frontKeys    = []string{"属性", "前端变量", "front", "field"}
	backKeys     = []string{"值", "后端数据库", "back", "db"}
	cssKeys      = []string{"样式", "绑定css样式", "css", "class", "style"}
	srcKeys      = []string{"组件", "导入的页面", "src", "page"}
	styleKeys    = []string{"样式", "style", "class"}
	propKeys     = []string{"属性", "property", "prop", "名", "name"}
	valKeys      = []string{"值", "value"}
	selKeys      = []string{"选择器", "selector", "sel"}
	mediaKeys    = []string{"媒体", "media", "mq", "@media"}
	whenKeys     = []string{"当", "when", "if", "条件", "visible"}
	navLabelKeys = []string{"标签", "label", "title", "text", "名"}
	navHrefKeys  = []string{"链接", "href", "url", "path", "地址"}
	uniqueKeys   = []string{"唯一", "unique", "uniq"}
	indexKeys    = []string{"索引", "index", "idx"}
	fkKeys       = []string{"外键", "fk", "references", "ref", "引用"}
	metaKeyKeys  = []string{"键", "key", "name"}
	metaValKeys  = []string{"值", "value"}
)

// NormalizeRef strips surrounding backticks from ref cells (paired `…`).
func NormalizeRef(s string) string {
	s = strings.TrimSpace(s)
	var out strings.Builder
	out.Grow(len(s))
	rest := s
	for {
		start := strings.IndexByte(rest, '`')
		if start < 0 {
			out.WriteString(rest)
			return out.String()
		}
		out.WriteString(rest[:start])
		rest = rest[start+1:]
		end := strings.IndexByte(rest, '`')
		if end < 0 {
			out.WriteByte('`')
			out.WriteString(rest)
			return out.String()
		}
		out.WriteString(rest[:end])
		rest = rest[end+1:]
	}
}

func pick(m map[string]any, keys []string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v, true
		}
	}
	return nil, false
}

func cellStr(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case bool:
		return strconv.FormatBool(t)
	case float64:
		if t == float64(int64(t)) {
			return strconv.FormatInt(int64(t), 10)
		}
		return strconv.FormatFloat(t, 'f', -1, 64)
	case int:
		return strconv.Itoa(t)
	case int64:
		return strconv.FormatInt(t, 10)
	default:
		return fmt.Sprint(t)
	}
}

func asStrList(v any) []string {
	switch t := v.(type) {
	case nil:
		return nil
	case []any:
		out := make([]string, len(t))
		for i, x := range t {
			out[i] = cellStr(x)
		}
		return out
	default:
		return []string{cellStr(v)}
	}
}

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		s := strings.TrimSpace(t)
		switch s {
		case "1", "true", "True", "yes", "是", "可":
			return true
		}
		return false
	case float64:
		return int64(t) != 0
	case int:
		return t != 0
	case int64:
		return t != 0
	default:
		return false
	}
}

func asMap(v any) (map[string]any, bool) {
	m, ok := v.(map[string]any)
	return m, ok
}

func asArray(v any) ([]any, bool) {
	a, ok := v.([]any)
	return a, ok
}

// AsFields converts a schema table → [{name,type,nullable,unique,index,fk}, …].
func AsFields(table any) any {
	switch t := table.(type) {
	case []any:
		out := make([]any, 0, len(t))
		for _, row := range t {
			m, ok := asMap(row)
			if !ok {
				continue
			}
			name := ""
			if v, ok := pick(m, fieldKeys); ok {
				name = cellStr(v)
			}
			if name == "" {
				continue
			}
			ty := "text"
			if v, ok := pick(m, typeKeys); ok {
				ty = cellStr(v)
			}
			nullable := true
			if v, ok := pick(m, nullKeys); ok {
				nullable = boolish(v)
			}
			unique := false
			if v, ok := pick(m, uniqueKeys); ok {
				unique = boolish(v)
			}
			index := false
			if v, ok := pick(m, indexKeys); ok {
				index = boolish(v)
			}
			fk := ""
			if v, ok := pick(m, fkKeys); ok {
				fk = cellStr(v)
			}
			out = append(out, map[string]any{
				"name":     name,
				"type":     ty,
				"nullable": nullable,
				"unique":   unique,
				"index":    index,
				"fk":       fk,
			})
		}
		return out
	case map[string]any:
		names := []string{}
		if v, ok := pick(t, fieldKeys); ok {
			names = asStrList(v)
		}
		types := []string{}
		if v, ok := pick(t, typeKeys); ok {
			types = asStrList(v)
		}
		nulls := []string{}
		if v, ok := pick(t, nullKeys); ok {
			nulls = asStrList(v)
		}
		uniques := []string{}
		if v, ok := pick(t, uniqueKeys); ok {
			uniques = asStrList(v)
		}
		indexes := []string{}
		if v, ok := pick(t, indexKeys); ok {
			indexes = asStrList(v)
		}
		fks := []string{}
		if v, ok := pick(t, fkKeys); ok {
			fks = asStrList(v)
		}
		out := make([]any, 0, len(names))
		for i, name := range names {
			if name == "" {
				continue
			}
			ty := "text"
			if i < len(types) {
				ty = types[i]
			}
			nullable := true
			if i < len(nulls) {
				nullable = boolish(nulls[i])
			}
			unique := false
			if i < len(uniques) {
				unique = boolish(uniques[i])
			}
			index := false
			if i < len(indexes) {
				index = boolish(indexes[i])
			}
			fk := ""
			if i < len(fks) {
				fk = fks[i]
			}
			out = append(out, map[string]any{
				"name":     name,
				"type":     ty,
				"nullable": nullable,
				"unique":   unique,
				"index":    index,
				"fk":       fk,
			})
		}
		return out
	default:
		return []any{}
	}
}

// AsRows converts a seed / row table → list of maps.
func AsRows(table any) any {
	switch t := table.(type) {
	case []any:
		out := make([]any, len(t))
		copy(out, t)
		return out
	case map[string]any:
		keys := make([]string, 0, len(t))
		for k := range t {
			keys = append(keys, k)
		}
		var rowKey string
		found := false
		for _, k := range []string{"@", "行", "row"} {
			if _, ok := t[k]; ok {
				rowKey = k
				found = true
				break
			}
		}
		if found {
			n := len(asStrList(t[rowKey]))
			out := make([]any, 0, n)
			for i := 0; i < n; i++ {
				row := map[string]any{}
				for _, k := range keys {
					if k == rowKey {
						continue
					}
					list := asStrList(t[k])
					val := ""
					if i < len(list) {
						val = list[i]
					}
					row[k] = val
				}
				ids := asStrList(t[rowKey])
				if i < len(ids) && ids[i] != "" {
					row["id"] = ids[i]
				}
				out = append(out, row)
			}
			return out
		}
		// Columnar GFM table: each value is a same-length array → zip into rows.
		colLens := make([]int, 0, len(t))
		allArrays := len(t) > 0
		for _, v := range t {
			a, ok := v.([]any)
			if !ok {
				allArrays = false
				break
			}
			colLens = append(colLens, len(a))
		}
		if allArrays {
			n := 0
			for _, l := range colLens {
				if l > n {
					n = l
				}
			}
			okLens := n > 0
			for _, l := range colLens {
				if l != n && l != 0 {
					okLens = false
					break
				}
			}
			if okLens {
				out := make([]any, 0, n)
				for i := 0; i < n; i++ {
					row := map[string]any{}
					for k, v := range t {
						a := v.([]any)
						if i < len(a) {
							row[k] = a[i]
						} else {
							row[k] = nil
						}
					}
					out = append(out, row)
				}
				return out
			}
		}
		// single map
		cp := map[string]any{}
		for k, v := range t {
			cp[k] = v
		}
		return []any{cp}
	default:
		return []any{}
	}
}

// AsBind converts a bind table → [{front,back,css,media,when}, …].
func AsBind(table any) any {
	switch t := table.(type) {
	case []any:
		out := make([]any, 0, len(t))
		for _, row := range t {
			m, ok := asMap(row)
			if !ok {
				continue
			}
			front, back, css, media, when := "", "", "", "", ""
			if v, ok := pick(m, frontKeys); ok {
				front = NormalizeRef(cellStr(v))
			}
			if v, ok := pick(m, backKeys); ok {
				back = NormalizeRef(cellStr(v))
			}
			if v, ok := pick(m, cssKeys); ok {
				css = NormalizeRef(cellStr(v))
			}
			if v, ok := pick(m, mediaKeys); ok {
				media = NormalizeRef(cellStr(v))
			}
			if v, ok := pick(m, whenKeys); ok {
				when = NormalizeRef(cellStr(v))
			}
			if front != "" || back != "" {
				out = append(out, map[string]any{
					"front": front,
					"back":  back,
					"css":   css,
					"media": media,
					"when":  when,
				})
			}
		}
		return out
	case map[string]any:
		fronts, backs, csses, medias, whens := []string{}, []string{}, []string{}, []string{}, []string{}
		if v, ok := pick(t, frontKeys); ok {
			fronts = asStrList(v)
		}
		if v, ok := pick(t, backKeys); ok {
			backs = asStrList(v)
		}
		if v, ok := pick(t, cssKeys); ok {
			csses = asStrList(v)
		}
		if v, ok := pick(t, mediaKeys); ok {
			medias = asStrList(v)
		}
		if v, ok := pick(t, whenKeys); ok {
			whens = asStrList(v)
		}
		n := len(fronts)
		if len(backs) > n {
			n = len(backs)
		}
		out := make([]any, 0, n)
		for i := 0; i < n; i++ {
			front, back, css, media, when := "", "", "", "", ""
			if i < len(fronts) {
				front = NormalizeRef(fronts[i])
			}
			if i < len(backs) {
				back = NormalizeRef(backs[i])
			}
			if i < len(csses) {
				css = NormalizeRef(csses[i])
			}
			if i < len(medias) {
				media = NormalizeRef(medias[i])
			}
			if i < len(whens) {
				when = NormalizeRef(whens[i])
			}
			if front != "" || back != "" {
				out = append(out, map[string]any{
					"front": front,
					"back":  back,
					"css":   css,
					"media": media,
					"when":  when,
				})
			}
		}
		return out
	default:
		return []any{}
	}
}

// AsNavRows converts a nav / link table → row maps with label/href.
func AsNavRows(table any) []map[string]any {
	binds := AsBind(table)
	if arr, ok := binds.([]any); ok && len(arr) > 0 {
		out := make([]map[string]any, 0, len(arr))
		for _, v := range arr {
			if m, ok := asMap(v); ok {
				out = append(out, m)
			}
		}
		return out
	}
	rows := AsRows(table)
	arr, _ := rows.([]any)
	out := make([]map[string]any, 0, len(arr))
	for _, v := range arr {
		if m, ok := asMap(v); ok {
			out = append(out, m)
		}
	}
	return out
}

// NavLabelHref picks label/href from a nav row.
func NavLabelHref(m map[string]any) (string, string) {
	label := ""
	if v, ok := pick(m, navLabelKeys); ok {
		label = NormalizeRef(cellStr(v))
	} else if v, ok := m["front"]; ok {
		label = NormalizeRef(cellStr(v))
	}
	href := "#"
	if v, ok := pick(m, navHrefKeys); ok {
		s := NormalizeRef(cellStr(v))
		if s != "" {
			href = s
		}
	} else if v, ok := m["back"]; ok {
		s := NormalizeRef(cellStr(v))
		if s != "" {
			href = s
		}
	}
	return label, href
}

// NavMediaWhenClass picks media/when/class from a nav row.
func NavMediaWhenClass(m map[string]any) (string, string, string) {
	media := ""
	if v, ok := pick(m, mediaKeys); ok {
		media = NormalizeRef(cellStr(v))
	} else if v, ok := m["media"]; ok {
		media = NormalizeRef(cellStr(v))
	}
	when := ""
	if v, ok := pick(m, whenKeys); ok {
		when = NormalizeRef(cellStr(v))
	} else if v, ok := m["when"]; ok {
		when = NormalizeRef(cellStr(v))
	}
	class := ""
	if v, ok := pick(m, cssKeys); ok {
		class = NormalizeRef(cellStr(v))
	} else if v, ok := m["css"]; ok {
		class = NormalizeRef(cellStr(v))
	}
	return media, when, class
}

// AsCompose converts a page table → [{src,style,slot}, …].
func AsCompose(table any) any {
	switch t := table.(type) {
	case map[string]any:
		srcs, styles := []string{}, []string{}
		if v, ok := pick(t, srcKeys); ok {
			srcs = asStrList(v)
		}
		if v, ok := pick(t, styleKeys); ok {
			styles = asStrList(v)
		}
		out := make([]any, 0, len(srcs))
		for i, srcRaw := range srcs {
			src := NormalizeRef(srcRaw)
			if src == "" {
				continue
			}
			style := ""
			if i < len(styles) {
				style = NormalizeRef(styles[i])
			}
			out = append(out, map[string]any{
				"src":   src,
				"style": style,
				"slot":  inferSlot(src),
			})
		}
		return out
	case []any:
		out := make([]any, 0, len(t))
		for _, row := range t {
			m, ok := asMap(row)
			if !ok {
				continue
			}
			src := ""
			if v, ok := pick(m, srcKeys); ok {
				src = NormalizeRef(cellStr(v))
			}
			if src == "" {
				continue
			}
			style := ""
			if v, ok := pick(m, styleKeys); ok {
				style = NormalizeRef(cellStr(v))
			}
			out = append(out, map[string]any{
				"src":   src,
				"style": style,
				"slot":  inferSlot(src),
			})
		}
		return out
	default:
		return []any{}
	}
}

func inferSlot(src string) string {
	base := src
	if i := strings.LastIndex(src, "."); i >= 0 {
		base = src[i+1:]
	}
	switch base {
	case "nav", "links", "导航":
		return "nav"
	case "side", "sidebar", "侧栏":
		return "sidebar"
	case "foot", "footer", "页脚":
		return "footer"
	default:
		return base
	}
}

// NormalizeSlot maps author slot names to canonical slots.
func NormalizeSlot(name string) string {
	switch name {
	case "nav", "links", "导航":
		return "nav"
	case "side", "sidebar", "侧栏":
		return "sidebar"
	case "foot", "footer", "页脚":
		return "footer"
	case "main", "index", "", "主体":
		return "main"
	default:
		return name
	}
}

// AsCSSNamed converts a style table → CSS text (non-strict).
func AsCSSNamed(name string, table any) string {
	s, _ := AsCSSNamedChecked(name, table, false)
	return s
}

// AsCSSNamedChecked is like AsCSSNamed; when strict, reject numeric/bool CSS value cells.
func AsCSSNamedChecked(name string, table any, strict bool) (string, error) {
	name = NormalizeRef(name)
	if name == "" {
		return "", nil
	}
	var out strings.Builder
	switch t := table.(type) {
	case map[string]any:
		sels, props, vals, medias := []string{}, []string{}, []string{}, []string{}
		if v, ok := pick(t, selKeys); ok {
			sels = asStrList(v)
		}
		if v, ok := pick(t, propKeys); ok {
			props = asStrList(v)
		}
		var valsRaw any
		if v, ok := pick(t, valKeys); ok {
			valsRaw = v
			vals = asStrList(v)
		}
		if v, ok := pick(t, mediaKeys); ok {
			medias = asStrList(v)
		}
		n := len(props)
		if len(sels) > n {
			n = len(sels)
		}
		if len(vals) > n {
			n = len(vals)
		}
		if n == 0 {
			return "", nil
		}
		if valsRaw != nil {
			if err := checkCSSValueCells(valsRaw, props, strict); err != nil {
				return "", err
			}
		}
		rows := make([][4]string, n)
		for i := 0; i < n; i++ {
			media, sel, p, v := "", "", "", ""
			if i < len(medias) {
				media = medias[i]
			}
			if i < len(sels) {
				sel = sels[i]
			}
			if i < len(props) {
				p = props[i]
			}
			if i < len(vals) {
				v = vals[i]
			}
			rows[i] = [4]string{media, sel, p, v}
		}
		emitCSSRows(rows, name, &out)
	case []any:
		collected := make([][4]string, 0, len(t))
		for _, row := range t {
			m, ok := asMap(row)
			if !ok {
				continue
			}
			prop := ""
			if v, ok := pick(m, propKeys); ok {
				prop = cellStr(v)
			}
			if v, ok := pick(m, valKeys); ok {
				if err := noteCSSValueCell(prop, v, strict); err != nil {
					return "", err
				}
			}
			media, sel, val := "", "", ""
			if v, ok := pick(m, mediaKeys); ok {
				media = cellStr(v)
			}
			if v, ok := pick(m, selKeys); ok {
				sel = cellStr(v)
			}
			if v, ok := pick(m, valKeys); ok {
				val = cellStr(v)
			}
			collected = append(collected, [4]string{media, sel, prop, val})
		}
		emitCSSRows(collected, name, &out)
	}
	return out.String(), nil
}

func cssValueSuspicious(v any) string {
	// Numbers are normal CSS (z-index, opacity, font-weight, unitless 0).
	// Do not treat Markup-evaluated ints/floats as "bare `/` division".
	switch v.(type) {
	case bool:
		return "boolean CSS value (quote the cell if intentional)"
	default:
		return ""
	}
}

func noteCSSValueCell(prop string, val any, strict bool) error {
	why := cssValueSuspicious(val)
	if why == "" {
		return nil
	}
	p := strings.TrimSpace(prop)
	if p == "" {
		p = "(value)"
	}
	msg := fmt.Sprintf("ext/web style: suspicious cell for `%s`: %s", p, why)
	if strict {
		return fmt.Errorf("%s", msg)
	}
	fmt.Fprintf(os.Stderr, "warning: %s\n", msg)
	return nil
}

func checkCSSValueCells(vals any, props []string, strict bool) error {
	switch t := vals.(type) {
	case []any:
		for i, v := range t {
			prop := ""
			if i < len(props) {
				prop = props[i]
			}
			if err := noteCSSValueCell(prop, v, strict); err != nil {
				return err
			}
		}
	default:
		prop := ""
		if len(props) > 0 {
			prop = props[0]
		}
		return noteCSSValueCell(prop, vals, strict)
	}
	return nil
}

func parseKeyframesSel(sel string) (anim string, stop *string, ok bool) {
	t := strings.TrimSpace(sel)
	if len(t) < 10 || !strings.EqualFold(t[:10], "@keyframes") {
		return "", nil, false
	}
	rest := strings.TrimSpace(t[10:])
	if rest == "" {
		return "", nil, false
	}
	parts := strings.Fields(rest)
	if len(parts) == 0 {
		return "", nil, false
	}
	anim = parts[0]
	if len(parts) > 1 {
		s := parts[1]
		stop = &s
	}
	return anim, stop, true
}

func isKeyframeStop(s string) bool {
	t := strings.TrimSpace(s)
	if t == "" {
		return false
	}
	if strings.EqualFold(t, "from") || strings.EqualFold(t, "to") {
		return true
	}
	stripped := strings.TrimSuffix(t, "%")
	if stripped == "" || !strings.HasSuffix(t, "%") {
		return false
	}
	for _, c := range stripped {
		if !(c >= '0' && c <= '9' || c == '.') {
			return false
		}
	}
	return true
}

func keyframeDeclLines(cssProp, value string) []string {
	v := strings.TrimSpace(strings.TrimSuffix(strings.TrimSpace(value), ";"))
	if v == "" {
		return nil
	}
	if strings.Contains(v, ":") {
		parts := strings.Split(v, ";")
		out := make([]string, 0, len(parts))
		for _, p := range parts {
			p = strings.TrimSpace(p)
			if p == "" {
				continue
			}
			out = append(out, "    "+p+";")
		}
		return out
	}
	if cssProp != "" && !isKeyframeStop(cssProp) {
		return []string{fmt.Sprintf("    %s: %s;", cssProp, v)}
	}
	return []string{"    " + v + ";"}
}

type kfStop struct {
	stop  string
	decls []string
}

type kfAnim struct {
	anim  string
	stops []kfStop
}

func pushKeyframe(groups *[]kfAnim, anim, stop string, decls []string) {
	if anim == "" || stop == "" || len(decls) == 0 {
		return
	}
	var animGroup *[]kfStop
	for i := range *groups {
		if (*groups)[i].anim == anim {
			animGroup = &(*groups)[i].stops
			break
		}
	}
	if animGroup == nil {
		*groups = append(*groups, kfAnim{anim: anim})
		animGroup = &(*groups)[len(*groups)-1].stops
	}
	for i := range *animGroup {
		if (*animGroup)[i].stop == stop {
			(*animGroup)[i].decls = append((*animGroup)[i].decls, decls...)
			return
		}
	}
	*animGroup = append(*animGroup, kfStop{stop: stop, decls: decls})
}

type ruleGroup struct {
	sel   string
	decls []string
}

type mediaGroup struct {
	media string
	rules []ruleGroup
}

func pushRule(groups *[]ruleGroup, sel, decl string) {
	if sel == "" {
		return
	}
	for i := range *groups {
		if (*groups)[i].sel == sel {
			(*groups)[i].decls = append((*groups)[i].decls, decl)
			return
		}
	}
	*groups = append(*groups, ruleGroup{sel: sel, decls: []string{decl}})
}

func emitCSSRows(rows [][4]string, className string, out *strings.Builder) {
	var byMedia []mediaGroup
	var byKeyframes []kfAnim
	var plain []string

	for _, row := range rows {
		media := strings.TrimSpace(row[0])
		sel := strings.TrimSpace(row[1])
		p := strings.TrimSpace(row[2])
		v := strings.TrimSpace(row[3])

		if anim, stopInSel, ok := parseKeyframesSel(sel); ok {
			stopFromSel := stopInSel != nil
			var stop, propForDecl string
			if stopInSel != nil {
				stop = *stopInSel
				propForDecl = p
			} else {
				if !isKeyframeStop(p) {
					continue
				}
				stop = p
				propForDecl = ""
			}
			var decls []string
			if stopFromSel {
				decls = keyframeDeclLines(propForDecl, v)
			} else {
				decls = keyframeDeclLines("", v)
			}
			if len(decls) == 0 && v != "" && propForDecl != "" {
				decls = keyframeDeclLines(propForDecl, v)
			}
			pushKeyframe(&byKeyframes, anim, stop, decls)
			continue
		}

		if p == "" {
			continue
		}
		decl := fmt.Sprintf("  %s: %s;", p, v)
		if media != "" {
			var group *[]ruleGroup
			for i := range byMedia {
				if byMedia[i].media == media {
					group = &byMedia[i].rules
					break
				}
			}
			if group == nil {
				byMedia = append(byMedia, mediaGroup{media: media})
				group = &byMedia[len(byMedia)-1].rules
			}
			pushRule(group, sel, decl)
		} else if sel != "" {
			var group *[]ruleGroup
			for i := range byMedia {
				if byMedia[i].media == "" {
					group = &byMedia[i].rules
					break
				}
			}
			if group == nil {
				byMedia = append(byMedia, mediaGroup{media: ""})
				group = &byMedia[len(byMedia)-1].rules
			}
			pushRule(group, sel, decl)
		} else {
			plain = append(plain, decl)
		}
	}

	for _, mg := range byMedia {
		if mg.media == "" {
			for _, r := range mg.rules {
				out.WriteString(fmt.Sprintf("%s {\n%s\n}\n", r.sel, strings.Join(r.decls, "\n")))
			}
		} else {
			inners := make([]string, 0, len(mg.rules))
			for _, r := range mg.rules {
				inners = append(inners, fmt.Sprintf("%s {\n%s\n}", r.sel, strings.Join(r.decls, "\n")))
			}
			out.WriteString(fmt.Sprintf("@media %s {\n%s\n}\n", mg.media, strings.Join(inners, "\n")))
		}
	}
	for _, kf := range byKeyframes {
		out.WriteString(fmt.Sprintf("@keyframes %s {\n", kf.anim))
		for _, st := range kf.stops {
			out.WriteString(fmt.Sprintf("  %s {\n", st.stop))
			out.WriteString(strings.Join(st.decls, "\n"))
			out.WriteByte('\n')
			out.WriteString("  }\n")
		}
		out.WriteString("}\n")
	}
	if len(plain) > 0 {
		out.WriteString(fmt.Sprintf(".%s {\n%s\n}\n", className, strings.Join(plain, "\n")))
	}
}

// SitePathKind distinguishes parse_site_path variants.
type SitePathKind int

const (
	SitePathPlain SitePathKind = iota
	SitePathLibMember
	SitePathDbField
)

// SitePath is the result of ParseSitePath.
type SitePath struct {
	Kind   SitePathKind
	Lib    string // LibMember
	Member string // LibMember
	Table  string // DbField
	Field  string // DbField
	Plain  string // Plain
}

// ParseSitePath parses `lib.member`, `x.table.field`, or plain refs.
func ParseSitePath(raw string) SitePath {
	s := NormalizeRef(raw)
	if s == "" {
		return SitePath{Kind: SitePathPlain, Plain: ""}
	}
	parts := make([]string, 0)
	for _, p := range strings.Split(s, ".") {
		if p != "" {
			parts = append(parts, p)
		}
	}
	switch len(parts) {
	case 2:
		return SitePath{Kind: SitePathLibMember, Lib: parts[0], Member: parts[1]}
	case 3:
		return SitePath{Kind: SitePathDbField, Table: parts[1], Field: parts[2]}
	default:
		return SitePath{Kind: SitePathPlain, Plain: s}
	}
}

// BindTableName returns the single DB table referenced by binds, or "" if none/ambiguous.
func BindTableName(binds []any) (string, bool) {
	var found string
	have := false
	for _, b := range binds {
		m, ok := asMap(b)
		if !ok {
			continue
		}
		back, _ := m["back"].(string)
		sp := ParseSitePath(back)
		if sp.Kind == SitePathDbField {
			if !have {
				found = sp.Table
				have = true
			} else if found != sp.Table {
				return "", false
			}
		} else if strings.Contains(back, ".") {
			p := strings.SplitN(back, ".", 2)
			if len(p) == 2 && p[0] != "" && p[1] != "" {
				t := p[0]
				if !have {
					found = t
					have = true
				} else if found != t {
					return "", false
				}
			}
		}
	}
	return found, have
}

// ProjectRows projects DB rows through bind front/back/css mappings.
func ProjectRows(binds []any, rows []any) any {
	out := make([]any, 0, len(rows))
	for _, row := range rows {
		obj, _ := asMap(row)
		if obj == nil {
			obj = map[string]any{}
		}
		m := map[string]any{}
		css := map[string]any{}
		for _, b := range binds {
			bm, ok := asMap(b)
			if !ok {
				continue
			}
			front, _ := bm["front"].(string)
			back, _ := bm["back"].(string)
			class, _ := bm["css"].(string)
			var col string
			switch sp := ParseSitePath(back); sp.Kind {
			case SitePathDbField:
				col = sp.Field
			case SitePathLibMember:
				col = sp.Member
			case SitePathPlain:
				if strings.Contains(sp.Plain, ".") {
					parts := strings.Split(sp.Plain, ".")
					col = parts[len(parts)-1]
				} else {
					col = sp.Plain
				}
			}
			if v, ok := obj[col]; ok {
				m[front] = v
			} else if v, ok := obj[front]; ok {
				m[front] = v
			}
			if class != "" {
				css[front] = class
			}
		}
		if id, ok := obj["id"]; ok {
			m["id"] = id
		}
		if len(css) > 0 {
			m["_css"] = css
		}
		out = append(out, m)
	}
	return out
}

// AsMetaMap converts an SEO / OpenGraph meta table → map.
func AsMetaMap(table any) map[string]any {
	out := map[string]any{}
	switch t := table.(type) {
	case []any:
		for _, row := range t {
			obj, ok := asMap(row)
			if !ok {
				continue
			}
			key := ""
			for _, k := range metaKeyKeys {
				if v, ok := obj[k]; ok {
					key = NormalizeRef(metaText(v))
					break
				}
			}
			val := ""
			for _, k := range metaValKeys {
				if v, ok := obj[k]; ok {
					val = metaText(v)
					break
				}
			}
			if key != "" {
				out[key] = val
			}
		}
	case map[string]any:
		for k, v := range t {
			out[k] = metaText(v)
		}
	}
	return out
}

func metaText(v any) string {
	return cellStr(v)
}
