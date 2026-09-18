// Package compose ports page assemble helpers from the Rust web plugin.
package compose

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// CallLib loads a lib.member value (host callback).
type CallLib func(path string) (any, error)

func cloneMap(m map[string]any) map[string]any {
	if m == nil {
		return map[string]any{}
	}
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func asObject(v any) map[string]any {
	if m, ok := v.(map[string]any); ok {
		return cloneMap(m)
	}
	return map[string]any{}
}

func strField(m map[string]any, key string) string {
	if m == nil {
		return ""
	}
	s, _ := m[key].(string)
	return s
}

// ComposeComponents merges layout component rows into a page bag.
func ComposeComponents(page any, layout any, callLib CallLib) (any, error) {
	obj := asObject(page)
	rows := table.AsCompose(layout)
	arr, _ := rows.([]any)

	css := strField(obj, "styles_css")
	slotClass := map[string]any{}
	if sc, ok := obj["slot_class"].(map[string]any); ok {
		slotClass = cloneMap(sc)
	}
	parts := map[string]any{}
	composeOut := make([]any, 0, len(arr))

	for _, row := range arr {
		rm, ok := row.(map[string]any)
		if !ok {
			continue
		}
		srcRaw := strField(rm, "src")
		styleRaw := strField(rm, "style")
		if srcRaw == "" {
			continue
		}

		var name string
		var bindTable any
		switch sp := table.ParseSitePath(srcRaw); sp.Kind {
		case table.SitePathLibMember:
			t, err := callLib(sp.Lib + "." + sp.Member)
			if err != nil {
				return nil, err
			}
			name = sp.Member
			bindTable = t
		case table.SitePathPlain:
			t, err := callLib(sp.Plain + "." + sp.Plain)
			if err != nil {
				return nil, err
			}
			name = sp.Plain
			bindTable = t
		case table.SitePathDbField:
			return nil, fmt.Errorf("component cell cannot be a db field: %s", srcRaw)
		}

		slot := table.NormalizeSlot(name)
		styleName := ""
		if styleRaw != "" {
			switch sp := table.ParseSitePath(styleRaw); sp.Kind {
			case table.SitePathLibMember:
				st, err := callLib(sp.Lib + "." + sp.Member)
				if err != nil {
					return nil, err
				}
				css += table.AsCSSNamed(sp.Member, st)
				styleName = sp.Member
			case table.SitePathPlain:
				styleName = sp.Plain
			case table.SitePathDbField:
				styleName = table.NormalizeRef(styleRaw)
			}
		}

		binds := table.AsBind(bindTable)
		switch slot {
		case "nav":
			obj["nav"] = binds
		case "sidebar":
			obj["sidebar"] = binds
		case "footer":
			obj["footer"] = binds
		default:
			obj["main"] = binds
		}
		if styleName != "" {
			slotClass[slot] = styleName
		}
		composeOut = append(composeOut, map[string]any{
			"src":   name,
			"style": styleName,
			"slot":  slot,
		})

		part := map[string]any{
			"_type":    "page",
			"slot":     slot,
			"fragment": slot,
		}
		switch slot {
		case "nav":
			part["nav"] = binds
		case "sidebar":
			part["sidebar"] = binds
		case "footer":
			part["footer"] = binds
		default:
			part["main"] = binds
		}
		parts[name] = part
	}

	obj["compose"] = composeOut
	if css != "" {
		obj["styles_css"] = css
	}
	if len(slotClass) > 0 {
		obj["slot_class"] = slotClass
	}
	if len(parts) > 0 {
		obj["parts"] = parts
	}
	return obj, nil
}

func resolveBinds(mainTable any, callLib CallLib) (binds []any, css string, err error) {
	raw := table.AsBind(mainTable)
	arr, _ := raw.([]any)
	binds = make([]any, 0, len(arr))
	for _, b := range arr {
		bm, ok := b.(map[string]any)
		if !ok {
			continue
		}
		front := strField(bm, "front")
		backRaw := strField(bm, "back")
		cssRaw := strField(bm, "css")

		var back string
		switch sp := table.ParseSitePath(backRaw); sp.Kind {
		case table.SitePathDbField:
			back = sp.Table + "." + sp.Field
		case table.SitePathPlain:
			back = sp.Plain
		case table.SitePathLibMember:
			v, err := callLib(sp.Lib + "." + sp.Member)
			if err != nil {
				back = sp.Lib + "." + sp.Member
			} else if s, ok := v.(string); ok {
				back = s
			} else {
				back = fmt.Sprint(v)
			}
		}

		cssName := ""
		switch sp := table.ParseSitePath(cssRaw); sp.Kind {
		case table.SitePathLibMember:
			st, err := callLib(sp.Lib + "." + sp.Member)
			if err != nil {
				return nil, "", err
			}
			css += table.AsCSSNamed(sp.Member, st)
			cssName = sp.Member
		case table.SitePathPlain:
			cssName = sp.Plain
		case table.SitePathDbField:
			cssName = table.NormalizeRef(cssRaw)
		}

		binds = append(binds, map[string]any{
			"front": front,
			"back":  back,
			"css":   cssName,
		})
	}
	return binds, css, nil
}

// ComposeMain merges a main bind table into a page bag.
func ComposeMain(page any, mainTable any, callLib CallLib) (any, error) {
	obj := asObject(page)
	binds, cssExtra, err := resolveBinds(mainTable, callLib)
	if err != nil {
		return nil, err
	}
	css := strField(obj, "styles_css") + cssExtra

	if t, ok := table.BindTableName(binds); ok {
		obj["table"] = t
	}
	obj["main"] = binds
	if css != "" {
		obj["styles_css"] = css
	}

	parts := map[string]any{}
	if p, ok := obj["parts"].(map[string]any); ok {
		parts = cloneMap(p)
	}
	part := map[string]any{
		"_type":    "page",
		"slot":     "main",
		"fragment": "main",
		"main":     binds,
	}
	if intro, ok := obj["intro"]; ok {
		part["intro"] = intro
	}
	parts["index"] = part
	obj["parts"] = parts
	return obj, nil
}

// ComposeList appends a secondary bind (detail + child list), with optional
// query/order/target (inject into intro element id, same as form_target).
func ComposeList(page any, mainTable any, query any, order, target string, callLib CallLib) (any, error) {
	obj := asObject(page)
	binds, cssExtra, err := resolveBinds(mainTable, callLib)
	if err != nil {
		return nil, err
	}
	if cssExtra != "" {
		prev := strField(obj, "styles_css")
		if prev != "" {
			obj["styles_css"] = prev + cssExtra
		} else {
			obj["styles_css"] = cssExtra
		}
	}
	entry := map[string]any{"main": binds}
	if query != nil {
		entry["query"] = query
	}
	if strings.TrimSpace(order) != "" {
		entry["order"] = strings.TrimSpace(order)
	}
	if t := strings.TrimSpace(target); t != "" {
		entry["target"] = t
	}
	lists := []any{}
	if existing, ok := obj["lists"].([]any); ok {
		lists = append(lists, existing...)
	}
	lists = append(lists, entry)
	obj["lists"] = lists
	return obj, nil
}

// ComposeForm embeds a form into the page main slot.
func ComposeForm(page any, form any, formID string, formTarget *string) (any, error) {
	id := strings.TrimSpace(formID)
	if id == "" {
		return nil, fmt.Errorf("compose_form requires non-empty `id`")
	}
	obj := asObject(page)

	frm := form
	if m, ok := form.(map[string]any); ok {
		fm := cloneMap(m)
		if _, ok := fm["show_meta"]; !ok {
			fm["show_meta"] = false
		}
		if _, ok := fm["title"]; !ok {
			fm["title"] = id
		}
		frm = fm
	}

	obj["form_id"] = id
	obj["form"] = frm

	var target *string
	if formTarget != nil {
		t := strings.TrimSpace(*formTarget)
		if t != "" {
			target = &t
			obj["form_target"] = t
		} else {
			delete(obj, "form_target")
		}
	} else {
		delete(obj, "form_target")
	}

	forms := map[string]any{}
	if f, ok := obj["forms"].(map[string]any); ok {
		forms = cloneMap(f)
	}
	forms[id] = frm
	obj["forms"] = forms

	parts := map[string]any{}
	if p, ok := obj["parts"].(map[string]any); ok {
		parts = cloneMap(p)
	}
	part := map[string]any{
		"_type":    "page",
		"slot":     "main",
		"fragment": "main",
		"form_id":  id,
		"form":     frm,
	}
	if target != nil {
		part["form_target"] = *target
	}
	if intro, ok := obj["intro"]; ok {
		part["intro"] = intro
	}
	parts["form_"+id] = part
	obj["parts"] = parts
	return obj, nil
}
