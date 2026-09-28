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
	"strings"

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
	var homeMeta map[string]string
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
			homeMeta = f.Meta
			break
		}
	}
	home := page.New(homeArgs)
	stampArtifactData(home, homeMeta)
	a := app.New(map[string]any{
		"page": home,
		"db":   args["db"],
		"host": host,
		"port": port,
	})
	artifactRoutes := map[string]any{}
	for _, f := range files {
		if f.Kind == "web" {
			if f.Route == "/" {
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
			stampArtifactData(pg, f.Meta)
			out, err := app.Route(a, f.Route, pg)
			if err != nil {
				return replyJSON(outJSON, errMsg, nil, err)
			}
			a = out
			continue
		}
		// endpoint
		req := f.Meta["request"]
		if req == "" {
			req = f.Meta["请求"]
		}
		if req == "" {
			req = "json"
		}
		resp := f.Meta["response"]
		if resp == "" {
			resp = f.Meta["响应"]
		}
		if resp == "" {
			resp = "json"
		}
		auth := f.Meta["auth"]
		if auth == "" {
			auth = f.Meta["鉴权"]
		}
		key := f.Method + " " + f.Route
		artifactRoutes[key] = map[string]any{
			"method":   f.Method,
			"path":     f.Route,
			"file":     f.Path,
			"request":  req,
			"response": resp,
			"auth":     auth,
			"kind":     "endpoint",
		}
	}
	if len(artifactRoutes) > 0 {
		a["artifact_routes"] = artifactRoutes
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

// stampArtifactData applies flat Document metadata (ADR 0007) onto a page bag.
func stampArtifactData(pg map[string]any, meta map[string]string) {
	if pg == nil || meta == nil {
		return
	}
	src := meta["data_source"]
	if src == "" {
		src = meta["数据源"]
	}
	if src != "" {
		pg["data_source"] = src
	}
	order := meta["data_order"]
	if order == "" {
		order = meta["排序"]
	}
	if order != "" {
		pg["order"] = order
	}
	where := meta["data_where"]
	if where == "" {
		where = meta["条件"]
	}
	if where != "" {
		// col=val,col2=val2 → query map (values may contain {param})
		q := map[string]any{}
		for _, part := range strings.Split(where, ",") {
			part = strings.TrimSpace(part)
			if part == "" {
				continue
			}
			k, v, ok := strings.Cut(part, "=")
			if !ok {
				continue
			}
			q[strings.TrimSpace(k)] = strings.TrimSpace(v)
		}
		if len(q) > 0 {
			pg["query"] = q
			// Filtered Document routes render as article detail (007 /post/{slug}).
			pg["detail"] = true
		}
	}
	if d := meta["detail"]; d != "" {
		pg["detail"] = d == "true" || d == "True" || d == "1"
	}
	if d := meta["详情"]; d != "" {
		pg["detail"] = d == "true" || d == "True" || d == "1" || d == "是"
	}
	prefix := meta["link_prefix"]
	if prefix == "" {
		prefix = meta["链接前缀"]
	}
	if prefix != "" {
		pg["link_prefix"] = prefix
	} else if src != "" && where == "" {
		// List pages default card links to /post/{slug}.
		pg["link_prefix"] = "/post/"
	}
}
