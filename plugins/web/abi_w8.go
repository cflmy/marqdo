package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_app_route_ws(char *args_json, char **out_json, char **err_msg);
extern int web_ws_connect(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"github.com/marqdo/marqdo/plugins/web/internal/ws"
)

//export web_app_route_ws
func web_app_route_ws(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := ws.RouteWS(asPageMap(args["app"]), path, args)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_ws_connect
func web_ws_connect(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url", "地址")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	message, _ := argStr(args, "message", "消息")
	var headers map[string]any
	if h, ok := args["headers"]; ok && h != nil {
		if m, ok := h.(map[string]any); ok {
			headers = m
		}
	}
	timeout := argUint64(args, 30, "timeout_sec", "timeout", "超时")
	out := ws.Connect(url, message, headers, timeout)
	return replyJSON(outJSON, errMsg, out, nil)
}
