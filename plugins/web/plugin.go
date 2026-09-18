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
extern int web_page_query(char *args_json, char **out_json, char **err_msg);
extern int web_page_order(char *args_json, char **out_json, char **err_msg);
extern int web_page_link_prefix(char *args_json, char **out_json, char **err_msg);
extern int web_page_css(char *args_json, char **out_json, char **err_msg);
extern int web_page_detail(char *args_json, char **out_json, char **err_msg);
extern int web_page_chrome(char *args_json, char **out_json, char **err_msg);
extern int web_style(char *args_json, char **out_json, char **err_msg);
extern int web_compose_components(char *args_json, char **out_json, char **err_msg);
extern int web_compose_main(char *args_json, char **out_json, char **err_msg);
extern int web_compose_list(char *args_json, char **out_json, char **err_msg);
extern int web_compose_intro(char *args_json, char **out_json, char **err_msg);
extern int web_intro(char *args_json, char **out_json, char **err_msg);
extern int web_render(char *args_json, char **out_json, char **err_msg);
extern int web_db_new(char *args_json, char **out_json, char **err_msg);
extern int web_db_init(char *args_json, char **out_json, char **err_msg);
extern int web_db_insert(char *args_json, char **out_json, char **err_msg);
extern int web_db_select(char *args_json, char **out_json, char **err_msg);
extern int web_db_get(char *args_json, char **out_json, char **err_msg);
extern int web_db_exec(char *args_json, char **out_json, char **err_msg);
extern int web_form_new(char *args_json, char **out_json, char **err_msg);
extern int web_form_fields(char *args_json, char **out_json, char **err_msg);
extern int web_compose_form(char *args_json, char **out_json, char **err_msg);
	extern int web_compose_auth_form(char *args_json, char **out_json, char **err_msg);
	extern int web_compose_nav_brand(char *args_json, char **out_json, char **err_msg);
	extern int web_compose_client(char *args_json, char **out_json, char **err_msg);
extern int web_app_new(char *args_json, char **out_json, char **err_msg);
extern int web_app_route(char *args_json, char **out_json, char **err_msg);
extern int web_db_update(char *args_json, char **out_json, char **err_msg);
extern int web_db_delete(char *args_json, char **out_json, char **err_msg);
extern int web_db_query(char *args_json, char **out_json, char **err_msg);
extern int web_db_count(char *args_json, char **out_json, char **err_msg);
extern int web_db_begin(char *args_json, char **out_json, char **err_msg);
extern int web_db_commit(char *args_json, char **out_json, char **err_msg);
extern int web_db_rollback(char *args_json, char **out_json, char **err_msg);
extern int web_db_migrate(char *args_json, char **out_json, char **err_msg);
extern int web_db_fts_create(char *args_json, char **out_json, char **err_msg);
extern int web_db_search(char *args_json, char **out_json, char **err_msg);
extern int web_db_table_info(char *args_json, char **out_json, char **err_msg);
	extern int web_form_rules(char *args_json, char **out_json, char **err_msg);
	extern int web_form_labels(char *args_json, char **out_json, char **err_msg);
extern int web_form_validate(char *args_json, char **out_json, char **err_msg);
extern int web_form_render(char *args_json, char **out_json, char **err_msg);
extern int web_form_submit(char *args_json, char **out_json, char **err_msg);
extern int web_form_from_schema(char *args_json, char **out_json, char **err_msg);
extern int web_app_mount_form(char *args_json, char **out_json, char **err_msg);
extern int web_app_static(char *args_json, char **out_json, char **err_msg);
extern int web_app_middleware(char *args_json, char **out_json, char **err_msg);
extern int web_listen(char *args_json, char **out_json, char **err_msg);
extern int web_password_hash(char *args_json, char **out_json, char **err_msg);
extern int web_session_new(char *args_json, char **out_json, char **err_msg);
extern int web_session_set(char *args_json, char **out_json, char **err_msg);
extern int web_session_get(char *args_json, char **out_json, char **err_msg);
extern int web_session_del(char *args_json, char **out_json, char **err_msg);
extern int web_session_destroy(char *args_json, char **out_json, char **err_msg);
extern int web_auth_login(char *args_json, char **out_json, char **err_msg);
extern int web_auth_check(char *args_json, char **out_json, char **err_msg);
extern int web_auth_logout(char *args_json, char **out_json, char **err_msg);
extern int web_auth_new(char *args_json, char **out_json, char **err_msg);
extern int web_app_auth(char *args_json, char **out_json, char **err_msg);
extern int web_app_gate(char *args_json, char **out_json, char **err_msg);
extern int web_app_rbac(char *args_json, char **out_json, char **err_msg);
extern int web_app_tenant(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_can(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_ensure(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_assign_role(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_create_role(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_set_role_permissions(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_list_roles(char *args_json, char **out_json, char **err_msg);
extern int web_rbac_list_permissions(char *args_json, char **out_json, char **err_msg);
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
extern int web_page_paginate(char *args_json, char **out_json, char **err_msg);
extern int web_rss_build(char *args_json, char **out_json, char **err_msg);
extern int web_app_route_rss(char *args_json, char **out_json, char **err_msg);
extern int web_app_redirect(char *args_json, char **out_json, char **err_msg);
extern int web_app_error_page(char *args_json, char **out_json, char **err_msg);
extern int web_app_sitemap(char *args_json, char **out_json, char **err_msg);
extern int web_app_robots(char *args_json, char **out_json, char **err_msg);
extern int web_sitemap_build(char *args_json, char **out_json, char **err_msg);
extern int web_app_route_ws(char *args_json, char **out_json, char **err_msg);
extern int web_ws_connect(char *args_json, char **out_json, char **err_msg);
extern int web_cache_new(char *args_json, char **out_json, char **err_msg);
extern int web_cache_get(char *args_json, char **out_json, char **err_msg);
extern int web_cache_set(char *args_json, char **out_json, char **err_msg);
extern int web_cache_del(char *args_json, char **out_json, char **err_msg);
extern int web_cache_exists(char *args_json, char **out_json, char **err_msg);
extern int web_cache_ttl(char *args_json, char **out_json, char **err_msg);
extern int web_app_proxy(char *args_json, char **out_json, char **err_msg);
extern int web_app_invoke(char *args_json, char **out_json, char **err_msg);
extern int web_api_key_check(char *args_json, char **out_json, char **err_msg);

static int register_core(void) {
	if (host_register((char *)"web_go_ready", (char *)"", web_go_ready) != 0) return 1;
	if (host_register((char *)"web_page_new", (char *)"title,intro,shell_css,layout,asset_version", web_page_new) != 0) return 1;
	if (host_register((char *)"web_page_query", (char *)"page,query", web_page_query) != 0) return 1;
	if (host_register((char *)"web_page_order", (char *)"page,order", web_page_order) != 0) return 1;
	if (host_register((char *)"web_page_link_prefix", (char *)"page,prefix", web_page_link_prefix) != 0) return 1;
	if (host_register((char *)"web_page_css", (char *)"page,css", web_page_css) != 0) return 1;
	if (host_register((char *)"web_page_detail", (char *)"page,detail", web_page_detail) != 0) return 1;
	if (host_register((char *)"web_page_chrome", (char *)"page,nav_html,footer_html,body_class", web_page_chrome) != 0) return 1;
	if (host_register((char *)"web_style", (char *)"name,table,strict", web_style) != 0) return 1;
	if (host_register((char *)"web_compose_components", (char *)"page,components", web_compose_components) != 0) return 1;
	if (host_register((char *)"web_compose_main", (char *)"page,main", web_compose_main) != 0) return 1;
	if (host_register((char *)"web_compose_list", (char *)"page,main,query,order,target", web_compose_list) != 0) return 1;
	if (host_register((char *)"web_compose_intro", (char *)"page,intro", web_compose_intro) != 0) return 1;
	if (host_register((char *)"web_intro", (char *)"table", web_intro) != 0) return 1;
	if (host_register((char *)"web_compose_form", (char *)"page,form,id,target", web_compose_form) != 0) return 1;
	if (host_register((char *)"web_compose_auth_form", (char *)"page,action,submit,form_id,err_id,target,next,kind", web_compose_auth_form) != 0) return 1;
	if (host_register((char *)"web_compose_nav_brand", (char *)"page,title,href,logo,logo_light,theme_key", web_compose_nav_brand) != 0) return 1;
	if (host_register((char *)"web_compose_client", (char *)"page,source,bridge,wasm", web_compose_client) != 0) return 1;
	if (host_register((char *)"web_render", (char *)"page", web_render) != 0) return 1;
	if (host_register((char *)"web_db_new", (char *)"url", web_db_new) != 0) return 1;
	if (host_register((char *)"web_db_init", (char *)"url,name,fields", web_db_init) != 0) return 1;
	if (host_register((char *)"web_db_insert", (char *)"url,table,rows,txn", web_db_insert) != 0) return 1;
	if (host_register((char *)"web_db_select", (char *)"url,table,where,limit,order,offset,txn", web_db_select) != 0) return 1;
	if (host_register((char *)"web_db_get", (char *)"url,table,id,txn", web_db_get) != 0) return 1;
	if (host_register((char *)"web_db_update", (char *)"url,table,id,row,txn", web_db_update) != 0) return 1;
	if (host_register((char *)"web_db_delete", (char *)"url,table,id,txn", web_db_delete) != 0) return 1;
	if (host_register((char *)"web_db_exec", (char *)"url,sql,args,txn", web_db_exec) != 0) return 1;
	if (host_register((char *)"web_db_query", (char *)"url,sql,args,txn", web_db_query) != 0) return 1;
	if (host_register((char *)"web_db_count", (char *)"url,table,where,txn", web_db_count) != 0) return 1;
	if (host_register((char *)"web_db_begin", (char *)"url", web_db_begin) != 0) return 1;
	if (host_register((char *)"web_db_commit", (char *)"txn", web_db_commit) != 0) return 1;
	if (host_register((char *)"web_db_rollback", (char *)"txn", web_db_rollback) != 0) return 1;
	if (host_register((char *)"web_db_migrate", (char *)"url,steps", web_db_migrate) != 0) return 1;
	if (host_register((char *)"web_db_fts_create", (char *)"url,table,columns,name", web_db_fts_create) != 0) return 1;
	if (host_register((char *)"web_db_search", (char *)"url,table,q,limit,name", web_db_search) != 0) return 1;
	if (host_register((char *)"web_db_table_info", (char *)"url,table", web_db_table_info) != 0) return 1;
	if (host_register((char *)"web_form_new", (char *)"table,action,id", web_form_new) != 0) return 1;
	if (host_register((char *)"web_form_fields", (char *)"form,fields", web_form_fields) != 0) return 1;
	if (host_register((char *)"web_form_rules", (char *)"form,rules", web_form_rules) != 0) return 1;
	if (host_register((char *)"web_form_labels", (char *)"form,submit,cancel,cancel_href", web_form_labels) != 0) return 1;
	if (host_register((char *)"web_form_validate", (char *)"form,rules,data", web_form_validate) != 0) return 1;
	if (host_register((char *)"web_form_render", (char *)"form,id", web_form_render) != 0) return 1;
	if (host_register((char *)"web_form_submit", (char *)"form,data,url", web_form_submit) != 0) return 1;
	if (host_register((char *)"web_form_from_schema", (char *)"url,table,action", web_form_from_schema) != 0) return 1;
	if (host_register((char *)"web_app_new", (char *)"page,db,admin,host,port,admin_prefix,login_redirect,logout_redirect,shell_css,layout,asset_version", web_app_new) != 0) return 1;
	if (host_register((char *)"web_app_route", (char *)"app,path,page", web_app_route) != 0) return 1;
	if (host_register((char *)"web_app_mount_form", (char *)"app,id,form", web_app_mount_form) != 0) return 1;
	if (host_register((char *)"web_app_static", (char *)"app,dir,mount", web_app_static) != 0) return 1;
	if (host_register((char *)"web_app_middleware", (char *)"app,cors,security,compress,body_limit,json_routes,access_log,cache_control,proxy,invoke", web_app_middleware) != 0) return 1;
	if (host_register((char *)"web_listen", (char *)"app", web_listen) != 0) return 1;
	if (host_register((char *)"web_password_hash", (char *)"password", web_password_hash) != 0) return 1;
	if (host_register((char *)"web_session_new", (char *)"ttl_sec", web_session_new) != 0) return 1;
	if (host_register((char *)"web_session_set", (char *)"id,key,value", web_session_set) != 0) return 1;
	if (host_register((char *)"web_session_get", (char *)"id,key", web_session_get) != 0) return 1;
	if (host_register((char *)"web_session_del", (char *)"id,key", web_session_del) != 0) return 1;
	if (host_register((char *)"web_session_destroy", (char *)"id", web_session_destroy) != 0) return 1;
	if (host_register((char *)"web_auth_login", (char *)"username,password,users,session_ttl", web_auth_login) != 0) return 1;
	if (host_register((char *)"web_auth_check", (char *)"session_id", web_auth_check) != 0) return 1;
	if (host_register((char *)"web_auth_logout", (char *)"session_id", web_auth_logout) != 0) return 1;
	if (host_register((char *)"web_auth_new", (char *)"users,session_ttl", web_auth_new) != 0) return 1;
	if (host_register((char *)"web_app_auth", (char *)"app,users,session_ttl,admin_prefix,login_redirect,logout_redirect,login_path,register,register_path,default_role", web_app_auth) != 0) return 1;
	if (host_register((char *)"web_app_gate", (char *)"app,path,roles,permissions,match,on_deny,exclude", web_app_gate) != 0) return 1;
	if (host_register((char *)"web_app_rbac", (char *)"app,catalog", web_app_rbac) != 0) return 1;
	if (host_register((char *)"web_app_tenant", (char *)"app,mode,param,column,default_scope", web_app_tenant) != 0) return 1;
	if (host_register((char *)"web_rbac_can", (char *)"permissions,needed", web_rbac_can) != 0) return 1;
	if (host_register((char *)"web_rbac_ensure", (char *)"url", web_rbac_ensure) != 0) return 1;
	if (host_register((char *)"web_rbac_assign_role", (char *)"url,username,role", web_rbac_assign_role) != 0) return 1;
	if (host_register((char *)"web_rbac_create_role", (char *)"url,name", web_rbac_create_role) != 0) return 1;
	if (host_register((char *)"web_rbac_set_role_permissions", (char *)"url,role,permissions,grantable", web_rbac_set_role_permissions) != 0) return 1;
	if (host_register((char *)"web_rbac_list_roles", (char *)"url", web_rbac_list_roles) != 0) return 1;
	if (host_register((char *)"web_rbac_list_permissions", (char *)"url", web_rbac_list_permissions) != 0) return 1;
	if (host_register((char *)"web_storage_new", (char *)"url", web_storage_new) != 0) return 1;
	if (host_register((char *)"web_storage_put", (char *)"url,key,body,path,content_type", web_storage_put) != 0) return 1;
	if (host_register((char *)"web_storage_get", (char *)"url,key", web_storage_get) != 0) return 1;
	if (host_register((char *)"web_storage_delete", (char *)"url,key", web_storage_delete) != 0) return 1;
	if (host_register((char *)"web_storage_list", (char *)"url,prefix", web_storage_list) != 0) return 1;
	if (host_register((char *)"web_media_new", (char *)"storage", web_media_new) != 0) return 1;
	if (host_register((char *)"web_upload_validate", (char *)"filename,content_type,size,max_bytes,types", web_upload_validate) != 0) return 1;
	if (host_register((char *)"web_upload_save", (char *)"storage,path,key,content_type,prefix", web_upload_save) != 0) return 1;
	if (host_register((char *)"web_app_upload", (char *)"app,path,field,storage,prefix,max_bytes,types", web_app_upload) != 0) return 1;
	if (host_register((char *)"web_app_download", (char *)"app,path,storage,disposition", web_app_download) != 0) return 1;
	if (host_register((char *)"web_app_gallery", (char *)"app,path,storage,prefix,title,download_base", web_app_gallery) != 0) return 1;
	if (host_register((char *)"web_page_meta", (char *)"page,meta", web_page_meta) != 0) return 1;
	if (host_register((char *)"web_page_head", (char *)"page,table", web_page_head) != 0) return 1;
	if (host_register((char *)"web_page_images", (char *)"page,table", web_page_images) != 0) return 1;
	if (host_register((char *)"web_head", (char *)"table", web_head) != 0) return 1;
	if (host_register((char *)"web_images", (char *)"table", web_images) != 0) return 1;
	if (host_register((char *)"web_app_icons", (char *)"app,table", web_app_icons) != 0) return 1;
	if (host_register((char *)"web_page_paginate", (char *)"page,offset,limit,path", web_page_paginate) != 0) return 1;
	if (host_register((char *)"web_rss_build", (char *)"title,link,description,items", web_rss_build) != 0) return 1;
	if (host_register((char *)"web_app_route_rss", (char *)"app,path,table,limit,order,title,link,description", web_app_route_rss) != 0) return 1;
	if (host_register((char *)"web_app_redirect", (char *)"app,from,to,permanent", web_app_redirect) != 0) return 1;
	if (host_register((char *)"web_app_error_page", (char *)"app,status,page", web_app_error_page) != 0) return 1;
	if (host_register((char *)"web_app_sitemap", (char *)"app,path,base,table,loc,limit,items", web_app_sitemap) != 0) return 1;
	if (host_register((char *)"web_app_robots", (char *)"app,body,sitemap", web_app_robots) != 0) return 1;
	if (host_register((char *)"web_sitemap_build", (char *)"base,items", web_sitemap_build) != 0) return 1;
	if (host_register((char *)"web_app_route_ws", (char *)"app,path,echo,mode,room_key,on_message,presence", web_app_route_ws) != 0) return 1;
	if (host_register((char *)"web_ws_connect", (char *)"url,message,headers,timeout_sec", web_ws_connect) != 0) return 1;
	if (host_register((char *)"web_cache_new", (char *)"url", web_cache_new) != 0) return 1;
	if (host_register((char *)"web_cache_get", (char *)"url,key", web_cache_get) != 0) return 1;
	if (host_register((char *)"web_cache_set", (char *)"url,key,value,ttl", web_cache_set) != 0) return 1;
	if (host_register((char *)"web_cache_del", (char *)"url,key", web_cache_del) != 0) return 1;
	if (host_register((char *)"web_cache_exists", (char *)"url,key", web_cache_exists) != 0) return 1;
	if (host_register((char *)"web_cache_ttl", (char *)"url,key", web_cache_ttl) != 0) return 1;
	if (host_register((char *)"web_app_proxy", (char *)"app,path,upstream,stream,strip_prefix,methods,headers_from_env,timeout_ms", web_app_proxy) != 0) return 1;
	if (host_register((char *)"web_app_invoke", (char *)"app,path,method,fn,body,return", web_app_invoke) != 0) return 1;
	if (host_register((char *)"web_api_key_check", (char *)"key,authorization,pepper,keys", web_api_key_check) != 0) return 1;
	return 0;
}
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"strconv"
	"strings"
	"unsafe"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/compose"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/page"
	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
	"github.com/marqdo/marqdo/plugins/web/internal/render"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
	"github.com/marqdo/marqdo/plugins/web/internal/style"
)

func main() {}

//export marqdo_plugin_abi_version
func marqdo_plugin_abi_version() C.uint32_t { return 2 }

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
	session.CloseRedis()
	db.ResetPool()
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

func argStr(args map[string]any, keys ...string) (string, bool) {
	for _, k := range keys {
		v, ok := args[k]
		if !ok || v == nil {
			continue
		}
		switch t := v.(type) {
		case string:
			return t, true
		case float64:
			if t == float64(int64(t)) {
				return strconv.FormatInt(int64(t), 10), true
			}
			return strconv.FormatFloat(t, 'f', -1, 64), true
		case bool:
			return strconv.FormatBool(t), true
		case json.Number:
			return t.String(), true
		}
	}
	return "", false
}

func argStrReq(args map[string]any, keys ...string) (string, error) {
	s, ok := argStr(args, keys...)
	if !ok || s == "" {
		return "", fmt.Errorf("missing `%s`", keys[0])
	}
	return s, nil
}

func entryDir() string {
	raw, err := plugin.Query("entry_dir", "{}")
	if err != nil {
		return "."
	}
	var s string
	if json.Unmarshal([]byte(raw), &s) == nil && s != "" {
		return s
	}
	return "."
}

func resolveDbURL(url string) string {
	lower := strings.ToLower(url)
	if strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://") {
		return url
	}
	stripped := url
	prefix := "sqlite:"
	switch {
	case strings.HasPrefix(url, "sqlite:"):
		stripped = strings.TrimPrefix(url, "sqlite:")
	case strings.HasPrefix(url, "SQLITE:"):
		stripped = strings.TrimPrefix(url, "SQLITE:")
		prefix = "SQLITE:"
	default:
		prefix = "sqlite:"
	}
	if filepath.IsAbs(stripped) {
		return prefix + stripped
	}
	abs := filepath.Join(entryDir(), stripped)
	return prefix + abs
}

func dbURLOf(args map[string]any) (string, error) {
	if s, ok := argStr(args, "url", "db_url", "数据库"); ok && s != "" {
		return resolveDbURL(s), nil
	}
	for _, key := range []string{"url", "db", "数据库"} {
		if dbObj, ok := args[key].(map[string]any); ok {
			if s, ok := argStr(dbObj, "url"); ok && s != "" {
				return resolveDbURL(s), nil
			}
		}
	}
	return "", fmt.Errorf("missing db url")
}

func callLib(path string) (any, error) {
	payload, _ := json.Marshal(map[string]any{"path": path})
	raw, err := plugin.Query("call_lib_path", string(payload))
	if err != nil {
		return nil, err
	}
	var v any
	if err := json.Unmarshal([]byte(raw), &v); err != nil {
		return nil, err
	}
	return v, nil
}

func asPageMap(v any) map[string]any {
	if m, ok := v.(map[string]any); ok {
		return m
	}
	return map[string]any{}
}

//export web_go_ready
func web_go_ready(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	_ = argsJSON
	return replyJSON(outJSON, errMsg, map[string]any{"ok": true, "impl": "go", "abi": 2}, nil)
}

//export web_page_new
func web_page_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, page.New(args), nil)
}

//export web_style
func web_style(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	css, err := style.Style(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, css, nil)
}

//export web_compose_components
func web_compose_components(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	components := args["components"]
	if components == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `components`"))
	}
	out, err := compose.ComposeComponents(pageBag, components, callLib)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_main
func web_compose_main(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	main := args["main"]
	if main == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `main`"))
	}
	out, err := compose.ComposeMain(pageBag, main, callLib)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_list
func web_compose_list(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	main := args["main"]
	if main == nil {
		main = args["主体"]
	}
	if main == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `main`"))
	}
	query := args["query"]
	if query == nil {
		query = args["条件"]
	}
	order, _ := argStr(args, "order", "排序")
	target, _ := argStr(args, "target", "插槽", "表单插槽")
	out, err := compose.ComposeList(pageBag, main, query, order, target, callLib)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_intro
func web_compose_intro(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	intro := args["intro"]
	if intro == nil {
		intro = args["引言"]
	}
	if intro == nil {
		intro = args["table"]
	}
	if intro == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `intro`"))
	}
	out, err := compose.ComposeIntro(pageBag, intro, callLib)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_intro
func web_intro(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tbl := args["table"]
	if tbl == nil {
		tbl = args["intro"]
	}
	if tbl == nil {
		tbl = args["表"]
	}
	if tbl == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `table`"))
	}
	return replyJSON(outJSON, errMsg, compose.MakeIntroHTML(tbl), nil)
}

//export web_render
func web_render(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := asPageMap(args["page"])
	if len(pageBag) == 0 {
		pageBag = args
	}
	dbURL, _ := dbURLOf(args)
	html := render.RenderPage(pageBag, dbURL, "")
	return replyJSON(outJSON, errMsg, html, nil)
}

//export web_db_new
func web_db_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, _ := argStr(args, "url")
	if url == "" {
		url = "sqlite:site.db"
	}
	return replyJSON(outJSON, errMsg, map[string]any{
		"url":   resolveDbURL(url),
		"_type": "db",
	}, nil)
}

//export web_db_init
func web_db_init(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	name, err := argStrReq(args, "name", "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	fields := args["fields"]
	if fields == nil {
		fields = []any{}
	}
	out, err := db.Init(url, name, fields)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_insert
func web_db_insert(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	rows := args["rows"]
	if rows == nil {
		rows = args["row"]
	}
	if rows == nil {
		rows = []any{}
	}
	out, err := db.Insert(url, tableName, rows, txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_select
func web_db_select(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	limit := int64(200)
	if v, ok := args["limit"]; ok {
		switch t := v.(type) {
		case float64:
			limit = int64(t)
		case string:
			if n, e := strconv.ParseInt(t, 10, 64); e == nil {
				limit = n
			}
		}
	}
	opts := db.SelectOpts{}
	if w, ok := args["where"]; ok && w != nil {
		if s, ok := w.(string); ok {
			if s != "" && !strings.EqualFold(s, "none") && !strings.EqualFold(s, "null") {
				opts.Where = w
			}
		} else {
			opts.Where = w
		}
	}
	if o, ok := argStr(args, "order"); ok {
		opts.Order = o
	}
	if v, ok := args["offset"]; ok && v != nil {
		switch t := v.(type) {
		case float64:
			n := int64(t)
			opts.Offset = &n
		case string:
			if n, e := strconv.ParseInt(t, 10, 64); e == nil {
				opts.Offset = &n
			}
		}
	} else if v, ok := args["跳过"]; ok && v != nil {
		if t, ok := v.(float64); ok {
			n := int64(t)
			opts.Offset = &n
		}
	}
	out, err := db.Select(url, tableName, limit, opts, txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_get
func web_db_get(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Get(url, tableName, id, txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_exec
func web_db_exec(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	sqlStmt, err := argStrReq(args, "sql")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Exec(url, sqlStmt, args["args"], txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_form_new
func web_form_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, _ := argStr(args, "table")
	action, _ := argStr(args, "action")
	id, _ := argStr(args, "id")
	return replyJSON(outJSON, errMsg, form.New(tableName, action, id), nil)
}

//export web_form_fields
func web_form_fields(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	formBag := asPageMap(args["form"])
	fields := args["fields"]
	if fields == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `fields`"))
	}
	return replyJSON(outJSON, errMsg, form.SetFields(formBag, fields), nil)
}

//export web_compose_form
func web_compose_form(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := args["page"]
	formV := args["form"]
	id, _ := argStr(args, "id")
	var target *string
	if t, ok := argStr(args, "target"); ok && t != "" {
		target = &t
	}
	out, err := compose.ComposeForm(pageBag, formV, id, target)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_auth_form
func web_compose_auth_form(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageBag := args["page"]
	action, _ := argStr(args, "action")
	if action == "" {
		action, _ = argStr(args, "动作")
	}
	submit, _ := argStr(args, "submit")
	if submit == "" {
		submit, _ = argStr(args, "提交")
	}
	formID, _ := argStr(args, "form_id")
	if formID == "" {
		formID, _ = argStr(args, "表单id")
	}
	errID, _ := argStr(args, "err_id")
	if errID == "" {
		errID, _ = argStr(args, "错误id")
	}
	target, _ := argStr(args, "target")
	if target == "" {
		target, _ = argStr(args, "表单插槽")
	}
	next, _ := argStr(args, "next")
	if next == "" {
		next, _ = argStr(args, "回跳")
	}
	kind, _ := argStr(args, "kind")
	if kind == "" {
		kind, _ = argStr(args, "种类")
	}
	out, err := compose.ComposeAuthForm(pageBag, action, submit, formID, errID, target, next, kind)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_nav_brand
func web_compose_nav_brand(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	title, _ := argStr(args, "title")
	if title == "" {
		title, _ = argStr(args, "标题")
	}
	href, _ := argStr(args, "href")
	if href == "" {
		href, _ = argStr(args, "链接")
	}
	logo, _ := argStr(args, "logo")
	if logo == "" {
		logo, _ = argStr(args, "标志")
	}
	logoLight, _ := argStr(args, "logo_light")
	if logoLight == "" {
		logoLight, _ = argStr(args, "浅色标志")
	}
	themeKey, _ := argStr(args, "theme_key")
	if themeKey == "" {
		themeKey, _ = argStr(args, "主题键")
	}
	out, err := compose.ComposeNavBrand(args["page"], title, href, logo, logoLight, themeKey)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_compose_client
func web_compose_client(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	source, _ := argStr(args, "source")
	if source == "" {
		source, _ = argStr(args, "源")
	}
	bridge, _ := argStr(args, "bridge")
	if bridge == "" {
		bridge, _ = argStr(args, "桥")
	}
	wasm, _ := argStr(args, "wasm")
	out, err := compose.ComposeClient(args["page"], bridge, wasm, source)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_app_new
func web_app_new(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	return replyJSON(outJSON, errMsg, app.New(args), nil)
}

//export web_app_route
func web_app_route(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	appBag := asPageMap(args["app"])
	path, err := argStrReq(args, "path")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	pageV := args["page"]
	if pageV == nil {
		return replyJSON(outJSON, errMsg, nil, fmt.Errorf("missing `page` for route"))
	}
	out, err := app.Route(appBag, path, pageV)
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_update
func web_db_update(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	row := args["row"]
	if row == nil {
		row = map[string]any{}
	}
	out, err := db.Update(url, tableName, id, row, txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_delete
func web_db_delete(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	id, err := argStrReq(args, "id")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Delete(url, tableName, id, txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_query
func web_db_query(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	sqlStmt, err := argStrReq(args, "sql")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Query(url, sqlStmt, args["args"], txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}

//export web_db_count
func web_db_count(argsJSON *C.char, outJSON **C.char, errMsg **C.char) C.int {
	args, err := parseArgs(argsJSON)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	url, err := dbURLOf(args)
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	tableName, err := argStrReq(args, "table")
	if err != nil {
		return replyJSON(outJSON, errMsg, nil, err)
	}
	out, err := db.Count(url, tableName, args["where"], txnOpt(args))
	return replyJSON(outJSON, errMsg, out, err)
}
