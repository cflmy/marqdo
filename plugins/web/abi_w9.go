package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_cache_new(char *args_json, char **out_json, char **err_msg);
extern int web_cache_get(char *args_json, char **out_json, char **err_msg);
extern int web_cache_set(char *args_json, char **out_json, char **err_msg);
extern int web_cache_del(char *args_json, char **out_json, char **err_msg);
extern int web_cache_exists(char *args_json, char **out_json, char **err_msg);
extern int web_cache_ttl(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"github.com/marqdo/marqdo/plugins/web/internal/cache"
)

func argOptUint64(args map[string]any, keys ...string) *uint64 {
	for _, k := range keys {
		v, ok := args[k]
		if !ok || v == nil {
			continue
		}
		switch t := v.(type) {
		case float64:
			n := uint64(t)
			return &n
		case int:
			n := uint64(t)
			return &n
		case int64:
			n := uint64(t)
			return &n
		case string:
			// ignore parse errors — treat as absent
			continue
		}
	}
	return nil
}

//export web_cache_new
func web_cache_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, _ := argStr(args, "url", "地址")
	if url == "" {
		url = "memory:"
	}
	out, err := cache.Open(url)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_cache_get
func web_cache_get(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key", "键")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := cache.Get(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_cache_set
func web_cache_set(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key", "键")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	value, err := argStrReq(args, "value", "值")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	ttl := argOptUint64(args, "ttl", "过期")
	out, err := cache.Set(url, key, value, ttl)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_cache_del
func web_cache_del(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key", "键")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := cache.Del(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_cache_exists
func web_cache_exists(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key", "键")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := cache.Exists(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_cache_ttl
func web_cache_ttl(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	key, err := argStrReq(args, "key", "键")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := cache.TTL(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}
