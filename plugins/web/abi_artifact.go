package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_render_nodes(char *args_json, char **out_json, char **err_msg);
extern int web_render_document(char *args_json, char **out_json, char **err_msg);
extern int web_dom_markdown(char *args_json, char **out_json, char **err_msg);
extern int web_serve_root(char *args_json, char **out_json, char **err_msg);
extern int web_inspect(char *args_json, char **out_json, char **err_msg);
extern int web_app_use(char *args_json, char **out_json, char **err_msg);
extern int web_route_use(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/artifact"
	"github.com/marqdo/marqdo/plugins/web/internal/httpx"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/page"
)

//export web_render_nodes
func web_render_nodes(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	frag, err := artifact.RenderNodes(args["nodes"])
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pg := asPageMap(args["page"])
	title := ""
	if pg != nil {
		if t, ok := pg["title"].(string); ok {
			title = t
		}
	}
	if t, ok := argStr(args, "title"); ok && t != "" {
		title = t
	}
	return replyJSON(outJSON, errMsg, artifact.RenderHTMLPage(title, frag), nil)
}

//export web_render_document
func web_render_document(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pg := asPageMap(args["page"])
	title := ""
	body := ""
	if pg != nil {
		if t, ok := pg["title"].(string); ok {
			title = t
		}
		if b, ok := pg["body"].(string); ok {
			body = b
		}
	}
	if b, ok := argStr(args, "body"); ok && b != "" {
		body = b
	}
	if t, ok := argStr(args, "title"); ok && t != "" {
		title = t
	}
	return replyJSON(outJSON, errMsg, artifact.RenderDocument(title, body), nil)
}

//export web_dom_markdown
func web_dom_markdown(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	text, _ := argStr(args, "text")
	return replyJSON(outJSON, errMsg, artifact.MarkdownHTML(text), nil)
}

//export web_serve_root
func web_serve_root(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	root, _ := argStr(args, "root")
	if root == "" {
		root = "."
	}
	abs, err := filepath.Abs(root)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	if st, err := os.Stat(abs); err != nil || !st.IsDir() {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("web.serve: root not a directory: %s", abs))
	}
	files, err := artifact.ScanRoot(abs)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	host, _ := argStr(args, "host")
	if host == "" {
		host = "127.0.0.1"
	}
	port := float64(18081)
	if v, ok := args["port"]; ok && v != nil {
		switch t := v.(type) {
		case float64:
			port = t
		case string:
			if n, e := strconv.Atoi(t); e == nil {
				port = float64(n)
			}
		}
	}
	homeArgs := map[string]any{"title": "Marqdo"}
	for _, f := range files {
		if f.Kind == "web" && (f.Route == "/" || f.Rel == "index.mq.md") {
			title := f.Meta["title"]
			if title == "" {
				title = f.Meta["标题"]
			}
			if title == "" {
				title = "Marqdo"
			}
			homeArgs = map[string]any{
				"title": title,
				"intro": artifact.MarkdownHTML(f.Body),
			}
			break
		}
	}
	home := page.New(homeArgs)
	a := app.New(map[string]any{
		"page": home,
		"db":   args["db"],
		"host": host,
		"port": port,
	})
	for _, f := range files {
		if f.Kind != "web" || f.Route == "/" {
			continue
		}
		title := f.Meta["title"]
		if title == "" {
			title = f.Meta["标题"]
		}
		pg := page.New(map[string]any{
			"title": title,
			"intro": artifact.MarkdownHTML(f.Body),
		})
		out, err := app.Route(a, f.Route, pg)
		if err != nil {
			return replyJSON(outJSON, errMsg, nil, err)
		}
		a = out
	}
	if mw := args["middleware"]; mw != nil {
		out, err := middleware.Configure(a, map[string]any{"security": mw})
		if err != nil {
			return replyJSON(outJSON, errMsg, nil, err)
		}
		a = out
	}
	if sd, ok := argStr(args, "static_dir"); ok && sd != "" {
		out, err := app.Static(a, sd, "/static")
		if err != nil {
			return replyJSON(outJSON, errMsg, nil, err)
		}
		a = out
	}
	a["inspect"] = artifact.InspectBag(a, files)
	_, err = httpx.Listen(a, abs)
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "value": a["inspect"], "error": nil}, err)
}

//export web_inspect
func web_inspect(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	a := asPageMap(args["app"])
	if a != nil {
		if insp, ok := a["inspect"].(map[string]any); ok {
			return replyJSON(outJSON, errMsg, insp, nil)
		}
	}
	return replyJSON(outJSON, errMsg, artifact.InspectBag(a, nil), nil)
}

//export web_app_use
func web_app_use(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	a := asPageMap(args["app"])
	out, err := middleware.Configure(a, map[string]any{"security": args["middleware"]})
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_route_use
func web_route_use(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	rt := asPageMap(args["route"])
	if rt == nil {
		rt = map[string]any{}
	}
	rt["middleware"] = args["middleware"]
	return replyJSON(outJSON, errMsg, rt, nil)
}
