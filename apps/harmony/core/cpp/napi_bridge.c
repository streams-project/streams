/* The core's four functions, as ArkTS sees them. Bytes go in and bytes come out, and no rules live
 * here. The same job as apps/android/core/src/main/cpp/bridge.c, through Node-API instead of JNI.
 *
 * Text, not byte arrays: a Kotlin String is UTF-16, so the Android side passes ByteArray to avoid
 * converting twice. Node-API hands out UTF-8 directly, which is what the core reads, so these
 * carry strings and nothing is converted at all.
 *
 * No malloc: the one buffer this file needs is taken from core_alloc and given back to core_free,
 * which core.h already promises. So this file has no allocator of its own to get wrong.
 */
#include <stddef.h>
#include <stdint.h>

#include "napi/native_api.h"

#include "core.h"

/* A handle is a pointer, carried to ArkTS as a number. A number holds whole values up to 2^53 and
 * a user pointer on aarch64 or armv7 is smaller than that, so nothing is lost. If a future runtime
 * tags the top bits of a pointer this has to become a table of handles instead. */
static napi_value handle_of(napi_env env, CoreHandle *handle) {
    napi_value answer;
    napi_create_int64(env, (int64_t)(intptr_t)handle, &answer);
    return answer;
}

static CoreHandle *handle_in(napi_env env, napi_value given) {
    int64_t number = 0;
    napi_get_value_int64(env, given, &number);
    return (CoreHandle *)(intptr_t)number;
}

/* The bytes of one string argument, from core_alloc. Hands back NULL when it cannot be read.
 * `len` is how many bytes the core should read, and `room` is what core_free must be given back:
 * the two differ, and freeing a block by its content length instead of its own size corrupts a
 * heap quietly. */
static char *text_in(napi_env env, napi_value given, size_t *len, size_t *room) {
    size_t size = 0;
    if (napi_get_value_string_utf8(env, given, NULL, 0, &size) != napi_ok) {
        return NULL;
    }
    char *text = (char *)core_alloc(size + 1);
    if (text == NULL) {
        return NULL;
    }
    size_t written = 0;
    if (napi_get_value_string_utf8(env, given, text, size + 1, &written) != napi_ok) {
        core_free((uint8_t *)text, size + 1);
        return NULL;
    }
    *len = written;
    *room = size + 1;
    return text;
}

static napi_value nothing(napi_env env) {
    napi_value answer;
    napi_get_undefined(env, &answer);
    return answer;
}

static napi_value abi(napi_env env, napi_callback_info info) {
    (void)info;
    napi_value answer;
    napi_create_int32(env, (int32_t)core_abi(), &answer);
    return answer;
}

static napi_value open(napi_env env, napi_callback_info info) {
    size_t count = 1;
    napi_value args[1];
    if (napi_get_cb_info(env, info, &count, args, NULL, NULL) != napi_ok || count < 1) {
        napi_throw_error(env, NULL, "open takes one config");
        return nothing(env);
    }
    size_t len = 0;
    size_t room = 0;
    char *config = text_in(env, args[0], &len, &room);
    if (config == NULL) {
        napi_throw_error(env, NULL, "the config did not read");
        return nothing(env);
    }
    CoreHandle *handle = core_open((const uint8_t *)config, len);
    core_free((uint8_t *)config, room);
    /* A null handle is 0 on the ArkTS side, and the host checks for it, the same as Android does. */
    return handle_of(env, handle);
}

static napi_value step(napi_env env, napi_callback_info info) {
    size_t count = 2;
    napi_value args[2];
    if (napi_get_cb_info(env, info, &count, args, NULL, NULL) != napi_ok || count < 2) {
        napi_throw_error(env, NULL, "step takes a handle and a message");
        return nothing(env);
    }
    size_t len = 0;
    size_t room = 0;
    char *input = text_in(env, args[1], &len, &room);
    if (input == NULL) {
        napi_throw_error(env, NULL, "the message did not read");
        return nothing(env);
    }
    uint8_t *out = NULL;
    size_t out_len = 0;
    int32_t status = core_step(handle_in(env, args[0]), (const uint8_t *)input, len, &out, &out_len);
    core_free((uint8_t *)input, room);
    if (status != CORE_OK) {
        /* The same refusal the Android side throws. CORE_PANICKED means this handle is finished and
         * the host must close it and open another; the host decides that, not this file. */
        napi_throw_error(env, NULL, "the core refused a step");
        return nothing(env);
    }
    napi_value answer;
    napi_status made = napi_create_string_utf8(env, (const char *)out, out_len, &answer);
    core_free(out, out_len);
    if (made != napi_ok) {
        napi_throw_error(env, NULL, "the answer did not fit");
        return nothing(env);
    }
    return answer;
}

static napi_value close(napi_env env, napi_callback_info info) {
    size_t count = 1;
    napi_value args[1];
    if (napi_get_cb_info(env, info, &count, args, NULL, NULL) != napi_ok || count < 1) {
        napi_throw_error(env, NULL, "close takes a handle");
        return nothing(env);
    }
    core_close(handle_in(env, args[0]));
    return nothing(env);
}

static napi_value start(napi_env env, napi_value exports) {
    napi_property_descriptor four[] = {
        {"abi", NULL, abi, NULL, NULL, NULL, napi_default, NULL},
        {"open", NULL, open, NULL, NULL, NULL, napi_default, NULL},
        {"step", NULL, step, NULL, NULL, NULL, napi_default, NULL},
        {"close", NULL, close, NULL, NULL, NULL, napi_default, NULL},
    };
    napi_define_properties(env, exports, sizeof(four) / sizeof(four[0]), four);
    return exports;
}

/* UNVERIFIED on this machine, which has no HarmonyOS SDK: everything above is plain Node-API, and
 * this block alone is OpenHarmony's own way of registering a module. See OPEN-QUESTIONS.md Q1.
 */
static napi_module streamscore = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = NULL,
    .nm_register_func = start,
    .nm_modname = "streamscore",
    .nm_priv = NULL,
    .reserved = {0},
};

__attribute__((constructor)) static void announce(void) {
    napi_module_register(&streamscore);
}
