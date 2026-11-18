package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_page_paginate(char *args_json, char **out_json, char **err_msg);
extern int web_rss_build(char *args_json, char **out_json, char **err_msg);
extern int web_app_route_rss(char *args_json, char **out_json, char **err_msg);
extern int web_app_redirect(char *args_json, char **out_json, char **err_msg);
extern int web_app_error_page(char *args_json, char **out_json, char **err_msg);
extern int web_app_sitemap(char *args_json, char **out_json, char **err_msg);
extern int web_app_robots(char *args_json, char **out_json, char **err_msg);
extern int web_sitemap_build(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"fmt"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/page"
	"github.com/marqdo/marqdo/plugins/web/internal/rss"
	"github.com/marqdo/marqdo/plugins/web/internal/sitemap"
)

func argInt64(args map[string]any, def int64, keys ...string) int64 {
	for _, k := range keys {
		v, ok := args[k]
		if !ok || v == nil {
			continue
		}
		switch t := v.(type) {
		case float64:
			return int64(t)
		case int:
			return int64(t)
		case int64:
			return t
		case string:
			if n, err := parseInt64(t); err == nil {
				return n
			}
		}
	}
	return def
}

func parseInt64(s string) (int64, error) {
	var n int64
	_, err := fmt.Sscanf(s, "%d", &n)
	return n, err
}

func argStatus(args map[string]any) uint16 {
	v, ok := first(args, "status", "状态")
	if !ok || v == nil {
		return 404
	}
	switch t := v.(type) {
	case float64:
		return uint16(t)
	case int:
		if t < 0 {
			return 404
		}
		return uint16(t)
	case int64:
		if t < 0 {
			return 404
		}
		return uint16(t)
	case string:
		if n, err := parseInt64(t); err == nil && n >= 0 {
			return uint16(n)
		}
	}
	return 404
}

func first(args map[string]any, keys ...string) (any, bool) {
	for _, k := range keys {
		if v, ok := args[k]; ok && v != nil {
			return v, true
		}
	}
	return nil, false
}

func argPermanent(args map[string]any) bool {
	v, ok := first(args, "permanent", "永久")
	if !ok {
		return false
	}
	switch t := v.(type) {
	case bool:
		return t
	case string:
		switch t {
		case "true", "True", "1", "yes", "真":
			return true
		}
	case float64:
		return int64(t) != 0
	}
	return false
}

func itemsOf(args map[string]any) any {
	if v, ok := first(args, "items", "rows", "条目"); ok {
		return v
	}
	return []any{}
}

//export web_page_paginate
func web_page_paginate(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	offset := argInt64(args, 0, "offset", "偏移")
	limit := argInt64(args, 10, "limit", "上限")
	path, _ := argStr(args, "path", "路径")
	out := page.Paginate(asPageMap(args["page"]), offset, limit, path)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_rss_build
func web_rss_build(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	title, _ := argStr(args, "title", "标题")
	if title == "" {
		title = "Feed"
	}
	link, _ := argStr(args, "link", "链接")
	if link == "" {
		link = "/"
	}
	description, _ := argStr(args, "description", "描述")
	items, _ := itemsOf(args).([]any)
	if items == nil {
		items = []any{}
	}
	xml := rss.BuildRSS(title, link, description, items)
	return replyJSON(outJSON, errMsg, xml, nil)
}

//export web_app_route_rss
func web_app_route_rss(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	table, err := argStrReq(args, "table", "表")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	limit := argInt64(args, 20, "limit", "上限")
	order, _ := argStr(args, "order", "排序")
	title, _ := argStr(args, "title", "标题")
	link, _ := argStr(args, "link", "链接")
	description, _ := argStr(args, "description", "描述")
	out, err := app.RouteRSS(asPageMap(args["app"]), path, table, order, title, link, description, limit)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_redirect
func web_app_redirect(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	from, err := argStrReq(args, "from", "来源", "path", "路径")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	to, err := argStrReq(args, "to", "目标")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := app.Redirect(asPageMap(args["app"]), from, to, argPermanent(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_error_page
func web_app_error_page(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageV, ok := first(args, "page", "页面")
	if !ok {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `page`"))
	}
	out, err := app.ErrorPage(asPageMap(args["app"]), argStatus(args), pageV)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_sitemap
func web_app_sitemap(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, _ := argStr(args, "path", "路径")
	base, _ := argStr(args, "base", "基址")
	table, _ := argStr(args, "table", "表")
	loc, _ := argStr(args, "loc", "定位列")
	limit := argInt64(args, 1000, "limit", "上限")
	items := itemsOf(args)
	out, err := app.Sitemap(asPageMap(args["app"]), path, base, table, loc, limit, items)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_robots
func web_app_robots(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	body, _ := argStr(args, "body", "正文")
	sitemapURL, _ := argStr(args, "sitemap", "站点地图")
	out, err := app.Robots(asPageMap(args["app"]), body, sitemapURL)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_sitemap_build
func web_sitemap_build(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	base, _ := argStr(args, "base", "基址")
	out := sitemap.SitemapJSON(base, itemsOf(args))
	return replyJSON(outJSON, errMsg, out, nil)
}
