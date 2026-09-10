package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_page_query(char *args_json, char **out_json, char **err_msg);
extern int web_page_order(char *args_json, char **out_json, char **err_msg);
extern int web_page_link_prefix(char *args_json, char **out_json, char **err_msg);
extern int web_page_css(char *args_json, char **out_json, char **err_msg);
extern int web_page_detail(char *args_json, char **out_json, char **err_msg);
extern int web_page_chrome(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"github.com/marqdo/marqdo/plugins/web/internal/page"
)

//export web_page_query
func web_page_query(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out := page.SetQuery(asPageMap(args["page"]), args["query"])
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_order
func web_page_order(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	order, _ := argStr(args, "order", "排序")
	out := page.SetOrder(asPageMap(args["page"]), order)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_link_prefix
func web_page_link_prefix(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	prefix, err := argStrReq(args, "prefix", "前缀")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out := page.SetLinkPrefix(asPageMap(args["page"]), prefix)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_css
func web_page_css(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	css, _ := argStr(args, "css", "样式")
	out := page.AppendCSS(asPageMap(args["page"]), css)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_detail
func web_page_detail(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	on := page.AsBool(args["detail"], true)
	if v, ok := args["详情"]; ok {
		on = page.AsBool(v, on)
	}
	out := page.SetDetail(asPageMap(args["page"]), on)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_chrome
func web_page_chrome(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	nav, _ := argStr(args, "nav_html", "导航")
	foot, _ := argStr(args, "footer_html", "页脚")
	body, _ := argStr(args, "body_class", "体类")
	out := page.SetChromeHTML(asPageMap(args["page"]), nav, foot, body)
	return replyJSON(outJSON, errMsg, out, nil)
}
