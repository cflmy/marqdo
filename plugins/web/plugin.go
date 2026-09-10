// Package main builds the Marqdo web plugin as a C shared library (ABI v2).
//
//	go build -buildmode=c-shared -o build/libweb.so .
//
// See doc/design/ext-web-go-rewrite.md (W-G0+).
package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"
#include <stdlib.h>
#include <string.h>

static void *g_userdata;
static int (*g_register_fn)(void *, char *, char *, MarqdoPluginFn);
static void *(*g_alloc)(size_t);
static void (*g_free)(void *);
static MarqdoHostQueryFn g_host_query;

static void host_save(MarqdoHostApi *h) {
	g_userdata = h->userdata;
	g_register_fn = h->register_fn;
	g_alloc = h->alloc;
	g_free = h->free;
	g_host_query = h->host_query;
}

static char *host_strdup(const char *s) {
	if (!g_alloc || !s) return NULL;
	size_t n = strlen(s);
	char *p = (char *)g_alloc(n + 1);
	if (!p) return NULL;
	memcpy(p, s, n);
	p[n] = 0;
	return p;
}

static void host_free(void *p) {
	if (g_free && p) g_free(p);
}

static int host_register(char *name, char *params, MarqdoPluginFn fn) {
	if (!g_register_fn) return 1;
	return g_register_fn(g_userdata, name, params, fn);
}

static int host_query(char *name, char *args_json, char **out_json, char **err_msg) {
	if (!g_host_query) return 1;
	return g_host_query(g_userdata, name, args_json, out_json, err_msg);
}

extern int web_go_ready(char *args_json, char **out_json, char **err_msg);
extern int web_page_new(char *args_json, char **out_json, char **err_msg);

static int register_core(void) {
	if (host_register((char *)"web_go_ready", (char *)"", web_go_ready) != 0) return 1;
	if (host_register((char *)"web_page_new", (char *)"title,intro,shell_css,layout,asset_version", web_page_new) != 0) return 1;
	return 0;
}
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"

	"github.com/marqdo/marqdo/plugins/web/internal/page"
	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
)

func main() {}

//export marqdo_plugin_abi_version
func marqdo_plugin_abi_version() C.uint32_t {
	return 2
}

//export marqdo_plugin_init
func marqdo_plugin_init(host *C.MarqdoHostApi) C.int {
	if host == nil {
		return 1
	}
	C.host_save(host)
	plugin.SetHost(plugin.HostFns{Query: hostQueryJSON})
	return C.register_core()
}

//export marqdo_plugin_shutdown
func marqdo_plugin_shutdown() {
	plugin.Shutdown()
}

func hostQueryJSON(name, argsJSON string) (string, error) {
	cname := C.CString(name)
	cargs := C.CString(argsJSON)
	defer C.free(unsafe.Pointer(cname))
	defer C.free(unsafe.Pointer(cargs))
	var out, errMsg *C.char
	rc := C.host_query(cname, cargs, &out, &errMsg)
	outGo := cstrTakeHost(out)
	errGo := cstrTakeHost(errMsg)
	if rc != 0 {
		if errGo == "" {
			errGo = "host_query failed"
		}
		return "", fmt.Errorf("%s", errGo)
	}
	if outGo == "" {
		outGo = "null"
	}
	return outGo, nil
}

func cstrTakeHost(p *C.char) string {
	if p == nil {
		return ""
	}
	s := C.GoString(p)
	C.host_free(unsafe.Pointer(p))
	return s
}

func hostDup(s string) *C.char {
	cs := C.CString(s)
	defer C.free(unsafe.Pointer(cs))
	return C.host_strdup(cs)
}

func replyJSON(outJSON **C.char, errMsg **C.char, v any, err error) C.int {
	if err != nil {
		if errMsg != nil {
			*errMsg = hostDup(err.Error())
		}
		return 1
	}
	b, e := json.Marshal(v)
	if e != nil {
		if errMsg != nil {
			*errMsg = hostDup(e.Error())
		}
		return 1
	}
	if outJSON != nil {
		*outJSON = hostDup(string(b))
	}
	return 0
}

func parseArgs(argsJSON *C.char) (map[string]any, error) {
	if argsJSON == nil {
		return map[string]any{}, nil
	}
	s := C.GoString(argsJSON)
	if s == "" {
		return map[string]any{}, nil
	}
	var m map[string]any
	if err := json.Unmarshal([]byte(s), &m); err != nil {
		return nil, err
	}
	if m == nil {
		m = map[string]any{}
	}
	return m, nil
}

//export web_go_ready
func web_go_ready(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	_ = argsJSON
	return replyJSON(outJSON, errMsg, map[string]any{
		"ok":   true,
		"impl": "go",
		"abi":  2,
	}, nil)
}

//export web_page_new
func web_page_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, page.New(args), nil)
}
