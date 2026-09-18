// Package form ports form fields / validate / render / submit (design §5.5, W-G3).
package form

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

var (
	fieldKeys     = []string{"字段", "name", "field", "列"}
	labelKeys     = []string{"标签", "label"}
	typeKeys      = []string{"类型", "type"}
	reqKeys       = []string{"必填", "required"}
	defKeys       = []string{"默认", "default"}
	sourceKeys    = []string{"来源", "source"}
	ruleFieldKeys = []string{"字段", "field", "name"}
	ruleKeys      = []string{"规则", "rule"}
	msgKeys       = []string{"消息", "message", "msg"}
)

// New matches Rust form_new.
func New(tableName, action, id string) map[string]any {
	action = normalizeAction(action)
	out := map[string]any{
		"_type":    "form",
		"action":   action,
		"fields":   []any{},
		"rules":    []any{},
		"redirect": "/",
	}
	if tableName != "" {
		out["table"] = tableName
	}
	if id != "" {
		out["id"] = id
	}
	return out
}

func normalizeAction(action string) string {
	switch action {
	case "update", "更新":
		return "update"
	case "":
		return "insert"
	default:
		return "insert"
	}
}

// SetFields attaches normalized field rows to the form bag.
func SetFields(formBag map[string]any, fields any) map[string]any {
	out := cloneMap(formBag)
	out["fields"] = asFormFields(fields)
	return out
}

// SetRules attaches normalized rule rows to the form bag.
func SetRules(formBag map[string]any, rules any) map[string]any {
	out := cloneMap(formBag)
	out["rules"] = asFormRules(rules)
	return out
}

func cloneMap(m map[string]any) map[string]any {
	out := map[string]any{}
	for k, v := range m {
		out[k] = v
	}
	return out
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

func pick(m map[string]any, keys []string) (any, bool) {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v, true
		}
	}
	return nil, false
}

func boolish(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		s := strings.TrimSpace(t)
		switch s {
		case "1", "true", "True", "yes", "是", "必填", "required", "可":
			return true
		}
	case float64:
		return t != 0
	case int:
		return t != 0
	case int64:
		return t != 0
	}
	return false
}

func asFormFields(fields any) []any {
	switch t := fields.(type) {
	case []any:
		var out []any
		for _, row := range t {
			m, ok := row.(map[string]any)
			if !ok {
				continue
			}
			nameRaw := ""
			if v, ok := pick(m, fieldKeys); ok {
				nameRaw = cellStr(v)
			}
			name := table.NormalizeRef(nameRaw)
			if name == "" {
				continue
			}
			label := name
			if v, ok := pick(m, labelKeys); ok {
				if s := cellStr(v); s != "" {
					label = s
				}
			}
			ty := "text"
			if v, ok := pick(m, typeKeys); ok {
				if s := cellStr(v); s != "" {
					ty = s
				}
			}
			required := false
			if v, ok := pick(m, reqKeys); ok {
				required = boolish(v)
			}
			def := ""
			if v, ok := pick(m, defKeys); ok {
				def = cellStr(v)
			}
			source := "client"
			if v, ok := pick(m, sourceKeys); ok {
				if s := strings.TrimSpace(cellStr(v)); s != "" {
					source = s
				}
			}
			item := map[string]any{
				"name":     name,
				"label":    label,
				"type":     ty,
				"required": required,
				"default":  def,
				"source":   normalizeSource(source),
			}
			if v, ok := m["pk"]; ok {
				item["pk"] = boolish(v)
			}
			out = append(out, item)
		}
		return out
	case map[string]any:
		names := []string{}
		if v, ok := pick(t, fieldKeys); ok {
			names = asStrList(v)
		}
		labels := []string{}
		if v, ok := pick(t, labelKeys); ok {
			labels = asStrList(v)
		}
		types := []string{}
		if v, ok := pick(t, typeKeys); ok {
			types = asStrList(v)
		}
		reqs := []string{}
		if v, ok := pick(t, reqKeys); ok {
			reqs = asStrList(v)
		}
		defs := []string{}
		if v, ok := pick(t, defKeys); ok {
			defs = asStrList(v)
		}
		sources := []string{}
		if v, ok := pick(t, sourceKeys); ok {
			sources = asStrList(v)
		}
		var out []any
		for i, nameRaw := range names {
			name := table.NormalizeRef(nameRaw)
			if name == "" {
				continue
			}
			label := name
			if i < len(labels) && labels[i] != "" {
				label = labels[i]
			}
			ty := "text"
			if i < len(types) && types[i] != "" {
				ty = types[i]
			}
			required := false
			if i < len(reqs) {
				required = boolish(reqs[i])
			}
			def := ""
			if i < len(defs) {
				def = defs[i]
			}
			source := "client"
			if i < len(sources) && strings.TrimSpace(sources[i]) != "" {
				source = sources[i]
			}
			out = append(out, map[string]any{
				"name":     name,
				"label":    label,
				"type":     ty,
				"required": required,
				"default":  def,
				"source":   normalizeSource(source),
			})
		}
		return out
	default:
		return []any{}
	}
}

func asFormRules(rules any) []any {
	switch t := rules.(type) {
	case []any:
		var out []any
		for _, row := range t {
			m, ok := row.(map[string]any)
			if !ok {
				continue
			}
			fieldRaw := ""
			if v, ok := pick(m, ruleFieldKeys); ok {
				fieldRaw = cellStr(v)
			}
			field := table.NormalizeRef(fieldRaw)
			rule := ""
			if v, ok := pick(m, ruleKeys); ok {
				rule = cellStr(v)
			}
			if field == "" || rule == "" {
				continue
			}
			msg := ""
			if v, ok := pick(m, msgKeys); ok {
				msg = cellStr(v)
			}
			out = append(out, map[string]any{
				"field":   field,
				"rule":    rule,
				"message": msg,
			})
		}
		return out
	case map[string]any:
		fields := []string{}
		if v, ok := pick(t, ruleFieldKeys); ok {
			fields = asStrList(v)
		}
		ruleList := []string{}
		if v, ok := pick(t, ruleKeys); ok {
			ruleList = asStrList(v)
		}
		msgs := []string{}
		if v, ok := pick(t, msgKeys); ok {
			msgs = asStrList(v)
		}
		n := len(fields)
		if len(ruleList) > n {
			n = len(ruleList)
		}
		var out []any
		for i := 0; i < n; i++ {
			field := ""
			if i < len(fields) {
				field = table.NormalizeRef(fields[i])
			}
			rule := ""
			if i < len(ruleList) {
				rule = ruleList[i]
			}
			if field == "" || rule == "" {
				continue
			}
			msg := ""
			if i < len(msgs) {
				msg = msgs[i]
			}
			out = append(out, map[string]any{
				"field":   field,
				"rule":    rule,
				"message": msg,
			})
		}
		return out
	default:
		return []any{}
	}
}

func sqlTypeToInput(sqlType, name string) string {
	t := strings.ToUpper(sqlType)
	if strings.Contains(t, "INT") || strings.Contains(t, "REAL") ||
		strings.Contains(t, "FLOAT") || strings.Contains(t, "DOUBLE") {
		return "number"
	}
	if name == "body" || strings.HasSuffix(name, "_body") || strings.HasSuffix(name, "_text") {
		return "textarea"
	}
	if strings.Contains(name, "email") {
		return "email"
	}
	if strings.Contains(name, "url") || name == "href" {
		return "url"
	}
	return "text"
}

// FromSchema builds a form handle from live SQLite schema (admin new/edit).
func FromSchema(dbURL, tableName, action, id, adminPrefix string) (map[string]any, error) {
	action = normalizeAction(action)
	cols, err := db.TableInfo(dbURL, tableName)
	if err != nil {
		return nil, err
	}
	if len(cols) == 0 {
		return nil, fmt.Errorf("unknown table `%s`", tableName)
	}
	fields := make([]any, 0, len(cols))
	for _, c := range cols {
		if action == "insert" && c.PK && c.Name == "id" {
			continue
		}
		required := c.NotNull || (action == "update" && c.PK)
		fields = append(fields, map[string]any{
			"name":     c.Name,
			"label":    c.Name,
			"type":     sqlTypeToInput(c.SQLType, c.Name),
			"required": required,
			"default":  "",
			"pk":       c.PK,
		})
	}
	if adminPrefix == "" {
		adminPrefix = "/admin"
	}
	base := strings.TrimRight(adminPrefix, "/")
	tableHref := base + "/" + tableName
	formBag := New(tableName, action, id)
	formBag["fields"] = fields
	formBag["redirect"] = tableHref
	formBag["cancel_href"] = tableHref
	if action == "insert" {
		formBag["action_url"] = tableHref + "/new"
		formBag["title"] = "New " + tableName
	} else if id != "" {
		formBag["action_url"] = tableHref + "/" + id + "/edit"
		formBag["title"] = fmt.Sprintf("Edit %s #%s", tableName, id)
		formBag["id"] = id
	}
	return formBag, nil
}

func dataMap(data any) map[string]any {
	rows := table.AsRows(data)
	if arr, ok := rows.([]any); ok && len(arr) > 0 {
		if m, ok := arr[0].(map[string]any); ok {
			return cloneMap(m)
		}
	}
	if m, ok := data.(map[string]any); ok {
		return cloneMap(m)
	}
	return map[string]any{}
}

func fieldText(data map[string]any, name string) string {
	if data == nil {
		return ""
	}
	if v, ok := data[name]; ok {
		return cellStr(v)
	}
	return ""
}

func defaultMessage(rule, field string) string {
	if rest, ok := strings.CutPrefix(rule, "min:"); ok {
		return fmt.Sprintf("%s must be at least %s", field, rest)
	}
	if rest, ok := strings.CutPrefix(rule, "max:"); ok {
		return fmt.Sprintf("%s must be at most %s", field, rest)
	}
	if other, ok := strings.CutPrefix(rule, "match:"); ok {
		return fmt.Sprintf("%s must match %s", field, other)
	}
	if strings.HasPrefix(rule, "in:") {
		return fmt.Sprintf("%s is not an allowed value", field)
	}
	switch rule {
	case "required":
		return field + " is required"
	case "email":
		return field + " must be an email"
	case "url":
		return field + " must be a URL"
	default:
		return fmt.Sprintf("%s failed `%s`", field, rule)
	}
}

func checkRule(rule, field string, data map[string]any) (string, bool) {
	val := fieldText(data, field)
	trimmed := strings.TrimSpace(val)
	if rule == "required" {
		if trimmed == "" {
			return defaultMessage(rule, field), true
		}
		return "", false
	}
	if trimmed == "" {
		return "", false
	}
	if n, ok := strings.CutPrefix(rule, "min:"); ok {
		if min, err := strconv.ParseInt(n, 10, 64); err == nil {
			if num, err := strconv.ParseFloat(trimmed, 64); err == nil {
				if num < float64(min) {
					return defaultMessage(rule, field), true
				}
			} else if int64(utf8.RuneCountInString(trimmed)) < min {
				return defaultMessage(rule, field), true
			}
		}
		return "", false
	}
	if n, ok := strings.CutPrefix(rule, "max:"); ok {
		if max, err := strconv.ParseInt(n, 10, 64); err == nil {
			if num, err := strconv.ParseFloat(trimmed, 64); err == nil {
				if num > float64(max) {
					return defaultMessage(rule, field), true
				}
			} else if int64(utf8.RuneCountInString(trimmed)) > max {
				return defaultMessage(rule, field), true
			}
		}
		return "", false
	}
	if other, ok := strings.CutPrefix(rule, "match:"); ok {
		if val != fieldText(data, other) {
			return defaultMessage(rule, field), true
		}
		return "", false
	}
	if list, ok := strings.CutPrefix(rule, "in:"); ok {
		allowed := strings.Split(list, ",")
		found := false
		for _, a := range allowed {
			if strings.TrimSpace(a) == trimmed {
				found = true
				break
			}
		}
		if !found {
			return defaultMessage(rule, field), true
		}
		return "", false
	}
	switch rule {
	case "email":
		if !(strings.Contains(trimmed, "@") && strings.Contains(trimmed, ".")) {
			return defaultMessage(rule, field), true
		}
	case "url":
		if !(strings.HasPrefix(trimmed, "http://") || strings.HasPrefix(trimmed, "https://")) {
			return defaultMessage(rule, field), true
		}
	}
	return "", false
}

func fieldSlice(formBag map[string]any) []any {
	if formBag == nil {
		return nil
	}
	arr, _ := formBag["fields"].([]any)
	return arr
}

func ruleSlice(formBag map[string]any) []any {
	if formBag == nil {
		return nil
	}
	arr, _ := formBag["rules"].([]any)
	return arr
}

// Validate merges field-required + author rules; returns {ok, errors:[{field,message}]}.
// extraRules may be nil.
func Validate(formBag map[string]any, extraRules any, data any) map[string]any {
	dataM := dataMap(data)
	fields := fieldSlice(formBag)
	rules := append([]any{}, ruleSlice(formBag)...)
	if extraRules != nil {
		rules = append(rules, asFormRules(extraRules)...)
	}
	for _, f := range fields {
		fm, ok := f.(map[string]any)
		if !ok {
			continue
		}
		name, _ := fm["name"].(string)
		required, _ := fm["required"].(bool)
		if !required || name == "" {
			continue
		}
		already := false
		for _, r := range rules {
			rm, ok := r.(map[string]any)
			if !ok {
				continue
			}
			if rm["field"] == name && rm["rule"] == "required" {
				already = true
				break
			}
		}
		if !already {
			rules = append([]any{map[string]any{
				"field":   name,
				"rule":    "required",
				"message": name + " is required",
			}}, rules...)
		}
	}

	var errors []any
	for _, r := range rules {
		rm, ok := r.(map[string]any)
		if !ok {
			continue
		}
		field, _ := rm["field"].(string)
		rule, _ := rm["rule"].(string)
		msg, _ := rm["message"].(string)
		if field == "" || rule == "" {
			continue
		}
		if defMsg, fail := checkRule(rule, field, dataM); fail {
			outMsg := defMsg
			if msg != "" {
				outMsg = msg
			}
			errors = append(errors, map[string]any{
				"field":   field,
				"message": outMsg,
			})
		}
	}
	if errors == nil {
		errors = []any{}
	}
	return map[string]any{
		"ok":     len(errors) == 0,
		"errors": errors,
	}
}

func esc(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, `"`, "&quot;")
	return s
}

func strOf(m map[string]any, key, def string) string {
	if m == nil {
		return def
	}
	if v, ok := m[key]; ok {
		if s := cellStr(v); s != "" {
			return s
		}
	}
	return def
}

// RenderBody emits form markup only (no document shell).
func RenderBody(formBag map[string]any, formID string, data, errors any, csrf string) string {
	return RenderBodyCtx(formBag, formID, data, errors, csrf, nil)
}

// RenderBodyCtx is RenderBody plus request-context hidden fields for POST.
func RenderBodyCtx(formBag map[string]any, formID string, data, errors any, csrf string, ctx *RequestContext) string {
	fields := fieldSlice(formBag)
	dataM := map[string]any{}
	if data != nil {
		dataM = dataMap(data)
	}
	errMap := map[string]string{}
	switch e := errors.(type) {
	case []any:
		for _, item := range e {
			if m, ok := item.(map[string]any); ok {
				f, _ := m["field"].(string)
				msg := cellStr(m["message"])
				if f != "" && msg != "" {
					errMap[f] = msg
				}
			}
		}
	case map[string]any:
		for k, v := range e {
			errMap[k] = cellStr(v)
		}
	}

	action := strOf(formBag, "action", "insert")
	tableName := strOf(formBag, "table", "")
	heading := strOf(formBag, "title", formID)
	postTo := strOf(formBag, "action_url", "/_form/"+formID)
	cancel := strOf(formBag, "cancel_href", "/")
	rowID := ""
	if formBag != nil {
		rowID = cellStr(formBag["id"])
	}
	showMeta := true
	if formBag != nil {
		if v, ok := formBag["show_meta"].(bool); ok {
			showMeta = v
		}
	}

	var body strings.Builder
	body.WriteString(`<div class="site-form">`)
	if showMeta {
		fmt.Fprintf(&body, `<p class="meta">table=<code>%s</code> · action=<code>%s</code> · id=<code>%s</code></p>`,
			esc(tableName), esc(action), esc(heading))
	}
	hasFile := false
	for _, f := range fields {
		if fm, ok := f.(map[string]any); ok {
			ty, _ := fm["type"].(string)
			if strings.EqualFold(ty, "file") {
				hasFile = true
				break
			}
		}
	}
	body.WriteString(`<form method="post" action="` + esc(postTo) + `"`)
	if hasFile {
		body.WriteString(` enctype="multipart/form-data"`)
	}
	body.WriteString(`>`)
	if csrf != "" {
		fmt.Fprintf(&body, `<input type="hidden" name="_csrf" value="%s"/>`, esc(csrf))
	}
	if action == "update" && rowID != "" {
		fmt.Fprintf(&body, `<input type="hidden" name="id" value="%s"/>`, esc(rowID))
	}
	if ctx != nil {
		if len(ctx.Params) > 0 {
			if b, err := json.Marshal(ctx.Params); err == nil {
				fmt.Fprintf(&body, `<input type="hidden" name="_mq_params" value="%s"/>`, esc(string(b)))
			}
		}
		ret := strings.TrimSpace(ctx.ReturnPath)
		if ret == "" {
			ret = strings.TrimSpace(ctx.Path)
		}
		if ret != "" && strings.HasPrefix(ret, "/") && !strings.HasPrefix(ret, "//") {
			fmt.Fprintf(&body, `<input type="hidden" name="_mq_return" value="%s"/>`, esc(ret))
		}
	}
	for _, f := range fields {
		fm, ok := f.(map[string]any)
		if !ok {
			continue
		}
		name, _ := fm["name"].(string)
		if name == "" {
			continue
		}
		if !IsClientSource(strOf(fm, "source", "client")) {
			continue
		}
		label := strOf(fm, "label", name)
		ty := strOf(fm, "type", "text")
		required, _ := fm["required"].(bool)
		def := strOf(fm, "default", "")
		pk, _ := fm["pk"].(bool)
		var value string
		if _, ok := dataM[name]; ok {
			value = fieldText(dataM, name)
		} else if name == "id" && rowID != "" {
			value = rowID
		} else {
			value = def
		}
		readonly := action == "update" && pk && name == "id"
		if readonly {
			fmt.Fprintf(&body, `<label>%s<input type="text" value="%s" readonly/></label>`,
				esc(label), esc(value))
			continue
		}
		reqAttr := ""
		if required {
			reqAttr = " required"
		}
		if ty == "hidden" {
			fmt.Fprintf(&body, `<input type="hidden" name="%s" value="%s"/>`, esc(name), esc(value))
			continue
		}
		body.WriteString("<label>")
		body.WriteString(esc(label))
		if ty == "textarea" {
			fmt.Fprintf(&body, `<textarea name="%s" rows="5"%s>%s</textarea>`,
				esc(name), reqAttr, esc(value))
		} else {
			inputType := "text"
			switch ty {
			case "number", "email", "url", "checkbox", "file":
				inputType = ty
			}
			if ty == "checkbox" {
				checked := ""
				switch value {
				case "1", "true", "on", "yes":
					checked = " checked"
				}
				fmt.Fprintf(&body, `<input type="checkbox" name="%s" value="1"%s%s/>`,
					esc(name), checked, reqAttr)
			} else if ty == "file" {
				fmt.Fprintf(&body, `<input type="file" name="%s"%s/>`, esc(name), reqAttr)
			} else {
				fmt.Fprintf(&body, `<input type="%s" name="%s" value="%s"%s/>`,
					esc(inputType), esc(name), esc(value), reqAttr)
			}
		}
		if msg, ok := errMap[name]; ok {
			fmt.Fprintf(&body, `<span class="err">%s</span>`, esc(msg))
		}
		body.WriteString("</label>")
	}
	fmt.Fprintf(&body, `<div class="actions"><button type="submit">Submit</button><a href="%s">cancel</a></div>`,
		esc(cancel))
	body.WriteString(`</form></div>`)
	return body.String()
}

// Render returns a standalone form HTML document.
// data and errors may be nil; csrf is optional (pass "" when unused).
func Render(formBag map[string]any, formID string, data, errors any, csrf string) string {
	heading := formID
	if formBag != nil {
		if t := strOf(formBag, "title", ""); t != "" {
			heading = t
		}
	}
	var body strings.Builder
	body.WriteString(`<!DOCTYPE html><html lang="zh-CN"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>`)
	fmt.Fprintf(&body, `<title>%s</title>`, esc(heading))
	body.WriteString(`<style>
body{font-family:"IBM Plex Sans","Noto Sans SC",sans-serif;margin:1.5rem;background:#fafaf9;color:#1c1917}
.site-form form{max-width:28rem;display:grid;gap:.85rem}
.site-form label{display:grid;gap:.25rem;font-size:.9rem}
.site-form input,.site-form textarea{padding:.5rem .6rem;border:1px solid #e7e5e4;border-radius:4px;font:inherit}
.site-form input[readonly]{background:#f5f5f4;color:#57534e}
.site-form .err{color:#b91c1c;font-size:.85rem}
.site-form .actions{display:flex;gap:.75rem;align-items:center;flex-wrap:wrap}
.site-form button{background:#0f766e;color:#fff;border:0;padding:.55rem 1rem;border-radius:4px;cursor:pointer}
.site-form a{color:#0f766e}
.site-form .meta{color:#57534e;font-size:.9rem}
</style></head><body>`)
	fmt.Fprintf(&body, `<h1>%s</h1>`, esc(heading))
	body.WriteString(RenderBody(formBag, formID, data, errors, csrf))
	body.WriteString(`</body></html>`)
	return body.String()
}

// Submit validates then insert/update via db. On validation failure returns
// {ok:false, errors:[…]} with a nil error (matches Rust).
func Submit(formBag map[string]any, data any, dbURL string) (map[string]any, error) {
	action := strOf(formBag, "action", "insert")
	rowMap := dataMap(data)
	if action == "update" {
		if formBag != nil {
			if fid := cellStr(formBag["id"]); fid != "" {
				if _, ok := rowMap["id"]; !ok {
					rowMap["id"] = fid
				}
			}
		}
	}
	v := Validate(formBag, nil, rowMap)
	ok, _ := v["ok"].(bool)
	if !ok {
		errs := v["errors"]
		if errs == nil {
			errs = []any{}
		}
		return map[string]any{"ok": false, "errors": errs}, nil
	}
	tableName := ""
	if formBag != nil {
		tableName, _ = formBag["table"].(string)
	}
	if tableName == "" {
		return nil, fmt.Errorf("form missing `table`")
	}
	var result map[string]any
	var err error
	if action == "update" {
		id := ""
		if formBag != nil {
			id = cellStr(formBag["id"])
		}
		if id == "" {
			id = cellStr(rowMap["id"])
		}
		if id == "" {
			return nil, fmt.Errorf("update requires `id`")
		}
		result, err = db.Update(dbURL, tableName, id, rowMap)
	} else {
		result, err = db.Insert(dbURL, tableName, []any{rowMap})
	}
	if err != nil {
		return nil, err
	}
	redirect := strOf(formBag, "redirect", "/")
	return map[string]any{
		"ok":       true,
		"result":   result,
		"redirect": redirect,
	}, nil
}
