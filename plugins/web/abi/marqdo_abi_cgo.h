/* Go/cgo local ABI view — same layout as include/marqdo_abi.h.
 * cgo //export cannot emit `const` on parameters; this header matches
 * the generated prototypes so -buildmode=c-shared links cleanly.
 * Wire format & semantics remain ABI v2 (see include/marqdo_abi.h).
 */
#ifndef MARQDO_ABI_CGO_H
#define MARQDO_ABI_CGO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MARQDO_ABI_VERSION 2u
#define MARQDO_ABI_VERSION_MIN 1u

typedef int (*MarqdoPluginFn)(char *args_json, char **out_json, char **err_msg);

typedef int (*MarqdoHostQueryFn)(void *userdata, char *name, char *args_json,
                                 char **out_json, char **err_msg);

typedef struct MarqdoHostApi {
    void *userdata;
    int (*register_fn)(void *userdata, char *name, char *params, MarqdoPluginFn fn);
    void *(*alloc)(size_t n);
    void (*free)(void *p);
    MarqdoHostQueryFn host_query;
} MarqdoHostApi;

uint32_t marqdo_plugin_abi_version(void);
int marqdo_plugin_init(MarqdoHostApi *host);
void marqdo_plugin_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif /* MARQDO_ABI_CGO_H */
