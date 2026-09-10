package db

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"sync"
	"sync/atomic"
	"time"
)

// dbConn is *sql.DB or a transaction-bound *sql.Conn.
type dbConn interface {
	Exec(query string, args ...any) (sql.Result, error)
	Query(query string, args ...any) (*sql.Rows, error)
	QueryRow(query string, args ...any) *sql.Row
}

type txnConn struct {
	conn *sql.Conn
	url  string
}

func (t *txnConn) Exec(query string, args ...any) (sql.Result, error) {
	return t.conn.ExecContext(context.Background(), query, args...)
}

func (t *txnConn) Query(query string, args ...any) (*sql.Rows, error) {
	return t.conn.QueryContext(context.Background(), query, args...)
}

func (t *txnConn) QueryRow(query string, args ...any) *sql.Row {
	return t.conn.QueryRowContext(context.Background(), query, args...)
}

var (
	txnsMu   sync.Mutex
	txns     = map[string]*txnConn{}
	txnSeq   atomic.Uint64
	txnSeedM sync.Mutex
	txnSeed  = uint64(0x243f_6a88_85a3_08d3)
)

func nextTxnID() string {
	nanos := time.Now().UnixNano() & 0xffffffff
	pid := os.Getpid()
	txnSeedM.Lock()
	x := txnSeed
	x ^= x << 13
	x ^= x >> 7
	x ^= x << 17
	txnSeed = x
	txnSeedM.Unlock()
	n := txnSeq.Add(1)
	return fmt.Sprintf("txn-%x%x%x%x", pid, nanos, x, n)
}

func optTxn(txnID []string) string {
	if len(txnID) == 0 {
		return ""
	}
	return txnID[0]
}

// connFor returns the pooled DB or an active transaction connection.
func connFor(url, txnID string) (dbConn, error) {
	if txnID != "" {
		txnsMu.Lock()
		defer txnsMu.Unlock()
		t, ok := txns[txnID]
		if !ok {
			return nil, fmt.Errorf("unknown transaction `%s`", txnID)
		}
		return t, nil
	}
	return open(url)
}

func takeTxn(txnID string) (*txnConn, error) {
	txnsMu.Lock()
	defer txnsMu.Unlock()
	t, ok := txns[txnID]
	if !ok {
		return nil, fmt.Errorf("unknown transaction `%s`", txnID)
	}
	delete(txns, txnID)
	return t, nil
}

func resetTxns() {
	txnsMu.Lock()
	defer txnsMu.Unlock()
	for id, t := range txns {
		_, _ = t.conn.ExecContext(context.Background(), "ROLLBACK")
		_ = t.conn.Close()
		delete(txns, id)
	}
}

// Begin starts a SQLite transaction (BEGIN IMMEDIATE).
// Returns {"_type":"txn","txn":id,"url":url,"事务":id,"地址":url}.
func Begin(url string) (map[string]any, error) {
	db, err := open(url)
	if err != nil {
		return nil, err
	}
	conn, err := db.Conn(context.Background())
	if err != nil {
		return nil, err
	}
	if _, err := conn.ExecContext(context.Background(), "BEGIN IMMEDIATE"); err != nil {
		_ = conn.Close()
		return nil, err
	}
	id := nextTxnID()
	resolved := ResolveURL(url)
	txnsMu.Lock()
	txns[id] = &txnConn{conn: conn, url: resolved}
	txnsMu.Unlock()
	return map[string]any{
		"_type": "txn",
		"txn":   id,
		"url":   resolved,
		"事务":    id,
		"地址":    resolved,
	}, nil
}

// Commit commits a transaction. Returns {"ok":true}.
func Commit(txnID string) (map[string]any, error) {
	t, err := takeTxn(txnID)
	if err != nil {
		return nil, err
	}
	defer t.conn.Close()
	if _, err := t.conn.ExecContext(context.Background(), "COMMIT"); err != nil {
		return nil, err
	}
	return map[string]any{"ok": true}, nil
}

// Rollback rolls back a transaction. Returns {"ok":true}.
func Rollback(txnID string) (map[string]any, error) {
	t, err := takeTxn(txnID)
	if err != nil {
		return nil, err
	}
	defer t.conn.Close()
	if _, err := t.conn.ExecContext(context.Background(), "ROLLBACK"); err != nil {
		return nil, err
	}
	return map[string]any{"ok": true}, nil
}
