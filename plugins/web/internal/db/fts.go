package db

import (
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

func parseColumnList(columns any) ([]string, error) {
	switch t := columns.(type) {
	case string:
		parts := strings.Split(t, ",")
		var cols []string
		for _, p := range parts {
			p = strings.TrimSpace(p)
			if p == "" {
				continue
			}
			if _, err := ident(p); err != nil {
				return nil, err
			}
			cols = append(cols, p)
		}
		if len(cols) == 0 {
			return nil, fmt.Errorf("fts columns is empty")
		}
		return cols, nil
	case []any:
		var cols []string
		for _, v := range t {
			name := cellStr(v)
			if _, err := ident(name); err != nil {
				return nil, err
			}
			cols = append(cols, name)
		}
		if len(cols) == 0 {
			return nil, fmt.Errorf("fts columns is empty")
		}
		return cols, nil
	case map[string]any:
		rows := table.AsRows(columns)
		arr, ok := rows.([]any)
		if !ok {
			return nil, fmt.Errorf("fts columns must be a list or CSV string")
		}
		var cols []string
		for _, row := range arr {
			if obj, ok := row.(map[string]any); ok {
				name := ""
				for _, k := range []string{"name", "字段", "column"} {
					if v, ok := obj[k]; ok {
						name = cellStr(v)
						break
					}
				}
				if name == "" {
					for _, v := range obj {
						name = cellStr(v)
						break
					}
				}
				if name != "" {
					if _, err := ident(name); err != nil {
						return nil, err
					}
					cols = append(cols, name)
				}
			} else {
				name := cellStr(row)
				if name != "" {
					if _, err := ident(name); err != nil {
						return nil, err
					}
					cols = append(cols, name)
				}
			}
		}
		if len(cols) == 0 {
			return nil, fmt.Errorf("fts columns is empty")
		}
		return cols, nil
	case nil:
		return nil, fmt.Errorf("fts columns is empty")
	default:
		return nil, fmt.Errorf("fts columns must be a list or CSV string")
	}
}

func ftsName(tableName string, name string) (string, error) {
	name = strings.TrimSpace(name)
	if name == "" {
		return tableName + "_fts", nil
	}
	return ident(name)
}

// FtsCreate builds an FTS5 virtual table synced via content= + triggers.
// Returns {"ok":true,"table":…,"fts":…,"columns":[…]}.
func FtsCreate(url, tableName string, columns any, name string) (map[string]any, error) {
	lower := strings.ToLower(ResolveURL(url))
	if strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://") {
		return nil, fmt.Errorf("db.fts is SQLite-only in this wave")
	}
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	cols, err := parseColumnList(columns)
	if err != nil {
		return nil, err
	}
	fts, err := ftsName(tn, name)
	if err != nil {
		return nil, err
	}
	quoted := make([]string, len(cols))
	for i, c := range cols {
		quoted[i] = `"` + c + `"`
	}
	colSQL := strings.Join(quoted, ", ")

	db, err := open(url)
	if err != nil {
		return nil, err
	}
	create := fmt.Sprintf(
		`CREATE VIRTUAL TABLE IF NOT EXISTS "%s" USING fts5(%s, content="%s", content_rowid="id")`,
		fts, colSQL, tn,
	)
	if _, err := db.Exec(create); err != nil {
		return nil, err
	}

	ai, ad, au := fts+"_ai", fts+"_ad", fts+"_au"
	for _, trg := range []string{ai, ad, au} {
		if _, err := db.Exec(fmt.Sprintf(`DROP TRIGGER IF EXISTS "%s"`, trg)); err != nil {
			return nil, err
		}
	}

	newCols := make([]string, len(cols))
	oldCols := make([]string, len(cols))
	for i, c := range cols {
		newCols[i] = `new."` + c + `"`
		oldCols[i] = `old."` + c + `"`
	}
	newSQL := strings.Join(newCols, ", ")
	oldSQL := strings.Join(oldCols, ", ")

	aiSQL := fmt.Sprintf(
		`CREATE TRIGGER "%s" AFTER INSERT ON "%s" BEGIN
			INSERT INTO "%s"(rowid, %s) VALUES (new.id, %s);
		END`,
		ai, tn, fts, colSQL, newSQL,
	)
	adSQL := fmt.Sprintf(
		`CREATE TRIGGER "%s" AFTER DELETE ON "%s" BEGIN
			INSERT INTO "%s"("%s", rowid, %s) VALUES('delete', old.id, %s);
		END`,
		ad, tn, fts, fts, colSQL, oldSQL,
	)
	auSQL := fmt.Sprintf(
		`CREATE TRIGGER "%s" AFTER UPDATE ON "%s" BEGIN
			INSERT INTO "%s"("%s", rowid, %s) VALUES('delete', old.id, %s);
			INSERT INTO "%s"(rowid, %s) VALUES (new.id, %s);
		END`,
		au, tn, fts, fts, colSQL, oldSQL, fts, colSQL, newSQL,
	)
	for _, stmt := range []string{aiSQL, adSQL, auSQL} {
		if _, err := db.Exec(stmt); err != nil {
			return nil, err
		}
	}
	if _, err := db.Exec(fmt.Sprintf(`INSERT INTO "%s"("%s") VALUES('rebuild')`, fts, fts)); err != nil {
		return nil, err
	}

	colAny := make([]any, len(cols))
	for i, c := range cols {
		colAny[i] = c
	}
	return map[string]any{
		"ok":      true,
		"table":   tn,
		"fts":     fts,
		"columns": colAny,
	}, nil
}

// Search runs FTS5 MATCH against a table created by FtsCreate.
// Returns {"rows":[…],"count":N}.
func Search(url, tableName, q string, limit int64, name string) (map[string]any, error) {
	lower := strings.ToLower(ResolveURL(url))
	if strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://") {
		return nil, fmt.Errorf("db.search is SQLite-only in this wave")
	}
	tn, err := ident(tableName)
	if err != nil {
		return nil, err
	}
	fts, err := ftsName(tn, name)
	if err != nil {
		return nil, err
	}
	q = strings.TrimSpace(q)
	if q == "" {
		return map[string]any{"rows": []any{}, "count": float64(0)}, nil
	}
	if limit <= 0 {
		limit = 20
	} else if limit > 500 {
		limit = 500
	}
	sqlStmt := fmt.Sprintf(
		`SELECT t.*, bm25("%s") AS rank
		 FROM "%s"
		 JOIN "%s" t ON t.id = "%s".rowid
		 WHERE "%s" MATCH ?
		 ORDER BY rank
		 LIMIT ?`,
		fts, fts, tn, fts, fts,
	)
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(sqlStmt, q, limit)
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
