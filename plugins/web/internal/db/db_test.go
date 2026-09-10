package db_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
)

func setupArticles(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, "t.db")
	url := "sqlite:" + path
	t.Cleanup(db.ResetPool)

	fields := []any{
		map[string]any{"name": "id", "type": "integer", "nullable": false},
		map[string]any{"name": "title", "type": "text", "nullable": false},
		map[string]any{"name": "body", "type": "text", "nullable": true},
		map[string]any{"name": "updated_at", "type": "text", "nullable": true},
	}
	if _, err := db.Init(url, "articles", fields); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Insert(url, "articles", []any{
		map[string]any{"id": float64(1), "title": "Alpha", "body": "first"},
		map[string]any{"id": float64(2), "title": "Beta", "body": "second"},
		map[string]any{"id": float64(3), "title": "Gamma", "body": "third"},
	}); err != nil {
		t.Fatal(err)
	}
	return url
}

func TestInitInsertSelectGet(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "t.db")
	url := "sqlite:" + path
	defer db.ResetPool()

	fields := []any{
		map[string]any{"name": "id", "type": "integer", "nullable": false},
		map[string]any{"name": "title", "type": "text", "nullable": false},
		map[string]any{"name": "body", "type": "text", "nullable": true},
	}
	initOut, err := db.Init(url, "articles", fields)
	if err != nil {
		t.Fatal(err)
	}
	if initOut["_type"] != "db_table" || initOut["name"] != "articles" {
		t.Fatalf("init=%v", initOut)
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatal(err)
	}

	ins, err := db.Insert(url, "articles", []any{
		map[string]any{"id": float64(1), "title": "Hello Marqdo", "body": "First"},
		map[string]any{"id": float64(2), "title": "Two", "body": "Second"},
	})
	if err != nil {
		t.Fatal(err)
	}
	if ins["ok"] != true || ins["inserted"] != float64(2) {
		t.Fatalf("insert=%v", ins)
	}

	sel, err := db.Select(url, "articles", 10, db.SelectOpts{})
	if err != nil {
		t.Fatal(err)
	}
	rows, ok := sel["rows"].([]any)
	if !ok || len(rows) != 2 {
		t.Fatalf("select=%v", sel)
	}
	if _, hasTotal := sel["total"]; hasTotal {
		t.Fatalf("unexpected total without offset: %v", sel)
	}

	got, err := db.Get(url, "articles", "1")
	if err != nil {
		t.Fatal(err)
	}
	m, ok := got.(map[string]any)
	if !ok {
		t.Fatalf("get=%v", got)
	}
	if m["title"] != "Hello Marqdo" {
		t.Fatalf("title=%v", m["title"])
	}

	miss, err := db.Get(url, "articles", "999")
	if err != nil {
		t.Fatal(err)
	}
	if miss != nil {
		t.Fatalf("expected null, got %v", miss)
	}
}

func TestUpdateDelete(t *testing.T) {
	url := setupArticles(t)

	up, err := db.Update(url, "articles", "1", map[string]any{"title": "Alpha2", "body": "updated"})
	if err != nil {
		t.Fatal(err)
	}
	if up["ok"] != true || up["updated"] != float64(1) {
		t.Fatalf("update=%v", up)
	}
	got, err := db.Get(url, "articles", "1")
	if err != nil {
		t.Fatal(err)
	}
	m := got.(map[string]any)
	if m["title"] != "Alpha2" {
		t.Fatalf("title=%v", m["title"])
	}
	if ua, _ := m["updated_at"].(string); ua == "" {
		t.Fatalf("expected auto updated_at, got %v", m["updated_at"])
	}

	del, err := db.Delete(url, "articles", "2")
	if err != nil {
		t.Fatal(err)
	}
	if del["ok"] != true || del["deleted"] != float64(1) {
		t.Fatalf("delete=%v", del)
	}
	miss, err := db.Get(url, "articles", "2")
	if err != nil {
		t.Fatal(err)
	}
	if miss != nil {
		t.Fatalf("expected deleted, got %v", miss)
	}
}

func TestQueryAndCount(t *testing.T) {
	url := setupArticles(t)

	q, err := db.Query(url, `SELECT title FROM "articles" WHERE "id" = ?`, []any{"1"})
	if err != nil {
		t.Fatal(err)
	}
	if q["count"] != float64(1) {
		t.Fatalf("query=%v", q)
	}
	rows := q["rows"].([]any)
	if rows[0].(map[string]any)["title"] != "Alpha" {
		t.Fatalf("rows=%v", rows)
	}

	all, err := db.Count(url, "articles", nil)
	if err != nil {
		t.Fatal(err)
	}
	if all["count"] != float64(3) {
		t.Fatalf("count all=%v", all)
	}

	eq, err := db.Count(url, "articles", map[string]any{"title": "Alpha", "body": "first"})
	if err != nil {
		t.Fatal(err)
	}
	if eq["count"] != float64(1) {
		t.Fatalf("equality where count=%v", eq)
	}

	filt, err := db.Count(url, "articles", []any{
		map[string]any{"字段": "title", "操作": "=", "值": "Beta"},
	})
	if err != nil {
		t.Fatal(err)
	}
	if filt["count"] != float64(1) {
		t.Fatalf("filter table count=%v", filt)
	}

	like, err := db.Select(url, "articles", 10, db.SelectOpts{
		Where: []any{
			map[string]any{"field": "title", "op": "like", "value": "A%"},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(like["rows"].([]any)) != 1 {
		t.Fatalf("like=%v", like)
	}
}

func TestTxnCommitRollback(t *testing.T) {
	url := setupArticles(t)

	begin, err := db.Begin(url)
	if err != nil {
		t.Fatal(err)
	}
	if begin["_type"] != "txn" {
		t.Fatalf("begin=%v", begin)
	}
	tid, _ := begin["txn"].(string)
	if tid == "" {
		t.Fatalf("missing txn id: %v", begin)
	}
	if begin["事务"] != tid || begin["地址"] != begin["url"] {
		t.Fatalf("bilingual keys: %v", begin)
	}

	if _, err := db.Insert(url, "articles", []any{
		map[string]any{"id": float64(99), "title": "TxnRow", "body": "in-txn"},
	}, tid); err != nil {
		t.Fatal(err)
	}
	// Visible inside the transaction.
	inside, err := db.Count(url, "articles", nil, tid)
	if err != nil {
		t.Fatal(err)
	}
	if inside["count"] != float64(4) {
		t.Fatalf("in-txn count=%v", inside)
	}
	if _, err := db.Commit(tid); err != nil {
		t.Fatal(err)
	}
	after, err := db.Count(url, "articles", nil)
	if err != nil {
		t.Fatal(err)
	}
	if after["count"] != float64(4) {
		t.Fatalf("after commit count=%v", after)
	}

	begin2, err := db.Begin(url)
	if err != nil {
		t.Fatal(err)
	}
	tid2 := begin2["txn"].(string)
	if _, err := db.Insert(url, "articles", []any{
		map[string]any{"id": float64(100), "title": "Zeta", "body": "rolled"},
	}, tid2); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Rollback(tid2); err != nil {
		t.Fatal(err)
	}
	final, err := db.Count(url, "articles", nil)
	if err != nil {
		t.Fatal(err)
	}
	if final["count"] != float64(4) {
		t.Fatalf("after rollback count=%v", final)
	}
	miss, err := db.Get(url, "articles", "100")
	if err != nil {
		t.Fatal(err)
	}
	if miss != nil {
		t.Fatalf("expected rolled-back row absent, got %v", miss)
	}
}

func TestMigrate(t *testing.T) {
	dir := t.TempDir()
	url := "sqlite:" + filepath.Join(dir, "m.db")
	defer db.ResetPool()

	steps := []any{
		map[string]any{
			"版本": float64(1),
			"SQL":  `CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT, status TEXT NOT NULL DEFAULT 'draft')`,
		},
		map[string]any{
			"版本": float64(2),
			"SQL":  `CREATE TABLE IF NOT EXISTS comments (id INTEGER PRIMARY KEY AUTOINCREMENT, post_id INTEGER NOT NULL, body TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending')`,
		},
	}
	m1, err := db.Migrate(url, steps)
	if err != nil {
		t.Fatal(err)
	}
	if m1["ok"] != true || m1["from"] != float64(0) || m1["to"] != float64(2) {
		t.Fatalf("migrate1=%v", m1)
	}
	applied, ok := m1["applied"].([]any)
	if !ok || len(applied) != 2 {
		t.Fatalf("applied=%v", m1["applied"])
	}

	m2, err := db.Migrate(url, steps)
	if err != nil {
		t.Fatal(err)
	}
	applied2 := m2["applied"].([]any)
	if len(applied2) != 0 || m2["from"] != float64(2) || m2["to"] != float64(2) {
		t.Fatalf("idempotent migrate=%v", m2)
	}
}

func TestFtsCreateSearch(t *testing.T) {
	dir := t.TempDir()
	url := "sqlite:" + filepath.Join(dir, "fts.db")
	defer db.ResetPool()

	steps := []any{
		map[string]any{
			"version": float64(1),
			"sql":     `CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT, status TEXT NOT NULL DEFAULT 'draft')`,
		},
	}
	if _, err := db.Migrate(url, steps); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Insert(url, "posts", []any{
		map[string]any{"title": "Alpha note", "body": "hello world search", "status": "published"},
		map[string]any{"title": "Beta draft", "body": "secret draft body", "status": "draft"},
		map[string]any{"title": "Gamma published", "body": "another hello", "status": "published"},
	}); err != nil {
		t.Fatal(err)
	}

	fts, err := db.FtsCreate(url, "posts", "title,body", "")
	if err != nil {
		t.Fatal(err)
	}
	if fts["ok"] != true || fts["fts"] != "posts_fts" {
		t.Fatalf("fts=%v", fts)
	}

	hit, err := db.Search(url, "posts", "hello", 10, "")
	if err != nil {
		t.Fatal(err)
	}
	n, _ := hit["count"].(float64)
	if n < 1 {
		t.Fatalf("search=%v", hit)
	}
	rows := hit["rows"].([]any)
	first := rows[0].(map[string]any)
	if cell, _ := first["title"].(string); cell == "" {
		t.Fatalf("expected title on FTS row: %v", first)
	}
}
