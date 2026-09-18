package db

import (
	"database/sql"
	"strings"
)

// IsPostgres reports whether url selects the Postgres backend.
func IsPostgres(url string) bool {
	lower := strings.ToLower(strings.TrimSpace(url))
	return strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://")
}

// RewritePlaceholdersPG turns `?` into `$1,$2,…` for Postgres.
func RewritePlaceholdersPG(sqlStmt string) string {
	var b strings.Builder
	b.Grow(len(sqlStmt) + 8)
	n := 0
	for _, c := range sqlStmt {
		if c == '?' {
			n++
			b.WriteByte('$')
			b.WriteString(itoa(n))
		} else {
			b.WriteRune(c)
		}
	}
	return b.String()
}

func itoa(n int) string {
	// small positive ints only
	if n < 10 {
		return string(rune('0' + n))
	}
	var buf [12]byte
	i := len(buf)
	for n > 0 {
		i--
		buf[i] = byte('0' + n%10)
		n /= 10
	}
	return string(buf[i:])
}

// rewritingDB rewrites `?` placeholders when talking to Postgres.
type rewritingDB struct {
	db *sql.DB
	pg bool
}

func (d *rewritingDB) Exec(query string, args ...any) (sql.Result, error) {
	if d.pg {
		query = RewritePlaceholdersPG(query)
	}
	return d.db.Exec(query, args...)
}

func (d *rewritingDB) Query(query string, args ...any) (*sql.Rows, error) {
	if d.pg {
		query = RewritePlaceholdersPG(query)
	}
	return d.db.Query(query, args...)
}

func (d *rewritingDB) QueryRow(query string, args ...any) *sql.Row {
	if d.pg {
		query = RewritePlaceholdersPG(query)
	}
	return d.db.QueryRow(query, args...)
}

func (d *rewritingDB) Begin() (*sql.Tx, error) {
	return d.db.Begin()
}

func (d *rewritingDB) Underlying() *sql.DB { return d.db }

type rewritingTx struct {
	tx *sql.Tx
	pg bool
}

func (t *rewritingTx) Exec(query string, args ...any) (sql.Result, error) {
	if t.pg {
		query = RewritePlaceholdersPG(query)
	}
	return t.tx.Exec(query, args...)
}

func (t *rewritingTx) Query(query string, args ...any) (*sql.Rows, error) {
	if t.pg {
		query = RewritePlaceholdersPG(query)
	}
	return t.tx.Query(query, args...)
}

func (t *rewritingTx) QueryRow(query string, args ...any) *sql.Row {
	if t.pg {
		query = RewritePlaceholdersPG(query)
	}
	return t.tx.QueryRow(query, args...)
}

func (t *rewritingTx) Commit() error   { return t.tx.Commit() }
func (t *rewritingTx) Rollback() error { return t.tx.Rollback() }

// SQLDB is *sql.DB with optional Postgres `?` rewrite (RBAC / admin).
type SQLDB interface {
	Exec(query string, args ...any) (sql.Result, error)
	Query(query string, args ...any) (*sql.Rows, error)
	QueryRow(query string, args ...any) *sql.Row
	Begin() (SQLTx, error)
}

// SQLTx is a transaction with optional placeholder rewrite.
type SQLTx interface {
	Exec(query string, args ...any) (sql.Result, error)
	Query(query string, args ...any) (*sql.Rows, error)
	QueryRow(query string, args ...any) *sql.Row
	Commit() error
	Rollback() error
}

type sqlDBHandle struct {
	*rewritingDB
}

func (h *sqlDBHandle) Begin() (SQLTx, error) {
	tx, err := h.rewritingDB.Begin()
	if err != nil {
		return nil, err
	}
	return &rewritingTx{tx: tx, pg: h.pg}, nil
}

// OpenSQLDB returns a pooled handle that rewrites `?` on Postgres.
func OpenSQLDB(url string) (SQLDB, error) {
	raw, err := open(url)
	if err != nil {
		return nil, err
	}
	return &sqlDBHandle{rewritingDB: &rewritingDB{db: raw, pg: IsPostgres(url)}}, nil
}
