package rbac_test

import (
	"path/filepath"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/rbac"
)

func TestHasAnyAndExpandRole(t *testing.T) {
	if !rbac.HasAny([]string{"desk:access", "comments:create"}, []string{"desk:access"}) {
		t.Fatal("expected desk:access held")
	}
	if rbac.HasAny([]string{"comments:create"}, []string{"desk:access"}) {
		t.Fatal("member should not have desk")
	}
	if !rbac.HasAny([]string{"*"}, []string{"roles:manage"}) {
		t.Fatal("star should allow all")
	}
	admin := rbac.ExpandRole("admin")
	if !rbac.HasAny(admin, []string{"roles:manage", "desk:access"}) {
		t.Fatalf("admin expand incomplete: %v", admin)
	}
	member := rbac.ExpandRole("member")
	if rbac.HasAny(member, []string{"desk:access"}) {
		t.Fatal("member must not get desk:access")
	}
}

func TestEnsureSchemaAndAssign(t *testing.T) {
	dir := t.TempDir()
	url := "sqlite:" + filepath.Join(dir, "rbac.db")
	if err := rbac.EnsureSchema(url); err != nil {
		t.Fatal(err)
	}
	sqlDB, err := db.OpenSQL(url)
	if err != nil {
		t.Fatal(err)
	}
	_, err = sqlDB.Exec(`INSERT INTO web_users(username, password_hash, created_at) VALUES('alice','x','now')`)
	if err != nil {
		t.Fatal(err)
	}
	if err := rbac.AssignRole(url, "alice", "member"); err != nil {
		t.Fatal(err)
	}
	perms, err := rbac.PermissionsForUser(url, "alice")
	if err != nil {
		t.Fatal(err)
	}
	if !rbac.HasAny(perms, []string{"comments:create"}) {
		t.Fatalf("alice perms=%v", perms)
	}
	if rbac.HasAny(perms, []string{"roles:manage"}) {
		t.Fatal("member should not manage roles")
	}
}
