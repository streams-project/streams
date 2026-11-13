/* A runtime for the four calls the bridge registers. See `stub.h` for what it is and is not.
 *
 * Two behaviours are copied from Node-API's documented ones and are the reason this file is not a
 * handful of stubs that answer `napi_ok`:
 *
 *  1. `napi_get_value_string_utf8` is a two-call protocol. With a null buffer it answers the length
 *     in bytes, not counting a terminator; with a buffer it copies at most `bufsize - 1` bytes,
 *     writes a terminator, and answers how many it copied. `text_in` in the bridge is built on
 *     exactly that, and a stub that answered the same number to both calls would hide an off by one
 *     for ever.
 *  2. `napi_get_cb_info`'s `argc` is in and out: in, the size of the array the caller provided;
 *     out, how many arguments really arrived, which may be more. The bridge's `count < 1` and
 *     `count < 2` tests read that, so a stub that answered the provided size would make every
 *     missing-argument branch unreachable.
 *
 * One behaviour is deliberately simpler than the real one, and it is stated rather than hidden: a
 * thrown error here does not make later calls answer `napi_pending_exception`. Under the real
 * runtime, whether `napi_get_undefined` would still hold after `napi_throw_error` decides whether
 * the bridge's `nothing(env)` gives back undefined or NULL -- and both mean the same thing to a
 * runtime with an exception pending, so the bridge is correct either way. The test asserts the
 * throw, which is the part that is not a matter of taste.
 */
#include "stub.h"

#include <stdlib.h>
#include <string.h>

enum { Room = 64 };

enum Kind { KindUndefined, KindNumber, KindText };

struct napi_value__ {
    enum Kind kind;
    int64_t number;
    char *bytes;
    size_t len;
};

struct napi_callback_info__ {
    napi_value *args;
    size_t count;
};

struct napi_env__ {
    int unused;
};

static struct napi_env__ the_env;

static struct {
    napi_value made[Room];
    size_t count;

    int throws;
    char thrown[256];

    int fail_sizing;
    int fail_reading;
    int fail_making;

    napi_module *registered;
    int started;

    napi_property_descriptor methods[Room];
    size_t method_count;
} stub;

static napi_value fresh(enum Kind kind) {
    if (stub.count >= Room) {
        return NULL;
    }
    napi_value made = (napi_value)calloc(1, sizeof(struct napi_value__));
    if (made == NULL) {
        return NULL;
    }
    made->kind = kind;
    stub.made[stub.count] = made;
    stub.count += 1;
    return made;
}

void stub_reset(void) {
    for (size_t at = 0; at < stub.count; at++) {
        free(stub.made[at]->bytes);
        free(stub.made[at]);
    }
    stub.count = 0;
    stub.throws = 0;
    stub.thrown[0] = '\0';
    stub.fail_sizing = 0;
    stub.fail_reading = 0;
    stub.fail_making = 0;
    stub.started = 0;
    stub.method_count = 0;
}

napi_env stub_env(void) { return &the_env; }

napi_value stub_text(const char *bytes, size_t len) {
    napi_value made = fresh(KindText);
    if (made == NULL) {
        return NULL;
    }
    made->bytes = (char *)malloc(len + 1);
    if (made->bytes == NULL) {
        return NULL;
    }
    memcpy(made->bytes, bytes, len);
    made->bytes[len] = '\0';
    made->len = len;
    return made;
}

napi_value stub_say(const char *text) { return stub_text(text, strlen(text)); }

napi_value stub_number(int64_t value) {
    napi_value made = fresh(KindNumber);
    if (made != NULL) {
        made->number = value;
    }
    return made;
}

int stub_is_undefined(napi_value value) {
    return value != NULL && value->kind == KindUndefined;
}

int stub_is_text(napi_value value) { return value != NULL && value->kind == KindText; }

int stub_is_number(napi_value value) { return value != NULL && value->kind == KindNumber; }

size_t stub_text_len(napi_value value) { return value == NULL ? 0 : value->len; }

const char *stub_text_bytes(napi_value value) { return value == NULL ? NULL : value->bytes; }

int64_t stub_number_of(napi_value value) { return value == NULL ? 0 : value->number; }

int stub_throws(void) { return stub.throws; }

const char *stub_thrown(void) { return stub.thrown; }

void stub_fail_sizing_a_string(void) { stub.fail_sizing = 1; }

void stub_fail_reading_a_string(void) { stub.fail_reading = 1; }

void stub_fail_making_a_string(void) { stub.fail_making = 1; }

napi_module *stub_module(void) { return stub.registered; }

int stub_started(void) { return stub.started; }

void stub_start(void) {
    stub.method_count = 0;
    stub.started = 0;
    if (stub.registered == NULL || stub.registered->nm_register_func == NULL) {
        return;
    }
    napi_value exports = fresh(KindUndefined);
    napi_value answer = stub.registered->nm_register_func(stub_env(), exports);
    stub.started = answer == exports;
}

napi_callback stub_method(const char *name) {
    for (size_t at = 0; at < stub.method_count; at++) {
        if (stub.methods[at].utf8name != NULL && strcmp(stub.methods[at].utf8name, name) == 0) {
            return stub.methods[at].method;
        }
    }
    return NULL;
}

size_t stub_methods(void) { return stub.method_count; }

napi_value stub_call(napi_callback method, napi_value *args, size_t count) {
    struct napi_callback_info__ info;
    info.args = args;
    info.count = count;
    return method(stub_env(), &info);
}

/* ------------------------------------------------------------------ the Node-API calls */

napi_status napi_get_undefined(napi_env env, napi_value *result) {
    (void)env;
    napi_value made = fresh(KindUndefined);
    if (made == NULL) {
        return napi_generic_failure;
    }
    *result = made;
    return napi_ok;
}

napi_status napi_create_int32(napi_env env, int32_t value, napi_value *result) {
    (void)env;
    napi_value made = fresh(KindNumber);
    if (made == NULL) {
        return napi_generic_failure;
    }
    made->number = value;
    *result = made;
    return napi_ok;
}

napi_status napi_create_int64(napi_env env, int64_t value, napi_value *result) {
    (void)env;
    napi_value made = fresh(KindNumber);
    if (made == NULL) {
        return napi_generic_failure;
    }
    made->number = value;
    *result = made;
    return napi_ok;
}

napi_status napi_create_string_utf8(napi_env env, const char *str, size_t length,
                                    napi_value *result) {
    (void)env;
    if (stub.fail_making) {
        stub.fail_making = 0;
        return napi_generic_failure;
    }
    napi_value made = stub_text(str, length);
    if (made == NULL) {
        return napi_generic_failure;
    }
    *result = made;
    return napi_ok;
}

napi_status napi_get_value_int64(napi_env env, napi_value value, int64_t *result) {
    (void)env;
    if (value == NULL || value->kind != KindNumber) {
        return napi_number_expected;
    }
    *result = value->number;
    return napi_ok;
}

napi_status napi_get_value_string_utf8(napi_env env, napi_value value, char *buf, size_t bufsize,
                                       size_t *result) {
    (void)env;
    if (value == NULL || value->kind != KindText) {
        return napi_string_expected;
    }
    if (buf == NULL) {
        if (stub.fail_sizing) {
            stub.fail_sizing = 0;
            return napi_generic_failure;
        }
        *result = value->len;
        return napi_ok;
    }
    if (stub.fail_reading) {
        stub.fail_reading = 0;
        return napi_generic_failure;
    }
    if (bufsize == 0) {
        *result = 0;
        return napi_ok;
    }
    size_t room = bufsize - 1;
    size_t copied = value->len < room ? value->len : room;
    memcpy(buf, value->bytes, copied);
    buf[copied] = '\0';
    *result = copied;
    return napi_ok;
}

napi_status napi_get_cb_info(napi_env env, napi_callback_info cbinfo, size_t *argc, napi_value *argv,
                             napi_value *this_arg, void **data) {
    (void)env;
    if (cbinfo == NULL) {
        return napi_invalid_arg;
    }
    if (argv != NULL && argc != NULL) {
        size_t room = *argc;
        size_t at = 0;
        for (; at < room && at < cbinfo->count; at++) {
            argv[at] = cbinfo->args[at];
        }
        for (; at < room; at++) {
            napi_value nothing = fresh(KindUndefined);
            argv[at] = nothing;
        }
    }
    if (argc != NULL) {
        /* How many really arrived, which is what the bridge's own count test reads. */
        *argc = cbinfo->count;
    }
    if (this_arg != NULL) {
        *this_arg = NULL;
    }
    if (data != NULL) {
        *data = NULL;
    }
    return napi_ok;
}

napi_status napi_throw_error(napi_env env, const char *code, const char *msg) {
    (void)env;
    (void)code;
    stub.throws += 1;
    if (msg != NULL) {
        size_t len = strlen(msg);
        if (len >= sizeof(stub.thrown)) {
            len = sizeof(stub.thrown) - 1;
        }
        memcpy(stub.thrown, msg, len);
        stub.thrown[len] = '\0';
    }
    return napi_ok;
}

napi_status napi_define_properties(napi_env env, napi_value object, size_t property_count,
                                   const napi_property_descriptor *properties) {
    (void)env;
    (void)object;
    if (properties == NULL) {
        return napi_invalid_arg;
    }
    for (size_t at = 0; at < property_count && stub.method_count < Room; at++) {
        stub.methods[stub.method_count] = properties[at];
        stub.method_count += 1;
    }
    return napi_ok;
}

void napi_module_register(napi_module *mod) { stub.registered = mod; }
