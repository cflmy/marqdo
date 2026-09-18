package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"
*/
import "C"

import (
	"fmt"
	"strconv"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/httpx"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/password"
	"github.com/marqdo/marqdo/plugins/web/internal/rbac"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func argUint64(args map[string]any, def uint64, keys ...string) uint64 {
	for _, k := range keys {
		v, ok := args[k]
		if !ok || v == nil {
			continue
		}
		switch t := v.(type) {
		case float64:
			if t > 0 {
				return uint64(t)
			}
		case int:
			if t > 0 {
				return uint64(t)
			}
		case int64:
			if t > 0 {
				return uint64(t)
			}
		case uint64:
			if t > 0 {
				return t
			}
		case string:
			if n, e := strconv.ParseUint(t, 10, 64); e == nil && n > 0 {
				return n
			}
		}
	}
	return def
}

func txnOpt(args map[string]any) string {
	s, _ := argStr(args, "txn", "事务")
	return s
}

//export web_db_begin
func web_db_begin(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Begin(url)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_commit
func web_db_commit(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	txn, err := argStrReq(args, "txn", "事务")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Commit(txn)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_rollback
func web_db_rollback(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	txn, err := argStrReq(args, "txn", "事务")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Rollback(txn)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_migrate
func web_db_migrate(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	steps := args["steps"]
	if steps == nil {
		steps = []any{}
	}
	out, err := db.Migrate(url, steps)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_fts_create
func web_db_fts_create(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	name, _ := argStr(args, "name")
	out, err := db.FtsCreate(url, tableName, args["columns"], name)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_search
func web_db_search(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	q, _ := argStr(args, "q")
	limit := int64(50)
	if v, ok := args["limit"].(float64); ok {
		limit = int64(v)
	}
	name, _ := argStr(args, "name")
	out, err := db.Search(url, tableName, q, limit, name)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_form_rules
func web_form_rules(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	formBag := asPageMap(args["form"])
	if args["rules"] == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `rules`"))
	}
	return replyJSON(outJSON, errMsg, form.SetRules(formBag, args["rules"]), nil)
}

//export web_form_labels
func web_form_labels(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	submit, _ := argStr(args, "submit")
	if submit == "" {
		submit, _ = argStr(args, "提交")
	}
	cancel, _ := argStr(args, "cancel")
	if cancel == "" {
		cancel, _ = argStr(args, "取消")
	}
	cancelHref, _ := argStr(args, "cancel_href")
	if cancelHref == "" {
		cancelHref, _ = argStr(args, "取消链接")
	}
	return replyJSON(outJSON, errMsg, form.SetLabels(asPageMap(args["form"]), submit, cancel, cancelHref), nil)
}

//export web_form_validate
func web_form_validate(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	if args["data"] == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `data`"))
	}
	return replyJSON(outJSON, errMsg, form.Validate(asPageMap(args["form"]), args["rules"], args["data"]), nil)
}

//export web_form_render
func web_form_render(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, _ := argStr(args, "id")
	if id == "" {
		id = "form"
	}
	csrf, _ := argStr(args, "csrf")
	html := form.Render(asPageMap(args["form"]), id, args["data"], args["errors"], csrf)
	return replyJSON(outJSON, errMsg, html, nil)
}

//export web_form_submit
func web_form_submit(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	if args["data"] == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `data`"))
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := form.Submit(asPageMap(args["form"]), args["data"], url)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_table_info
func web_db_table_info(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	cols, err := db.TableInfo(url, tableName)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	arr := make([]any, len(cols))
	for i, c := range cols {
		arr[i] = map[string]any{
			"name":    c.Name,
			"type":    c.SQLType,
			"notnull": c.NotNull,
			"pk":      c.PK,
		}
	}
	return replyJSON(outJSON, errMsg, map[string]any{"columns": arr}, nil)
}

//export web_form_from_schema
func web_form_from_schema(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	action, _ := argStr(args, "action")
	id, _ := argStr(args, "id")
	adminPrefix, _ := argStr(args, "admin_prefix", "后台前缀")
	if adminPrefix == "" {
		adminPrefix = "/admin"
	}
	out, err := form.FromSchema(url, tableName, action, id, adminPrefix)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_mount_form
func web_app_mount_form(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	if args["form"] == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `form`"))
	}
	out, err := app.MountForm(asPageMap(args["app"]), id, args["form"])
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_static
func web_app_static(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	dir, err := argStrReq(args, "dir")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	mount, _ := argStr(args, "mount")
	out, err := app.Static(asPageMap(args["app"]), dir, mount)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_middleware
func web_app_middleware(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := middleware.Configure(asPageMap(args["app"]), args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_listen
func web_listen(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	appBag := asPageMap(args["app"])
	if len(appBag) == 0 {
		appBag = args
	}
	out, err := httpx.Listen(appBag, entryDir())
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_password_hash
func web_password_hash(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pw, err := argStrReq(args, "password", "密码")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := password.HashResult(pw)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_session_new
func web_session_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	ttl := argUint64(args, 3600, "ttl_sec", "ttl", "session_ttl")
	return replyJSON(outJSON, errMsg, session.New(ttl), nil)
}

//export web_session_set
func web_session_set(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id", "session_id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, session.Set(id, key, args["value"]), nil)
}

//export web_session_get
func web_session_get(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id", "session_id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, session.Get(id, key), nil)
}

//export web_session_del
func web_session_del(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id", "session_id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, session.Del(id, key), nil)
}

//export web_session_destroy
func web_session_destroy(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id", "session_id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, session.Destroy(id), nil)
}

//export web_auth_login
func web_auth_login(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	username, err := argStrReq(args, "username", "用户名", "用户")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pw, err := argStrReq(args, "password", "密码")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	ttl := argUint64(args, 3600, "session_ttl", "ttl_sec", "ttl")
	out := auth.Login(username, pw, args["users"], ttl)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_auth_check
func web_auth_check(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "session_id", "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, auth.Check(id), nil)
}

//export web_auth_logout
func web_auth_logout(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "session_id", "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, auth.Logout(id), nil)
}

//export web_auth_new
func web_auth_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	ttl := argUint64(args, 3600, "session_ttl", "ttl_sec", "ttl")
	return replyJSON(outJSON, errMsg, auth.New(args["users"], ttl), nil)
}

//export web_app_auth
func web_app_auth(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := app.Auth(asPageMap(args["app"]), args["users"], args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_gate
func web_app_gate(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := app.Gate(asPageMap(args["app"]), path, args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_rbac
func web_app_rbac(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := app.Rbac(asPageMap(args["app"]), args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_tenant
func web_app_tenant(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := app.Tenant(asPageMap(args["app"]), args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_rbac_can
func web_rbac_can(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	heldRaw, _ := argStrReq(args, "permissions", "权限", "held")
	neededRaw, _ := argStrReq(args, "needed", "需要")
	if neededRaw == "" {
		neededRaw, _ = argStrReq(args, "permission", "权限码")
	}
	ok := rbac.HasAny(rbac.ParseCSV(heldRaw), rbac.ParseCSV(neededRaw))
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "allowed": ok}, nil)
}

//export web_rbac_ensure
func web_rbac_ensure(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	if err := rbac.EnsureSchema(url); err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true}, nil)
}

//export web_rbac_assign_role
func web_rbac_assign_role(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	user, err := argStrReq(args, "username", "用户名", "user")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	role, err := argStrReq(args, "role", "角色")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	err = rbac.AssignRole(url, user, role)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true}, nil)
}

//export web_rbac_create_role
func web_rbac_create_role(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	name, err := argStrReq(args, "name", "名称")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := rbac.CreateRole(url, name)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "id": id, "name": name}, nil)
}

//export web_rbac_set_role_permissions
func web_rbac_set_role_permissions(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	roleName, _ := argStrReq(args, "role", "角色")
	permsRaw, _ := argStrReq(args, "permissions", "权限")
	grantRaw, _ := argStrReq(args, "grantable", "可授")
	err = rbac.SetRolePermissionsByName(url, roleName, rbac.ParseCSV(permsRaw), rbac.ParseCSV(grantRaw))
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true}, nil)
}

//export web_rbac_list_roles
func web_rbac_list_roles(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	roles, err := rbac.ListRoles(url)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "roles": roles}, nil)
}

//export web_rbac_list_permissions
func web_rbac_list_permissions(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	perms, err := rbac.ListPermissions(url)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "permissions": perms}, nil)
}
