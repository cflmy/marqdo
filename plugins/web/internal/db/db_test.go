package db_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
)

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
