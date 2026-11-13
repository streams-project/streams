/* The Node-API surface `../../napi_bridge.c` uses, so that the bridge can be built and run on a
 * machine with no HarmonyOS SDK.
 *
 * ## Where these declarations come from
 *
 * Not from memory. Every type and every function below was copied from the upstream Node-API
 * headers, downloaded and read on 2026-11-13:
 *
 *   js_native_api_types.h  napi_env, napi_value, napi_callback_info, napi_callback, napi_status,
 *                          napi_property_attributes, napi_property_descriptor
 *   js_native_api.h        napi_get_undefined, napi_create_int32, napi_create_int64,
 *                          napi_create_string_utf8, napi_get_value_int64,
 *                          napi_get_value_string_utf8, napi_get_cb_info, napi_throw_error,
 *                          napi_define_properties
 *   node_api.h             napi_module, napi_addon_register_func, napi_module_register
 *
 * from https://raw.githubusercontent.com/nodejs/node-api-headers/main/include/. The field order of
 * `napi_property_descriptor` and of `napi_module` matters here, because `napi_bridge.c` fills both
 * positionally, and both were checked field for field against those files.
 *
 * ## What it is not
 *
 * **ASSUMED**: that OpenHarmony's own `napi/native_api.h` declares these the same way. It is
 * Node-API, and OpenHarmony's `arkui_napi` publishes it as such, but that header is not on this
 * machine and was not read, so a difference between it and this file would let every test beside it
 * pass while the real build fails. The first build against a real SDK is what settles that, and it
 * is the one thing this folder cannot settle itself. See `OPEN-QUESTIONS.md` Q1.
 *
 * **This folder must never be on the include path of a real build.** It is reached only by
 * `apps/harmony/check.sh`, which passes `-I apps/harmony/core/cpp/test`; `CMakeLists.txt` names
 * `napi_bridge.c` alone and adds only `platform/bridges/c`, so nothing here can reach a device. A
 * header with this name earlier on a real include path would shadow the SDK's own and the library
 * would build against a fake.
 *
 * Only what the bridge calls is here. There is no handle scope, no reference, no async work, no
 * object, no array and no exception machinery beyond `napi_throw_error`, because the bridge uses
 * none of them. A declaration nobody calls would be a claim nobody checks.
 */
#ifndef STREAMS_HARMONY_TEST_NAPI_NATIVE_API_H
#define STREAMS_HARMONY_TEST_NAPI_NATIVE_API_H

#include <stddef.h>
#include <stdint.h>

/* The upstream header defines both of these; on this machine they are empty, which keeps the
 * declarations below the same shape as the ones they were copied from. */
#define NAPI_CDECL
#define NAPI_EXTERN extern

#ifdef __cplusplus
extern "C" {
#endif

typedef struct napi_env__ *napi_env;
typedef struct napi_value__ *napi_value;
typedef struct napi_callback_info__ *napi_callback_info;

typedef enum {
    napi_default = 0,
    napi_writable = 1 << 0,
    napi_enumerable = 1 << 1,
    napi_configurable = 1 << 2,
    napi_static = 1 << 10,
    napi_default_method = napi_writable | napi_configurable,
    napi_default_jsproperty = napi_writable | napi_enumerable | napi_configurable,
} napi_property_attributes;

typedef enum {
    napi_ok,
    napi_invalid_arg,
    napi_object_expected,
    napi_string_expected,
    napi_name_expected,
    napi_function_expected,
    napi_number_expected,
    napi_boolean_expected,
    napi_array_expected,
    napi_generic_failure,
    napi_pending_exception,
    napi_cancelled,
    napi_escape_called_twice,
    napi_handle_scope_mismatch,
    napi_callback_scope_mismatch,
    napi_queue_full,
    napi_closing,
    napi_bigint_expected,
    napi_date_expected,
    napi_arraybuffer_expected,
    napi_detachable_arraybuffer_expected,
    napi_would_deadlock,
    napi_no_external_buffers_allowed,
    napi_cannot_run_js,
} napi_status;

typedef napi_value(NAPI_CDECL *napi_callback)(napi_env env, napi_callback_info info);

typedef struct {
    /* One of utf8name or name should be NULL. */
    const char *utf8name;
    napi_value name;

    napi_callback method;
    napi_callback getter;
    napi_callback setter;
    napi_value value;

    napi_property_attributes attributes;
    void *data;
} napi_property_descriptor;

NAPI_EXTERN napi_status NAPI_CDECL napi_get_undefined(napi_env env, napi_value *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_create_int32(napi_env env, int32_t value,
                                                     napi_value *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_create_int64(napi_env env, int64_t value,
                                                     napi_value *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_create_string_utf8(napi_env env, const char *str,
                                                           size_t length, napi_value *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_get_value_int64(napi_env env, napi_value value,
                                                        int64_t *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_get_value_string_utf8(napi_env env, napi_value value,
                                                              char *buf, size_t bufsize,
                                                              size_t *result);

NAPI_EXTERN napi_status NAPI_CDECL napi_get_cb_info(napi_env env, napi_callback_info cbinfo,
                                                    size_t *argc, napi_value *argv,
                                                    napi_value *this_arg, void **data);

NAPI_EXTERN napi_status NAPI_CDECL napi_throw_error(napi_env env, const char *code,
                                                    const char *msg);

NAPI_EXTERN napi_status NAPI_CDECL napi_define_properties(napi_env env, napi_value object,
                                                          size_t property_count,
                                                          const napi_property_descriptor *properties);

typedef napi_value(NAPI_CDECL *napi_addon_register_func)(napi_env env, napi_value exports);

typedef struct napi_module {
    int nm_version;
    unsigned int nm_flags;
    const char *nm_filename;
    napi_addon_register_func nm_register_func;
    const char *nm_modname;
    void *nm_priv;
    void *reserved[4];
} napi_module;

NAPI_EXTERN void NAPI_CDECL napi_module_register(napi_module *mod);

#ifdef __cplusplus
}
#endif

#endif /* STREAMS_HARMONY_TEST_NAPI_NATIVE_API_H */
