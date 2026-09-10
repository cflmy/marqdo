package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"
*/
import "C"

import (
	"fmt"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
)

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
