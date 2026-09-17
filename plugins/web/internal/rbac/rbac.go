// Package rbac implements configurable role ⊥ permission for ext/web.
package rbac

import (
	"database/sql"
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
)

// Perm is one catalog entry.
type Perm struct {
	Code        string
	Resource    string
	Action      string
	Description string
}

// DefaultCatalog is seeded when EnsureSchema runs.
func DefaultCatalog() []Perm {
	return []Perm{
		{"comments:create", "comments", "create", "Create a comment"},
		{"comments:delete", "comments", "delete", "Delete any comment"},
		{"comments:delete_own", "comments", "delete_own", "Delete own comment"},
		{"desk:access", "desk", "access", "Access writing desk / admin chrome"},
		{"roles:manage", "roles", "manage", "Create roles and assign permissions"},
		{"posts:edit", "posts", "edit", "Create or edit posts"},
		{"users:manage", "users", "manage", "Manage user accounts"},
	}
}

// AllCodes returns every default permission code.
func AllCodes() []string {
	cat := DefaultCatalog()
	out := make([]string, len(cat))
	for i, p := range cat {
		out[i] = p.Code
	}
	return out
}

// ExpandRole maps legacy/system role names to permission codes when DB grants are empty.
func ExpandRole(role string) []string {
	switch strings.ToLower(strings.TrimSpace(role)) {
	case "superadmin", "admin":
		return AllCodes()
	case "member", "user":
		return []string{"comments:create", "comments:delete_own"}
	case "editor", "author":
		return []string{"comments:create", "comments:delete_own", "posts:edit", "desk:access"}
	default:
		return nil
	}
}

// ParseCSV splits a CSV of permission or role tokens.
func ParseCSV(s string) []string {
	var out []string
	for _, part := range strings.Split(s, ",") {
		part = strings.ToLower(strings.TrimSpace(part))
		if part != "" {
			out = append(out, part)
		}
	}
	return out
}

// HasAny reports whether held contains any of needed (case-insensitive).
// Empty needed → false. Held containing "*" → true.
func HasAny(held, needed []string) bool {
	if len(needed) == 0 {
		return false
	}
	set := map[string]struct{}{}
	for _, h := range held {
		h = strings.ToLower(strings.TrimSpace(h))
		if h == "*" {
			return true
		}
		if h != "" {
			set[h] = struct{}{}
		}
	}
	for _, n := range needed {
		n = strings.ToLower(strings.TrimSpace(n))
		if _, ok := set[n]; ok {
			return true
		}
	}
	return false
}

// MergeUnique concatenates permission lists.
func MergeUnique(lists ...[]string) []string {
	seen := map[string]struct{}{}
	var out []string
	for _, list := range lists {
		for _, p := range list {
			p = strings.ToLower(strings.TrimSpace(p))
			if p == "" {
				continue
			}
			if _, ok := seen[p]; ok {
				continue
			}
			seen[p] = struct{}{}
			out = append(out, p)
		}
	}
	return out
}

const schemaSQL = `
CREATE TABLE IF NOT EXISTS web_permissions (
  code TEXT PRIMARY KEY,
  resource TEXT NOT NULL,
  action TEXT NOT NULL,
  description TEXT
);
CREATE TABLE IF NOT EXISTS web_roles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  is_system INTEGER NOT NULL DEFAULT 0,
  tenant_id TEXT
);
CREATE TABLE IF NOT EXISTS web_role_permissions (
  role_id INTEGER NOT NULL,
  permission_code TEXT NOT NULL,
  PRIMARY KEY (role_id, permission_code),
  FOREIGN KEY (role_id) REFERENCES web_roles(id) ON DELETE CASCADE,
  FOREIGN KEY (permission_code) REFERENCES web_permissions(code) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS web_users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at TEXT
);
CREATE TABLE IF NOT EXISTS web_user_roles (
  user_id INTEGER NOT NULL,
  role_id INTEGER NOT NULL,
  tenant_id TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (user_id, role_id, tenant_id),
  FOREIGN KEY (user_id) REFERENCES web_users(id) ON DELETE CASCADE,
  FOREIGN KEY (role_id) REFERENCES web_roles(id) ON DELETE CASCADE
);
`

// EnsureSchema creates RBAC tables and seeds default catalog + system roles.
func EnsureSchema(dbURL string) error {
	if strings.TrimSpace(dbURL) == "" {
		return fmt.Errorf("rbac: empty db url")
	}
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return err
	}
	if _, err := sqlDB.Exec(schemaSQL); err != nil {
		return err
	}
	for _, p := range DefaultCatalog() {
		_, _ = sqlDB.Exec(
			`INSERT OR IGNORE INTO web_permissions(code, resource, action, description) VALUES(?,?,?,?)`,
			p.Code, p.Resource, p.Action, p.Description,
		)
	}
	if err := ensureSystemRole(sqlDB, "superadmin", true, AllCodes()); err != nil {
		return err
	}
	if err := ensureSystemRole(sqlDB, "member", true, []string{"comments:create", "comments:delete_own"}); err != nil {
		return err
	}
	return nil
}

func ensureSystemRole(sqlDB *sql.DB, name string, isSystem bool, perms []string) error {
	var id int64
	err := sqlDB.QueryRow(`SELECT id FROM web_roles WHERE name = ?`, name).Scan(&id)
	if err == sql.ErrNoRows {
		sys := 0
		if isSystem {
			sys = 1
		}
		res, err := sqlDB.Exec(`INSERT INTO web_roles(name, is_system) VALUES(?,?)`, name, sys)
		if err != nil {
			return err
		}
		id, err = res.LastInsertId()
		if err != nil {
			return err
		}
	} else if err != nil {
		return err
	}
	for _, code := range perms {
		_, _ = sqlDB.Exec(
			`INSERT OR IGNORE INTO web_role_permissions(role_id, permission_code) VALUES(?,?)`,
			id, code,
		)
	}
	return nil
}

// PermissionsForUser returns permission codes granted via user_roles for username.
func PermissionsForUser(dbURL, username string) ([]string, error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return nil, err
	}
	rows, err := sqlDB.Query(`
SELECT DISTINCT rp.permission_code
FROM web_users u
JOIN web_user_roles ur ON ur.user_id = u.id
JOIN web_role_permissions rp ON rp.role_id = ur.role_id
WHERE u.username = ?`, username)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []string
	for rows.Next() {
		var code string
		if err := rows.Scan(&code); err != nil {
			return nil, err
		}
		out = append(out, strings.ToLower(code))
	}
	return out, rows.Err()
}

// AssignRole binds username to role name (creates user row link only; user must exist in web_users).
func AssignRole(dbURL, username, roleName string) error {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return err
	}
	var uid, rid int64
	if err := sqlDB.QueryRow(`SELECT id FROM web_users WHERE username = ?`, username).Scan(&uid); err != nil {
		return fmt.Errorf("rbac assign: user %q: %w", username, err)
	}
	if err := sqlDB.QueryRow(`SELECT id FROM web_roles WHERE name = ?`, roleName).Scan(&rid); err != nil {
		return fmt.Errorf("rbac assign: role %q: %w", roleName, err)
	}
	_, err = sqlDB.Exec(
		`INSERT OR IGNORE INTO web_user_roles(user_id, role_id, tenant_id) VALUES(?,?,?)`,
		uid, rid, "",
	)
	return err
}

// CreateRole inserts a non-system role; returns role id.
func CreateRole(dbURL, name string) (int64, error) {
	if err := EnsureSchema(dbURL); err != nil {
		return 0, err
	}
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return 0, err
	}
	res, err := sqlDB.Exec(`INSERT INTO web_roles(name, is_system) VALUES(?,0)`, name)
	if err != nil {
		return 0, err
	}
	return res.LastInsertId()
}

// SetRolePermissions replaces role's permissions. grantable restricts what can be set (anti-escalation);
// empty grantable means allow all catalog codes.
func SetRolePermissions(dbURL string, roleID int64, codes, grantable []string) error {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return err
	}
	var isSystem int
	if err := sqlDB.QueryRow(`SELECT is_system FROM web_roles WHERE id = ?`, roleID).Scan(&isSystem); err != nil {
		return err
	}
	if isSystem == 1 {
		return fmt.Errorf("rbac: cannot rewrite system role permissions via set")
	}
	allow := map[string]struct{}{}
	if len(grantable) == 0 {
		for _, c := range AllCodes() {
			allow[c] = struct{}{}
		}
	} else {
		for _, c := range grantable {
			allow[strings.ToLower(c)] = struct{}{}
		}
	}
	tx, err := sqlDB.Begin()
	if err != nil {
		return err
	}
	defer func() { _ = tx.Rollback() }()
	if _, err := tx.Exec(`DELETE FROM web_role_permissions WHERE role_id = ?`, roleID); err != nil {
		return err
	}
	for _, code := range codes {
		code = strings.ToLower(strings.TrimSpace(code))
		if code == "" {
			continue
		}
		if _, ok := allow[code]; !ok {
			return fmt.Errorf("rbac: cannot grant permission %q (not held by actor)", code)
		}
		if _, err := tx.Exec(
			`INSERT INTO web_role_permissions(role_id, permission_code) VALUES(?,?)`,
			roleID, code,
		); err != nil {
			return err
		}
	}
	return tx.Commit()
}

// CreateUser inserts a web_users row with hashed password; returns user id.
func CreateUser(dbURL, username, passwordHash string) (int64, error) {
	username = strings.TrimSpace(username)
	if username == "" || passwordHash == "" {
		return 0, fmt.Errorf("rbac: username and password_hash required")
	}
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return 0, err
	}
	res, err := sqlDB.Exec(
		`INSERT INTO web_users(username, password_hash, created_at) VALUES(?,?,datetime('now'))`,
		username, passwordHash,
	)
	if err != nil {
		return 0, err
	}
	return res.LastInsertId()
}

// UserExists reports whether username is in web_users.
func UserExists(dbURL, username string) (bool, error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return false, err
	}
	var id int64
	err = sqlDB.QueryRow(`SELECT id FROM web_users WHERE username = ?`, username).Scan(&id)
	if err == sql.ErrNoRows {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, nil
}

// Authenticate checks username/password against web_users; returns primary role name.
func Authenticate(dbURL, username, pass string, verify func(password, encoded string) bool) (role string, ok bool, err error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return "", false, err
	}
	var hash string
	var uid int64
	err = sqlDB.QueryRow(`SELECT id, password_hash FROM web_users WHERE username = ?`, username).Scan(&uid, &hash)
	if err == sql.ErrNoRows {
		return "", false, nil
	}
	if err != nil {
		return "", false, err
	}
	if verify == nil || !verify(pass, hash) {
		return "", false, nil
	}
	var roleName string
	err = sqlDB.QueryRow(`
SELECT r.name FROM web_user_roles ur
JOIN web_roles r ON r.id = ur.role_id
WHERE ur.user_id = ?
ORDER BY r.is_system DESC, r.name
LIMIT 1`, uid).Scan(&roleName)
	if err == sql.ErrNoRows {
		return "member", true, nil
	}
	if err != nil {
		return "", false, err
	}
	return strings.ToLower(roleName), true, nil
}

// RegisterUser creates web_users + assigns defaultRole (default member).
func RegisterUser(dbURL, username, passwordHash, defaultRole string) error {
	if defaultRole == "" {
		defaultRole = "member"
	}
	exists, err := UserExists(dbURL, username)
	if err != nil {
		return err
	}
	if exists {
		return fmt.Errorf("rbac: username already taken")
	}
	if _, err := CreateUser(dbURL, username, passwordHash); err != nil {
		return err
	}
	return AssignRole(dbURL, username, defaultRole)
}

// EnsureUserWithRole creates user if missing (hash may be empty → skip create) and assigns role.
func EnsureUserWithRole(dbURL, username, passwordHash, roleName string) error {
	exists, err := UserExists(dbURL, username)
	if err != nil {
		return err
	}
	if !exists {
		if passwordHash == "" {
			return fmt.Errorf("rbac: cannot create user %q without hash", username)
		}
		if _, err := CreateUser(dbURL, username, passwordHash); err != nil {
			return err
		}
	}
	return AssignRole(dbURL, username, roleName)
}

// ListRoles returns [{id,name,is_system}, …].
func ListRoles(dbURL string) ([]map[string]any, error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return nil, err
	}
	rows, err := sqlDB.Query(`SELECT id, name, is_system FROM web_roles ORDER BY name`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []map[string]any
	for rows.Next() {
		var id int64
		var name string
		var sys int
		if err := rows.Scan(&id, &name, &sys); err != nil {
			return nil, err
		}
		out = append(out, map[string]any{"id": id, "name": name, "is_system": sys == 1})
	}
	return out, rows.Err()
}

// ListPermissions returns catalog rows.
func ListPermissions(dbURL string) ([]map[string]any, error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return nil, err
	}
	rows, err := sqlDB.Query(`SELECT code, resource, action, description FROM web_permissions ORDER BY code`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []map[string]any
	for rows.Next() {
		var code, resource, action, desc string
		if err := rows.Scan(&code, &resource, &action, &desc); err != nil {
			return nil, err
		}
		out = append(out, map[string]any{
			"code": code, "resource": resource, "action": action, "description": desc,
		})
	}
	return out, rows.Err()
}

// RolePermissionCodes returns permission codes for a role id.
func RolePermissionCodes(dbURL string, roleID int64) ([]string, error) {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return nil, err
	}
	rows, err := sqlDB.Query(`SELECT permission_code FROM web_role_permissions WHERE role_id = ? ORDER BY permission_code`, roleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []string
	for rows.Next() {
		var code string
		if err := rows.Scan(&code); err != nil {
			return nil, err
		}
		out = append(out, code)
	}
	return out, rows.Err()
}

// SetRolePermissionsByName looks up role by name then SetRolePermissions.
func SetRolePermissionsByName(dbURL, roleName string, codes, grantable []string) error {
	sqlDB, err := openDB(dbURL)
	if err != nil {
		return err
	}
	var id int64
	if err := sqlDB.QueryRow(`SELECT id FROM web_roles WHERE name = ?`, roleName).Scan(&id); err != nil {
		return fmt.Errorf("rbac role %q: %w", roleName, err)
	}
	return SetRolePermissions(dbURL, id, codes, grantable)
}

func openDB(dbURL string) (*sql.DB, error) {
	// Reuse db package pool via a no-op select path: open through Ensure-compatible helper.
	return db.OpenSQL(dbURL)
}
