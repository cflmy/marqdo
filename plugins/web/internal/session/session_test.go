package session_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func TestSessionSetGetDel(t *testing.T) {
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)
	id := session.NewID(0)
	if id == "" {
		t.Fatal("empty id")
	}
	if !session.SetValue(id, "username", "admin") {
		t.Fatal("set")
	}
	v, ok := session.GetValue(id, "username")
	if !ok || v != "admin" {
		t.Fatalf("get=%v ok=%v", v, ok)
	}
	got := session.Get(id, "username")
	if got["ok"] != true || got["value"] != "admin" {
		t.Fatalf("Get ABI: %v", got)
	}
	if !session.DelKey(id, "username") {
		t.Fatal("del")
	}
	if _, ok := session.GetValue(id, "username"); ok {
		t.Fatal("del leftover")
	}
	if !session.DestroyID(id) {
		t.Fatal("destroy")
	}
}

func TestCSRFAndCookie(t *testing.T) {
	session.Reset(3600)
	id := session.NewID(0)
	tok, ok := session.CSRFFor(id)
	if !ok || tok == "" {
		t.Fatal("csrf")
	}
	if !session.ValidateCSRF(id, tok) {
		t.Fatal("validate")
	}
	if session.ValidateCSRF(id, "bad") {
		t.Fatal("bad csrf")
	}
	cookie := session.SessionCookie(id, 3600, false)
	got, ok := session.IDFromCookie("theme=light; " + cookie)
	if !ok || got != id {
		t.Fatalf("cookie parse %q != %q", got, id)
	}
}

func TestSessionPersistsInSQLite(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "persist.db")
	url := "sqlite:" + path
	db.ResetPool()
	session.Configure(session.Config{DBURL: url, TTLSec: 3600})
	id := session.NewID(0)
	if !session.SetValue(id, "theme", "dark") {
		t.Fatal("set")
	}
	session.Configure(session.Config{DBURL: url, TTLSec: 3600})
	v, ok := session.GetValue(id, "theme")
	if !ok || v != "dark" {
		t.Fatalf("persist get=%v ok=%v", v, ok)
	}
	_ = os.RemoveAll(dir)
}

func TestNewABI(t *testing.T) {
	session.Reset(120)
	out := session.New(120)
	id, _ := out["id"].(string)
	if id == "" {
		t.Fatal(out)
	}
}
