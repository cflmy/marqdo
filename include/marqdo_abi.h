/* Marqdo plugin ABI v1 — C contract (source of truth).
 *
 * Plugins are optional shared libraries (.dll / .so / .dylib), never linked
 * into the marqdo binary. Values cross the boundary as UTF-8 JSON.
 */
#ifndef MARQDO_ABI_H
#define MARQDO_ABI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MARQDO_ABI_VERSION 1u

/* Plugin function: args_json is a UTF-8 JSON object whose keys are param names.
 * On success: return 0, set *out_json to a heap string (host frees via free).
 * On failure: return non-zero; optional *err_msg heap string (host frees).
 * out_json / err_msg may be left NULL.
 */
typedef int (*MarqdoPluginFn)(const char *args_json, char **out_json, char **err_msg);

typedef struct MarqdoHostApi {
    void *userdata;
    /* params: comma-separated param names in order, e.g. "a,b". May be "". */
    int (*register_fn)(void *userdata, const char *name, const char *params,
                       MarqdoPluginFn fn);
    void *(*alloc)(size_t n);
    void (*free)(void *p);
} MarqdoHostApi;

/* Required exports from every plugin shared library. */
uint32_t marqdo_plugin_abi_version(void);
int marqdo_plugin_init(const MarqdoHostApi *host);
void marqdo_plugin_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif /* MARQDO_ABI_H */
