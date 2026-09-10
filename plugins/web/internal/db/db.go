// Package db ports minimal SQLite CRUD from the Rust web plugin (sqlite: URLs).
package db

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
	_ "modernc.org/sqlite"
)

var (
	poolMu sync.Mutex
	pool   = map[string]*sql.DB{}
)

// ResolveURL normalizes sqlite: paths. Absolute paths stay; relative stay as given
// (caller / host resolve cwd). Always returns a sqlite:… URL when prefixed.
func ResolveURL(url string) string {
	lower := strings.ToLower(url)
	if strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://") {
		return url
	}
	stripped := url
	prefix := ""
	if strings.HasPrefix(url, "sqlite:") {
		stripped = strings.TrimPrefix(url, "sqlite:")
		prefix = "sqlite:"
	} else if strings.HasPrefix(url, "SQLITE:") {
		stripped = strings.TrimPrefix(url, "SQLITE:")
		prefix = "SQLITE:"
	}
	if prefix == "" {
		return stripped
	}
	return prefix + stripped
}

func pathOf(url string) string {
	u := ResolveURL(url)
	if strings.HasPrefix(u, "sqlite:") {
		return strings.TrimPrefix(u, "sqlite:")
	}
	if strings.HasPrefix(u, "SQLITE:") {
		return strings.TrimPrefix(u, "SQLITE:")
	}
	return u
}

func open(url string) (*sql.DB, error) {
	path := pathOf(url)
	poolMu.Lock()
	defer poolMu.Unlock()
	key := ResolveURL(url)
	if db, ok := pool[key]; ok {
		return db, nil
	}
	if parent := filepath.Dir(path); parent != "" && parent != "." {
		if err := os.MkdirAll(parent, 0o755); err != nil {
			return nil, err
		}
	}
	db, err := sql.Open("sqlite", path)
	if err != nil {
		return nil, err
	}
	if _, err := db.Exec(`PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;`); err != nil {
		_ = db.Close()
		return nil, err
	}
	pool[key] = db
	return db, nil
}

// ResetPool closes pooled connections (tests).
func ResetPool() {
	poolMu.Lock()
	defer poolMu.Unlock()
	for k, db := range pool {
		_ = db.Close()
		delete(pool, k)
	}
}

func ident(s string) (string, error) {
	s = strings.TrimSpace(s)
	if s == "" {
		return "", fmt.Errorf("bad identifier `%s`", s)
	}
	for _, c := range s {
		if !(c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c >= '0' && c <= '9' || c == '_') {
			return "", fmt.Errorf("bad identifier `%s`", s)
		}
	}
	return s, nil
}

func sqlType(t string) string {
	switch strings.ToLower(strings.TrimSpace(t)) {
	case "int", "integer":
		return "INTEGER"
	case "real", "float", "double":
		return "REAL"
	case "blob":
		return "BLOB"
	case "timestamp", "datetime", "timestamptz", "审计":
		return "TEXT"
	default:
		return "TEXT"
	}
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

func boolField(m map[string]any, keys []string, def bool) bool {
	for _, k := range keys {
		v, ok := m[k]
		if !ok {
			continue
		}
		switch t := v.(type) {
		case bool:
			return t
		case string:
			switch t {
			case "true", "True", "1", "yes", "是", "可", "唯一", "索引":
				return true
			case "false", "False", "0", "no", "否":
				return false
			}
		case float64:
			return int64(t) != 0
		}
	}
	return def
}

func parseFK(raw string) (string, string, error) {
	s := strings.TrimSpace(raw)
	if s == "" {
		return "", "", fmt.Errorf("empty foreign key")
	}
	if i := strings.IndexByte(s, '('); i >= 0 {
		t, err := ident(s[:i])
		if err != nil {
			return "", "", err
		}
		col := strings.TrimSpace(strings.TrimSuffix(s[i+1:], ")"))
		if col == "" {
			return t, "id", nil
		}
		c, err := ident(col)
		return t, c, err
	}
	if i := strings.IndexByte(s, '.'); i >= 0 {
		t, err := ident(s[:i])
		if err != nil {
			return "", "", err
		}
		c, err := ident(s[i+1:])
		return t, c, err
	}
	t, err := ident(s)
	return t, "id", err
}

func utcNowISO() string {
	return time.Now().UTC().Format("2006-01-02T15:04:05Z")
}

// Init creates a table from AsFields-shaped schema.
// Returns {"_type":"db_table","name":table,"url":url}.
func Init(url, name string, fields any) (map[string]any, error) {
	tableName, err := ident(name)
	if err != nil {
		return nil, err
	}
	colsAny := table.AsFields(fields)
	arr, ok := colsAny.([]any)
	if !ok {
		return nil, fmt.Errorf("fields must be a list")
	}
	if len(arr) == 0 {
		return nil, fmt.Errorf("empty fields")
	}
	var parts []string
	var indexSQL []string
	hasPK := false
	for _, c := range arr {
		m, ok := c.(map[string]any)
		if !ok {
			continue
		}
		nm := cellStr(m["name"])
		if nm == "" {
			nm = cellStr(m["字段"])
		}
		colName, err := ident(nm)
		if err != nil {
			return nil, err
		}
		ty := cellStr(m["type"])
		if ty == "" {
			ty = cellStr(m["类型"])
		}
		if ty == "" {
			ty = "text"
		}
		nullable := boolField(m, []string{"nullable", "可空"}, true)
		unique := boolField(m, []string{"unique", "唯一"}, false)
		index := boolField(m, []string{"index", "索引"}, false)
		col := fmt.Sprintf(`"%s" %s`, colName, sqlType(ty))
		if colName == "id" && !hasPK {
			col += " PRIMARY KEY"
			if sqlType(ty) == "INTEGER" {
				col += " AUTOINCREMENT"
			}
			hasPK = true
		} else if !nullable {
			col += " NOT NULL"
		}
		if unique && colName != "id" {
			col += " UNIQUE"
		}
		fkRaw := cellStr(m["fk"])
		if fkRaw == "" {
			fkRaw = cellStr(m["外键"])
		}
		if fkRaw != "" {
			rt, rc, err := parseFK(fkRaw)
			if err != nil {
				return nil, err
			}
			col += fmt.Sprintf(` REFERENCES "%s"("%s")`, rt, rc)
		}
		parts = append(parts, col)
		if (index || unique) && colName != "id" {
			kind := ""
			if unique {
				kind = "UNIQUE "
			}
			indexSQL = append(indexSQL, fmt.Sprintf(
				`CREATE %sINDEX IF NOT EXISTS "idx_%s_%s" ON "%s" ("%s")`,
				kind, tableName, colName, tableName, colName,
			))
		}
	}
	sqlStmt := fmt.Sprintf(`CREATE TABLE IF NOT EXISTS "%s" (%s)`, tableName, strings.Join(parts, ", "))
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	if _, err := db.Exec(sqlStmt); err != nil {
		return nil, err
	}
	for _, stmt := range indexSQL {
		if _, err := db.Exec(stmt); err != nil {
			return nil, err
		}
	}
	return map[string]any{
		"_type": "db_table",
		"name":  tableName,
		"url":   ResolveURL(url),
	}, nil
}

func columnNames(db *sql.DB, tableName string) ([]string, error) {
	rows, err := db.Query(fmt.Sprintf(`PRAGMA table_info("%s")`, tableName))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []string
	for rows.Next() {
		var cid int
		var name, ctype string
		var notnull, pk int
		var dflt any
		if err := rows.Scan(&cid, &name, &ctype, &notnull, &dflt, &pk); err != nil {
			return nil, err
		}
		out = append(out, name)
	}
	return out, rows.Err()
}

func toSQLArg(v any) any {
	switch t := v.(type) {
	case nil:
		return nil
	case bool:
		if t {
			return int64(1)
		}
		return int64(0)
	case float64:
		if t == float64(int64(t)) {
			return int64(t)
		}
		return t
	case string:
		s := strings.TrimSpace(t)
		if i, err := strconv.ParseInt(s, 10, 64); err == nil {
			return i
		}
		if f, err := strconv.ParseFloat(s, 64); err == nil && strings.Contains(s, ".") && !strings.ContainsAny(s, "eE") {
			return f
		}
		return t
	case int:
		return int64(t)
	case int64:
		return t
	default:
		return fmt.Sprint(t)
	}
}

func fromSQL(v any) any {
	switch t := v.(type) {
	case nil:
		return nil
	case int64:
		return float64(t) // JSON numbers are float64
	case float64:
		return t
	case bool:
		return t
	case []byte:
		return string(t)
	case string:
		return t
	default:
		return fmt.Sprint(t)
	}
}

// Insert inserts row maps. Returns {"ok":true,"inserted":n}.
func Insert(url, tableName string, rows any) (map[string]any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	colsPresent, err := columnNames(db, tn)
	if err != nil {
		return nil, err
	}
	hasCreated, hasUpdated := false, false
	for _, c := range colsPresent {
		if c == "created_at" {
			hasCreated = true
		}
		if c == "updated_at" {
			hasUpdated = true
		}
	}
	normalized := table.AsRows(rows)
	arr, ok := normalized.([]any)
	if !ok {
		return nil, fmt.Errorf("rows must be a list")
	}
	now := utcNowISO()
	var n int64
	for _, row := range arr {
		obj, ok := row.(map[string]any)
		if !ok {
			return nil, fmt.Errorf("row must be a map")
		}
		var cols, placeholders []string
		var vals []any
		seenCreated, seenUpdated := false, false
		for k, v := range obj {
			if k == "@" || k == "行" || k == "row" {
				continue
			}
			if _, err := ident(k); err != nil {
				return nil, err
			}
			if k == "created_at" {
				seenCreated = true
			}
			if k == "updated_at" {
				seenUpdated = true
			}
			cols = append(cols, `"`+k+`"`)
			placeholders = append(placeholders, "?")
			vals = append(vals, toSQLArg(v))
		}
		if hasCreated && !seenCreated {
			cols = append(cols, `"created_at"`)
			placeholders = append(placeholders, "?")
			vals = append(vals, now)
		}
		if hasUpdated && !seenUpdated {
			cols = append(cols, `"updated_at"`)
			placeholders = append(placeholders, "?")
			vals = append(vals, now)
		}
		if len(cols) == 0 {
			continue
		}
		sqlStmt := fmt.Sprintf(
			`INSERT INTO "%s" (%s) VALUES (%s)`,
			tn, strings.Join(cols, ", "), strings.Join(placeholders, ", "),
		)
		if _, err := db.Exec(sqlStmt, vals...); err != nil {
			return nil, err
		}
		n++
	}
	return map[string]any{"ok": true, "inserted": float64(n)}, nil
}

func skipWhereKey(k string) bool {
	return k == "@" || k == "行" || k == "row" || strings.HasPrefix(k, "_")
}

func whereOp(raw string) (string, error) {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "", "=", "eq", "==", "等于":
		return "=", nil
	case "!=", "<>", "ne", "不等于":
		return "!=", nil
	case ">", "gt", "大于":
		return ">", nil
	case ">=", "gte", "大于等于":
		return ">=", nil
	case "<", "lt", "小于":
		return "<", nil
	case "<=", "lte", "小于等于":
		return "<=", nil
	case "like", "contains", "包含", "匹配":
		return "LIKE", nil
	case "in", "在":
		return "IN", nil
	case "between", "介于", "在之间":
		return "BETWEEN", nil
	case "is null", "isnull", "为空", "空":
		return "IS NULL", nil
	default:
		return "", fmt.Errorf("unsupported where op `%s`", raw)
	}
}

type whereClause struct {
	expr string
	vals []any
	or   bool
}

func inValues(v any) []any {
	switch t := v.(type) {
	case []any:
		out := make([]any, len(t))
		for i, x := range t {
			out[i] = toSQLArg(x)
		}
		return out
	case string:
		parts := strings.Split(t, ",")
		out := make([]any, 0, len(parts))
		for _, p := range parts {
			out = append(out, toSQLArg(strings.TrimSpace(p)))
		}
		return out
	default:
		return []any{toSQLArg(v)}
	}
}

func betweenValues(v any) ([]any, error) {
	switch t := v.(type) {
	case []any:
		if len(t) != 2 {
			return nil, fmt.Errorf(`between expects two bounds ("lo,hi" or [lo,hi])`)
		}
		return []any{toSQLArg(t[0]), toSQLArg(t[1])}, nil
	case string:
		parts := strings.SplitN(t, ",", 2)
		lo, hi := "", ""
		if len(parts) > 0 {
			lo = strings.TrimSpace(parts[0])
		}
		if len(parts) > 1 {
			hi = strings.TrimSpace(parts[1])
		}
		return []any{toSQLArg(lo), toSQLArg(hi)}, nil
	default:
		return nil, fmt.Errorf(`between expects two bounds ("lo,hi" or [lo,hi])`)
	}
}

func boolOrFlag(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case string:
		switch strings.ToLower(strings.TrimSpace(t)) {
		case "1", "true", "是", "or", "或者":
			return true
		}
	case float64:
		return int64(t) != 0
	}
	return false
}

func clauseFromRow(m map[string]any) (*whereClause, error) {
	field := ""
	for _, k := range []string{"field", "字段", "列"} {
		if v, ok := m[k]; ok {
			field = cellStr(v)
			break
		}
	}
	if field == "" {
		return nil, nil
	}
	col, err := ident(field)
	if err != nil {
		return nil, err
	}
	opRaw := ""
	if v, ok := m["op"]; ok {
		opRaw = cellStr(v)
	} else if v, ok := m["操作"]; ok {
		opRaw = cellStr(v)
	}
	op, err := whereOp(opRaw)
	if err != nil {
		return nil, err
	}
	var val any
	if v, ok := m["value"]; ok {
		val = v
	} else if v, ok := m["值"]; ok {
		val = v
	}
	or := false
	if v, ok := m["or"]; ok {
		or = boolOrFlag(v)
	} else if v, ok := m["或"]; ok {
		or = boolOrFlag(v)
	}
	var expr string
	var vals []any
	switch op {
	case "IS NULL":
		expr = fmt.Sprintf(`"%s" IS NULL`, col)
	case "IN":
		vs := inValues(val)
		ph := make([]string, len(vs))
		for i := range vs {
			ph[i] = "?"
		}
		expr = fmt.Sprintf(`"%s" IN (%s)`, col, strings.Join(ph, ", "))
		vals = vs
	case "BETWEEN":
		vs, err := betweenValues(val)
		if err != nil {
			return nil, err
		}
		expr = fmt.Sprintf(`"%s" BETWEEN ? AND ?`, col)
		vals = vs
	default:
		expr = fmt.Sprintf(`"%s" %s ?`, col, op)
		vals = []any{toSQLArg(val)}
	}
	return &whereClause{expr: expr, vals: vals, or: or}, nil
}

// parseWhere builds AND/OR filter clauses.
// Supports:
//   - equality map `{title:"Alpha", body:"first"}` → col = ?
//   - filter rows `{field|字段|列, op|操作, value|值, or|或?}`
//   - ops: = != > >= < <= LIKE IN BETWEEN IS NULL (+ Chinese aliases)
//   - GFM columnar tables via table.AsRows
func parseWhere(where any) (exprs []string, vals []any, err error) {
	if where == nil {
		return nil, nil, nil
	}
	if s, ok := where.(string); ok {
		if s == "" || strings.EqualFold(s, "none") || strings.EqualFold(s, "null") {
			return nil, nil, nil
		}
		// Non-empty bare strings: AsRows yields []; treat as no filter (Rust parity).
	}
	w := table.AsRows(where)
	arr, ok := w.([]any)
	if !ok {
		return nil, nil, fmt.Errorf("where must be a map or filter table")
	}
	var clauses []whereClause
	for _, item := range arr {
		m, ok := item.(map[string]any)
		if !ok {
			return nil, nil, fmt.Errorf("where must be a map or filter table")
		}
		field := ""
		for _, k := range []string{"field", "字段", "列"} {
			if v, ok := m[k]; ok {
				field = cellStr(v)
				break
			}
		}
		_, hasVal := m["value"]
		if !hasVal {
			_, hasVal = m["值"]
		}
		_, hasOp := m["op"]
		if !hasOp {
			_, hasOp = m["操作"]
		}
		if field != "" && (hasVal || hasOp) {
			cl, err := clauseFromRow(m)
			if err != nil {
				return nil, nil, err
			}
			if cl != nil {
				clauses = append(clauses, *cl)
			}
			continue
		}
		for k, v := range m {
			if skipWhereKey(k) {
				continue
			}
			col, err := ident(k)
			if err != nil {
				return nil, nil, err
			}
			clauses = append(clauses, whereClause{
				expr: fmt.Sprintf(`"%s" = ?`, col),
				vals: []any{toSQLArg(v)},
			})
		}
	}
	// Assemble: AND by default; OR-marked clauses join the current group.
	var group []string
	for _, cl := range clauses {
		if cl.or {
			group = append(group, cl.expr)
		} else {
			if len(group) > 0 {
				exprs = append(exprs, "("+strings.Join(group, " OR ")+")")
				group = group[:0]
			}
			group = append(group, cl.expr)
		}
		vals = append(vals, cl.vals...)
	}
	if len(group) > 0 {
		exprs = append(exprs, "("+strings.Join(group, " OR ")+")")
	}
	return exprs, vals, nil
}

func scanRows(rows *sql.Rows) ([]any, error) {
	cols, err := rows.Columns()
	if err != nil {
		return nil, err
	}
	var out []any
	for rows.Next() {
		raw := make([]any, len(cols))
		ptrs := make([]any, len(cols))
		for i := range raw {
			ptrs[i] = &raw[i]
		}
		if err := rows.Scan(ptrs...); err != nil {
			return nil, err
		}
		m := map[string]any{}
		for i, name := range cols {
			m[name] = fromSQL(raw[i])
		}
		out = append(out, m)
	}
	return out, rows.Err()
}

// SelectOpts controls select_order behavior.
type SelectOpts struct {
	Where  any
	Order  string
	Offset *int64
}

// Select runs SELECT * FROM table with optional where/order/offset.
// Without offset: {"rows":[…]}; with offset: {"rows":[…],"total":N}.
func Select(url, tableName string, limit int64, opts SelectOpts) (map[string]any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	exprs, vals, err := parseWhere(opts.Where)
	if err != nil {
		return nil, err
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	sqlStmt := fmt.Sprintf(`SELECT * FROM "%s"`, tn)
	whereSQL := ""
	if len(exprs) > 0 {
		whereSQL = " WHERE " + strings.Join(exprs, " AND ")
		sqlStmt += whereSQL
	}
	if order := strings.TrimSpace(opts.Order); order != "" {
		parts := strings.Split(order, ",")
		var orderExprs []string
		for _, part := range parts {
			part = strings.TrimSpace(part)
			if part == "" {
				continue
			}
			dir := ""
			col := part
			if strings.HasPrefix(part, "-") {
				col = part[1:]
				dir = " DESC"
			}
			c, err := ident(col)
			if err != nil {
				return nil, err
			}
			orderExprs = append(orderExprs, fmt.Sprintf(`"%s"%s`, c, dir))
		}
		if len(orderExprs) > 0 {
			sqlStmt += " ORDER BY " + strings.Join(orderExprs, ", ")
		}
	}
	sqlStmt += " LIMIT ?"
	vals = append(vals, limit)
	if opts.Offset != nil && *opts.Offset > 0 {
		sqlStmt += " OFFSET ?"
		vals = append(vals, *opts.Offset)
	}
	rows, err := db.Query(sqlStmt, vals...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	list, err := scanRows(rows)
	if err != nil {
		return nil, err
	}
	if list == nil {
		list = []any{}
	}
	if opts.Offset != nil {
		countSQL := fmt.Sprintf(`SELECT COUNT(*) FROM "%s"%s`, tn, whereSQL)
		_, whereOnly, _ := parseWhere(opts.Where)
		var total int64
		if err := db.QueryRow(countSQL, whereOnly...).Scan(&total); err != nil {
			total = int64(len(list))
		}
		return map[string]any{"rows": list, "total": float64(total)}, nil
	}
	return map[string]any{"rows": list}, nil
}

// Get returns one row by id, or nil (JSON null).
func Get(url, tableName, id string) (any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	sqlStmt := fmt.Sprintf(`SELECT * FROM "%s" WHERE "id" = ? LIMIT 1`, tn)
	rows, err := db.Query(sqlStmt, toSQLArg(id))
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	list, err := scanRows(rows)
	if err != nil {
		return nil, err
	}
	if len(list) == 0 {
		return nil, nil
	}
	return list[0], nil
}

func sqlArgs(args any) []any {
	switch t := args.(type) {
	case []any:
		out := make([]any, len(t))
		for i, v := range t {
			out[i] = toSQLArg(v)
		}
		return out
	case nil:
		return nil
	default:
		return []any{toSQLArg(t)}
	}
}

// Exec runs a non-SELECT statement. Returns {"ok":true,"changes":N}.
func Exec(url, sqlStmt string, args any) (map[string]any, error) {
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	vals := sqlArgs(args)
	res, err := db.Exec(sqlStmt, vals...)
	if err != nil {
		return nil, err
	}
	n, _ := res.RowsAffected()
	return map[string]any{"ok": true, "changes": float64(n)}, nil
}

// Update sets columns from row (skips id). Auto-sets updated_at when present.
// Returns {"ok":true,"updated":N}.
func Update(url, tableName, id string, row any) (map[string]any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	obj, ok := row.(map[string]any)
	if !ok {
		return nil, fmt.Errorf("row must be a map")
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	colsPresent, err := columnNames(db, tn)
	if err != nil {
		return nil, err
	}
	hasUpdated := false
	for _, c := range colsPresent {
		if c == "updated_at" {
			hasUpdated = true
			break
		}
	}
	var sets []string
	var vals []any
	seenUpdated := false
	for k, v := range obj {
		if k == "id" {
			continue
		}
		if _, err := ident(k); err != nil {
			return nil, err
		}
		if k == "updated_at" {
			seenUpdated = true
		}
		sets = append(sets, fmt.Sprintf(`"%s" = ?`, k))
		vals = append(vals, toSQLArg(v))
	}
	if hasUpdated && !seenUpdated {
		sets = append(sets, `"updated_at" = ?`)
		vals = append(vals, utcNowISO())
	}
	if len(sets) == 0 {
		return nil, fmt.Errorf("nothing to update")
	}
	vals = append(vals, id) // Rust binds id as text; SQLite coerces
	sqlStmt := fmt.Sprintf(`UPDATE "%s" SET %s WHERE "id" = ?`, tn, strings.Join(sets, ", "))
	res, err := db.Exec(sqlStmt, vals...)
	if err != nil {
		return nil, err
	}
	n, _ := res.RowsAffected()
	return map[string]any{"ok": true, "updated": float64(n)}, nil
}

// Delete removes a row by id. Returns {"ok":true,"deleted":N}.
func Delete(url, tableName, id string) (map[string]any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	sqlStmt := fmt.Sprintf(`DELETE FROM "%s" WHERE "id" = ?`, tn)
	res, err := db.Exec(sqlStmt, id)
	if err != nil {
		return nil, err
	}
	n, _ := res.RowsAffected()
	return map[string]any{"ok": true, "deleted": float64(n)}, nil
}

// Query runs a SELECT (or any result-set SQL). Returns {"rows":[…],"count":N}.
func Query(url, sqlStmt string, args any) (map[string]any, error) {
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	vals := sqlArgs(args)
	rows, err := db.Query(sqlStmt, vals...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	list, err := scanRows(rows)
	if err != nil {
		return nil, err
	}
	if list == nil {
		list = []any{}
	}
	return map[string]any{"rows": list, "count": float64(len(list))}, nil
}

// Count returns {"count":N} for rows matching where (nil where → all).
func Count(url, tableName string, where any) (map[string]any, error) {
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	exprs, vals, err := parseWhere(where)
	if err != nil {
		return nil, err
	}
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	sqlStmt := fmt.Sprintf(`SELECT COUNT(*) FROM "%s"`, tn)
	if len(exprs) > 0 {
		sqlStmt += " WHERE " + strings.Join(exprs, " AND ")
	}
	var n int64
	if err := db.QueryRow(sqlStmt, vals...).Scan(&n); err != nil {
		return nil, err
	}
	return map[string]any{"count": float64(n)}, nil
}
