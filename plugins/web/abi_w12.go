package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_api_key_check(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"github.com/marqdo/marqdo/plugins/web/internal/apikey"
)

//export web_api_key_check
func web_api_key_check(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, _ := argStr(args, "key", "密钥")
	auth, _ := argStr(args, "authorization", "auth", "授权")
	pepper, _ := argStr(args, "pepper", "椒盐")
	keys := args["keys"]
	if keys == nil {
		keys = args["hashes"]
	}
	out := apikey.Check(key, auth, pepper, keys)
	return replyJSON(outJSON, errMsg, out, nil)
}
