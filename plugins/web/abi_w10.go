package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_app_proxy(char *args_json, char **out_json, char **err_msg);
extern int web_app_invoke(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"github.com/marqdo/marqdo/plugins/web/internal/invoke"
	"github.com/marqdo/marqdo/plugins/web/internal/proxy"
)

//export web_app_proxy
func web_app_proxy(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	opts, err := proxy.OptionsFromArgs(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := proxy.Register(asPageMap(args["app"]), opts)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_invoke
func web_app_invoke(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	opts, err := invoke.OptionsFromArgs(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := invoke.Register(asPageMap(args["app"]), opts)
	return replyJSON(outJSON, errMsg, out, err)
}
