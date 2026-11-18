package main

/*
#cgo CFLAGS: -I${SRCDIR}/abi
#include "marqdo_abi_cgo.h"

extern int web_storage_new(char *args_json, char **out_json, char **err_msg);
extern int web_storage_put(char *args_json, char **out_json, char **err_msg);
extern int web_storage_get(char *args_json, char **out_json, char **err_msg);
extern int web_storage_delete(char *args_json, char **out_json, char **err_msg);
extern int web_storage_list(char *args_json, char **out_json, char **err_msg);
extern int web_media_new(char *args_json, char **out_json, char **err_msg);
extern int web_upload_validate(char *args_json, char **out_json, char **err_msg);
extern int web_upload_save(char *args_json, char **out_json, char **err_msg);
extern int web_app_upload(char *args_json, char **out_json, char **err_msg);
extern int web_app_download(char *args_json, char **out_json, char **err_msg);
extern int web_app_gallery(char *args_json, char **out_json, char **err_msg);
extern int web_page_meta(char *args_json, char **out_json, char **err_msg);
extern int web_page_head(char *args_json, char **out_json, char **err_msg);
extern int web_page_images(char *args_json, char **out_json, char **err_msg);
extern int web_head(char *args_json, char **out_json, char **err_msg);
extern int web_images(char *args_json, char **out_json, char **err_msg);
extern int web_app_icons(char *args_json, char **out_json, char **err_msg);
*/
import "C"

import (
	"fmt"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/assets"
	"github.com/marqdo/marqdo/plugins/web/internal/storage"
	"github.com/marqdo/marqdo/plugins/web/internal/upload"
)

func storageURLOf(args map[string]any, keys ...string) (string, error) {
	for _, k := range keys {
		if v, ok := args[k]; ok {
			if s, err := app.StorageURL(v); err == nil {
				return s, nil
			}
		}
	}
	return "", fmt.Errorf("missing storage url")
}

//export web_storage_new
func web_storage_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, _ := argStr(args, "url", "地址")
	if url == "" {
		url = "file:data/blobs"
	}
	out, err := storage.Open(url)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_storage_put
func web_storage_put(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
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
	var body, path, ct *string
	if s, ok := argStr(args, "body", "正文"); ok {
		body = &s
	}
	if s, ok := argStr(args, "path", "路径"); ok {
		path = &s
	}
	if s, ok := argStr(args, "content_type", "类型"); ok {
		ct = &s
	}
	out, err := storage.Put(url, key, body, path, ct)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_storage_get
func web_storage_get(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
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
	out, err := storage.Get(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_storage_delete
func web_storage_delete(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
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
	out, err := storage.Delete(url, key)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_storage_list
func web_storage_list(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := argStrReq(args, "url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	prefix, _ := argStr(args, "prefix", "前缀")
	out, err := storage.List(url, prefix)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_media_new
func web_media_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	st := args["storage"]
	if st == nil {
		st = args["存储"]
	}
	return replyJSON(outJSON, errMsg, map[string]any{"_type": "media", "storage": st}, nil)
}

//export web_upload_validate
func web_upload_validate(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	filename, err := argStrReq(args, "filename", "文件名")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	ct, _ := argStr(args, "content_type", "类型")
	if ct == "" {
		ct = "application/octet-stream"
	}
	size := argUint64(args, 0, "size", "大小")
	if size == 0 {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `size`"))
	}
	maxBytes := argUint64(args, 5_242_880, "max_bytes", "最大字节")
	types := args["types"]
	if types == nil {
		types = args["允许"]
	}
	out := upload.Validate(filename, ct, size, maxBytes, types)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_upload_save
func web_upload_save(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	storageURL, err := storageURLOf(args, "storage", "url", "storage_url")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, err := argStrReq(args, "path", "路径")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	var key, ct, prefix *string
	if s, ok := argStr(args, "key", "键"); ok && s != "" {
		key = &s
	}
	if s, ok := argStr(args, "content_type", "类型"); ok {
		ct = &s
	}
	if s, ok := argStr(args, "prefix", "前缀"); ok {
		prefix = &s
	}
	out, err := upload.Save(storageURL, key, path, ct, prefix)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_upload
func web_app_upload(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, _ := argStr(args, "path", "路径")
	field, _ := argStr(args, "field")
	storageURL, err := storageURLOf(args, "storage", "存储")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	prefix, _ := argStr(args, "prefix", "前缀")
	maxBytes := argUint64(args, 5_242_880, "max_bytes", "最大字节")
	types := args["types"]
	if types == nil {
		types = args["允许"]
	}
	if types == nil {
		types = nil // explicit null ok
	}
	out, err := app.Upload(asPageMap(args["app"]), path, field, storageURL, prefix, maxBytes, types)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_download
func web_app_download(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, _ := argStr(args, "path", "路径")
	storageURL, err := storageURLOf(args, "storage", "存储")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	disposition, _ := argStr(args, "disposition")
	out, err := app.Download(asPageMap(args["app"]), path, storageURL, disposition)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_gallery
func web_app_gallery(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	path, _ := argStr(args, "path", "路径")
	storageURL, err := storageURLOf(args, "storage", "存储")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	prefix, _ := argStr(args, "prefix", "前缀")
	title, _ := argStr(args, "title", "标题")
	downloadBase, _ := argStr(args, "download_base", "media", "下载基址")
	out, err := app.Gallery(asPageMap(args["app"]), path, storageURL, prefix, title, downloadBase)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_page_meta
func web_page_meta(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	meta := args["meta"]
	if meta == nil {
		meta = args["table"]
	}
	if meta == nil {
		meta = args["元数据"]
	}
	out := assets.AttachMeta(pageBag, meta)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_head
func web_page_head(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	table := args["table"]
	if table == nil {
		table = args["head"]
	}
	if table == nil {
		table = args["表"]
	}
	out := assets.AttachHead(pageBag, table)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_page_images
func web_page_images(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	table := args["table"]
	if table == nil {
		table = args["images"]
	}
	out := assets.AttachImages(pageBag, table)
	return replyJSON(outJSON, errMsg, out, nil)
}

//export web_head
func web_head(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	table := args["table"]
	if table == nil {
		table = args["head"]
	}
	if table == nil {
		table = args["表"]
	}
	html := assets.MakeHeadHTML(table)
	return replyJSON(outJSON, errMsg, html, nil)
}

//export web_images
func web_images(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	table := args["table"]
	if table == nil {
		table = args["images"]
	}
	html := assets.MakeImagesHTML(table)
	return replyJSON(outJSON, errMsg, html, nil)
}

//export web_app_icons
func web_app_icons(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	table := args["table"]
	if table == nil {
		table = args["icons"]
	}
	out, err := app.Icons(asPageMap(args["app"]), table)
	return replyJSON(outJSON, errMsg, out, err)
}
